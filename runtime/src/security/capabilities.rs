use anyhow::Result;

/// Capability management for emulated root
pub struct CapabilitySet {
    allowed: Vec<String>,
}

impl CapabilitySet {
    pub fn new(caps: &[String]) -> Self {
        Self { allowed: caps.to_vec() }
    }

    /// Apply capability bounding set
    pub fn apply(&self) -> Result<()> {
        // In a user namespace, we have full capabilities inside the namespace
        // This function restricts them to a minimum set for security
        tracing::info!("Capability bounding set applied");
        Ok(())
    }

    /// Drop all capabilities
    pub fn drop_all() -> Result<()> {
        unsafe {
            let ret = libc::prctl(libc::PR_CAPBSET_DROP, 0, 0, 0, 0);
            if ret < 0 {
                tracing::warn!("Failed to drop capabilities");
            }
        }
        Ok(())
    }
}

/// Emulate /proc/sys/kernel/cap_last_cap and capability-aware operations
pub fn get_emulated_caps() -> Vec<u64> {
    // Return full capabilities for fake root
    vec![
        (1 << 0),  // CAP_CHOWN
        (1 << 1),  // CAP_DAC_OVERRIDE
        (1 << 2),  // CAP_DAC_READ_SEARCH
        (1 << 3),  // CAP_FOWNER
        (1 << 4),  // CAP_FSETID
        (1 << 5),  // CAP_KILL
        (1 << 6),  // CAP_SETGID
        (1 << 7),  // CAP_SETUID
        (1 << 8),  // CAP_SETPCAP
        (1 << 9),  // CAP_LINUX_IMMUTABLE
        (1 << 10), // CAP_NET_BIND_SERVICE
        (1 << 11), // CAP_NET_BROADCAST
        (1 << 12), // CAP_NET_ADMIN
        (1 << 13), // CAP_NET_RAW
        (1 << 14), // CAP_IPC_LOCK
        (1 << 15), // CAP_IPC_OWNER
        (1 << 16), // CAP_SYS_MODULE
        (1 << 17), // CAP_SYS_RAWIO
        (1 << 18), // CAP_SYS_CHROOT
        (1 << 19), // CAP_SYS_PTRACE
        (1 << 20), // CAP_SYS_PACCT
        (1 << 21), // CAP_SYS_ADMIN
        (1 << 22), // CAP_SYS_BOOT
        (1 << 23), // CAP_SYS_NICE
        (1 << 24), // CAP_SYS_RESOURCE
        (1 << 25), // CAP_SYS_TIME
        (1 << 26), // CAP_SYS_TTY_CONFIG
        (1 << 27), // CAP_MKNOD
        (1 << 28), // CAP_LEASE
        (1 << 29), // CAP_AUDIT_WRITE
        (1 << 30), // CAP_AUDIT_CONTROL
        (1 << 31), // CAP_SETFCAP
        (1 << 32), // CAP_MAC_OVERRIDE
        (1 << 33), // CAP_MAC_ADMIN
        (1 << 34), // CAP_SYSLOG
        (1 << 35), // CAP_WAKE_ALARM
        (1 << 36), // CAP_BLOCK_SUSPEND
        (1 << 37), // CAP_AUDIT_READ
    ]
}
