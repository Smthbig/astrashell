use crate::engine::Engine;
use crate::Config;
use crate::syscall::intercept::SyscallIntercept;
use crate::namespace::user::UserNamespace;
use crate::namespace::mount::MountContext;
use crate::fs::overlay::OverlayFS;
use crate::security::seccomp::SeccompProfile;
use crate::net::slirp::SlirpNetwork;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Native execution engine - fast path using LD_PRELOAD + seccomp
/// Does NOT require root. Uses seccomp user notification for syscall interception.
pub struct NativeEngine;

impl NativeEngine {
    pub fn new() -> Self {
        Self
    }

    fn probe_seccomp_notify() -> bool {
        // Try to create a seccomp filter with SECCOMP_FILTER_FLAG_NEW_LISTENER
        unsafe {
            let ret = libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_SET_MODE_FILTER,
                libc::SECCOMP_FILTER_FLAG_NEW_LISTENER,
                std::ptr::null(),
            );
            // If ENOSYS, seccomp not available. If EFAULT, filter not set but syscall exists
            ret >= 0 || *libc::__errno_location() != libc::ENOSYS
        }
    }
}

#[async_trait]
impl Engine for NativeEngine {
    fn name(&self) -> &'static str { "native" }
    fn priority(&self) -> u32 { 10 }

    async fn available(&self) -> bool {
        // Check if user namespaces or seccomp notify is available
        Self::probe_seccomp_notify() || crate::util::kernel::has_user_namespaces().await
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        tracing::info!("Starting native execution engine");

        // 1. Setup overlay filesystem
        let overlay = OverlayFS::new(
            &config.runtime.rootfs,
            config.filesystem.upper_dir.as_deref(),
            config.filesystem.work_dir.as_deref(),
        )?;
        overlay.mount()?;

        // 2. Setup mount namespace context
        let mount_ctx = MountContext::new(&config.runtime.rootfs)?;
        mount_ctx.setup(&config.filesystem).await?;

        // 3. Setup seccomp interception
        let seccomp = SeccompProfile::new(&config.security.seccomp_profile);
        let mut interceptor = SyscallIntercept::new();
        interceptor.attach_current_thread(seccomp.get_filter()?)?;

        // 4. Start network (slirp)
        if config.network.enabled {
            let net = SlirpNetwork::new();
            tokio::spawn(async move { net.run().await });
        }

        // 5. Execute PID 1 (or shell)
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

        // Override linker for glibc compatibility on bionic
        // When running on Android, use LD_PRELOAD to intercept and translate
        std::env::set_var("LD_PRELOAD", "libastrashell-preload.so");
        std::env::set_var("ASTRASHELL_ROOT", &config.runtime.rootfs);
        std::env::set_var("ASTRASHELL_HOSTNAME", &config.runtime.hostname);

        let status = std::process::Command::new(args[0])
            .args(&args[1..])
            .env_clear()
            .env("PATH", &path_env)
            .env("HOME", &config.runtime.workdir)
            .env("TERM", "xterm-256color")
            .envs(config.runtime.env.iter().filter_map(|e| {
                let mut parts = e.splitn(2, '=');
                Some((parts.next()?.to_string(), parts.next()?.to_string()))
            }))
            .spawn()?
            .wait()?;

        Ok(status.code().unwrap_or(-1))
    }
}
