pub mod engine;
pub mod syscall;
pub mod namespace;
pub mod container;
pub mod vm;
pub mod fs;
pub mod net;
pub mod gui;
pub mod ipc;
pub mod security;
pub mod util;
pub mod platform;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// AstraShell runtime version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Codename
pub const CODENAME: &str = "AstraShell";
/// Supported Android API level min
pub const MIN_ANDROID_API: u32 = 30;
/// Default rootfs path
pub const DEFAULT_ROOTFS: &str = "@astrashell_rootfs@";

/// AstraShell runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub runtime: RuntimeConfig,
    pub engine: EngineConfig,
    pub filesystem: FilesystemConfig,
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub gui: GuiConfig,
    pub logging: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub rootfs: String,
    pub tmpdir: String,
    pub workdir: String,
    pub hostname: String,
    pub dns: Vec<String>,
    pub env: Vec<String>,
    pub uid_map: Option<String>,
    pub gid_map: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub mode: ExecutionMode,
    pub autodetect: bool,
    pub max_containers: u32,
    pub default_timeout_ms: u64,
    pub pid1: String,
    pub init_system: InitSystem,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMode {
    /// Native fast path - LD_PRELOAD + seccomp user notif
    Native,
    /// Full namespace isolation
    Container,
    /// AVF-backed microVM
    MicroVM,
    /// Auto-detect best mode
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InitSystem {
    None,
    Systemd,
    OpenRC,
    Runit,
    S6,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    pub overlay: bool,
    pub upper_dir: Option<String>,
    pub work_dir: Option<String>,
    pub fuse: bool,
    pub bind_mounts: Vec<String>,
    pub tmpfs: Vec<String>,
    pub proc: bool,
    pub sys: bool,
    pub dev: bool,
    pub pts: bool,
    pub shm_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub enabled: bool,
    pub type_: NetworkType,
    pub port_forwards: Vec<String>,
    pub dns: Vec<String>,
    pub mtu: u32,
    pub bridge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkType {
    None,
    Host,
    Nat,
    Bridged,
    Slirp,
    Vsock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub seccomp_profile: String,
    pub capabilities: Vec<String>,
    pub no_new_privs: bool,
    pub readonly_rootfs: bool,
    pub mask_paths: Vec<String>,
    pub landlock: bool,
    pub apparmor_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub enabled: bool,
    pub display: DisplayServer,
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub vulkan: bool,
    pub opengl: bool,
    pub clipboard: bool,
    pub audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayServer {
    None,
    Wayland,
    X11,
    Web,
    AndroidSurface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub file: Option<String>,
    pub json: bool,
    pub android_log: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            runtime: RuntimeConfig {
                rootfs: DEFAULT_ROOTFS.to_string(),
                tmpdir: "/tmp".to_string(),
                workdir: "/home/astra".to_string(),
                hostname: "astrashell".to_string(),
                dns: vec!["8.8.8.8".into(), "1.1.1.1".into()],
                env: vec!["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".into()],
                uid_map: None,
                gid_map: None,
                capabilities: vec![],
            },
            engine: EngineConfig {
                mode: ExecutionMode::Auto,
                autodetect: true,
                max_containers: 4,
                default_timeout_ms: 30000,
                pid1: "/sbin/init".into(),
                init_system: InitSystem::None,
            },
            filesystem: FilesystemConfig {
                overlay: true,
                upper_dir: None,
                work_dir: None,
                fuse: false,
                bind_mounts: vec![],
                tmpfs: vec!["/tmp".into(), "/run".into()],
                proc: true,
                sys: true,
                dev: true,
                pts: true,
                shm_size: "64M".into(),
            },
            network: NetworkConfig {
                enabled: true,
                type_: NetworkType::Slirp,
                port_forwards: vec![],
                dns: vec!["8.8.8.8".into(), "1.1.1.1".into()],
                mtu: 1500,
                bridge: None,
            },
            security: SecurityConfig {
                seccomp_profile: "default.json".into(),
                capabilities: vec![],
                no_new_privs: true,
                readonly_rootfs: false,
                mask_paths: vec![],
                landlock: false,
                apparmor_profile: None,
            },
            gui: GuiConfig {
                enabled: false,
                display: DisplayServer::None,
                width: 1080,
                height: 1920,
                dpi: 320,
                vulkan: false,
                opengl: false,
                clipboard: false,
                audio: false,
            },
            logging: LogConfig {
                level: "info".into(),
                file: None,
                json: false,
                android_log: true,
            },
        }
    }
}

/// AstraShell runtime handle
pub struct AstraShell {
    config: Config,
    engine: Box<dyn engine::Engine>,
    state: RuntimeState,
}

enum RuntimeState {
    Initializing,
    Running,
    Stopped,
    Error(String),
}

impl AstraShell {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            engine: Box::new(engine::NullEngine),
            state: RuntimeState::Initializing,
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        tracing::info!("AstraShell v{} initializing", VERSION);
        self.engine = engine::detect_best(&self.config).await?;
        self.state = RuntimeState::Running;
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        self.engine.execute(&self.config).await
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn status(&self) -> &str {
        match &self.state {
            RuntimeState::Initializing => "initializing",
            RuntimeState::Running => "running",
            RuntimeState::Stopped => "stopped",
            RuntimeState::Error(e) => e,
        }
    }
}
