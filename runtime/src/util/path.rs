/// Path and binary utilities
use std::path::{Path, PathBuf};

/// Find a binary in PATH
pub fn which(name: &str) -> Option<String> {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths).find_map(|dir| {
                let full_path = dir.join(name);
                if full_path.is_file() {
                    Some(full_path.to_string_lossy().to_string())
                } else {
                    None
                }
            })
        })
}

/// Find binary in PATH or return default
pub fn which_or(name: &str, default: &str) -> String {
    which(name).unwrap_or_else(|| default.to_string())
}

/// Expand ~ to home directory
pub fn expand_home(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(&path[2..]).to_string_lossy().to_string();
        }
    }
    path.to_string()
}

/// Ensure a directory exists
pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

/// Copy with sparse file awareness for Android
pub fn copy_sparse(src: &Path, dst: &Path) -> std::io::Result<u64> {
    // On Android f2fs, use fallocate for sparse
    let mut src_file = std::fs::File::open(src)?;
    let dst_file = std::fs::File::create(dst)?;
    let metadata = src_file.metadata()?;
    let len = metadata.len();

    // Pre-allocate
    unsafe {
        libc::fallocate(
            std::os::unix::io::AsRawFd::as_raw_fd(&dst_file),
            0x02, // FALLOC_FL_KEEP_SIZE
            0,
            len as i64,
        );
    }

    std::io::copy(&mut src_file, &mut dst_file)
}
