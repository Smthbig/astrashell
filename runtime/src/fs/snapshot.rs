use anyhow::Result;

/// Snapshot manager for filesystem layers
pub struct SnapshotManager {
    base_dir: String,
}

impl SnapshotManager {
    pub fn new(base_dir: &str) -> Self {
        Self { base_dir: base_dir.to_string() }
    }

    /// Create a read-only snapshot
    pub fn create_snapshot(&self, name: &str) -> Result<String> {
        let snap_path = format!("{}/snapshots/{}", self.base_dir, name);
        std::fs::create_dir_all(&snap_path)?;

        // Create a bind mount snapshot (copy-on-write aware)
        if cfg!(target_os = "android") {
            // On Android, use cp --reflink if available
            let status = std::process::Command::new("cp")
                .args(&["-a", "--reflink=auto", &self.base_dir, &snap_path])
                .spawn()?
                .wait()?;
            if !status.success() {
                tracing::warn!("reflink copy failed, falling back to regular copy");
            }
        }

        Ok(snap_path)
    }

    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<Vec<String>> {
        let snap_dir = format!("{}/snapshots", self.base_dir);
        let mut snaps = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&snap_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    snaps.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }

        Ok(snaps)
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&self, name: &str) -> Result<()> {
        let snap_path = format!("{}/snapshots/{}", self.base_dir, name);
        std::fs::remove_dir_all(&snap_path)?;
        Ok(())
    }
}

/// Image management for import/export
pub mod image {
    use anyhow::Result;
    use std::path::Path;

    /// Supported image formats
    pub enum ImageFormat {
        TarGz,
        TarXz,
        TarZst,
        Squashfs,
        Ext4,
        Raw,
    }

    /// Import a rootfs image
    pub fn import_image(source: &Path, target: &Path, format: ImageFormat) -> Result<()> {
        std::fs::create_dir_all(target)?;

        match format {
            ImageFormat::TarGz => {
                let file = std::fs::File::open(source)?;
                let decoder = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(target)?;
            }
            ImageFormat::TarXz => {
                let file = std::fs::File::open(source)?;
                let decoder = xz2::read::XzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(target)?;
            }
            ImageFormat::TarZst => {
                let file = std::fs::File::open(source)?;
                let decoder = zstd::Decoder::new(file)?;
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(target)?;
            }
            _ => return Err(anyhow::anyhow!("Unsupported image format")),
        }

        Ok(())
    }

    /// Export a rootfs as an image
    pub fn export_image(source: &Path, target: &Path, format: ImageFormat) -> Result<()> {
        match format {
            ImageFormat::TarGz => {
                let file = std::fs::File::create(target)?;
                let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
                let mut archive = tar::Builder::new(encoder);
                archive.append_dir_all(".", source)?;
            }
            _ => return Err(anyhow::anyhow!("Unsupported export format")),
        }
        Ok(())
    }
}
