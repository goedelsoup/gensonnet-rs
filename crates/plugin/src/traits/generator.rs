//! Schema generator traits and interfaces

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::super::ExtractedSchema;
use super::processor::OutputFormat;

/// Schema generator trait for generating code from schemas
#[async_trait]
pub trait SchemaGenerator: Send + Sync {
    /// Get generator name
    fn name(&self) -> &str;

    /// Get supported output formats
    fn supported_formats(&self) -> Vec<OutputFormat>;

    /// Generate code from schemas
    async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &GenerationContext,
    ) -> Result<GenerationResult>;

    /// Get generator capabilities
    fn capabilities(&self) -> Vec<GeneratorCapability>;
}

/// Generation context
#[derive(Debug, Clone)]
pub struct GenerationContext {
    /// Output directory
    pub output_dir: PathBuf,

    /// Configuration
    pub config: serde_yaml::Value,

    /// Generation options
    pub options: GenerationOptions,
}

/// Generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// Output format
    pub format: OutputFormat,

    /// Whether to include validation
    pub include_validation: bool,

    /// Whether to include documentation
    pub include_docs: bool,

    /// Whether to generate helper functions
    pub generate_helpers: bool,

    /// Template to use
    pub template: Option<String>,

    /// Custom options
    pub custom: HashMap<String, serde_yaml::Value>,
}

/// Generation result
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Generated files
    pub files: Vec<GeneratedFile>,

    /// Generation statistics
    pub statistics: GenerationStatistics,

    /// Warnings
    pub warnings: Vec<String>,

    /// Errors
    pub errors: Vec<String>,
}

/// Generated file
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// File path
    pub path: PathBuf,

    /// File content
    pub content: String,

    /// File metadata
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Generation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStatistics {
    /// Generation time in milliseconds
    pub generation_time_ms: u64,

    /// Number of files generated
    pub files_generated: usize,

    /// Total lines of code generated
    pub lines_generated: usize,

    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
}

/// Generator capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratorCapability {
    /// Can generate Jsonnet code
    JsonnetGeneration,

    /// Can generate JSON Schema
    JsonSchemaGeneration,

    /// Can generate OpenAPI specs
    OpenApiGeneration,

    /// Can generate documentation
    DocumentationGeneration,

    /// Can generate validation code
    ValidationGeneration,

    /// Can generate helper functions
    HelperGeneration,

    /// Can use templates
    TemplateSupport,

    /// Can customize output
    Customization,
}
