//! Jsonnet code generation from schema sources

use crate::config::OutputConfig;
use crate::crd::CrdSchema;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub struct JsonnetGenerator {
    output_config: OutputConfig,
}

impl JsonnetGenerator {
    pub fn new(output_config: OutputConfig) -> Self {
        Self { output_config }
    }

    /// Generate Jsonnet library from CRD schemas
    pub async fn generate_crd_library(
        &self,
        schemas: &[CrdSchema],
        output_path: &Path,
    ) -> Result<SourceResult> {
        info!(
            "Generating Jsonnet library for {} CRD schemas",
            schemas.len()
        );

        // Create output directory
        std::fs::create_dir_all(output_path)?;

        let mut generated_files = Vec::new();
        let mut errors = Vec::new();

        // Group schemas by API version
        let grouped_schemas = self.group_schemas_by_version(schemas);

        for (api_version, version_schemas) in &grouped_schemas {
            match self
                .generate_version_library(api_version, version_schemas, output_path)
                .await
            {
                Ok(files) => generated_files.extend(files),
                Err(e) => {
                    let error = format!("Failed to generate library for {api_version}: {e}");
                    errors.push(error);
                }
            }
        }

        // Generate index file
        if let Err(e) = self
            .generate_index_file(&grouped_schemas, output_path)
            .await
        {
            errors.push(format!("Failed to generate index file: {e}"));
        }

        // Generate metadata file
        if let Err(e) = self.generate_metadata_file(schemas, output_path).await {
            errors.push(format!("Failed to generate metadata file: {e}"));
        }

        // Generate validation utilities
        if let Err(e) = self.generate_validation_utilities(output_path).await {
            errors.push(format!("Failed to generate validation utilities: {e}"));
        }

        Ok(SourceResult {
            source_type: "crd".to_string(),
            files_generated: generated_files.len(),
            errors,
            output_path: output_path.to_path_buf(),
            processing_time_ms: 0, // Will be set by the caller
            warnings: Vec::new(),
        })
    }

    /// Group schemas by API version
    fn group_schemas_by_version<'a>(
        &self,
        schemas: &'a [CrdSchema],
    ) -> HashMap<String, Vec<&'a CrdSchema>> {
        let mut grouped = HashMap::new();

        for schema in schemas {
            let api_version = schema.api_version.clone();
            grouped
                .entry(api_version)
                .or_insert_with(Vec::new)
                .push(schema);
        }

        grouped
    }

    /// Generate library for a specific API version
    async fn generate_version_library(
        &self,
        api_version: &str,
        schemas: &[&CrdSchema],
        output_path: &Path,
    ) -> Result<Vec<PathBuf>> {
        let version_path = match self.output_config.organization {
            crate::config::OrganizationStrategy::ApiVersion => {
                let version_dir = api_version.replace('/', "_");
                output_path.join(version_dir)
            }
            crate::config::OrganizationStrategy::Flat => output_path.to_path_buf(),
            crate::config::OrganizationStrategy::Hierarchical => {
                let parts: Vec<&str> = api_version.split('/').collect();
                if parts.len() == 2 {
                    output_path.join(parts[0]).join(parts[1])
                } else {
                    output_path.join(api_version)
                }
            }
        };

        std::fs::create_dir_all(&version_path)?;

        let mut generated_files = Vec::new();

        for schema in schemas {
            let file_path = version_path.join(format!("{}.libsonnet", schema.name.to_lowercase()));

            match self.generate_schema_file(schema, &file_path).await {
                Ok(_) => generated_files.push(file_path),
                Err(e) => {
                    warn!("Failed to generate schema file for {}: {}", schema.name, e);
                }
            }
        }

        // Generate version index file
        let index_path = version_path.join("_index.libsonnet");
        if let Err(e) = self.generate_version_index(schemas, &index_path).await {
            warn!("Failed to generate version index: {}", e);
        } else {
            generated_files.push(index_path);
        }

        Ok(generated_files)
    }

    /// Generate Jsonnet file for a single schema
    async fn generate_schema_file(&self, schema: &CrdSchema, file_path: &Path) -> Result<()> {
        let content = self.generate_schema_content(schema)?;
        std::fs::write(file_path, content)?;
        info!("Generated schema file: {:?}", file_path);
        Ok(())
    }

    /// Generate Jsonnet content for a schema
    fn generate_schema_content(&self, schema: &CrdSchema) -> Result<String> {
        let mut content = String::new();

        // Add header comment
        content.push_str(&format!("// Generated from CRD: {}\n", schema.name));
        content.push_str(&format!("// API Version: {}\n", schema.api_version));
        content.push_str(&format!("// Source: {}\n\n", schema.source_path.display()));

        // Add imports
        content.push_str("local k = import \"k.libsonnet\";\n");
        content.push_str("local validate = import \"_validation.libsonnet\";\n\n");

        // Generate the main resource function
        content.push_str(&self.generate_resource_function(schema)?);
        content.push_str("\n\n");

        // Generate validation functions
        content.push_str(&self.generate_validation_functions(schema)?);
        content.push_str("\n\n");

        // Generate field-specific functions
        content.push_str(&self.generate_field_functions(schema)?);
        content.push_str("\n\n");

        // Generate helper functions
        content.push_str(&self.generate_helper_functions(schema)?);

        Ok(content)
    }

    /// Generate the main resource function
    fn generate_resource_function(&self, schema: &CrdSchema) -> Result<String> {
        let mut content = String::new();

        let resource_name = schema.resource_name();
        let kind = schema.kind();

        content.push_str(&format!("// Create a new {kind} resource\n"));
        content.push_str(&format!(
            "function({}) {{\n",
            self.generate_function_params(schema)
        ));

        // Add validation call
        content.push_str(&format!(
            "  local validated = validate.{resource_name}(metadata, spec);\n"
        ));

        content.push_str(&format!("  apiVersion: \"{}\",\n", schema.api_version));
        content.push_str(&format!("  kind: \"{kind}\",\n"));
        content.push_str("  metadata: validated.metadata,\n");

        if self.generate_spec_object(schema)?.is_some() {
            content.push_str("  spec: validated.spec,\n");
        }

        content.push_str("}\n");

        Ok(content)
    }

    /// Generate function parameters based on schema
    fn generate_function_params(&self, schema: &CrdSchema) -> String {
        let _required_fields = schema.required_fields();
        let mut params = vec!["metadata".to_string()];

        // Add spec if it exists
        if schema.is_object() && schema.properties().is_some() {
            params.push("spec={}".to_string());
        }

        params.join(", ")
    }

    /// Generate spec object based on schema properties
    fn generate_spec_object(&self, schema: &CrdSchema) -> Result<Option<String>> {
        if !schema.is_object() {
            return Ok(None);
        }

        if let Some(properties) = schema.properties() {
            let mut spec_content = String::new();
            spec_content.push_str("{\n");

            for (field_name, field_schema) in properties {
                if let Some(field_name_str) = field_name.as_str() {
                    let _field_type = self.get_field_type(field_schema)?;
                    let default_value = self.get_field_default_value(field_schema)?;
                    spec_content.push_str(&format!("    {field_name_str}: {default_value},\n"));
                }
            }

            spec_content.push_str("  }");
            Ok(Some(spec_content))
        } else {
            Ok(None)
        }
    }

    /// Get Jsonnet type for a field
    fn get_field_type(&self, field_schema: &serde_yaml::Value) -> Result<String> {
        let field_type = field_schema
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("object");

        match field_type {
            "string" => Ok("\"\"".to_string()),
            "integer" => Ok("0".to_string()),
            "number" => Ok("0.0".to_string()),
            "boolean" => Ok("false".to_string()),
            "array" => Ok("[]".to_string()),
            "object" => Ok("{}".to_string()),
            _ => Ok("{}".to_string()),
        }
    }

    /// Get default value for a field
    fn get_field_default_value(&self, field_schema: &serde_yaml::Value) -> Result<String> {
        // Check for default value first
        if let Some(default) = field_schema.get("default") {
            return self.serialize_yaml_to_jsonnet(default);
        }

        // Fall back to type-based default
        self.get_field_type(field_schema)
    }

    /// Serialize YAML value to Jsonnet
    fn serialize_yaml_to_jsonnet(&self, value: &serde_yaml::Value) -> Result<String> {
        match value {
            serde_yaml::Value::Null => Ok("null".to_string()),
            serde_yaml::Value::Bool(b) => Ok(b.to_string()),
            serde_yaml::Value::Number(n) => Ok(n.to_string()),
            serde_yaml::Value::String(s) => Ok(format!("\"{s}\"")),
            serde_yaml::Value::Sequence(seq) => {
                let items: Vec<String> = seq
                    .iter()
                    .map(|v| self.serialize_yaml_to_jsonnet(v))
                    .collect::<Result<Vec<_>>>()?;
                Ok(format!("[{}]", items.join(", ")))
            }
            serde_yaml::Value::Mapping(map) => {
                let items: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        let key = k.as_str().unwrap_or("unknown");
                        let value = self.serialize_yaml_to_jsonnet(v)?;
                        Ok(format!("{key}: {value}"))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(format!("{{{}}}", items.join(", ")))
            }
            serde_yaml::Value::Tagged(tagged) => {
                // Handle tagged values by extracting the inner value
                self.serialize_yaml_to_jsonnet(&tagged.value)
            }
        }
    }

    /// Generate validation functions
    fn generate_validation_functions(&self, schema: &CrdSchema) -> Result<String> {
        let mut content = String::new();

        let _resource_name = schema.resource_name();

        content.push_str(&format!("// Validation function for {}\n", schema.name));
        content.push_str(&format!(
            "function validate{}(metadata, spec) {{\n",
            schema.name
        ));

        // Add metadata validation
        content.push_str("  // Validate metadata\n");
        content.push_str("  assert metadata != null : \"metadata is required\";\n");
        content.push_str("  assert metadata.name != null : \"metadata.name is required\";\n");

        // Add spec validation if it exists
        if schema.is_object() && schema.properties().is_some() {
            content.push_str("  // Validate spec\n");
            content.push_str("  local validated_spec = spec + {\n");

            for (field_name, field_schema) in schema.properties().unwrap() {
                if let Some(field_name_str) = field_name.as_str() {
                    content
                        .push_str(&self.generate_field_validation(field_name_str, field_schema)?);
                }
            }

            content.push_str("  };\n");
            content.push_str("  {\n");
            content.push_str("    metadata: metadata,\n");
            content.push_str("    spec: validated_spec,\n");
            content.push_str("  }\n");
        } else {
            content.push_str("  {\n");
            content.push_str("    metadata: metadata,\n");
            content.push_str("    spec: spec,\n");
            content.push_str("  }\n");
        }

        content.push_str("}\n");

        Ok(content)
    }

    /// Generate field validation
    fn generate_field_validation(
        &self,
        field_name: &str,
        field_schema: &serde_yaml::Value,
    ) -> Result<String> {
        let mut content = String::new();

        // Check if field is required
        if let Some(required) = field_schema.get("required").and_then(|r| r.as_bool()) {
            if required {
                content.push_str(&format!("    // {field_name} is required\n"));
                content.push_str(&format!(
                    "    assert spec.{field_name} != null : \"{field_name} is required\";\n"
                ));
            }
        }

        // Add type-specific validation
        if let Some(field_type) = field_schema.get("type").and_then(|t| t.as_str()) {
            match field_type {
                "string" => {
                    content.push_str(&self.generate_string_validation(field_name, field_schema)?);
                }
                "integer" | "number" => {
                    content.push_str(&self.generate_number_validation(field_name, field_schema)?);
                }
                "array" => {
                    content.push_str(&self.generate_array_validation(field_name, field_schema)?);
                }
                "object" => {
                    content.push_str(&self.generate_object_validation(field_name, field_schema)?);
                }
                _ => {}
            }
        }

        // Add enum validation
        if let Some(enum_values) = field_schema.get("enum").and_then(|e| e.as_sequence()) {
            content.push_str(&self.generate_enum_validation(field_name, enum_values)?);
        }

        Ok(content)
    }

    /// Generate string validation
    fn generate_string_validation(
        &self,
        field_name: &str,
        field_schema: &serde_yaml::Value,
    ) -> Result<String> {
        let mut content = String::new();

        if let Some(min_length) = field_schema.get("minLength").and_then(|v| v.as_u64()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert std.length(spec.{field_name}) >= {min_length} : \"{field_name} must be at least {min_length} characters\";\n"
            ));
        }

        if let Some(max_length) = field_schema.get("maxLength").and_then(|v| v.as_u64()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert std.length(spec.{field_name}) <= {max_length} : \"{field_name} must be at most {max_length} characters\";\n"
            ));
        }

        if let Some(pattern) = field_schema.get("pattern").and_then(|v| v.as_str()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert std.regexMatch(\"{pattern}\", spec.{field_name}) : \"{field_name} must match pattern {pattern}\";\n"
            ));
        }

        Ok(content)
    }

    /// Generate number validation
    fn generate_number_validation(
        &self,
        field_name: &str,
        field_schema: &serde_yaml::Value,
    ) -> Result<String> {
        let mut content = String::new();

        if let Some(minimum) = field_schema.get("minimum").and_then(|v| v.as_f64()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert spec.{field_name} >= {minimum} : \"{field_name} must be at least {minimum}\";\n"
            ));
        }

        if let Some(maximum) = field_schema.get("maximum").and_then(|v| v.as_f64()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert spec.{field_name} <= {maximum} : \"{field_name} must be at most {maximum}\";\n"
            ));
        }

        Ok(content)
    }

    /// Generate array validation
    fn generate_array_validation(
        &self,
        field_name: &str,
        field_schema: &serde_yaml::Value,
    ) -> Result<String> {
        let mut content = String::new();

        if let Some(min_items) = field_schema.get("minItems").and_then(|v| v.as_u64()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert std.length(spec.{field_name}) >= {min_items} : \"{field_name} must have at least {min_items} items\";\n"
            ));
        }

        if let Some(max_items) = field_schema.get("maxItems").and_then(|v| v.as_u64()) {
            content.push_str(&format!("    if spec.{field_name} != null then\n"));
            content.push_str(&format!(
                "      assert std.length(spec.{field_name}) <= {max_items} : \"{field_name} must have at most {max_items} items\";\n"
            ));
        }

        Ok(content)
    }

    /// Generate object validation
    fn generate_object_validation(
        &self,
        field_name: &str,
        _field_schema: &serde_yaml::Value,
    ) -> Result<String> {
        let mut content = String::new();

        content.push_str(&format!("    if spec.{field_name} != null then\n"));
        content.push_str(&format!(
            "      assert std.type(spec.{field_name}) == \"object\" : \"{field_name} must be an object\";\n"
        ));

        Ok(content)
    }

    /// Generate enum validation
    fn generate_enum_validation(
        &self,
        field_name: &str,
        enum_values: &serde_yaml::Sequence,
    ) -> Result<String> {
        let mut content = String::new();

        let enum_strings: Vec<String> = enum_values
            .iter()
            .filter_map(|v| v.as_str().map(|s| format!("\"{s}\"")))
            .collect();

        content.push_str(&format!("    if spec.{field_name} != null then\n"));
        content.push_str(&format!(
            "      assert std.member(spec.{}, [{}]) : \"{} must be one of [{}]\";\n",
            field_name,
            enum_strings.join(", "),
            field_name,
            enum_strings.join(", ")
        ));

        Ok(content)
    }

    /// Generate field-specific functions
    fn generate_field_functions(&self, schema: &CrdSchema) -> Result<String> {
        let mut content = String::new();

        if let Some(properties) = schema.properties() {
            for (field_name, field_schema) in properties {
                if let Some(field_name_str) = field_name.as_str() {
                    content.push_str(&self.generate_field_function(field_name_str, field_schema)?);
                    content.push_str("\n\n");
                }
            }
        }

        Ok(content)
    }

    /// Generate a field-specific function
    fn generate_field_function(
        &self,
        field_name: &str,
        _field_schema: &serde_yaml::Value,
    ) -> Result<String> {
        let mut content = String::new();

        let function_name = format!(
            "with{}",
            field_name
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .chain(field_name.chars().skip(1))
                .collect::<String>()
        );

        content.push_str(&format!("// Set the {field_name} field\n"));
        content.push_str(&format!("function({function_name}) {{\n"));
        content.push_str("  spec +: {\n");
        content.push_str(&format!("    {field_name}: {field_name},\n"));
        content.push_str("  },\n");
        content.push_str("}\n");

        Ok(content)
    }

    /// Generate helper functions
    fn generate_helper_functions(&self, schema: &CrdSchema) -> Result<String> {
        let mut content = String::new();

        // Generate factory functions for common patterns
        content.push_str("// Helper functions\n");
        content.push_str(&format!("local {} = {{\n", schema.name.to_lowercase()));
        content.push_str(&format!("  new: {},\n", schema.name.to_lowercase()));

        // Add common field setters
        if let Some(properties) = schema.properties() {
            for (field_name, _field_schema) in properties {
                if let Some(field_name_str) = field_name.as_str() {
                    let setter_name = format!(
                        "with{}",
                        field_name_str
                            .chars()
                            .next()
                            .unwrap()
                            .to_uppercase()
                            .chain(field_name_str.chars().skip(1))
                            .collect::<String>()
                    );
                    content.push_str(&format!("  {setter_name}: {setter_name},\n"));
                }
            }
        }

        content.push_str("};\n");

        Ok(content)
    }

    /// Generate validation utilities
    async fn generate_validation_utilities(&self, output_path: &Path) -> Result<()> {
        let validation_path = output_path.join("_validation.libsonnet");
        let content = r#"// Validation utilities
{
  // Common validation functions
  assertRequired: function(field, value, fieldName) {
    assert value != null : fieldName + " is required";
    value
  },
  
  assertString: function(value, fieldName) {
    assert std.type(value) == "string" : fieldName + " must be a string";
    value
  },
  
  assertNumber: function(value, fieldName) {
    assert std.type(value) == "number" : fieldName + " must be a number";
    value
  },
  
  assertBoolean: function(value, fieldName) {
    assert std.type(value) == "boolean" : fieldName + " must be a boolean";
    value
  },
  
  assertArray: function(value, fieldName) {
    assert std.type(value) == "array" : fieldName + " must be an array";
    value
  },
  
  assertObject: function(value, fieldName) {
    assert std.type(value) == "object" : fieldName + " must be an object";
    value
  },
  
  assertEnum: function(value, allowedValues, fieldName) {
    assert std.member(value, allowedValues) : fieldName + " must be one of " + std.join(", ", allowedValues);
    value
  },
  
  assertPattern: function(value, pattern, fieldName) {
    assert std.regexMatch(pattern, value) : fieldName + " must match pattern " + pattern;
    value
  },
  
  assertMinLength: function(value, minLength, fieldName) {
    assert std.length(value) >= minLength : fieldName + " must be at least " + minLength + " characters";
    value
  },
  
  assertMaxLength: function(value, maxLength, fieldName) {
    assert std.length(value) <= maxLength : fieldName + " must be at most " + maxLength + " characters";
    value
  },
  
  assertMinValue: function(value, minValue, fieldName) {
    assert value >= minValue : fieldName + " must be at least " + minValue;
    value
  },
  
  assertMaxValue: function(value, maxValue, fieldName) {
    assert value <= maxValue : fieldName + " must be at most " + maxValue;
    value
  },
}
"#;

        std::fs::write(validation_path, content)?;
        Ok(())
    }

    /// Generate version index file
    async fn generate_version_index(
        &self,
        schemas: &[&CrdSchema],
        index_path: &Path,
    ) -> Result<()> {
        let mut content = String::new();

        content.push_str("// Version index file\n");
        content.push_str("{\n");

        for schema in schemas {
            let import_name = schema.name.to_lowercase();
            content.push_str(&format!(
                "  {import_name}: import \"./{import_name}.libsonnet\",\n"
            ));
        }

        content.push_str("}\n");

        std::fs::write(index_path, content)?;
        Ok(())
    }

    /// Generate main index file
    async fn generate_index_file(
        &self,
        grouped_schemas: &HashMap<String, Vec<&CrdSchema>>,
        output_path: &Path,
    ) -> Result<()> {
        let index_path = output_path.join("index.libsonnet");
        let mut content = String::new();

        content.push_str("// Main index file\n");
        content.push_str("{\n");

        for api_version in grouped_schemas.keys() {
            let version_path = match self.output_config.organization {
                crate::config::OrganizationStrategy::ApiVersion => api_version.replace('/', "_"),
                crate::config::OrganizationStrategy::Flat => ".".to_string(),
                crate::config::OrganizationStrategy::Hierarchical => {
                    let parts: Vec<&str> = api_version.split('/').collect();
                    if parts.len() == 2 {
                        format!("{}/{}", parts[0], parts[1])
                    } else {
                        api_version.clone()
                    }
                }
            };

            content.push_str(&format!(
                "  {}: import \"./{}/_index.libsonnet\",\n",
                api_version.replace('/', "_"),
                version_path
            ));
        }

        content.push_str("}\n");

        std::fs::write(index_path, content)?;
        Ok(())
    }

    /// Generate metadata file
    async fn generate_metadata_file(
        &self,
        schemas: &[CrdSchema],
        output_path: &Path,
    ) -> Result<()> {
        let metadata_path = output_path.join("_meta.libsonnet");
        let mut content = String::new();

        content.push_str("// Generation metadata\n");
        content.push_str("{\n");
        content.push_str(&format!(
            "  generated_at: \"{}\",\n",
            chrono::Utc::now().to_rfc3339()
        ));
        content.push_str(&format!(
            "  tool_version: \"{}\",\n",
            env!("CARGO_PKG_VERSION")
        ));
        content.push_str("  schemas: [\n");

        for schema in schemas {
            content.push_str("    {\n");
            content.push_str(&format!("      name: \"{}\",\n", schema.name));
            content.push_str(&format!("      api_version: \"{}\",\n", schema.api_version));
            content.push_str(&format!(
                "      source: \"{}\",\n",
                schema.source_path.display()
            ));
            content.push_str("    },\n");
        }

        content.push_str("  ],\n");
        content.push_str("}\n");

        std::fs::write(metadata_path, content)?;
        Ok(())
    }
}

/// Result of processing a source
#[derive(Debug, Clone)]
pub struct SourceResult {
    pub source_type: String,
    pub files_generated: usize,
    pub errors: Vec<String>,
    pub output_path: PathBuf,
    pub processing_time_ms: u64,
    pub warnings: Vec<String>,
}

/// Overall generation result
#[derive(Debug)]
pub struct GenerationResult {
    pub sources_processed: usize,
    pub total_sources: usize,
    pub results: Vec<SourceResult>,
    pub statistics: crate::GenerationStatistics,
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_group_schemas_by_version() {
        let generator = JsonnetGenerator::new(OutputConfig::default());

        let schemas = vec![
            CrdSchema {
                name: "Test1".to_string(),
                group: "test.example.com".to_string(),
                version: "v1".to_string(),
                api_version: "test.example.com/v1".to_string(),
                kind: "Test1".to_string(),
                schema: serde_yaml::Value::Null,
                source_path: PathBuf::from("test1.yaml"),
                validation_rules: crate::crd::ValidationRules::default(),
                schema_analysis: crate::crd::SchemaAnalysis::default(),
            },
            CrdSchema {
                name: "Test2".to_string(),
                group: "test.example.com".to_string(),
                version: "v1".to_string(),
                api_version: "test.example.com/v1".to_string(),
                kind: "Test2".to_string(),
                schema: serde_yaml::Value::Null,
                source_path: PathBuf::from("test2.yaml"),
                validation_rules: crate::crd::ValidationRules::default(),
                schema_analysis: crate::crd::SchemaAnalysis::default(),
            },
        ];

        let grouped = generator.group_schemas_by_version(&schemas);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped.get("test.example.com/v1").unwrap().len(), 2);
    }

    #[test]
    fn test_generate_function_params() {
        let generator = JsonnetGenerator::new(OutputConfig::default());

        let schema = CrdSchema {
            name: "Test".to_string(),
            group: "test.example.com".to_string(),
            version: "v1".to_string(),
            api_version: "test.example.com/v1".to_string(),
            kind: "Test".to_string(),
            schema: serde_yaml::Value::Null,
            source_path: PathBuf::from("test.yaml"),
            validation_rules: crate::crd::ValidationRules::default(),
            schema_analysis: crate::crd::SchemaAnalysis::default(),
        };

        let params = generator.generate_function_params(&schema);
        assert_eq!(params, "metadata");
    }

    #[test]
    fn test_serialize_yaml_to_jsonnet() {
        let generator = JsonnetGenerator::new(OutputConfig::default());

        // Test string
        let yaml_str = serde_yaml::Value::String("test".to_string());
        assert_eq!(
            generator.serialize_yaml_to_jsonnet(&yaml_str).unwrap(),
            "\"test\""
        );

        // Test number
        let yaml_num = serde_yaml::Value::Number(serde_yaml::Number::from(42));
        assert_eq!(
            generator.serialize_yaml_to_jsonnet(&yaml_num).unwrap(),
            "42"
        );

        // Test boolean
        let yaml_bool = serde_yaml::Value::Bool(true);
        assert_eq!(
            generator.serialize_yaml_to_jsonnet(&yaml_bool).unwrap(),
            "true"
        );
    }
}
