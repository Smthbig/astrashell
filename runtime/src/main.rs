use astra_shell::Config;
use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "astrashelld")]
#[command(version = "0.1.0")]
#[command(about = "AstraShell - Next-generation Android Linux runtime")]
struct Cli {
    /// Path to configuration file
    #[arg(short = 'c', long = "config", default_value = "/etc/astrashell/config.toml")]
    config: String,

    /// Path to rootfs
    #[arg(short = 'r', long = "rootfs")]
    rootfs: Option<String>,

    /// Execution mode
    #[arg(short = 'm', long = "mode", value_enum)]
    mode: Option<String>,

    /// Command to execute (interactive shell if empty)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("astrashell=debug".parse().unwrap()),
        )
        .with_target(true)
        .init();

    tracing::info!("AstraShell daemon v{} starting", astra_shell::VERSION);

    // Load config
    let mut config = load_config(&cli.config)?;

    // Override from CLI
    if let Some(rootfs) = &cli.rootfs {
        config.runtime.rootfs = rootfs.clone();
    }
    if let Some(mode) = &cli.mode {
        config.engine.mode = match mode.as_str() {
            "native" => astra_shell::ExecutionMode::Native,
            "container" => astra_shell::ExecutionMode::Container,
            "vm" | "avf" | "microvm" => astra_shell::ExecutionMode::MicroVM,
            _ => astra_shell::ExecutionMode::Auto,
        };
    }

    // If command specified, exec it directly
    if !cli.command.is_empty() {
        return exec_command(&config, &cli.command).await;
    }

    // Start the runtime
    let mut runtime = astra_shell::AstraShell::new(config);
    runtime.init().await?;

    // Run
    runtime.run().await?;

    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        }
        Err(_) => {
            tracing::info!("No config found at {}, using defaults", path);
            Ok(Config::default())
        }
    }
}

async fn exec_command(config: &Config, cmd: &[String]) -> Result<()> {
    let mut runtime = astra_shell::AstraShell::new(config.clone());
    runtime.init().await?;
    runtime.run().await?;
    Ok(())
}
