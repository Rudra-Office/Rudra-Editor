//! Server configuration — loaded from TOML file and/or environment variables.
//!
//! Precedence (highest wins): environment variables > s1.toml > defaults.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Port to listen on.
    pub port: u16,
    /// Storage backend: "local", "s3", or "memory".
    pub storage: String,
    /// Local storage directory (when storage = "local").
    pub data_dir: String,
    /// Maximum upload size in bytes.
    #[allow(dead_code)]
    pub max_upload_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            storage: "local".to_string(),
            data_dir: "./data".to_string(),
            max_upload_size: 64 * 1024 * 1024, // 64MB
        }
    }
}

impl Config {
    /// Load configuration: defaults → s1.toml → env vars (highest precedence).
    ///
    /// Environment variables always take precedence over file config so that
    /// container/staging/production deployments can override without editing files.
    pub fn load() -> Self {
        // Start with defaults
        let mut config = Self::default();

        // Layer 2: merge TOML file on top of defaults
        if let Ok(contents) = std::fs::read_to_string("s1.toml") {
            if let Ok(file_config) = toml::from_str::<Config>(&contents) {
                config = file_config;
            }
        }

        // Layer 3: env vars override everything (highest precedence)
        if let Ok(port) = std::env::var("S1_PORT") {
            if let Ok(p) = port.parse() {
                config.port = p;
            }
        }
        if let Ok(storage) = std::env::var("S1_STORAGE") {
            config.storage = storage;
        }
        if let Ok(dir) = std::env::var("S1_DATA_DIR") {
            config.data_dir = dir;
        }
        if let Ok(size) = std::env::var("S1_MAX_UPLOAD_SIZE") {
            if let Ok(s) = size.parse() {
                config.max_upload_size = s;
            }
        }
        config
    }
}
