//! Generation configuration and merge strategies

use anyhow::Result;
use serde::{Deserialize, Serialize};

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
