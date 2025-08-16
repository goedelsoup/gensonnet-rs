//! Configuration management for JsonnetGen

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod source;
pub use source::*;

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

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Base path for generated files
    pub base_path: PathBuf,

    /// Organization strategy for output files
    pub organization: OrganizationStrategy,
}

impl OutputConfig {
    pub fn validate(&self) -> Result<()> {
        if self.base_path.to_string_lossy().is_empty() {
            return Err(anyhow!("Base path cannot be empty"));
        }
        Ok(())
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./generated"),
            organization: OrganizationStrategy::ApiVersion,
        }
    }
}

/// Organization strategy for output files
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationStrategy {
    /// Organize by API version (e.g., apps/v1/, networking.k8s.io/v1/)
    ApiVersion,

    /// Flat organization (all files in one directory)
    Flat,

    /// Hierarchical organization (nested directories)
    Hierarchical,
}

/// Generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Whether to fail fast on errors
    pub fail_fast: bool,

    /// Deep merge strategy
    pub deep_merge_strategy: MergeStrategy,
}

impl GenerationConfig {
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            deep_merge_strategy: MergeStrategy::Default,
        }
    }
}

/// Merge strategy for deep merging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    /// Default merge strategy
    Default,

    /// Replace strategy (overwrite existing values)
    Replace,

    /// Append strategy (add to existing arrays)
    Append,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin directories to scan for external plugins
    pub plugin_directories: Vec<PathBuf>,

    /// Whether to enable external plugin discovery
    pub enable_external_discovery: bool,

    /// Plugin registry URL (for remote plugin discovery)
    pub registry_url: Option<String>,

    /// Plugin cache directory
    pub cache_directory: PathBuf,

    /// Plugin validation settings
    pub validation: PluginValidationConfig,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_directories: vec![
                PathBuf::from("./plugins"),
                PathBuf::from("~/.config/gensonnet/plugins"),
            ],
            enable_external_discovery: true,
            registry_url: None,
            cache_directory: PathBuf::from("~/.cache/gensonnet/plugins"),
            validation: PluginValidationConfig::default(),
        }
    }
}

/// Plugin validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginValidationConfig {
    /// Whether to validate plugin signatures
    pub validate_signatures: bool,

    /// Whether to check plugin compatibility
    pub check_compatibility: bool,

    /// Allowed plugin sources
    pub allowed_sources: Vec<String>,

    /// Blocked plugin sources
    pub blocked_sources: Vec<String>,
}

impl Default for PluginValidationConfig {
    fn default() -> Self {
        Self {
            validate_signatures: false,
            check_compatibility: true,
            allowed_sources: vec!["local".to_string(), "registry".to_string()],
            blocked_sources: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.version, parsed.version);
    }

    #[test]
    fn test_config_from_file() {
        let mut config = Config::default();
        config
            .sources
            .push(crate::config::Source::Crd(crate::config::CrdSource {
                name: "test".to_string(),
                git: crate::config::GitSource {
                    url: "https://github.com/test/repo.git".to_string(),
                    ref_name: Some("main".to_string()),
                    auth: None,
                },
                filters: vec!["test.com/v1".to_string()],
                output_path: PathBuf::from("./output"),
            }));

        let temp_file = NamedTempFile::new().unwrap();
        config
            .save_to_file(&temp_file.path().to_path_buf())
            .unwrap();

        let loaded = Config::from_file(&temp_file.path().to_path_buf()).unwrap();
        assert_eq!(config.version, loaded.version);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        config.sources.push(Source::Crd(CrdSource {
            name: "test".to_string(),
            git: GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./output"),
        }));

        assert!(config.validate().is_ok());
    }
}
