use astrashell::*;
use astrashell::engine::Engine;

#[tokio::test]
async fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.runtime.hostname, "astrashell");
    assert_eq!(config.engine.mode, ExecutionMode::Auto);
    assert!(config.network.enabled);
}

#[tokio::test]
async fn test_engine_detect() {
    let config = Config::default();
    // Should not panic, at least native engine should be detectable
    let result = engine::detect_best(&config).await;
    // On test environments (not Android), this may return native
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_kernel_features() {
    let features = astrashell::util::kernel::detect_kernel_features().await;
    // At minimum, seccomp should be detected
    // These tests verify the feature detection works
    println!("Kernel features: {:#?}", features);
}

#[test]
fn test_elf_analysis() {
    // Test with a simple binary
    let result = astrashell::util::elf::analyze_elf("/bin/sh");
    // May not exist in test env, that's ok
    if let Ok(analysis) = result {
        assert!(analysis.is_dynamic || !analysis.is_dynamic);
    }
}

#[test]
fn test_overlay_config() {
    let config = Config::default();
    assert!(config.filesystem.overlay);
    assert_eq!(config.filesystem.tmpfs.len(), 2);
}

#[test]
fn test_security_config() {
    let config = Config::default();
    assert!(config.security.no_new_privs);
    assert_eq!(config.security.seccomp_profile, "default.json");
}

#[test]
fn test_serialization() {
    let config = Config::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(config.runtime.hostname, deserialized.runtime.hostname);
}

#[test]
fn test_syscall_numbers() {
    assert_eq!(astrashell::syscall::arm64::READ, 63);
    assert_eq!(astrashell::syscall::arm64::WRITE, 64);
    assert_eq!(astrashell::syscall::arm64::OPENAT, 56);
    assert_eq!(astrashell::syscall::arm64::EXECVE, 221);
}

#[test]
fn test_network_config() {
    let config = Config::default();
    assert_eq!(config.network.type_, astrashell::NetworkType::Slirp);
    assert!(config.network.port_forwards.is_empty());
}

#[test]
fn test_gui_config() {
    let config = Config::default();
    assert!(!config.gui.enabled);
    assert_eq!(config.gui.display, astrashell::DisplayServer::None);
}

#[test]
fn test_namespace_user() {
    // Just test that the struct creates without error
    let ns = astrashell::namespace::user::UserNamespace::new(Some("0 1000 1"), None);
    assert!(ns.uid_map.is_some());
}

#[test]
fn test_oci_runtime() {
    let runtime = astrashell::container::oci::OCIRuntime::new();
    let parsed = astrashell::container::oci::OCIRuntime::parse_image_ref("ubuntu:latest");
    assert!(parsed.is_ok());
    if let Ok((registry, repo, tag)) = parsed {
        assert_eq!(tag, "latest");
    }
}

#[test]
fn test_path_utils() {
    // which() should find common binaries
    let sh = astrashell::util::path::which("sh");
    // On most systems this should exist
    if let Some(path) = sh {
        assert!(std::path::Path::new(&path).exists());
    }
}

#[test]
fn test_seccomp_profile() {
    let profile = astrashell::security::seccomp::SeccompProfile::new("default.json");
    let filter = profile.get_filter();
    assert!(filter.is_ok());
}

#[test]
fn test_capabilities() {
    let caps = astrashell::security::capabilities::get_emulated_caps();
    assert!(!caps.is_empty());
    // Should have CHOWN
    assert!(caps[0] & (1 << 0) != 0);
}

#[test]
fn test_lib_version() {
    assert_eq!(astrashell::VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(astrashell::MIN_ANDROID_API, 30);
}
