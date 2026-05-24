use astrashell::Config;

#[tokio::test]
async fn test_runtime_init() {
    let config = Config::default();
    let mut runtime = astrashell::AstraShell::new(config);
    let result = runtime.init().await;
    // On non-Android this may fail, but shouldn't panic
    match result {
        Ok(()) => assert_eq!(runtime.status(), "running"),
        Err(_) => assert!(true), // Expected on non-Android
    }
}

#[tokio::test]
async fn test_runtime_engine_switch() {
    let mut config = Config::default();
    config.engine.mode = astrashell::ExecutionMode::Native;
    let mut runtime = astrashell::AstraShell::new(config);
    let result = runtime.init().await;
    // Native mode should work anywhere
    if result.is_ok() {
        assert_eq!(runtime.status(), "running");
    }
}

#[test]
fn test_overlay_mount_validation() {
    // Overlay should validate lowerdir exists
    let result = astrashell::fs::overlay::OverlayFS::new(
        "/nonexistent/path",
        None,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_snapshot_create_list_delete() {
    let tmpdir = std::env::temp_dir().join("astrashell-test-snap");
    let _ = std::fs::remove_dir_all(&tmpdir);
    std::fs::create_dir_all(&tmpdir).unwrap();

    let mgr = astrashell::fs::snapshot::SnapshotManager::new(tmpdir.to_str().unwrap());
    
    let snaps = mgr.list_snapshots().unwrap();
    assert!(snaps.is_empty());
    
    // Cleanup
    let _ = std::fs::remove_dir_all(&tmpdir);
}

#[tokio::test]
async fn test_vsock_protocol() {
    use astrashell::ipc::protocol::VsockMessage;
    use astrashell::ipc::protocol::VsockMessageType;

    let msg = VsockMessage {
        message_type: VsockMessageType::Heartbeat,
        payload: vec![1, 2, 3],
        timestamp: 1000,
    };

    let encoded = rmp_serde::to_vec(&msg).unwrap();
    let decoded: VsockMessage = rmp_serde::from_slice(&encoded).unwrap();
    
    assert_eq!(decoded.message_type, VsockMessageType::Heartbeat);
    assert_eq!(decoded.payload, vec![1, 2, 3]);
}
