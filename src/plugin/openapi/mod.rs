//! OpenAPI (Swagger) specification processing

pub mod factory;
pub mod parser;
pub mod plugin;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use factory::OpenApiPluginFactory;
pub use parser::OpenApiParser;
pub use plugin::OpenApiPlugin;
pub use types::*;
