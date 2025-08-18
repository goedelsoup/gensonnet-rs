//! Configuration management for JsonnetGen

pub mod core;
pub mod generation;
pub mod plugins;
pub mod source;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use core::Config;
pub use generation::{GenerationConfig, MergeStrategy};
pub use plugins::{PluginConfig, PluginValidationConfig};
pub use source::*;
