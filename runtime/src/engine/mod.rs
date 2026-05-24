use crate::Config;
use anyhow::Result;
use async_trait::async_trait;

pub mod native;
pub mod container;
pub mod vm;

/// Execution engine trait - the core abstraction for all execution modes
#[async_trait]
pub trait Engine: Send + Sync {
    /// Name of this engine
    fn name(&self) -> &'static str;

    /// Priority (higher = preferred)
    fn priority(&self) -> u32;

    /// Check if this engine is available on current system
    async fn available(&self) -> bool;

    /// Execute a command/environment with this engine
    async fn execute(&self, config: &Config) -> Result<()>;

    /// Execute a specific command
    async fn exec(&self, config: &Config, cmd: &[String]) -> Result<i32>;
}

/// Null engine (fallback)
pub struct NullEngine;

#[async_trait]
impl Engine for NullEngine {
    fn name(&self) -> &'static str { "null" }
    fn priority(&self) -> u32 { 0 }
    async fn available(&self) -> bool { true }
    async fn execute(&self, _: &Config) -> Result<()> {
        Err(anyhow::anyhow!("No execution engine configured"))
    }
    async fn exec(&self, _: &Config, _: &[String]) -> Result<i32> {
        Err(anyhow::anyhow!("No execution engine configured"))
    }
}

/// Detect the best available engine based on system capabilities
pub async fn detect_best(config: &Config) -> Result<Box<dyn Engine>> {
    let candidates: Vec<Box<dyn Engine>> = vec![
        Box::new(vm::AVFEngine::new()),
        Box::new(container::ContainerEngine::new()),
        Box::new(native::NativeEngine::new()),
    ];

    let mode = &config.engine.mode;
    if *mode != crate::ExecutionMode::Auto {
        for e in candidates {
            if e.name() == match mode {
                crate::ExecutionMode::Native => "native",
                crate::ExecutionMode::Container => "container",
                crate::ExecutionMode::MicroVM => "avf",
                _ => "",
            } && e.available().await {
                return Ok(e);
            }
        }
        return Err(anyhow::anyhow!("Requested engine {:?} not available", mode));
    }

    let mut best: Option<Box<dyn Engine>> = None;
    let mut best_prio = 0u32;
    for e in candidates {
        if e.available().await && e.priority() > best_prio {
            best_prio = e.priority();
            best = Some(e);
        }
    }

    best.ok_or_else(|| anyhow::anyhow!("No execution engine available"))
}
