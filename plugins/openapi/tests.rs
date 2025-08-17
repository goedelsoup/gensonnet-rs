//! OpenAPI plugin tests

use super::*;
use tempfile::TempDir;
use crate::plugin::{Plugin, PluginConfig, PluginCapability, PluginContext};

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
