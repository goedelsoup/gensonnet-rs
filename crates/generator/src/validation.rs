//! Validation code generation for Jsonnet

use crate::crd::CrdSchema;
use anyhow::Result;
use std::path::Path;

pub struct ValidationGenerator;

impl ValidationGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate validation functions for a schema
    pub fn generate_validation_functions(&self, schema: &CrdSchema) -> Result<String> {
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

    /// Generate validation utilities
    pub async fn generate_validation_utilities(&self, output_path: &Path) -> Result<()> {
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
}
