//! Configuration types for the generator

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
