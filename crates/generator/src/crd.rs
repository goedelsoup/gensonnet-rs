//! CRD types for the generator

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a parsed CRD schema with advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdSchema {
    /// CRD name
    pub name: String,

    /// API group
    pub group: String,

    /// API version
    pub version: String,

    /// Full API version (group/version)
    pub api_version: String,

    /// Resource kind (from spec.names.kind)
    pub kind: String,

    /// OpenAPI v3 schema
    pub schema: serde_yaml::Value,

    /// Source file path
    pub source_path: PathBuf,

    /// Extracted validation rules
    pub validation_rules: ValidationRules,

    /// Schema analysis
    pub schema_analysis: SchemaAnalysis,
}

impl CrdSchema {
    /// Get the kind name (from spec.names.kind)
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Get the resource name in plural form
    pub fn resource_name(&self) -> String {
        // This is a simplified implementation
        // In practice, you'd extract this from the CRD spec
        format!("{}s", self.name.to_lowercase())
    }

    /// Get the schema properties
    pub fn properties(&self) -> Option<&serde_yaml::Mapping> {
        self.schema.get("properties")?.as_mapping()
    }

    /// Get the schema type
    pub fn schema_type(&self) -> Option<&str> {
        self.schema.get("type")?.as_str()
    }

    /// Check if the schema is an object type
    pub fn is_object(&self) -> bool {
        self.schema_type() == Some("object")
    }

    /// Get required fields
    pub fn required_fields(&self) -> Vec<String> {
        self.validation_rules.required.clone()
    }

    /// Get validation rules for a specific field
    pub fn get_field_validation(&self, field_name: &str) -> Option<&ValidationRules> {
        self.schema_analysis
            .fields
            .get(field_name)
            .map(|field| &field.validation_rules)
    }

    /// Check if a field is required
    pub fn is_field_required(&self, field_name: &str) -> bool {
        self.required_fields().contains(&field_name.to_string())
    }

    /// Get field type information
    pub fn get_field_type(&self, field_name: &str) -> Option<&FieldAnalysis> {
        self.schema_analysis.fields.get(field_name)
    }
}

/// Validation rules extracted from OpenAPI schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationRules {
    /// Minimum length for strings
    pub min_length: Option<usize>,

    /// Maximum length for strings
    pub max_length: Option<usize>,

    /// Pattern for strings (regex)
    pub pattern: Option<String>,

    /// Minimum value for numbers
    pub minimum: Option<f64>,

    /// Maximum value for numbers
    pub maximum: Option<f64>,

    /// Exclusive minimum
    pub exclusive_minimum: Option<bool>,

    /// Exclusive maximum
    pub exclusive_maximum: Option<bool>,

    /// Multiple of value
    pub multiple_of: Option<f64>,

    /// Enum values
    pub enum_values: Vec<String>,

    /// Format (e.g., "date-time", "email", "uri")
    pub format: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Default value
    pub default_value: Option<serde_yaml::Value>,

    /// Additional properties allowed
    pub additional_properties: Option<serde_yaml::Value>,

    /// Array items schema
    pub items: Option<serde_yaml::Value>,

    /// Object properties
    pub properties: Option<serde_yaml::Value>,

    /// Required fields
    pub required: Vec<String>,
}

/// Analysis of schema structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaAnalysis {
    /// Schema type (object, array, string, etc.)
    pub schema_type: String,

    /// Field analysis for object types
    pub fields: HashMap<String, FieldAnalysis>,

    /// Array item type analysis
    pub array_item_type: Option<FieldAnalysis>,

    /// OneOf schemas
    pub one_of: Option<serde_yaml::Value>,

    /// AnyOf schemas
    pub any_of: Option<serde_yaml::Value>,

    /// AllOf schemas
    pub all_of: Option<serde_yaml::Value>,

    /// Reference to another schema
    pub reference: Option<String>,
}

/// Analysis of a field schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FieldAnalysis {
    /// Field type
    pub field_type: String,

    /// Validation rules
    pub validation_rules: ValidationRules,

    /// Nested properties for object types
    pub nested_properties: Option<serde_yaml::Value>,

    /// Array items schema
    pub array_items: Option<serde_yaml::Value>,
}
