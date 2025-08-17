//! Main configuration structure and implementation

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{GenerationConfig, PluginConfig, Source};
use jsonnet_generator::config::OutputConfig;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration version
    pub version: String,

    /// List of sources to process
    pub sources: Vec<Source>,

    /// Output configuration
    pub output: OutputConfig,

    /// Generation settings
    pub generation: GenerationConfig,

    /// Plugin configuration
    pub plugins: PluginConfig,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a YAML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.version != "1.0" {
            return Err(anyhow!(
                "Unsupported configuration version: {}",
                self.version
            ));
        }

        if self.sources.is_empty() {
            return Err(anyhow!("At least one source must be configured"));
        }

        // Validate each source
        for source in &self.sources {
            source.validate()?;
        }

        // Validate output configuration
        self.output.validate()?;

        Ok(())
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            sources: Vec::new(),
            output: OutputConfig::default(),
            generation: GenerationConfig::default(),
            plugins: PluginConfig::default(),
        }
    }
}
