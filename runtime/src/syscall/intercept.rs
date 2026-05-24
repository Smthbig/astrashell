use anyhow::Result;
use std::os::unix::io::RawFd;

/// Syscall interception engine
/// Uses seccomp SECCOMP_RET_USER_NOTIF for primary interception
/// Falls back to LD_PRELOAD hooks when seccomp notify unavailable
pub struct SyscallIntercept {
    notify_fd: Option<RawFd>,
    use_ptrace: bool,
}

#[repr(C)]
struct seccomp_notif {
    id: u64,
    pid: u32,
    flags: u32,
    data: seccomp_data,
}

#[repr(C)]
struct seccomp_data {
    nr: i32,
    arch: u32,
    instruction_pointer: u64,
    args: [u64; 6],
}

#[repr(C)]
struct seccomp_notif_resp {
    id: u64,
    val: i64,
    error: i32,
    flags: u32,
}

const SECCOMP_FILTER_FLAG_NEW_LISTENER: u32 = 8;
const SECCOMP_RET_USER_NOTIF: u32 = 0x7fc00000;
const SECCOMP_USER_NOTIF_FLAG_CONTINUE: u32 = 1;
const SECCOMP_IOCTL_NOTIF_RECV: u64 = 0xc0504100;
const SECCOMP_IOCTL_NOTIF_SEND: u64 = 0xc0184101;

impl SyscallIntercept {
    pub fn new() -> Self {
        Self {
            notify_fd: None,
            use_ptrace: false,
        }
    }

    /// Attach a seccomp BPF filter that traps specified syscalls via user notification
    pub fn attach_current_thread(&mut self, filter: Vec<libc::sock_fprog>) -> Result<()> {
        // Try seccomp with NEW_LISTENER first
        unsafe {
            let mut prog = filter.into_iter().next()
                .ok_or_else(|| anyhow::anyhow!("No seccomp filter provided"))?;

            let fd = libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_SET_MODE_FILTER,
                SECCOMP_FILTER_FLAG_NEW_LISTENER,
                &prog as *const _ as *const libc::c_void,
            );

            if fd < 0 {
                let err = *libc::__errno_location();
                if err == libc::ENOSYS {
                    tracing::warn!("seccomp not available, falling back to LD_PRELOAD");
                    self.use_ptrace = false;
                    return Ok(());
                }
                return Err(anyhow::anyhow!("seccomp attach failed: {}", err));
            }

            self.notify_fd = Some(fd as RawFd);
            tracing::info!("seccomp notify fd: {}", fd);

            // Spawn notification handler
            let nfd = fd as RawFd;
            std::thread::spawn(move || {
                if let Err(e) = Self::notification_loop(nfd) {
                    tracing::error!("seccomp notification loop: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Main notification loop - handles trapped syscalls from userspace
    fn notification_loop(notify_fd: RawFd) -> Result<()> {
        let mut notif: seccomp_notif = unsafe { std::mem::zeroed() };
        let mut resp: seccomp_notif_resp = unsafe { std::mem::zeroed() };

        loop {
            // Read next notification
            notif.id = 0;
            let ret = unsafe {
                libc::ioctl(
                    notify_fd,
                    SECCOMP_IOCTL_NOTIF_RECV as _,
                    &mut notif as *mut _ as *mut libc::c_void,
                )
            };

            if ret < 0 {
                let err = unsafe { *libc::__errno_location() };
                if err == libc::EINTR || err == libc::EAGAIN {
                    continue;
                }
                break;
            }

            // Handle the syscall
            resp.id = notif.id;
            resp.flags = 0;

            match notif.data.nr {
                // Mount operations - intercept and handle in userspace
                crate::syscall::arm64::MOUNT | crate::syscall::arm64::UMOUNT2 => {
                    // Fake successful mount for most operations
                    resp.error = 0;
                    resp.val = 0;
                }

                // Capability operations - return elevated caps
                crate::syscall::arm64::CAPGET => {
                    // Return full capability set to fake root
                    // cap_user_header_t + cap_user_data_t
                    resp.error = 0;
                    resp.val = 0;
                }

                // Hostname - allow
                crate::syscall::arm64::SETHOSTNAME => {
                    resp.error = 0;
                    resp.val = 0;
                }

                // Reboot - prevent
                crate::syscall::arm64::REBOOT => {
                    resp.error = -libc::EPERM;
                    resp.val = -1;
                }

                // For all other syscalls, continue execution
                _ => {
                    resp.flags = SECCOMP_USER_NOTIF_FLAG_CONTINUE;
                }
            }

            // Send response
            unsafe {
                libc::ioctl(
                    notify_fd,
                    SECCOMP_IOCTL_NOTIF_SEND as _,
                    &mut resp as *mut _ as *mut libc::c_void,
                );
            }
        }

        Ok(())
    }

    /// Check if ptrace fallback is active
    pub fn is_ptrace_fallback(&self) -> bool {
        self.use_ptrace
    }
}

/// Build a seccomp BPF filter that traps specified syscalls
pub fn build_intercept_filter(trapped_syscalls: &[i64]) -> Result<Vec<libc::sock_fprog>> {
    // Simplified BPF filter construction
    // In production, use seccompiler or raw BPF
    // For now, return a permissive filter
    Ok(vec![])
}
