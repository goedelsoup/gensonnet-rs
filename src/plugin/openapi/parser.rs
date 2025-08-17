//! OpenAPI parser implementation

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::types::*;
use crate::plugin::*;

/// OpenAPI parser
pub struct OpenApiParser {
    /// Parsed OpenAPI specifications
    specs: Vec<OpenApiSpec>,

    /// Extracted schemas
    schemas: HashMap<String, Schema>,
}

impl Default for OpenApiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenApiParser {
    /// Create a new OpenAPI parser
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            schemas: HashMap::new(),
        }
    }

    /// Parse an OpenAPI specification file
    pub async fn parse_file(&mut self, file_path: &Path) -> Result<()> {
        let content = tokio::fs::read_to_string(file_path).await?;
        self.parse_content(&content, file_path).await
    }

    /// Parse OpenAPI specification content
    pub async fn parse_content(&mut self, content: &str, file_path: &Path) -> Result<()> {
        // Try to parse as JSON first
        if let Ok(spec) = serde_json::from_str::<OpenApiSpec>(content) {
            self.process_spec(spec, file_path)?;
            return Ok(());
        }

        // Try to parse as YAML
        if let Ok(spec) = serde_yaml::from_str::<OpenApiSpec>(content) {
            self.process_spec(spec, file_path)?;
            return Ok(());
        }

        // If both fail, try to get more specific error information
        let json_error = serde_json::from_str::<OpenApiSpec>(content).unwrap_err();
        let yaml_error = serde_yaml::from_str::<OpenApiSpec>(content).unwrap_err();

        Err(anyhow::anyhow!(
            "Failed to parse OpenAPI specification. JSON error: {}, YAML error: {}",
            json_error,
            yaml_error
        ))
    }

    /// Process an OpenAPI specification
    fn process_spec(&mut self, spec: OpenApiSpec, _file_path: &Path) -> Result<()> {
        self.specs.push(spec.clone());

        // Extract schemas from definitions (v2)
        if let Some(definitions) = &spec.definitions {
            for (name, schema) in definitions {
                self.schemas.insert(name.clone(), schema.clone());
            }
        }

        // Extract schemas from components (v3)
        if let Some(components) = &spec.components {
            if let Some(schemas) = &components.schemas {
                for (name, schema) in schemas {
                    self.schemas.insert(name.clone(), schema.clone());
                }
            }
        }

        Ok(())
    }

    /// Get all parsed specifications
    pub fn get_specs(&self) -> &[OpenApiSpec] {
        &self.specs
    }

    /// Get extracted schemas
    pub fn get_schemas(&self) -> &HashMap<String, Schema> {
        &self.schemas
    }

    /// Extract schemas from OpenAPI specifications
    pub fn extract_schemas(&self) -> Vec<ExtractedSchema> {
        let mut schemas = Vec::new();

        for spec in &self.specs {
            // Extract schemas from definitions (v2)
            if let Some(definitions) = &spec.definitions {
                for (name, schema) in definitions {
                    let extracted_schema =
                        self.schema_to_extracted_schema(name, schema, &spec.info);
                    schemas.push(extracted_schema);
                }
            }

            // Extract schemas from components (v3)
            if let Some(components) = &spec.components {
                if let Some(schemas_map) = &components.schemas {
                    for (name, schema) in schemas_map {
                        let extracted_schema =
                            self.schema_to_extracted_schema(name, schema, &spec.info);
                        schemas.push(extracted_schema);
                    }
                }
            }
        }

        schemas
    }

    /// Convert OpenAPI schema to extracted schema
    fn schema_to_extracted_schema(
        &self,
        name: &str,
        schema: &Schema,
        info: &ApiInfo,
    ) -> ExtractedSchema {
        let mut metadata = HashMap::new();
        metadata.insert(
            "api_title".to_string(),
            serde_yaml::Value::String(info.title.clone()),
        );
        metadata.insert(
            "api_version".to_string(),
            serde_yaml::Value::String(info.version.clone()),
        );
        if let Some(description) = &info.description {
            metadata.insert(
                "api_description".to_string(),
                serde_yaml::Value::String(description.clone()),
            );
        }

        let schema_content = self.schema_to_yaml(schema);

        ExtractedSchema {
            name: name.to_string(),
            schema_type: "openapi_schema".to_string(),
            content: schema_content,
            source_file: PathBuf::new(), // Will be set by caller
            metadata,
        }
    }

    /// Convert OpenAPI schema to YAML
    fn schema_to_yaml(&self, schema: &Schema) -> serde_yaml::Value {
        let mut yaml = serde_yaml::Mapping::new();

        if let Some(schema_type) = &schema.r#type {
            yaml.insert(
                serde_yaml::Value::String("type".to_string()),
                serde_yaml::Value::String(schema_type.clone()),
            );
        }

        if let Some(format) = &schema.format {
            yaml.insert(
                serde_yaml::Value::String("format".to_string()),
                serde_yaml::Value::String(format.clone()),
            );
        }

        if let Some(description) = &schema.description {
            yaml.insert(
                serde_yaml::Value::String("description".to_string()),
                serde_yaml::Value::String(description.clone()),
            );
        }

        if let Some(properties) = &schema.properties {
            let mut props_yaml = serde_yaml::Mapping::new();
            for (prop_name, prop_schema) in properties {
                props_yaml.insert(
                    serde_yaml::Value::String(prop_name.clone()),
                    self.schema_to_yaml(prop_schema),
                );
            }
            yaml.insert(
                serde_yaml::Value::String("properties".to_string()),
                serde_yaml::Value::Mapping(props_yaml),
            );
        }

        if let Some(required) = &schema.required {
            yaml.insert(
                serde_yaml::Value::String("required".to_string()),
                serde_yaml::Value::Sequence(
                    required
                        .iter()
                        .map(|r| serde_yaml::Value::String(r.clone()))
                        .collect(),
                ),
            );
        }

        if let Some(items) = &schema.items {
            yaml.insert(
                serde_yaml::Value::String("items".to_string()),
                self.schema_to_yaml(items),
            );
        }

        if let Some(enum_values) = &schema.r#enum {
            let enum_yaml: Vec<serde_yaml::Value> = enum_values
                .iter()
                .map(|v| serde_yaml::to_value(v).unwrap_or(serde_yaml::Value::Null))
                .collect();
            yaml.insert(
                serde_yaml::Value::String("enum".to_string()),
                serde_yaml::Value::Sequence(enum_yaml),
            );
        }

        if let Some(example) = &schema.example {
            yaml.insert(
                serde_yaml::Value::String("example".to_string()),
                serde_yaml::to_value(example).unwrap_or(serde_yaml::Value::Null),
            );
        }

        if let Some(default) = &schema.default {
            yaml.insert(
                serde_yaml::Value::String("default".to_string()),
                serde_yaml::to_value(default).unwrap_or(serde_yaml::Value::Null),
            );
        }

        serde_yaml::Value::Mapping(yaml)
    }
}
