//! CRD (CustomResourceDefinition) plugin for processing Kubernetes CRDs

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

use crate::crd::{CrdParser, CrdSchema};
use crate::plugin::*;

/// CRD plugin for processing Kubernetes CustomResourceDefinitions
pub struct CrdPlugin {
    /// Plugin configuration
    config: PluginConfig,

    /// CRD parser instance
    parser: CrdParser,
}

impl CrdPlugin {
    /// Create a new CRD plugin
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            parser: CrdParser::new(),
        }
    }

    /// Get plugin metadata
    fn get_metadata() -> PluginMetadata {
        PluginMetadata {
            id: "crd:builtin".to_string(),
            name: "CRD Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin for processing Kubernetes CustomResourceDefinitions".to_string(),
            supported_types: vec!["yaml".to_string(), "yml".to_string()],
            capabilities: vec![
                PluginCapability::Parse,
                PluginCapability::SchemaExtraction,
                PluginCapability::Validation,
            ],
        }
    }

    /// Extract schemas from CRD content
    async fn extract_schemas_from_crd(&self, crd_schemas: Vec<CrdSchema>) -> Vec<ExtractedSchema> {
        let mut extracted_schemas = Vec::new();

        for crd_schema in crd_schemas {
            let schema = ExtractedSchema {
                name: crd_schema.name.clone(),
                schema_type: "object".to_string(),
                content: crd_schema.schema.clone(),
                source_file: crd_schema.source_path.clone(),
                metadata: HashMap::from([
                    (
                        "group".to_string(),
                        serde_yaml::Value::String(crd_schema.group.clone()),
                    ),
                    (
                        "version".to_string(),
                        serde_yaml::Value::String(crd_schema.version.clone()),
                    ),
                    (
                        "api_version".to_string(),
                        serde_yaml::Value::String(crd_schema.api_version.clone()),
                    ),
                    (
                        "kind".to_string(),
                        serde_yaml::Value::String(crd_schema.kind().to_string()),
                    ),
                    (
                        "resource_name".to_string(),
                        serde_yaml::Value::String(crd_schema.resource_name()),
                    ),
                ]),
            };

            extracted_schemas.push(schema);
        }

        extracted_schemas
    }
}

#[async_trait]
impl Plugin for CrdPlugin {
    fn metadata(&self) -> PluginMetadata {
        Self::get_metadata()
    }

    async fn initialize(&self, _context: &PluginContext) -> Result<()> {
        info!("Initializing CRD plugin");
        Ok(())
    }

    async fn can_handle(&self, source_path: &Path) -> Result<bool> {
        // Check if it's a YAML file
        if let Some(ext) = source_path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if ext_str == "yaml" || ext_str == "yml" {
                // Try to read the file and check if it contains CRD content
                if let Ok(content) = tokio::fs::read_to_string(source_path).await {
                    if content.contains("kind:") && content.contains("CustomResourceDefinition") {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    async fn process_source(
        &self,
        source_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        info!("Processing CRD source: {:?}", source_path);

        let start_time = std::time::Instant::now();

        // Parse CRDs from the source
        let filters = self
            .config
            .config
            .get("filters")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let crd_schemas = self.parser.parse_from_directory(source_path, &filters)?;

        // Extract schemas
        let extracted_schemas = self.extract_schemas_from_crd(crd_schemas).await;
        let schemas_count = extracted_schemas.len();

        // Generate code
        let generated_files = self.generate_code(&extracted_schemas, context).await?;
        let files_count = generated_files.len();

        let processing_time = start_time.elapsed();

        Ok(PluginResult {
            schemas: extracted_schemas,
            generated_files,
            errors: Vec::new(),
            warnings: Vec::new(),
            statistics: PluginStatistics {
                processing_time_ms: processing_time.as_millis() as u64,
                files_processed: 1,
                schemas_extracted: schemas_count,
                files_generated: files_count,
            },
        })
    }

    async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &PluginContext,
    ) -> Result<Vec<PathBuf>> {
        info!("Generating code for {} CRD schemas", schemas.len());

        let mut generated_files = Vec::new();

        // Create output directory if it doesn't exist
        tokio::fs::create_dir_all(&context.output_dir).await?;

        // Generate Jsonnet library for each schema
        for schema in schemas {
            let schema_name = &schema.name;
            let output_file = context
                .output_dir
                .join(format!("{}.libsonnet", schema_name.to_lowercase()));

            // Generate Jsonnet content
            let jsonnet_content = self.generate_jsonnet_content(schema)?;

            // Write to file
            tokio::fs::write(&output_file, jsonnet_content).await?;
            generated_files.push(output_file);
        }

        // Generate index file
        let index_file = context.output_dir.join("_index.libsonnet");
        let index_content = self.generate_index_content(schemas)?;
        tokio::fs::write(&index_file, index_content).await?;
        generated_files.push(index_file);

        info!("Generated {} files", generated_files.len());
        Ok(generated_files)
    }

    async fn cleanup(&self, _context: &PluginContext) -> Result<()> {
        info!("Cleaning up CRD plugin");
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self::new(self.config.clone()))
    }
}

impl CrdPlugin {
    /// Generate Jsonnet content for a schema
    fn generate_jsonnet_content(&self, schema: &ExtractedSchema) -> Result<String> {
        let schema_name = &schema.name;
        let group = schema
            .metadata
            .get("group")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let version = schema
            .metadata
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let kind = schema
            .metadata
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or(schema_name);

        let mut content = String::new();

        // Add header comment
        content.push_str(&format!("// Generated from CRD: {}\n", schema_name));
        content.push_str(&format!("// API Group: {}\n", group));
        content.push_str(&format!("// API Version: {}\n", version));
        content.push_str(&format!("// Kind: {}\n\n", kind));

        // Add schema definition
        content.push_str(&format!("{{\n"));
        content.push_str(&format!("  // {} schema definition\n", schema_name));
        content.push_str(&format!("  {}:: {{\n", schema_name.to_lowercase()));

        // Add basic CRD structure
        content.push_str(&format!("    apiVersion: \"{}/{}\",\n", group, version));
        content.push_str(&format!("    kind: \"{}\",\n", kind));
        content.push_str(&format!("    metadata:: {{\n"));
        content.push_str(&format!(
            "      name: \"example-{}\",\n",
            schema_name.to_lowercase()
        ));
        content.push_str(&format!("    }},\n"));
        content.push_str(&format!("    spec:: {{}},\n"));
        content.push_str(&format!("  }},\n"));

        // Add helper functions
        content.push_str(&format!(
            "  // Helper function to create a new {}\n",
            schema_name
        ));
        content.push_str(&format!("  new{}:: function(name) {{\n", schema_name));
        content.push_str(&format!(
            "    self.{}(name) {{\n",
            schema_name.to_lowercase()
        ));
        content.push_str(&format!("      metadata+: {{\n"));
        content.push_str(&format!("        name: name,\n"));
        content.push_str(&format!("      }},\n"));
        content.push_str(&format!("    }}\n"));
        content.push_str(&format!("  }},\n"));

        content.push_str(&format!("}}\n"));

        Ok(content)
    }

    /// Generate index content
    fn generate_index_content(&self, schemas: &[ExtractedSchema]) -> Result<String> {
        let mut content = String::new();

        content.push_str("// CRD Library Index\n");
        content.push_str("// This file exports all CRD schemas\n\n");

        // Import all schema files
        for schema in schemas {
            let schema_name = &schema.name;
            content.push_str(&format!(
                "local {} = import \"{}.libsonnet\";\n",
                schema_name.to_lowercase(),
                schema_name.to_lowercase()
            ));
        }

        content.push_str("\n{\n");

        // Export all schemas
        for schema in schemas {
            let schema_name = &schema.name;
            content.push_str(&format!(
                "  {}: {}.{},\n",
                schema_name,
                schema_name.to_lowercase(),
                schema_name.to_lowercase()
            ));
        }

        content.push_str("}\n");

        Ok(content)
    }
}

/// CRD plugin factory
pub struct CrdPluginFactory;

#[async_trait]
impl PluginFactory for CrdPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(CrdPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["crd".to_string(), "yaml".to_string(), "yml".to_string()]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(CrdPluginFactory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_crd_plugin_creation() {
        let config = PluginConfig {
            plugin_id: "crd:test".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
        };

        let plugin = CrdPlugin::new(config);
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "crd:builtin");
        assert_eq!(metadata.name, "CRD Plugin");
        assert!(metadata.supported_types.contains(&"yaml".to_string()));
    }

    #[tokio::test]
    async fn test_crd_plugin_factory() {
        let factory = CrdPluginFactory;
        let config = PluginConfig {
            plugin_id: "crd:test".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
        };

        let plugin = factory.create_plugin(config).await.unwrap();
        let metadata = plugin.metadata();

        assert_eq!(metadata.id, "crd:builtin");
        assert!(factory.supported_types().contains(&"crd".to_string()));
    }

    #[tokio::test]
    async fn test_crd_plugin_can_handle() {
        let config = PluginConfig {
            plugin_id: "crd:test".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
        };

        let plugin = CrdPlugin::new(config);
        let temp_dir = TempDir::new().unwrap();

        // Test with non-CRD file
        let non_crd_file = temp_dir.path().join("test.txt");
        tokio::fs::write(&non_crd_file, "This is not a CRD")
            .await
            .unwrap();
        assert!(!plugin.can_handle(&non_crd_file).await.unwrap());

        // Test with CRD file
        let crd_content = r#"
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: examples.test.com
spec:
  group: test.com
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
"#;
        let crd_file = temp_dir.path().join("test-crd.yaml");
        tokio::fs::write(&crd_file, crd_content).await.unwrap();
        assert!(plugin.can_handle(&crd_file).await.unwrap());
    }
}
