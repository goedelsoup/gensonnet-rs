//! Plugin configuration and validation

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
