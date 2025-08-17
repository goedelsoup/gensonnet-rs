//! AST (Abstract Syntax Tree) processing for Go source code
//! See: https://tree-sitter.github.io/tree-sitter/

pub mod types;
pub mod parser;
pub mod plugin;
pub mod factory;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use types::*;
pub use parser::GoAstParser;
pub use plugin::GoAstPlugin;
pub use factory::GoAstPluginFactory;
