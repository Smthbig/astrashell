use anyhow::Result;
use std::ffi::CString;

/// User namespace manager - provides fake root capability
/// Maps UID/GID 0 inside namespace to unprivileged UID/GID outside
pub struct UserNamespace {
    uid_map: Option<String>,
    gid_map: Option<String>,
}

impl UserNamespace {
    pub fn new(uid_map: Option<&str>, gid_map: Option<&str>) -> Self {
        Self {
            uid_map: uid_map.map(|s| s.to_string()),
            gid_map: gid_map.map(|s| s.to_string()),
        }
    }

    /// Unshare into a new user namespace
    pub fn unshare(&self) -> Result<()> {
        unsafe {
            let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWUSER);
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::EINVAL || err == libc::EPERM {
                    tracing::warn!("User namespaces not supported (err {}), continuing without", err);
                    return Ok(());
                }
                return Err(anyhow::anyhow!("unshare(CLONE_NEWUSER) failed: {}", err));
            }
        }

        // Write UID/GID mappings if not specified
        // Default: map UID 0 in ns to current UID outside
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };

        // Write /proc/self/uid_map
        let uid_map = self.uid_map.as_deref().unwrap_or(&format!("0 {} 1", uid));
        std::fs::write("/proc/self/uid_map", uid_map.as_bytes())?;

        // Write /proc/self/setgroups (deny)
        let _ = std::fs::write("/proc/self/setgroups", b"deny");

        // Write /proc/self/gid_map
        let gid_map = self.gid_map.as_deref().unwrap_or(&format!("0 {} 1", gid));
        std::fs::write("/proc/self/gid_map", gid_map.as_bytes())?;

        tracing::info!("User namespace created: uid {} -> 0, gid {} -> 0", uid, gid);
        Ok(())
    }

    /// Check if we're already in a user namespace (UID 0 inside)
    pub fn is_root() -> bool {
        unsafe { libc::getuid() == 0 }
    }
}
