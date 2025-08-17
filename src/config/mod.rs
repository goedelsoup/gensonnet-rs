//! Configuration management for JsonnetGen

pub mod config;
pub mod generation;
pub mod plugins;
pub mod source;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use config::Config;
pub use generation::{GenerationConfig, MergeStrategy};
pub use plugins::{PluginConfig, PluginValidationConfig};
pub use source::*;
