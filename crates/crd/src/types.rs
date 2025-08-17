//! CRD types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
