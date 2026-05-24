pub mod intercept;
pub mod translate;
pub mod broker;

/// Syscall numbers for ARM64 Linux
pub mod arm64 {
    pub const READ: i64 = 63;
    pub const WRITE: i64 = 64;
    pub const OPENAT: i64 = 56;
    pub const CLOSE: i64 = 57;
    pub const FSTAT: i64 = 80;
    pub const STATX: i64 = 291;
    pub const MMAP: i64 = 222;
    pub const MPROTECT: i64 = 226;
    pub const MUNMAP: i64 = 215;
    pub const BRK: i64 = 214;
    pub const CLONE: i64 = 220;
    pub const FORK: i64 = 1079;  // clone(CLONE_CHILD_SETTID|SIGCHLD, 0)
    pub const VFORK: i64 = 1071; // clone(CLONE_VFORK|SIGCHLD, 0)
    pub const EXECVE: i64 = 221;
    pub const EXIT: i64 = 93;
    pub const EXIT_GROUP: i64 = 94;
    pub const GETDENTS64: i64 = 61;
    pub const MOUNT: i64 = 40;
    pub const UMOUNT2: i64 = 39;
    pub const PIVOT_ROOT: i64 = 41;
    pub const CHROOT: i64 = 51;
    pub const SETNS: i64 = 268;
    pub const UNAME: i64 = 160;
    pub const SETHOSTNAME: i64 = 161;
    pub const SETDOMAINNAME: i64 = 162;
    pub const REBOOT: i64 = 142;
    pub const CAPGET: i64 = 90;
    pub const CAPSET: i64 = 91;
}
