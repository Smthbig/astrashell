pub mod avf;

/// Android platform-specific code
pub mod android {
    /// Get app's data directory
    pub fn app_data_dir() -> Option<String> {
        std::env::var("ASTRASHELL_DATA_DIR").ok()
    }

    /// Detect if running on Android
    pub fn is_android() -> bool {
        std::env::var("ANDROID_ROOT").is_ok() ||
        std::env::var("ANDROID_DATA").is_ok()
    }

    /// Get Android API level
    pub fn api_level() -> i32 {
        std::env::var("ASTRASHELL_API_LEVEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    /// Get SELinux context
    pub fn selinux_context() -> Option<String> {
        std::fs::read_to_string("/proc/self/attr/current").ok()
    }

    /// Check if we're in a shell (app) context vs native
    pub fn is_shell_context() -> bool {
        std::env::var("ASTRASHELL_SHELL").is_ok()
    }
}
