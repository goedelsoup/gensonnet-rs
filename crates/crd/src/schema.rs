//! CRD schema definition and implementation

use crate::types::{FieldAnalysis, SchemaAnalysis, ValidationRules};
use serde::{Deserialize, Serialize};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crd_schema_creation() {
        let schema = CrdSchema {
            name: "TestResource".to_string(),
            group: "test.example.com".to_string(),
            version: "v1".to_string(),
            api_version: "test.example.com/v1".to_string(),
            kind: "TestResource".to_string(),
            schema: serde_yaml::Value::Null,
            source_path: PathBuf::from("test.yaml"),
            validation_rules: crate::types::ValidationRules::default(),
            schema_analysis: crate::types::SchemaAnalysis::default(),
        };

        assert_eq!(schema.kind(), "TestResource");
        assert_eq!(schema.resource_name(), "testresources");
        assert_eq!(schema.api_version, "test.example.com/v1");
    }
}
