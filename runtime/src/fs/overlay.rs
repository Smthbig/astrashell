use anyhow::Result;
use std::path::Path;

/// Overlay filesystem manager
/// Supports both kernel overlayfs (when available) and FUSE-overlayfs fallback
pub struct OverlayFS {
    lower: String,
    upper: Option<String>,
    work: Option<String>,
    merged: String,
}

impl OverlayFS {
    pub fn new(lower: &str, upper: Option<&str>, work: Option<&str>) -> Result<Self> {
        let lower_path = Path::new(lower);
        if !lower_path.exists() {
            return Err(anyhow::anyhow!("Lower directory does not exist: {}", lower));
        }

        Ok(Self {
            lower: lower.to_string(),
            upper: upper.map(|s| s.to_string()),
            work: work.map(|s| s.to_string()),
            merged: "/tmp/astrashell-merged".to_string(),
        })
    }

    /// Mount the overlay filesystem
    pub fn mount(&self) -> Result<()> {
        let _ = std::fs::create_dir_all(&self.merged);

        if let (Some(upper), Some(work)) = (&self.upper, &self.work) {
            // Try kernel overlayfs first
            let options = format!(
                "lowerdir={},upperdir={},workdir={}",
                self.lower, upper, work
            );

            unsafe {
                let ret = libc::mount(
                    std::ptr::null(),
                    std::ffi::CString::new(self.merged.as_str()).unwrap().as_ptr(),
                    std::ffi::CString::new("overlay").unwrap().as_ptr(),
                    libc::MS_NODEV,
                    std::ffi::CString::new(options.as_str()).unwrap().as_ptr() as *const libc::c_void,
                );

                if ret == 0 {
                    tracing::info!("Kernel overlayfs mounted at {}", self.merged);
                    return Ok(());
                }

                let err = *libc::__errno_location();
                if err == libc::ENOSYS || err == libc::ENODATA || err == libc::EPERM {
                    tracing::warn!("Kernel overlayfs not supported (err {}), trying FUSE", err);
                    return self.mount_fuse_overlayfs(upper, work);
                }
                return Err(anyhow::anyhow!("overlay mount failed: {}", err));
            }
        }

        // No overlay: bind mount lower directly
        unsafe {
            libc::mount(
                std::ffi::CString::new(self.lower.as_str()).unwrap().as_ptr(),
                std::ffi::CString::new(self.merged.as_str()).unwrap().as_ptr(),
                std::ptr::null(),
                libc::MS_BIND | libc::MS_REC,
                std::ptr::null(),
            );
        }
        tracing::info!("Bind-mounted {} to {}", self.lower, self.merged);
        Ok(())
    }

    /// Mount using FUSE-overlayfs when kernel overlayfs is unavailable
    fn mount_fuse_overlayfs(&self, _upper: &str, _work: &str) -> Result<()> {
        // Check if fuse-overlayfs binary exists
        let fuse_binary = crate::util::path::which("fuse-overlayfs");
        if let Some(bin) = fuse_binary {
            tracing::info!("Using fuse-overlayfs: {}", bin);
            let status = std::process::Command::new(&bin)
                .args(&[
                    "-o", &format!("lowerdir={},upperdir={},workdir={}", self.lower, _upper, _work),
                    &self.merged,
                ])
                .spawn()?
                .wait()?;
            if status.success() {
                return Ok(());
            }
        }

        tracing::warn!("FUSE-overlayfs not available, using bind mount (non-writable overlay)");
        Err(anyhow::anyhow!("No overlay filesystem available"))
    }

    /// Get the merged view path
    pub fn merged_path(&self) -> &str {
        &self.merged
    }

    /// Unmount overlay
    pub fn unmount(&self) -> Result<()> {
        unsafe {
            libc::umount2(
                std::ffi::CString::new(self.merged.as_str()).unwrap().as_ptr(),
                libc::MNT_DETACH,
            );
        }
        let _ = std::fs::remove_dir(&self.merged);
        Ok(())
    }
}

impl Drop for OverlayFS {
    fn drop(&mut self) {
        let _ = self.unmount();
    }
}
