//! OpenAPI (Swagger) specification processing

pub mod types;
pub mod parser;
pub mod plugin;
pub mod factory;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use types::*;
pub use parser::OpenApiParser;
pub use plugin::OpenApiPlugin;
pub use factory::OpenApiPluginFactory;
