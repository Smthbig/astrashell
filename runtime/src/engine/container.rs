use crate::engine::Engine;
use crate::Config;
use crate::namespace::user::UserNamespace;
use crate::namespace::mount::MountContext;
use crate::namespace::pid::PidNamespace;
use crate::namespace::net::NetNamespace;
use crate::namespace::uts::UtsNamespace;
use crate::namespace::ipc::IpcNamespace;
use crate::fs::overlay::OverlayFS;
use crate::container::oci::OCIRuntime;
use crate::container::cgroup::CgroupManager;
use crate::security::seccomp::SeccompProfile;
use crate::security::capabilities::CapabilitySet;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Full container execution engine
/// Uses Linux namespaces for process isolation
/// This is the most powerful mode - requires user namespace support
pub struct ContainerEngine {
    oci: OCIRuntime,
}

impl ContainerEngine {
    pub fn new() -> Self {
        Self {
            oci: OCIRuntime::new(),
        }
    }
}

#[async_trait]
impl Engine for ContainerEngine {
    fn name(&self) -> &'static str { "container" }
    fn priority(&self) -> u32 { 20 }

    async fn available(&self) -> bool {
        // Check for user namespace support and newuidmap/newgidmap
        let has_userns = crate::util::kernel::has_user_namespaces().await;
        let has_newuidmap = crate::util::path::which("newuidmap").is_some();
        let has_newgidmap = crate::util::path::which("newgidmap").is_some();
        has_userns // user namespace is the only hard requirement
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        tracing::info!("Starting container execution engine");

        // 1. Setup overlay filesystem
        let overlay = OverlayFS::new(
            &config.runtime.rootfs,
            config.filesystem.upper_dir.as_deref(),
            config.filesystem.work_dir.as_deref(),
        )?;

        // 2. Create user namespace (fake root)
        let userns = UserNamespace::new(
            config.runtime.uid_map.as_deref(),
            config.runtime.gid_map.as_deref(),
        );
        userns.unshare()?;

        // 3. Create mount namespace
        let mntns = MountContext::new(&config.runtime.rootfs)?;
        mntns.setup(&config.filesystem).await?;

        // 4. Create PID namespace
        let pidns = PidNamespace::new();
        pidns.unshare()?;

        // 5. Create UTS namespace
        let utsns = UtsNamespace::new(&config.runtime.hostname);
        utsns.setup()?;

        // 6. Create IPC namespace
        if config.filesystem.dev {
            let ipcns = IpcNamespace::new();
            ipcns.unshare()?;
        }

        // 7. Create network namespace (if isolated)
        if config.network.type_ != crate::NetworkType::Host {
            let netns = NetNamespace::new();
            netns.setup(&config.network).await?;
        }

        // 8. Mount proc, sys, dev
        mntns.mount_pseudo().await?;

        // 9. Setup seccomp
        let seccomp = SeccompProfile::new(&config.security.seccomp_profile);
        seccomp.apply()?;

        // 10. Drop capabilities
        let caps = CapabilitySet::new(&config.security.capabilities);
        caps.apply()?;

        // 11. pivot_root into the new rootfs
        mntns.pivot_root()?;

        // 12. Execute PID 1
        let pid1 = &config.engine.pid1;
        self.exec(config, &[pid1.to_string()]).await?;

        Ok(())
    }

    async fn exec(&self, config: &Config, cmd: &[String]) -> Result<i32> {
        let args: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        let path_env = config.runtime.env.iter()
            .flat_map(|e| e.split(':'))
            .collect::<Vec<_>>()
            .join(":");

        let mut child = std::process::Command::new(args[0])
            .args(&args[1..])
            .env_clear()
            .env("PATH", &path_env)
            .env("HOME", &config.runtime.workdir)
            .env("TERM", "xterm-256color")
            .env("container", "astrashell")
            .envs(config.runtime.env.iter().filter_map(|e| {
                let mut parts = e.splitn(2, '=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            }))
            .spawn()?;

        let status = child.wait()?;
        Ok(status.code().unwrap_or(-1))
    }
}
