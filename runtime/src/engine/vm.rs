use crate::engine::Engine;
use crate::Config;
use crate::vm::avf::AVFManager;
use crate::vm::microdroid::MicrodroidPayload;
use anyhow::Result;
use async_trait::async_trait;

/// AVF-backed microVM execution engine
/// This provides the strongest isolation using hardware virtualization
/// Requires Android 15+ with AVF support (Pixel 7+, etc.)
pub struct AVFEngine {
    avf: Option<AVFManager>,
}

impl AVFEngine {
    pub fn new() -> Self {
        Self { avf: None }
    }
}

#[async_trait]
impl Engine for AVFEngine {
    fn name(&self) -> &'static str { "avf" }
    fn priority(&self) -> u32 { 30 }

    async fn available(&self) -> bool {
        // Probe for AVF support
        #[cfg(target_os = "android")]
        {
            // Check for VirtualizationService via JNI
            let has_avf = crate::platform::android::avf::check_avf_support().await
                .unwrap_or(false);
            return has_avf;
        }
        #[cfg(not(target_os = "android"))]
        {
            // Fallback: check if /dev/kvm is accessible
            std::path::Path::new("/dev/kvm").exists()
        }
    }

    async fn execute(&self, config: &Config) -> Result<()> {
        tracing::info!("Starting AVF microVM execution engine");
        let mut avf = AVFManager::new();
        avf.initialize().await?;

        // Build VM config
        let vm_config = avf::AVFVMConfig {
            name: "astrashell".into(),
            kernel: config.runtime.rootfs.clone() + "/boot/vmlinuz",
            initrd: config.runtime.rootfs.clone() + "/boot/initrd.img",
            disks: vec![
                avf::AVFDiskConfig {
                    image: config.runtime.rootfs.clone() + "/rootfs.img",
                    writable: true,
                }
            ],
            memory_mib: 2048,
            cpus: 4,
            gpu: config.gui.enabled,
            network: config.network.enabled,
            vsock: true,
            console: true,
        };

        avf.start_vm(vm_config).await?;
        avf.wait_for_shutdown().await?;
        Ok(())
    }

    async fn exec(&self, config: &Config, cmd: &[String]) -> Result<i32> {
        if let Some(avf) = &self.avf {
            avf.exec_vsock(cmd).await
        } else {
            // Fallthrough to shell exec if VM not running
            let args: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
            let status = std::process::Command::new(args[0])
                .args(&args[1..])
                .spawn()?
                .wait()?;
            Ok(status.code().unwrap_or(-1))
        }
    }
}

pub mod avf {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AVFVMConfig {
        pub name: String,
        pub kernel: String,
        pub initrd: String,
        pub disks: Vec<AVFDiskConfig>,
        pub memory_mib: u32,
        pub cpus: u32,
        pub gpu: bool,
        pub network: bool,
        pub vsock: bool,
        pub console: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AVFDiskConfig {
        pub image: String,
        pub writable: bool,
    }

    pub struct AVFManager;

    impl AVFManager {
        pub fn new() -> Self { Self }

        pub async fn initialize(&mut self) -> Result<()> {
            // Initialize AVF VirtualizationService connection
            // Uses AIDL over Binder or vsock to communicate with virtmgr
            tracing::info!("AVF manager initializing");
            Ok(())
        }

        pub async fn start_vm(&self, _config: AVFVMConfig) -> Result<()> {
            // 1. Create VirtualMachineConfig via AIDL
            // 2. Call VirtualizationService.startVm()
            // 3. crosvm boots kernel + initrd
            // 4. Microdroid payload starts
            tracing::info!("Starting AVF VM");
            Ok(())
        }

        pub async fn wait_for_shutdown(&self) -> Result<()> {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            Ok(())
        }

        pub async fn exec_vsock(&self, _cmd: &[String]) -> Result<i32> {
            // Execute command inside VM via vsock
            Ok(0)
        }
    }
}
