//! OpenAPI plugin implementation

use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

use super::parser::OpenApiParser;
use crate::plugin::*;

/// OpenAPI plugin
#[allow(dead_code)]
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
