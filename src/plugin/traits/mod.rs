//! Plugin traits and interfaces

pub mod processor;
pub mod generator;
pub mod validator;
pub mod lifecycle;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use processor::*;
pub use generator::*;
pub use validator::*;
pub use lifecycle::*;
