use anyhow::Result;

pub struct PidNamespace;

impl PidNamespace {
    pub fn new() -> Self { Self }

    pub fn unshare(&self) -> Result<()> {
        unsafe {
            let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWPID);
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINVAL || err == libc::EPERM {
                    tracing::warn!("PID namespaces not supported, continuing");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("unshare(CLONE_NEWPID) failed: {}", err));
            }
        }
        tracing::debug!("PID namespace created");
        Ok(())
    }
}

pub struct NetNamespace;

impl NetNamespace {
    pub fn new() -> Self { Self }

    pub async fn setup(&self, _net_cfg: &crate::NetworkConfig) -> Result<()> {
        unsafe {
            let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWNET);
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINVAL || err == libc::EPERM {
                    tracing::warn!("Network namespaces not supported, continuing with host net");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("unshare(CLONE_NEWNET) failed: {}", err));
            }
        }
        tracing::debug!("Network namespace created");
        Ok(())
    }
}

pub struct UtsNamespace {
    hostname: String,
}

impl UtsNamespace {
    pub fn new(hostname: &str) -> Self {
        Self { hostname: hostname.to_string() }
    }

    pub fn setup(&self) -> Result<()> {
        unsafe {
            let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWUTS);
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINVAL || err == libc::EPERM {
                    tracing::warn!("UTS namespaces not supported");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("unshare(CLONE_NEWUTS) failed: {}", err));
            }
        }

        // Set hostname
        unsafe {
            libc::sethostname(
                self.hostname.as_ptr() as *const libc::c_char,
                self.hostname.len(),
            );
        }

        Ok(())
    }
}

pub struct IpcNamespace;

impl IpcNamespace {
    pub fn new() -> Self { Self }

    pub fn unshare(&self) -> Result<()> {
        unsafe {
            let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWIPC);
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINVAL || err == libc::EPERM {
                    tracing::warn!("IPC namespaces not supported");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("unshare(CLONE_NEWIPC) failed: {}", err));
            }
        }
        Ok(())
    }
}

pub mod cgroup {
    use anyhow::Result;

    pub struct CgroupManager;

    impl CgroupManager {
        pub fn new() -> Self { Self }

        pub fn setup(&self) -> Result<()> {
            // Attempt to unshare cgroup namespace
            unsafe {
                let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWCGROUP);
                if ret < 0 {
                    tracing::warn!("CGroup namespaces not supported");
                }
            }
            Ok(())
        }
    }
}
