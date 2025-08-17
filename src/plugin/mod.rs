//! Plugin architecture for extensible source processing
//!
//! This module contains the plugin registry and testing utilities.
//! The core plugin infrastructure is now in the `gensonnet-plugin` crate.

// Temporary plugin implementations (will be moved to dynamic loading)
pub mod ast;
pub mod crd;
pub mod openapi;

pub mod registry;
pub mod testing;

pub use registry::*;
pub use testing::*;

// Re-export common types from the plugin crate
pub use gensonnet_plugin::*;
