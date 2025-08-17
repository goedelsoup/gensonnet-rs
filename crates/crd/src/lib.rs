//! CRD (CustomResourceDefinition) parsing and schema extraction

pub mod parser;
pub mod schema;
pub mod types;

pub use parser::CrdParser;
pub use schema::CrdSchema;
pub use types::{FieldAnalysis, SchemaAnalysis, ValidationRules};
