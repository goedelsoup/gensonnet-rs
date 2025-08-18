//! CRD parser implementation

use crate::schema::CrdSchema;
use crate::types::{FieldAnalysis, SchemaAnalysis, ValidationRules};
use anyhow::{anyhow, Result};
use glob::Pattern;
use std::path::Path; 
use tracing::{debug, info};
use walkdir::WalkDir;

pub struct CrdParser;

impl Default for CrdParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CrdParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse CRDs from a directory, applying filters
    pub fn parse_from_directory(
        &self,
        dir_path: &Path,
        filters: &[String],
    ) -> Result<Vec<CrdSchema>> {
        info!("Parsing CRDs from directory: {:?}", dir_path);

        let mut schemas = Vec::new();

        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Check if it's a YAML file
            if let Some(ext) = path.extension() {
                if ext != "yaml" && ext != "yml" {
                    continue;
                }
            } else {
                continue;
            }

            // Try to parse as CRD
            match self.parse_crd_file(path) {
                Ok(mut crd_schemas) => {
                    // Apply filters
                    crd_schemas.retain(|schema| self.matches_filters(schema, filters));
                    schemas.extend(crd_schemas);
                }
                Err(e) => {
                    debug!("Failed to parse {} as CRD: {}", path.display(), e);
                    // Continue with other files
                }
            }
        }

        info!("Found {} CRD schemas after filtering", schemas.len());
        Ok(schemas)
    }

    /// Parse a single CRD file
    fn parse_crd_file(&self, path: &Path) -> Result<Vec<CrdSchema>> {
        let content = std::fs::read_to_string(path)?;

        // Try to parse as a single document first
        let doc: serde_yaml::Value = serde_yaml::from_str(&content)?;

        let mut schemas = Vec::new();

        if let Some(crd) = self.extract_crd_from_document(&doc, path)? {
            schemas.push(crd);
        }

        Ok(schemas)
    }

    /// Extract CRD information from a YAML document
    fn extract_crd_from_document(
        &self,
        doc: &serde_yaml::Value,
        source_path: &Path,
    ) -> Result<Option<CrdSchema>> {
        // Check if this is a CRD
        if let Some(kind) = doc.get("kind").and_then(|k| k.as_str()) {
            if kind != "CustomResourceDefinition" {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        // Extract metadata
        let metadata = doc
            .get("metadata")
            .ok_or_else(|| anyhow!("CRD missing metadata"))?;

        let name = metadata
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("CRD missing name"))?;

        // Extract spec
        let spec = doc.get("spec").ok_or_else(|| anyhow!("CRD missing spec"))?;

        let group = spec
            .get("group")
            .and_then(|g| g.as_str())
            .ok_or_else(|| anyhow!("CRD missing group"))?;

        // Extract the kind from spec.names.kind
        let kind = spec
            .get("names")
            .and_then(|n| n.get("kind"))
            .and_then(|k| k.as_str())
            .unwrap_or(name); // Fallback to CRD name if kind is not specified

        let versions = spec
            .get("versions")
            .and_then(|v| v.as_sequence())
            .ok_or_else(|| anyhow!("CRD missing versions"))?;

        let mut crd_schemas = Vec::new();

        for version_doc in versions {
            let version_name = version_doc
                .get("name")
                .and_then(|n| n.as_str())
                .ok_or_else(|| anyhow!("CRD version missing name"))?;

            let schema = version_doc
                .get("schema")
                .and_then(|s| s.get("openAPIV3Schema"))
                .ok_or_else(|| anyhow!("CRD version missing openAPIV3Schema"))?;

            let crd_schema = CrdSchema {
                name: name.to_string(),
                group: group.to_string(),
                version: version_name.to_string(),
                api_version: format!("{group}/{version_name}"),
                kind: kind.to_string(),
                schema: schema.clone(),
                source_path: source_path.to_path_buf(),
                validation_rules: self.extract_validation_rules(schema)?,
                schema_analysis: self.analyze_schema(schema)?,
            };

            crd_schemas.push(crd_schema);
        }

        Ok(Some(crd_schemas.into_iter().next().unwrap()))
    }

    /// Extract validation rules from OpenAPI schema
    fn extract_validation_rules(&self, schema: &serde_yaml::Value) -> Result<ValidationRules> {
        let mut rules = ValidationRules::default();

        // Extract basic validation rules
        if let Some(min_length) = schema.get("minLength").and_then(|v| v.as_u64()) {
            rules.min_length = Some(min_length as usize);
        }

        if let Some(max_length) = schema.get("maxLength").and_then(|v| v.as_u64()) {
            rules.max_length = Some(max_length as usize);
        }

        if let Some(pattern) = schema.get("pattern").and_then(|v| v.as_str()) {
            rules.pattern = Some(pattern.to_string());
        }

        if let Some(minimum) = schema.get("minimum").and_then(|v| v.as_f64()) {
            rules.minimum = Some(minimum);
        }

        if let Some(maximum) = schema.get("maximum").and_then(|v| v.as_f64()) {
            rules.maximum = Some(maximum);
        }

        if let Some(exclusive_minimum) = schema.get("exclusiveMinimum").and_then(|v| v.as_bool()) {
            rules.exclusive_minimum = Some(exclusive_minimum);
        }

        if let Some(exclusive_maximum) = schema.get("exclusiveMaximum").and_then(|v| v.as_bool()) {
            rules.exclusive_maximum = Some(exclusive_maximum);
        }

        if let Some(multiple_of) = schema.get("multipleOf").and_then(|v| v.as_f64()) {
            rules.multiple_of = Some(multiple_of);
        }

        // Extract enum values
        if let Some(enum_values) = schema.get("enum").and_then(|v| v.as_sequence()) {
            rules.enum_values = enum_values
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }

        // Extract format
        if let Some(format) = schema.get("format").and_then(|v| v.as_str()) {
            rules.format = Some(format.to_string());
        }

        // Extract description
        if let Some(description) = schema.get("description").and_then(|v| v.as_str()) {
            rules.description = Some(description.to_string());
        }

        // Extract default value
        if let Some(default) = schema.get("default") {
            rules.default_value = Some(default.clone());
        }

        // Extract additional properties
        if let Some(additional_properties) = schema.get("additionalProperties") {
            rules.additional_properties = Some(additional_properties.clone());
        }

        // Extract items for arrays
        if let Some(items) = schema.get("items") {
            rules.items = Some(items.clone());
        }

        // Extract properties for objects
        if let Some(properties) = schema.get("properties") {
            rules.properties = Some(properties.clone());
        }

        // Extract required fields
        if let Some(required) = schema.get("required").and_then(|v| v.as_sequence()) {
            rules.required = required
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }

        Ok(rules)
    }

    /// Analyze schema structure and types
    fn analyze_schema(&self, schema: &serde_yaml::Value) -> Result<SchemaAnalysis> {
        let schema_type = schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("object")
            .to_string();
        
        let mut analysis = SchemaAnalysis {
            schema_type,
            ..Default::default()
        };

        // Analyze object properties
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_mapping()) {
            for (key, value) in properties {
                if let Some(key_str) = key.as_str() {
                    let field_analysis = self.analyze_field_schema(value)?;
                    analysis.fields.insert(key_str.to_string(), field_analysis);
                }
            }
        }

        // Analyze array items
        if let Some(items) = schema.get("items") {
            analysis.array_item_type = Some(self.analyze_field_schema(items)?);
        }

        // Check for oneOf, anyOf, allOf
        if let Some(one_of) = schema.get("oneOf").and_then(|v| v.as_sequence()) {
            analysis.one_of = Some(serde_yaml::Value::Sequence(one_of.clone()));
        }

        if let Some(any_of) = schema.get("anyOf").and_then(|v| v.as_sequence()) {
            analysis.any_of = Some(serde_yaml::Value::Sequence(any_of.clone()));
        }

        if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_sequence()) {
            analysis.all_of = Some(serde_yaml::Value::Sequence(all_of.clone()));
        }

        // Check for references
        if let Some(reference) = schema.get("$ref").and_then(|v| v.as_str()) {
            analysis.reference = Some(reference.to_string());
        }

        Ok(analysis)
    }

    /// Analyze a field schema
    fn analyze_field_schema(&self, schema: &serde_yaml::Value) -> Result<FieldAnalysis> {
        let field_type = schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("object")
            .to_string();

        let validation_rules = self.extract_validation_rules(schema)?;

        let mut analysis = FieldAnalysis {
            field_type,
            validation_rules,
            ..Default::default()
        };

        // Check for nested objects
        if let Some(properties) = schema.get("properties") {
            analysis.nested_properties = Some(properties.clone());
        }

        // Check for arrays
        if let Some(items) = schema.get("items") {
            analysis.array_items = Some(items.clone());
        }

        Ok(analysis)
    }

    /// Check if a CRD schema matches the given filters
    fn matches_filters(&self, schema: &CrdSchema, filters: &[String]) -> bool {
        if filters.is_empty() {
            return true; // No filters means accept all
        }

        for filter in filters {
            if self.matches_filter(schema, filter) {
                return true;
            }
        }

        false
    }

    /// Check if a CRD schema matches a specific filter pattern
    fn matches_filter(&self, schema: &CrdSchema, filter: &str) -> bool {
        // Convert filter to glob pattern
        let pattern = match Pattern::new(filter) {
            Ok(p) => p,
            Err(_) => return false, // Invalid pattern, skip
        };

        // Check against API version
        pattern.matches(&schema.api_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        let parser = CrdParser::new();
        let schema = CrdSchema {
            name: "TestResource".to_string(),
            group: "test.example.com".to_string(),
            version: "v1".to_string(),
            api_version: "test.example.com/v1".to_string(),
            kind: "TestResource".to_string(),
            schema: serde_yaml::Value::Null,
            source_path: std::path::PathBuf::from("test.yaml"),
            validation_rules: crate::types::ValidationRules::default(),
            schema_analysis: crate::types::SchemaAnalysis::default(),
        };

        // Test exact match
        assert!(parser.matches_filter(&schema, "test.example.com/v1"));

        // Test wildcard match
        assert!(parser.matches_filter(&schema, "test.example.com/*"));

        // Test no match
        assert!(!parser.matches_filter(&schema, "other.example.com/v1"));
    }

    #[test]
    fn test_empty_filters() {
        let parser = CrdParser::new();
        let schema = CrdSchema {
            name: "TestResource".to_string(),
            group: "test.example.com".to_string(),
            version: "v1".to_string(),
            api_version: "test.example.com/v1".to_string(),
            kind: "TestResource".to_string(),
            schema: serde_yaml::Value::Null,
            source_path: std::path::PathBuf::from("test.yaml"),
            validation_rules: crate::types::ValidationRules::default(),
            schema_analysis: crate::types::SchemaAnalysis::default(),
        };

        assert!(parser.matches_filters(&schema, &[]));
    }

    #[test]
    fn test_validation_rules_extraction() {
        let parser = CrdParser::new();
        let schema_value = serde_yaml::from_str(
            r#"
            type: string
            minLength: 1
            maxLength: 100
            pattern: "^[a-zA-Z0-9-]+$"
            description: "A test field"
            enum: ["value1", "value2", "value3"]
        "#,
        )
        .unwrap();

        let rules = parser.extract_validation_rules(&schema_value).unwrap();

        assert_eq!(rules.min_length, Some(1));
        assert_eq!(rules.max_length, Some(100));
        assert_eq!(rules.pattern, Some("^[a-zA-Z0-9-]+$".to_string()));
        assert_eq!(rules.description, Some("A test field".to_string()));
        assert_eq!(rules.enum_values, vec!["value1", "value2", "value3"]);
    }
}
