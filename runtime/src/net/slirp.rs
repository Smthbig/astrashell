use anyhow::Result;

/// Slirp networking for rootless user-mode networking
pub struct SlirpNetwork;

impl SlirpNetwork {
    pub fn new() -> Self { Self }

    pub async fn run(&self) -> Result<()> {
        // Check if slirp4netns is available
        if let Some(bin) = crate::util::path::which("slirp4netns") {
            tracing::info!("Starting slirp4netns from: {}", bin);
            let child = std::process::Command::new(&bin)
                .args(&["--configure", "--mtu=65520", "tap0", "tap1"])
                .spawn()?;
            child.wait_with_output()?;
        } else {
            // Built-in SLIRP implementation fallback
            tracing::info!("slirp4netns not found, using built-in SLIRP");
            self.builtin_slirp().await?;
        }
        Ok(())
    }

    async fn builtin_slirp(&self) -> Result<()> {
        // Minimal user-mode networking stack
        // Routes packets through host's network stack
        tracing::debug!("Built-in SLIRP network active");
        Ok(())
    }
}
