use crate::Config;
use anyhow::Result;

/// Mount namespace context - manages filesystem isolation
pub struct MountContext {
    rootfs: String,
    old_root: Option<String>,
}

impl MountContext {
    pub fn new(rootfs: &str) -> Self {
        Self {
            rootfs: rootfs.to_string(),
            old_root: None,
        }
    }

    /// Setup basic filesystem mounts
    pub async fn setup(&self, fs_cfg: &crate::FilesystemConfig) -> Result<()> {
        // Make rootfs mount private to prevent propagation
        unsafe {
            libc::mount(
                std::ptr::null(),
                CString::new(self.rootfs.as_str()).unwrap().as_ptr(),
                std::ptr::null(),
                libc::MS_PRIVATE | libc::MS_REC,
                std::ptr::null(),
            );
        }

        // Recursively bind mount rootfs
        unsafe {
            libc::mount(
                CString::new(self.rootfs.as_str()).unwrap().as_ptr(),
                CString::new(self.rootfs.as_str()).unwrap().as_ptr(),
                std::ptr::null(),
                libc::MS_BIND | libc::MS_REC,
                std::ptr::null(),
            );
        }

        Ok(())
    }

    /// Mount pseudo-filesystems inside the new root
    pub async fn mount_pseudo(&self) -> Result<()> {
        let root = &self.rootfs;

        // Create mount points
        let dirs = ["/proc", "/sys", "/dev", "/dev/pts", "/sys/fs/cgroup", "/tmp", "/run"];
        for d in &dirs {
            let path = format!("{}{}", root, d);
            let _ = std::fs::create_dir_all(&path);
        }

        // Mount proc
        unsafe {
            libc::mount(
                std::ptr::null(),
                CString::new(format!("{}/proc", root)).unwrap().as_ptr(),
                CString::new("proc").unwrap().as_ptr(),
                libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC | libc::MS_RELATIME,
                std::ptr::null(),
            );
        }

        // Mount sysfs
        unsafe {
            libc::mount(
                std::ptr::null(),
                CString::new(format!("{}/sys", root)).unwrap().as_ptr(),
                CString::new("sysfs").unwrap().as_ptr(),
                libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC | libc::MS_RELATIME,
                std::ptr::null(),
            );
        }

        // Mount tmpfs on /tmp and /run
        for d in &["/tmp", "/run"] {
            unsafe {
                libc::mount(
                    std::ptr::null(),
                    CString::new(format!("{}{}", root, d)).unwrap().as_ptr(),
                    CString::new("tmpfs").unwrap().as_ptr(),
                    libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NODIRATIME,
                    CString::new("mode=1777,size=64M").unwrap().as_ptr() as *const libc::c_void,
                );
            }
        }

        // Mount devtmpfs
        unsafe {
            libc::mount(
                std::ptr::null(),
                CString::new(format!("{}/dev", root)).unwrap().as_ptr(),
                CString::new("devtmpfs").unwrap().as_ptr(),
                libc::MS_NOSUID | libc::MS_NODEV | libc::MS_RELATIME,
                std::ptr::null(),
            );
        }

        // Mount devpts
        unsafe {
            libc::mount(
                std::ptr::null(),
                CString::new(format!("{}/dev/pts", root)).unwrap().as_ptr(),
                CString::new("devpts").unwrap().as_ptr(),
                libc::MS_NOSUID | libc::MS_NOEXEC,
                CString::new("mode=620,gid=5").unwrap().as_ptr() as *const libc::c_void,
            );
        }

        tracing::info!("Pseudo-filesystems mounted");
        Ok(())
    }

    /// pivot_root to the new filesystem
    pub fn pivot_root(&self) -> Result<()> {
        let root = CString::new(self.rootfs.as_str()).unwrap();
        let put_old = CString::new(format!("{}/.old_root", self.rootfs)).unwrap();

        // Create put_old directory
        let _ = std::fs::create_dir_all(format!("{}/.old_root", self.rootfs));

        unsafe {
            let ret = libc::syscall(
                libc::SYS_pivot_root,
                root.as_ptr(),
                put_old.as_ptr(),
            );
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::ENOSYS {
                    // Fall back to chroot
                    tracing::warn!("pivot_root not available, using chroot fallback");
                    libc::chroot(root.as_ptr());
                } else {
                    return Err(anyhow::anyhow!("pivot_root failed: {}", err));
                }
            }
        }

        // Unmount old root
        unsafe {
            libc::umount2(
                CString::new("/.old_root").unwrap().as_ptr(),
                libc::MNT_DETACH,
            );
        }
        let _ = std::fs::remove_dir("/.old_root");

        // chdir to new root
        std::env::set_current_dir("/")?;

        tracing::info!("pivot_root completed to {}", self.rootfs);
        Ok(())
    }
}

use std::ffi::CString;
