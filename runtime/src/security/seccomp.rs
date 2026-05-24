use anyhow::Result;

/// Seccomp profile manager
pub struct SeccompProfile {
    profile_name: String,
}

impl SeccompProfile {
    pub fn new(profile: &str) -> Self {
        Self { profile_name: profile.to_string() }
    }

    pub fn get_filter(&self) -> Result<Vec<libc::sock_fprog>> {
        Ok(vec![])
    }

    /// Apply seccomp filter to current process
    pub fn apply(&self) -> Result<()> {
        // Build a static BPF filter (no dangling pointers)
        let filter = build_astrashell_filter();
        // The sock_fprog holds a pointer into the filter array which must live
        let filter_vec = filter.0;
        let prog = libc::sock_fprog {
            len: filter_vec.len() as u16,
            filter: filter_vec.as_ptr(),
        };

        unsafe {
            let ret = libc::syscall(
                libc::SYS_seccomp,
                libc::SECCOMP_SET_MODE_FILTER,
                0,
                &prog as *const libc::sock_fprog as *const libc::c_void,
            );
            if ret < 0 {
                let err = *libc::__errno_location();
                if err == libc::ENOSYS {
                    tracing::warn!("seccomp not available, skipping");
                    return Ok(());
                }
                return Err(anyhow::anyhow!("seccomp apply failed: {}", err));
            }
        }
        // filter_vec lives until function return, but seccomp syscall copies the program
        // into kernel memory, so this is safe
        Ok(())
    }
}

// BPF instruction
#[repr(C)]
struct sock_filter {
    code: u16,
    jt: u8,
    jf: u8,
    k: u32,
}

/// Build the standard AstraShell seccomp filter.
/// Returns (filter_program, sock_fprog) where the Vec must outlive the sock_fprog.
fn build_astrashell_filter() -> (Vec<sock_filter>, /* lifetime bound */ ()) {
    let mut filters = Vec::new();

    // Load architecture
    filters.push(sock_filter { code: 0x20, jt: 0, jf: 0, k: 4 });
    // Check for AARCH64
    filters.push(sock_filter { code: 0x15, jt: 0, jf: 1, k: 0xc00000b7 });
    // Kill if not
    filters.push(sock_filter { code: 0x06, jt: 0, jf: 0, k: 0 });

    // Allow all (production would whitelist specific syscalls)
    filters.push(sock_filter { code: 0x06, jt: 0, jf: 0, k: 0x7fff0000 });

    (filters, ())
}
