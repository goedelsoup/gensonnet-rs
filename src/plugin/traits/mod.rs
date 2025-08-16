//! Plugin traits and interfaces

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::plugin::*;

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

/// Schema validator trait for validating schemas
#[async_trait]
pub trait SchemaValidator: Send + Sync {
    /// Get validator name
    fn name(&self) -> &str;

    /// Get supported schema types
    fn supported_schema_types(&self) -> Vec<&str>;

    /// Validate a schema
    async fn validate_schema(
        &self,
        schema: &ExtractedSchema,
        context: &ValidationContext,
    ) -> Result<ValidationResult>;

    /// Get validator capabilities
    fn capabilities(&self) -> Vec<ValidatorCapability>;
}

/// Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Validation options
    pub options: ValidationOptions,

    /// Schema registry
    pub schema_registry: Option<SchemaRegistry>,
}

/// Validation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOptions {
    /// Whether to perform strict validation
    pub strict: bool,

    /// Whether to allow unknown fields
    pub allow_unknown_fields: bool,

    /// Whether to validate references
    pub validate_references: bool,

    /// Custom validation rules
    pub custom_rules: HashMap<String, serde_yaml::Value>,
}

/// Schema registry for managing schemas
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    /// Registered schemas
    pub schemas: HashMap<String, ExtractedSchema>,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,

    /// Validation statistics
    pub statistics: ValidationStatistics,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Error location
    pub location: Option<String>,

    /// Error code
    pub code: Option<String>,

    /// Error severity
    pub severity: ErrorSeverity,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Warning location
    pub location: Option<String>,

    /// Warning code
    pub code: Option<String>,
}

/// Error severity
#[derive(Debug, Clone)]
pub enum ErrorSeverity {
    /// Error
    Error,

    /// Warning
    Warning,

    /// Info
    Info,
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatistics {
    /// Validation time in milliseconds
    pub validation_time_ms: u64,

    /// Number of schemas validated
    pub schemas_validated: usize,

    /// Number of errors found
    pub errors_found: usize,

    /// Number of warnings found
    pub warnings_found: usize,
}

/// Validator capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorCapability {
    /// Can validate OpenAPI schemas
    OpenApiValidation,

    /// Can validate JSON Schema
    JsonSchemaValidation,

    /// Can validate custom schemas
    CustomValidation,

    /// Can validate references
    ReferenceValidation,

    /// Can provide detailed error messages
    DetailedErrors,

    /// Can provide suggestions
    Suggestions,
}

/// Plugin lifecycle manager trait
#[async_trait]
pub trait PluginLifecycleManager: Send + Sync {
    /// Initialize plugins
    async fn initialize_plugins(&self) -> Result<()>;

    /// Start plugins
    async fn start_plugins(&self) -> Result<()>;

    /// Stop plugins
    async fn stop_plugins(&self) -> Result<()>;

    /// Reload plugins
    async fn reload_plugins(&self) -> Result<()>;

    /// Get plugin status
    async fn get_plugin_status(&self) -> Result<Vec<PluginStatus>>;
}

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    /// Plugin ID
    pub plugin_id: String,

    /// Plugin name
    pub name: String,

    /// Plugin status
    pub status: String,

    /// Plugin version
    pub version: String,

    /// Last activity
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,

    /// Error message (if any)
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_options() {
        let options = ProcessingOptions {
            include_docs: true,
            include_validation: true,
            generate_helpers: false,
            output_format: OutputFormat::Jsonnet,
            mode: ProcessingMode::Full,
        };

        assert!(options.include_docs);
        assert!(options.include_validation);
        assert!(!options.generate_helpers);
    }

    #[test]
    fn test_generation_options() {
        let mut options = GenerationOptions {
            format: OutputFormat::Jsonnet,
            include_validation: true,
            include_docs: true,
            generate_helpers: true,
            template: None,
            custom: HashMap::new(),
        };

        options.custom.insert(
            "test_key".to_string(),
            serde_yaml::Value::String("test_value".to_string()),
        );

        assert_eq!(options.format, OutputFormat::Jsonnet);
        assert!(options.include_validation);
        assert!(options.custom.contains_key("test_key"));
    }

    #[test]
    fn test_validation_options() {
        let options = ValidationOptions {
            strict: true,
            allow_unknown_fields: false,
            validate_references: true,
            custom_rules: HashMap::new(),
        };

        assert!(options.strict);
        assert!(!options.allow_unknown_fields);
        assert!(options.validate_references);
    }
}
