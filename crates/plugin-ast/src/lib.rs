//! Abstract AST (Abstract Syntax Tree) processing plugin infrastructure
//!
//! This crate provides the common infrastructure for AST-based plugins
//! that can parse source code and extract type information.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[allow(unused_imports)]
use tracing::{debug, info, warn};

use gensonnet_plugin::*;

pub mod parser;
pub mod types;
pub mod visitor;

pub use parser::*;
pub use types::*;
pub use visitor::*;

/// AST node types that can be processed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AstNodeType {
    /// Function or method definition
    Function,

    /// Type definition (struct, class, interface, etc.)
    Type,

    /// Variable declaration
    Variable,

    /// Import statement
    Import,

    /// Package/module declaration
    Package,

    /// Comment
    Comment,

    /// Other/unknown node type
    Other(String),
}

/// AST node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    /// Node type
    pub node_type: AstNodeType,

    /// Node name/identifier
    pub name: String,

    /// Node content/source
    pub content: String,

    /// Line number in source file
    pub line: usize,

    /// Column number in source file
    pub column: usize,

    /// Node metadata
    pub metadata: HashMap<String, serde_yaml::Value>,

    /// Child nodes
    pub children: Vec<AstNode>,
}

/// AST parsing result
#[derive(Debug, Clone)]
pub struct AstParseResult {
    /// Root nodes of the AST
    pub root_nodes: Vec<AstNode>,

    /// Parsing errors
    pub errors: Vec<String>,

    /// Parsing warnings
    pub warnings: Vec<String>,

    /// Processing statistics
    pub statistics: AstParseStatistics,
}

/// AST parsing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstParseStatistics {
    /// Number of nodes parsed
    pub nodes_parsed: usize,

    /// Number of functions found
    pub functions_found: usize,

    /// Number of types found
    pub types_found: usize,

    /// Number of variables found
    pub variables_found: usize,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// AST parser trait for different languages
#[async_trait]
pub trait AstParser: Send + Sync {
    /// Get parser name
    fn name(&self) -> &str;

    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<&str>;

    /// Check if this parser can handle the given file
    async fn can_parse(&self, file_path: &Path) -> Result<bool>;

    /// Parse a file and return AST nodes
    async fn parse_file(&self, file_path: &Path) -> Result<AstParseResult>;

    /// Parse source code string
    async fn parse_source(&self, source: &str, file_path: Option<&Path>) -> Result<AstParseResult>;

    /// Clone the parser as a boxed trait object
    fn clone_box(&self) -> Box<dyn AstParser>;
}

/// AST visitor trait for traversing AST nodes
#[async_trait]
pub trait AstVisitor: Send + Sync {
    /// Visit a node
    async fn visit_node(&mut self, node: &AstNode) -> Result<()>;

    /// Visit function nodes
    async fn visit_function(&mut self, node: &AstNode) -> Result<()>;

    /// Visit type nodes
    async fn visit_type(&mut self, node: &AstNode) -> Result<()>;

    /// Visit variable nodes
    async fn visit_variable(&mut self, node: &AstNode) -> Result<()>;

    /// Visit import nodes
    async fn visit_import(&mut self, node: &AstNode) -> Result<()>;

    /// Get visitor results
    fn get_results(&self) -> AstVisitorResult;

    /// Clone the visitor as a boxed trait object
    fn clone_box(&self) -> Box<dyn AstVisitor>;
}

/// AST visitor result
#[derive(Debug, Clone)]
pub struct AstVisitorResult {
    /// Extracted schemas
    pub schemas: Vec<ExtractedSchema>,

    /// Processing statistics
    pub statistics: AstParseStatistics,

    /// Warnings
    pub warnings: Vec<String>,

    /// Errors
    pub errors: Vec<String>,
}

/// Abstract AST plugin that can be extended for different languages
pub struct AbstractAstPlugin {
    /// Parser instance
    parser: Box<dyn AstParser>,

    /// Plugin configuration
    config: PluginConfig,

    /// AST visitor
    visitor: Box<dyn AstVisitor>,
}

impl AbstractAstPlugin {
    /// Create a new abstract AST plugin
    pub fn new(
        parser: Box<dyn AstParser>,
        visitor: Box<dyn AstVisitor>,
        config: PluginConfig,
    ) -> Self {
        Self {
            parser,
            config,
            visitor,
        }
    }

    /// Get plugin metadata
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: self.config.plugin_id.clone(),
            name: format!("{} AST Plugin", self.parser.name()),
            version: "1.0.0".to_string(),
            description: format!("AST processing plugin for {}", self.parser.name()),
            supported_types: self
                .parser
                .supported_extensions()
                .iter()
                .map(|s| s.to_string())
                .collect(),
            capabilities: vec![
                PluginCapability::Parse,
                PluginCapability::SchemaExtraction,
                PluginCapability::AstProcessing,
            ],
        }
    }
}

#[async_trait]
impl Plugin for AbstractAstPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.get_metadata()
    }

    async fn initialize(&self, _context: &PluginContext) -> Result<()> {
        info!("Initializing {} AST plugin", self.parser.name());
        Ok(())
    }

    async fn can_handle(&self, source_path: &Path) -> Result<bool> {
        self.parser.can_parse(source_path).await
    }

    async fn process_source(
        &self,
        source_path: &Path,
        _context: &PluginContext,
    ) -> Result<PluginResult> {
        let start_time = std::time::Instant::now();

        debug!("Processing source file: {:?}", source_path);

        // Parse the source file
        let parse_result = self.parser.parse_file(source_path).await?;

        // Create a new visitor instance for this processing
        let mut visitor = self.visitor.clone_box();

        // Visit all root nodes
        for node in &parse_result.root_nodes {
            visitor.visit_node(node).await?;
        }

        // Get visitor results
        let visitor_result = visitor.get_results();

        let processing_time = start_time.elapsed();

        let schemas_count = visitor_result.schemas.len();
        Ok(PluginResult {
            schemas: visitor_result.schemas,
            generated_files: Vec::new(),
            statistics: PluginStatistics {
                processing_time_ms: processing_time.as_millis() as u64,
                files_processed: 1,
                schemas_extracted: schemas_count,
                files_generated: 0,
            },
            warnings: parse_result.warnings,
            errors: parse_result.errors,
        })
    }

    async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &PluginContext,
    ) -> Result<Vec<PathBuf>> {
        let mut generated_files = Vec::new();

        for schema in schemas {
            let output_file = context.output_dir.join(format!("{}.jsonnet", schema.name));

            // Generate Jsonnet code from schema
            let jsonnet_code = self.generate_jsonnet_code(schema)?;

            tokio::fs::write(&output_file, jsonnet_code).await?;
            generated_files.push(output_file);
        }

        Ok(generated_files)
    }

    async fn cleanup(&self, _context: &PluginContext) -> Result<()> {
        debug!("Cleaning up {} AST plugin", self.parser.name());
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(AbstractAstPlugin {
            parser: self.parser.clone_box(),
            visitor: self.visitor.clone_box(),
            config: self.config.clone(),
        })
    }
}

impl AbstractAstPlugin {
    /// Generate Jsonnet code from a schema
    fn generate_jsonnet_code(&self, schema: &ExtractedSchema) -> Result<String> {
        // Basic Jsonnet generation - can be overridden by specific implementations
        let mut jsonnet = String::new();

        jsonnet.push_str(&format!("// Generated from {}\n", schema.schema_type));
        jsonnet.push_str(&format!("// Source: {:?}\n\n", schema.source_file));

        // Convert schema content to Jsonnet
        match &schema.content {
            serde_yaml::Value::Mapping(map) => {
                jsonnet.push_str(&format!("{{\n"));
                for (key, value) in map {
                    if let Some(key_str) = key.as_str() {
                        jsonnet.push_str(&format!(
                            "  {}: {},\n",
                            key_str,
                            self.value_to_jsonnet(value)
                        ));
                    }
                }
                jsonnet.push_str(&format!("}}\n"));
            }
            _ => {
                jsonnet.push_str(&format!("{}\n", self.value_to_jsonnet(&schema.content)));
            }
        }

        Ok(jsonnet)
    }

    /// Convert a YAML value to Jsonnet representation
    fn value_to_jsonnet(&self, value: &serde_yaml::Value) -> String {
        match value {
            serde_yaml::Value::Null => "null".to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::String(s) => format!("\"{}\"", s),
            serde_yaml::Value::Sequence(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_jsonnet(v)).collect();
                format!("[{}]", items.join(", "))
            }
            serde_yaml::Value::Mapping(map) => {
                let items: Vec<String> = map
                    .iter()
                    .filter_map(|(k, v)| {
                        k.as_str()
                            .map(|key_str| format!("{}: {}", key_str, self.value_to_jsonnet(v)))
                    })
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            &serde_yaml::Value::Tagged(_) => {
                // For tagged values, we'll just convert them to a string representation
                format!("{:?}", value)
            }
        }
    }
}

/// AST parser factory trait
#[async_trait]
pub trait AstParserFactory: Send + Sync {
    /// Create a new parser instance
    fn create_parser(&self) -> Box<dyn AstParser>;

    /// Get parser name
    fn parser_name(&self) -> &str;

    /// Clone the factory
    fn clone_box(&self) -> Box<dyn AstParserFactory>;
}

/// AST visitor factory trait
#[async_trait]
pub trait AstVisitorFactory: Send + Sync {
    /// Create a new visitor instance
    fn create_visitor(&self) -> Box<dyn AstVisitor>;

    /// Get visitor name
    fn visitor_name(&self) -> &str;

    /// Clone the factory
    fn clone_box(&self) -> Box<dyn AstVisitorFactory>;
}

/// Abstract AST plugin factory
pub struct AbstractAstPluginFactory {
    /// Parser factory
    parser_factory: Box<dyn AstParserFactory>,

    /// Visitor factory
    visitor_factory: Box<dyn AstVisitorFactory>,
}

impl AbstractAstPluginFactory {
    /// Create a new abstract AST plugin factory
    pub fn new(
        parser_factory: Box<dyn AstParserFactory>,
        visitor_factory: Box<dyn AstVisitorFactory>,
    ) -> Self {
        Self {
            parser_factory,
            visitor_factory,
        }
    }
}

#[async_trait]
impl PluginFactory for AbstractAstPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        let parser = self.parser_factory.create_parser();
        let visitor = self.visitor_factory.create_visitor();

        Ok(Box::new(AbstractAstPlugin::new(parser, visitor, config)))
    }

    fn supported_types(&self) -> Vec<String> {
        let parser = self.parser_factory.create_parser();
        parser
            .supported_extensions()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(AbstractAstPluginFactory {
            parser_factory: self.parser_factory.clone_box(),
            visitor_factory: self.visitor_factory.clone_box(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let node = AstNode {
            node_type: AstNodeType::Function,
            name: "test_function".to_string(),
            content: "func test_function() {}".to_string(),
            line: 1,
            column: 1,
            metadata: HashMap::new(),
            children: Vec::new(),
        };

        assert_eq!(node.name, "test_function");
        assert_eq!(node.node_type, AstNodeType::Function);
    }

    #[test]
    fn test_ast_node_type_serialization() {
        let node_type = AstNodeType::Type;
        let serialized = serde_yaml::to_string(&node_type).unwrap();
        let deserialized: AstNodeType = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(node_type, deserialized);
    }

    #[tokio::test]
    async fn test_ast_parse_statistics() {
        let stats = AstParseStatistics {
            nodes_parsed: 10,
            functions_found: 3,
            types_found: 2,
            variables_found: 5,
            processing_time_ms: 100,
        };

        assert_eq!(stats.nodes_parsed, 10);
        assert_eq!(stats.functions_found, 3);
    }
}
