//! Source processor traits and interfaces

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::super::ExtractedSchema;

/// Source processor trait for handling different source types
#[async_trait]
pub trait SourceProcessor: Send + Sync {
    /// Get processor name
    fn name(&self) -> &str;

    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<&str>;

    /// Check if this processor can handle the given source
    async fn can_process(&self, source_path: &Path) -> Result<bool>;

    /// Process a source and extract schemas
    async fn process_source(
        &self,
        source_path: &Path,
        context: &ProcessingContext,
    ) -> Result<ProcessingResult>;

    /// Get processor capabilities
    fn capabilities(&self) -> Vec<ProcessorCapability>;
}

/// Processing context for source processors
#[derive(Debug, Clone)]
pub struct ProcessingContext {
    /// Working directory
    pub working_dir: PathBuf,

    /// Output directory
    pub output_dir: PathBuf,

    /// Configuration
    pub config: serde_yaml::Value,

    /// Processing options
    pub options: ProcessingOptions,
}

/// Processing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingOptions {
    /// Whether to include documentation
    pub include_docs: bool,

    /// Whether to include validation
    pub include_validation: bool,

    /// Whether to generate helper functions
    pub generate_helpers: bool,

    /// Output format
    pub output_format: OutputFormat,

    /// Processing mode
    pub mode: ProcessingMode,
}

/// Output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    /// Jsonnet format
    Jsonnet,

    /// JSON Schema format
    JsonSchema,

    /// OpenAPI format
    OpenApi,

    /// Custom format
    Custom(String),
}

/// Processing mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingMode {
    /// Full processing
    Full,

    /// Incremental processing
    Incremental,

    /// Dry run
    DryRun,
}

/// Processing result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Extracted schemas
    pub schemas: Vec<ExtractedSchema>,

    /// Generated files
    pub generated_files: Vec<PathBuf>,

    /// Processing statistics
    pub statistics: ProcessingStatistics,

    /// Warnings
    pub warnings: Vec<String>,

    /// Errors
    pub errors: Vec<String>,
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStatistics {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Number of files processed
    pub files_processed: usize,

    /// Number of schemas extracted
    pub schemas_extracted: usize,

    /// Number of files generated
    pub files_generated: usize,

    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
}

/// Processor capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessorCapability {
    /// Can parse source files
    Parse,

    /// Can extract schemas
    SchemaExtraction,

    /// Can validate schemas
    Validation,

    /// Can generate code
    CodeGeneration,

    /// Can process AST
    AstProcessing,

    /// Can handle dependencies
    DependencyResolution,

    /// Can handle incremental processing
    IncrementalProcessing,

    /// Can handle parallel processing
    ParallelProcessing,
}
