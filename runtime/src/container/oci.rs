use anyhow::Result;
use serde::{Deserialize, Serialize};

/// OCI Runtime interface - compatibility with Docker/Podman images
pub struct OCIRuntime;

impl OCIRuntime {
    pub fn new() -> Self { Self }

    /// Pull an OCI image from registry
    pub async fn pull_image(&self, image_ref: &str) -> Result<OCIImage> {
        tracing::info!("Pulling OCI image: {}", image_ref);
        let (_registry, _repo, _tag) = Self::parse_image_ref(image_ref)?;
        // TODO: implement actual registry client
        Err(anyhow::anyhow!("OCI pull not yet implemented"))
    }

    /// Unpack an OCI image to a rootfs
    pub async fn unpack_image(&self, image: &OCIImage, target: &str) -> Result<()> {
        tracing::info!("Unpacking OCI image to {}", target);
        std::fs::create_dir_all(target)?;

        for (i, layer) in image.layers.iter().enumerate() {
            let layer_path = format!("{}/.layers/{}", target, i);
            std::fs::create_dir_all(&layer_path)?;

            // Decompress and unpack layer
            let decoder = flate2::read::GzDecoder::new(&layer.data[..]);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&layer_path)?;

            tracing::debug!("Unpacked layer {}/{}", i + 1, image.layers.len());
        }

        // Flatten layers using overlay-style merging
        // (In production, use overlayfs with all layers as lowerdirs)
        Ok(())
    }

    /// Create an OCI container spec from config
    pub fn create_spec(&self, config: &crate::Config) -> Result<String> {
        let spec = serde_json::json!({
            "ociVersion": "1.0.2",
            "root": { "path": config.runtime.rootfs },
            "hostname": config.runtime.hostname,
            "process": {
                "args": [config.engine.pid1],
                "env": config.runtime.env,
                "cwd": config.runtime.workdir,
                "capabilities": {
                    "bounding": ["CAP_ALL"],
                    "effective": ["CAP_ALL"],
                    "permitted": ["CAP_ALL"]
                }
            },
            "linux": {
                "namespaces": [
                    {"type": "pid"},
                    {"type": "mount"},
                    {"type": "network"},
                    {"type": "uts"},
                    {"type": "ipc"}
                ]
            }
        });
        Ok(serde_json::to_string_pretty(&spec)?)
    }

    fn parse_image_ref(image_ref: &str) -> Result<(String, String, String)> {
        let parts: Vec<&str> = image_ref.splitn(2, '/').collect();
        if parts.len() == 2 {
            let registry = parts[0].to_string();
            let rest = parts[1];
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            let repo = parts[0].to_string();
            let tag = if parts.len() > 1 { parts[1].to_string() } else { "latest".into() };
            Ok((registry, repo, tag))
        } else {
            Ok(("docker.io".into(), parts[0].to_string(), "latest".into()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIImage {
    pub manifest: OCIManifest,
    pub config: OCIConfig,
    pub layers: Vec<OCILayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIManifest {
    pub schema_version: i32,
    pub media_type: String,
    pub config: OCIDescriptor,
    pub layers: Vec<OCIDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIDescriptor {
    pub media_type: String,
    pub digest: String,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIConfig {
    pub created: Option<String>,
    pub architecture: String,
    pub os: String,
    pub config: OCIConfigInner,
    pub rootfs: OCIRootFS,
    pub history: Vec<OCIHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIConfigInner {
    pub env: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIRootFS {
    #[serde(rename = "type")]
    pub type_: String,
    pub diff_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCIHistory {
    pub created: Option<String>,
    pub created_by: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OCILayer {
    pub data: Vec<u8>,
    pub digest: String,
    pub media_type: String,
}

/// Image store management
pub struct ImageStore {
    store_path: String,
}

impl ImageStore {
    pub fn new(path: &str) -> Self {
        Self { store_path: path.to_string() }
    }

    pub fn list_images(&self) -> Result<Vec<String>> {
        let mut images = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.store_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".json") {
                    images.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        Ok(images)
    }

    pub fn save_image(&self, name: &str, image: &OCIImage) -> Result<()> {
        std::fs::create_dir_all(&self.store_path)?;
        let path = format!("{}/{}.json", self.store_path, name);
        let data = serde_json::to_string_pretty(image)?;
        std::fs::write(&path, data)?;
        Ok(())
    }
}
