/// Syscall broker daemon
/// Runs as a privileged helper to perform operations on behalf of the sandboxed process
/// This replaces the traditional setuid-root helper with a more secure model
use anyhow::Result;
use tokio::net::UnixListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerRequest {
    Mount {
        source: String,
        target: String,
        fstype: String,
        flags: u64,
    },
    Umount {
        target: String,
        flags: i32,
    },
    PivotRoot {
        new_root: String,
        put_old: String,
    },
    SetHostname {
        name: String,
    },
    CreateNamespace {
        flags: i32,
    },
    Setns {
        fd: i64,
        nstype: i32,
    },
    CapSet {
        pid: i32,
        caps: Vec<u32>,
    },
    Chown {
        path: String,
        uid: u32,
        gid: u32,
    },
    Mknod {
        path: String,
        mode: u32,
        dev: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrokerResponse {
    Success,
    Error { code: i32, message: String },
    Data { value: i64 },
}

/// Syscall broker - handles privileged operations
pub struct SyscallBroker {
    socket_path: String,
}

impl SyscallBroker {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            socket_path: path.as_ref().to_string_lossy().into(),
        }
    }

    /// Start the broker listener
    pub async fn listen(&self) -> Result<()> {
        // Remove stale socket
        let _ = std::fs::remove_file(&self.socket_path);
        
        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("Syscall broker listening on {}", self.socket_path);

        loop {
            let (mut stream, _) = listener.accept().await?;
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                loop {
                    match stream.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let req: BrokerRequest = match rmp_serde::from_slice(&buf[..n]) {
                                Ok(r) => r,
                                Err(e) => {
                                    tracing::error!("Failed to deserialize request: {}", e);
                                    continue;
                                }
                            };

                            let resp = Self::handle_request(&req);
                            let encoded = rmp_serde::to_vec(&resp).unwrap_or_default();
                            if let Err(e) = stream.write_all(&encoded).await {
                                tracing::error!("Failed to send response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Read error: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }

    fn handle_request(req: &BrokerRequest) -> BrokerResponse {
        match req {
            BrokerRequest::Mount { source, target, fstype, flags } => {
                // Check mount against allowlist
                if !is_mount_allowed(source, target, fstype) {
                    return BrokerResponse::Error {
                        code: -libc::EPERM,
                        message: "Mount not allowed".into(),
                    };
                }

                unsafe {
                    let ret = libc::mount(
                        std::ffi::CString::new(source.as_str()).unwrap().as_ptr(),
                        std::ffi::CString::new(target.as_str()).unwrap().as_ptr(),
                        std::ffi::CString::new(fstype.as_str()).unwrap().as_ptr(),
                        *flags,
                        std::ptr::null(),
                    );
                    if ret == 0 {
                        BrokerResponse::Success
                    } else {
                        BrokerResponse::Error {
                            code: *libc::__errno_location(),
                            message: "Mount failed".into(),
                        }
                    }
                }
            }
            BrokerRequest::Umount { target, flags } => {
                unsafe {
                    let ret = libc::umount2(
                        std::ffi::CString::new(target.as_str()).unwrap().as_ptr(),
                        *flags,
                    );
                    if ret == 0 {
                        BrokerResponse::Success
                    } else {
                        BrokerResponse::Error {
                            code: *libc::__errno_location(),
                            message: "Umount failed".into(),
                        }
                    }
                }
            }
            BrokerRequest::PivotRoot { new_root, put_old } => {
                unsafe {
                    let ret = libc::syscall(
                        libc::SYS_pivot_root,
                        std::ffi::CString::new(new_root.as_str()).unwrap().as_ptr(),
                        std::ffi::CString::new(put_old.as_str()).unwrap().as_ptr(),
                    );
                    if ret == 0 {
                        BrokerResponse::Success
                    } else {
                        BrokerResponse::Error {
                            code: *libc::__errno_location(),
                            message: "pivot_root failed".into(),
                        }
                    }
                }
            }
            _ => BrokerResponse::Error {
                code: -libc::ENOSYS,
                message: "Unsupported operation".into(),
            },
        }
    }
}

/// Check if a mount operation is in the allowlist
fn is_mount_allowed(source: &str, target: &str, fstype: &str) -> bool {
    let allowed_fs = ["proc", "sysfs", "tmpfs", "devpts", "devtmpfs", "cgroup", "cgroup2"];
    let allowed_targets = ["/proc", "/sys", "/dev", "/dev/pts", "/sys/fs/cgroup", "/tmp", "/run"];

    // Check if this is a bind mount of an allowed path
    if target.starts_with("/proc") || target.starts_with("/sys") || target.starts_with("/dev") {
        return fstype == "bind" || fstype == "none" || allowed_fs.contains(&fstype);
    }

    allowed_fs.contains(&fstype)
}
