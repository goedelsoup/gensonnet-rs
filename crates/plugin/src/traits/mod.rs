//! Plugin traits and interfaces

pub mod generator;
pub mod lifecycle;
pub mod processor;
pub mod validator;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use generator::*;
pub use lifecycle::*;
pub use processor::*;
pub use validator::*;
