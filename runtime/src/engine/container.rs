use crate::engine::Engine;
use crate::Config;
use crate::namespace::mount::MountContext;
use crate::fs::overlay::OverlayFS;
use crate::security::seccomp::SeccompProfile;
use crate::security::capabilities::CapabilitySet;
use crate::container::oci::OCIRuntime;
use anyhow::Result;
use async_trait::async_trait;

/// Full container execution engine
pub struct ContainerEngine {
    oci: OCIRuntime,
}

impl ContainerEngine {
    pub fn new() -> Self {
        Self { oci: OCIRuntime::new() }
    }
}

#[async_trait]
impl Engine for ContainerEngine {
    fn name(&self) -> &'static str { "container" }
    fn priority(&self) -> u32 { 20 }

    async fn available(&self) -> bool {
        crate::util::kernel::has_user_namespaces().await
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        tracing::info!("Starting container execution engine");

        let overlay = OverlayFS::new(
            &config.runtime.rootfs,
            config.filesystem.upper_dir.as_deref(),
            config.filesystem.work_dir.as_deref(),
        )?;

        // User namespace (fake root)
        let userns = crate::namespace::user::UserNamespace::new(
            config.runtime.uid_map.as_deref(),
            config.runtime.gid_map.as_deref(),
        );
        userns.unshare()?;

        // Mount namespace
        let mntns = MountContext::new(&config.runtime.rootfs)?;
        mntns.setup(&config.filesystem).await?;

        // PID namespace
        let pidns = crate::namespace::pid::PidNamespace::new();
        pidns.unshare()?;

        // UTS namespace
        let utsns = crate::namespace::pid::UtsNamespace::new(&config.runtime.hostname);
        utsns.setup()?;

        // IPC namespace
        if config.filesystem.dev {
            let ipcns = crate::namespace::pid::IpcNamespace::new();
            ipcns.unshare()?;
        }

        // Network namespace
        if config.network.type_ != crate::NetworkType::Host {
            let netns = crate::namespace::pid::NetNamespace::new();
            netns.setup(&config.network).await?;
        }

        // Mount pseudo filesystems
        mntns.mount_pseudo().await?;

        // seccomp
        let seccomp = SeccompProfile::new(&config.security.seccomp_profile);
        seccomp.apply()?;

        // Drop capabilities
        let caps = CapabilitySet::new(&config.security.capabilities);
        caps.apply()?;

        // pivot_root
        mntns.pivot_root()?;

        // Execute PID 1
        let pid1 = &config.engine.pid1;
        self.exec(config, &[pid1.to_string()]).await?;

        Ok(())
    }

    async fn exec(&self, config: &Config, cmd: &[String]) -> Result<i32> {
        let args: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        let path = config.runtime.env.iter()
            .find(|e| e.starts_with("PATH="))
            .map(|e| &e[5..])
            .unwrap_or("/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");

        let mut child = std::process::Command::new(args[0])
            .args(&args[1..])
            .env_clear()
            .env("PATH", path)
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
