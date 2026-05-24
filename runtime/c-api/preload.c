/// LD_PRELOAD shim library for glibc compatibility on bionic
///
/// This library is LD_PRELOADED into all guest processes.
/// It intercepts libc calls and translates between:
/// - bionic (Android's libc) and glibc calling conventions
/// - Android's restricted syscall environment and full Linux
///
/// Key intercepts:
/// - fork/vfork -> use bionic-compatible clone
/// - execve -> ELF loader compatibility
/// - dynamic linker path manipulation
/// - /proc and /sys virtual filesystem redirects
/// - mount/umount -> broker to syscall daemon

use std::ffi::{CStr, CString};

/// Intercept fork() - use clone() with correct flags for bionic
#[no_mangle]
pub unsafe extern "C" fn fork() -> libc::pid_t {
    libc::syscall(libc::SYS_clone, libc::SIGCHLD) as libc::pid_t
}

/// Intercept open()  - translate paths for virtual filesystem
#[no_mangle]
pub unsafe extern "C" fn open(path: *const libc::c_char, flags: libc::c_int, ...) -> libc::c_int {
    let path_str = CStr::from_ptr(path).to_string_lossy().into_owned();

    // Redirect /proc/self/... to our virtual proc
    let translated = translate_path(&path_str);
    let cpath = CString::new(translated).unwrap();

    // Call real open via syscall directly
    libc::syscall(libc::SYS_openat, libc::AT_FDCWD, cpath.as_ptr(), flags) as libc::c_int
}

/// Intercept execve() - add LD_PRELOAD and handle ABI
#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const libc::c_char,
    argv: *const *const libc::c_char,
    envp: *const *const libc::c_char,
) -> libc::c_int {
    // Ensure our preload is in the new process's environment
    let envp = augment_env(envp);
    libc::syscall(libc::SYS_execve, path, argv, envp) as libc::c_int
}

/// Intercept uname() - return Linux hostname instead of Android
#[no_mangle]
pub unsafe extern "C" fn uname(buf: *mut libc::utsname) -> libc::c_int {
    let ret = libc::syscall(libc::SYS_uname, buf);
    if ret >= 0 {
        // Override release string to look like real Linux
        let nodename = &mut (*buf).nodename;
        if let Ok(hostname) = std::env::var("ASTRASHELL_HOSTNAME") {
            let bytes = hostname.as_bytes();
            let len = bytes.len().min(65);
            nodename[..len].copy_from_slice(bytes);
        }

        // Override machine to aarch64
        let machine = b"aarch64\0";
        (*buf).machine[..8].copy_from_slice(machine);
    }
    ret as libc::c_int
}

/// Path translation for virtual filesystem
fn translate_path(path: &str) -> String {
    let root = std::env::var("ASTRASHELL_ROOT").unwrap_or_default();

    if path.starts_with("/proc/") {
        // Redirect to virtual proc
        format!("{}/proc/{}", root, &path[6..])
    } else if path.starts_with("/sys/") {
        format!("{}/sys/{}", root, &path[5..])
    } else if path.starts_with("/dev/") {
        format!("{}/dev/{}", root, &path[5..])
    } else if path.starts_with("/etc/") {
        format!("{}/etc/{}", root, &path[5..])
    } else {
        path.to_string()
    }
}

/// Augment environment with our preload
unsafe fn augment_env(envp: *const *const libc::c_char) -> *const *const libc::c_char {
    // In production: walk envp, add LD_PRELOAD if not present
    envp
}

/// ELF loader entry point - invoked by the custom dynamic linker
#[no_mangle]
pub unsafe extern "C" fn __astrashell_entry(
    entry: usize,
    args: *const *const libc::c_char,
    envp: *const *const libc::c_char,
) -> ! {
    // Call the real entry point
    let entry_fn: extern "C" fn(*const *const libc::c_char, *const *const libc::c_char) -> ! =
        std::mem::transmute(entry);
    entry_fn(args, envp);
}
