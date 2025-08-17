//! AST (Abstract Syntax Tree) processing for Go source code
//! See: https://tree-sitter.github.io/tree-sitter/

pub mod factory;
pub mod parser;
pub mod plugin;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use factory::GoAstPluginFactory;
pub use parser::GoAstParser;
pub use plugin::GoAstPlugin;
pub use types::*;
