//! CRD (CustomResourceDefinition) plugin for processing Kubernetes CRDs

pub mod plugin;
pub mod factory;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use plugin::CrdPlugin;
pub use factory::CrdPluginFactory;
