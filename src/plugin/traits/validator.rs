//! Schema validator traits and interfaces

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::plugin::ExtractedSchema;

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
