//! CRD (CustomResourceDefinition) plugin for processing Kubernetes CRDs

pub mod factory;
pub mod plugin;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use factory::CrdPluginFactory;
pub use plugin::CrdPlugin;
