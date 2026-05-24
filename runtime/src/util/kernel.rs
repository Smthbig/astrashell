/// Kernel feature detection
use anyhow::Result;

/// Probe kernel for available features
pub struct KernelFeatures {
    pub has_user_namespaces: bool,
    pub has_pid_namespaces: bool,
    pub has_net_namespaces: bool,
    pub has_seccomp: bool,
    pub has_seccomp_notify: bool,
    pub has_overlayfs: bool,
    pub has_fuse: bool,
    pub has_cgroups_v2: bool,
    pub has_landlock: bool,
    pub has_bpf: bool,
    pub has_kvm: bool,
    pub has_avf: bool,
}

/// Detect kernel features
pub async fn detect_kernel_features() -> KernelFeatures {
    KernelFeatures {
        has_user_namespaces: has_user_namespaces().await,
        has_pid_namespaces: has_ns("pid").await,
        has_net_namespaces: has_ns("net").await,
        has_seccomp: probe_seccomp().await,
        has_seccomp_notify: probe_seccomp_notify().await,
        has_overlayfs: has_overlayfs().await,
        has_fuse: Path::new("/dev/fuse").exists(),
        has_cgroups_v2: Path::new("/sys/fs/cgroup/cgroup.controllers").exists(),
        has_landlock: probe_landlock().await,
        has_bpf: Path::new("/sys/fs/bpf").exists(),
        has_kvm: Path::new("/dev/kvm").exists(),
        has_avf: false, // detected via JNI
    }
}

pub async fn has_user_namespaces() -> bool {
    // Check via /proc/sys/kernel/unprivileged_userns_clone
    if let Ok(content) = std::fs::read_to_string("/proc/sys/kernel/unprivileged_userns_clone") {
        return content.trim() == "1";
    }
    if let Ok(content) = std::fs::read_to_string("/proc/sys/kernel/userns_restrict") {
        return content.trim() == "0";
    }
    // Try unshare directly
    unsafe {
        let ret = libc::syscall(libc::SYS_unshare, libc::CLONE_NEWUSER);
        if ret == 0 {
            return true;
        }
        let err = *libc::__errno_location();
        err != libc::EPERM && err != libc::EINVAL
    }
}

async fn has_ns(_ns_type: &str) -> bool {
    // Check if namespace type is available
    unsafe {
        let ret = libc::syscall(
            libc::SYS_unshare,
            match _ns_type {
                "pid" => libc::CLONE_NEWPID,
                "net" => libc::CLONE_NEWNET,
                "ipc" => libc::CLONE_NEWIPC,
                "uts" => libc::CLONE_NEWUTS,
                "cgroup" => libc::CLONE_NEWCGROUP,
                _ => return false,
            },
        );
        if ret == 0 {
            return true;
        }
        let err = *libc::__errno_location();
        err != libc::EPERM && err != libc::EINVAL
    }
}

async fn probe_seccomp() -> bool {
    unsafe {
        let ret = libc::syscall(
            libc::SYS_seccomp,
            libc::SECCOMP_SET_MODE_FILTER,
            0,
            std::ptr::null(),
        );
        ret < 0 && *libc::__errno_location() != libc::ENOSYS
    }
}

async fn probe_seccomp_notify() -> bool {
    unsafe {
        let ret = libc::syscall(
            libc::SYS_seccomp,
            libc::SECCOMP_SET_MODE_FILTER,
            libc::SECCOMP_FILTER_FLAG_NEW_LISTENER,
            std::ptr::null(),
        );
        ret >= 0 || *libc::__errno_location() != libc::EINVAL
    }
}

async fn has_overlayfs() -> bool {
    std::fs::read_to_string("/proc/filesystems")
        .map(|c| c.contains("overlay"))
        .unwrap_or(false)
}

async fn probe_landlock() -> bool {
    unsafe {
        let ret = libc::syscall(444, 0, 0); // landlock_create_ruleset
        ret >= 0 || *libc::__errno_location() != libc::ENOSYS
    }
}

use std::path::Path;
