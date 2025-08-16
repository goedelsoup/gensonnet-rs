//! OpenAPI (Swagger) specification processing

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::plugin::*;

/// OpenAPI specification version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenApiVersion {
    /// OpenAPI 2.0 (Swagger)
    V2,
    /// OpenAPI 3.0
    V3,
    /// OpenAPI 3.1
    V31,
}

/// OpenAPI specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// OpenAPI version
    #[serde(rename = "openapi")]
    pub version: Option<String>,
    
    /// Swagger version (v2)
    #[serde(rename = "swagger")]
    pub swagger_version: Option<String>,
    
    /// API information
    pub info: ApiInfo,
    
    /// Base path (v2)
    #[serde(rename = "basePath")]
    pub base_path: Option<String>,
    
    /// Servers (v3)
    pub servers: Option<Vec<Server>>,
    
    /// Paths/endpoints
    pub paths: HashMap<String, PathItem>,
    
    /// Definitions/schemas (v2)
    pub definitions: Option<HashMap<String, Schema>>,
    
    /// Components (v3)
    pub components: Option<Components>,
}

/// API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// API title
    pub title: String,
    
    /// API version
    pub version: String,
    
    /// API description
    pub description: Option<String>,
    
    /// Contact information
    pub contact: Option<Contact>,
    
    /// License information
    pub license: Option<License>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Contact name
    pub name: Option<String>,
    
    /// Contact email
    pub email: Option<String>,
    
    /// Contact URL
    pub url: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// License name
    pub name: String,
    
    /// License URL
    pub url: Option<String>,
}

/// Server information (v3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server URL
    pub url: String,
    
    /// Server description
    pub description: Option<String>,
}

/// Path item (endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    /// HTTP methods
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
    
    /// Parameters
    pub parameters: Option<Vec<Parameter>>,
}

/// API operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation summary
    pub summary: Option<String>,
    
    /// Operation description
    pub description: Option<String>,
    
    /// Operation tags
    pub tags: Option<Vec<String>>,
    
    /// Operation parameters
    pub parameters: Option<Vec<Parameter>>,
    
    /// Request body
    pub request_body: Option<RequestBody>,
    
    /// Responses
    pub responses: HashMap<String, Response>,
    
    /// Operation ID
    pub operation_id: Option<String>,
}

/// Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter location
    pub r#in: String,
    
    /// Parameter description
    pub description: Option<String>,
    
    /// Whether parameter is required
    pub required: Option<bool>,
    
    /// Parameter schema
    pub schema: Option<Schema>,
    
    /// Parameter type (v2)
    pub r#type: Option<String>,
    
    /// Parameter format (v2)
    pub format: Option<String>,
}

/// Request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Request body description
    pub description: Option<String>,
    
    /// Request body content
    pub content: HashMap<String, MediaType>,
    
    /// Whether request body is required
    pub required: Option<bool>,
}

/// Media type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    /// Media type schema
    pub schema: Option<Schema>,
    
    /// Media type examples
    pub examples: Option<HashMap<String, Example>>,
}

/// Example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example summary
    pub summary: Option<String>,
    
    /// Example description
    pub description: Option<String>,
    
    /// Example value
    pub value: Option<serde_json::Value>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Response description
    pub description: String,
    
    /// Response content
    pub content: Option<HashMap<String, MediaType>>,
    
    /// Response headers
    pub headers: Option<HashMap<String, Header>>,
}

/// Header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// Header description
    pub description: Option<String>,
    
    /// Header schema
    pub schema: Option<Schema>,
}

/// Components (v3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Component schemas
    pub schemas: Option<HashMap<String, Schema>>,
    
    /// Component responses
    pub responses: Option<HashMap<String, Response>>,
    
    /// Component parameters
    pub parameters: Option<HashMap<String, Parameter>>,
    
    /// Component examples
    pub examples: Option<HashMap<String, Example>>,
}

/// Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema type
    pub r#type: Option<String>,
    
    /// Schema format
    pub format: Option<String>,
    
    /// Schema description
    pub description: Option<String>,
    
    /// Schema properties
    pub properties: Option<HashMap<String, Schema>>,
    
    /// Required properties
    pub required: Option<Vec<String>>,
    
    /// Schema items (for arrays)
    pub items: Option<Box<Schema>>,
    
    /// Schema reference
    pub r#ref: Option<String>,
    
    /// Schema allOf
    pub all_of: Option<Vec<Schema>>,
    
    /// Schema anyOf
    pub any_of: Option<Vec<Schema>>,
    
    /// Schema oneOf
    pub one_of: Option<Vec<Schema>>,
    
    /// Schema not
    pub not: Option<Box<Schema>>,
    
    /// Schema additional properties
    pub additional_properties: Option<Box<Schema>>,
    
    /// Schema enum values
    pub r#enum: Option<Vec<serde_json::Value>>,
    
    /// Schema example
    pub example: Option<serde_json::Value>,
    
    /// Schema examples
    pub examples: Option<HashMap<String, Example>>,
    
    /// Schema default value
    pub default: Option<serde_json::Value>,
    
    /// Schema minimum value
    pub minimum: Option<f64>,
    
    /// Schema maximum value
    pub maximum: Option<f64>,
    
    /// Schema min length
    pub min_length: Option<u64>,
    
    /// Schema max length
    pub max_length: Option<u64>,
    
    /// Schema pattern
    pub pattern: Option<String>,
}

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
                    let extracted_schema = self.schema_to_extracted_schema(name, schema, &spec.info);
                    schemas.push(extracted_schema);
                }
            }

            // Extract schemas from components (v3)
            if let Some(components) = &spec.components {
                if let Some(schemas_map) = &components.schemas {
                    for (name, schema) in schemas_map {
                        let extracted_schema = self.schema_to_extracted_schema(name, schema, &spec.info);
                        schemas.push(extracted_schema);
                    }
                }
            }
        }

        schemas
    }

    /// Convert OpenAPI schema to extracted schema
    fn schema_to_extracted_schema(&self, name: &str, schema: &Schema, info: &ApiInfo) -> ExtractedSchema {
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

/// OpenAPI plugin
pub struct OpenApiPlugin {
    /// Parser instance
    parser: OpenApiParser,
    
    /// Plugin configuration
    config: PluginConfig,
}

impl OpenApiPlugin {
    /// Create a new OpenAPI plugin
    pub fn new(config: PluginConfig) -> Self {
        Self {
            parser: OpenApiParser::new(),
            config,
        }
    }
}

#[async_trait]
impl Plugin for OpenApiPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: self.config.plugin_id.clone(),
            name: "OpenAPI Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin for processing OpenAPI/Swagger specifications and extracting type information"
                .to_string(),
            supported_types: vec!["openapi".to_string(), "swagger".to_string(), "yaml".to_string(), "json".to_string()],
            capabilities: vec![
                PluginCapability::Parse,
                PluginCapability::SchemaExtraction,
                PluginCapability::Validation,
            ],
        }
    }

    async fn initialize(&self, _context: &PluginContext) -> Result<()> {
        // Initialize the parser
        Ok(())
    }

    async fn can_handle(&self, source_path: &Path) -> Result<bool> {
        // Check if it's an OpenAPI file
        if let Some(extension) = source_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            Ok(ext == "yaml" || ext == "yml" || ext == "json")
        } else {
            Ok(false)
        }
    }

    async fn process_source(
        &self,
        source_path: &Path,
        _context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        // Parse the OpenAPI specification file
        let mut parser = OpenApiParser::new();
        parser.parse_file(source_path).await?;

        // Extract schemas
        let schemas = parser.extract_schemas();

        let processing_time = start_time.elapsed();

        let schemas_count = schemas.len();
        Ok(PluginResult {
            schemas,
            generated_files: Vec::new(),
            statistics: PluginStatistics {
                processing_time_ms: processing_time.as_millis() as u64,
                files_processed: 1,
                schemas_extracted: schemas_count,
                files_generated: 0,
            },
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }

    async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &PluginContext,
    ) -> Result<Vec<PathBuf>> {
        let mut generated_files = Vec::new();

        for schema in schemas {
            let output_file = context
                .output_dir
                .join(format!("{}.libsonnet", schema.name.to_lowercase()));

            // Generate Jsonnet code from the schema
            let jsonnet_code = self.generate_jsonnet_code(schema)?;
            tokio::fs::write(&output_file, jsonnet_code).await?;

            generated_files.push(output_file);
        }

        Ok(generated_files)
    }

    async fn cleanup(&self, _context: &PluginContext) -> Result<()> {
        // Clean up any resources
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(OpenApiPlugin {
            parser: OpenApiParser::new(),
            config: self.config.clone(),
        })
    }
}

impl OpenApiPlugin {
    /// Generate Jsonnet code from schema
    fn generate_jsonnet_code(&self, schema: &ExtractedSchema) -> Result<String> {
        let mut code = String::new();

        code.push_str(&format!("// Generated from OpenAPI: {}\n", schema.name));
        code.push_str(&format!("// Source: {}\n\n", schema.source_file.display()));

        // Add imports
        code.push_str("local k = import \"k.libsonnet\";\n");
        code.push_str("local validate = import \"_validation.libsonnet\";\n\n");

        // Generate the main function
        code.push_str(&format!("// Create a new {} resource\n", schema.name));
        code.push_str("function(metadata, spec={}) {\n");
        code.push_str(&format!(
            "  apiVersion: \"{}\",\n",
            schema.name.to_lowercase()
        ));
        code.push_str(&format!("  kind: \"{}\",\n", schema.name));
        code.push_str("  metadata: metadata,\n");
        code.push_str("  spec: spec,\n");
        code.push_str("}\n");

        Ok(code)
    }
}

/// OpenAPI plugin factory
pub struct OpenApiPluginFactory;

#[async_trait]
impl PluginFactory for OpenApiPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(OpenApiPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["openapi".to_string(), "swagger".to_string(), "yaml".to_string(), "json".to_string()]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(OpenApiPluginFactory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_openapi_parser_basic() {
        let mut parser = OpenApiParser::new();

        let test_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /users:
    get:
      summary: Get users
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/User'
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
      required:
        - id
        - name
"#;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.yaml");
        tokio::fs::write(&test_file, test_content).await.unwrap();

        parser
            .parse_content(test_content, &test_file)
            .await
            .unwrap();

        let schemas = parser.extract_schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "User");
    }

    #[tokio::test]
    async fn test_openapi_plugin() {
        let config = PluginConfig {
            plugin_id: "test-openapi-plugin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
        };

        let plugin = OpenApiPlugin::new(config);
        let metadata = plugin.metadata();

        assert_eq!(metadata.name, "OpenAPI Plugin");
        assert!(metadata.supported_types.contains(&"openapi".to_string()));
    }

    #[tokio::test]
    async fn test_openapi_plugin_processing() {
        let config = PluginConfig {
            plugin_id: "test-openapi-plugin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
        };

        let plugin = OpenApiPlugin::new(config.clone());
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.yaml");
        
        let test_content = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths:
  /test:
    get:
      summary: Test endpoint
      responses:
        '200':
          description: Success
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
"#;
        
        tokio::fs::write(&test_file, &test_content).await.unwrap();

        let context = PluginContext::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("output"),
            config,
        );

        let result = plugin.process_source(&test_file, &context).await.unwrap();
        
        assert_eq!(result.schemas.len(), 1);
        assert_eq!(result.schemas[0].name, "User");
        assert_eq!(result.statistics.files_processed, 1);
        assert_eq!(result.statistics.schemas_extracted, 1);
    }
}
