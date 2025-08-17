//! AST visitor patterns and implementations

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, trace};

use super::*;

/// Default AST visitor implementation
pub struct DefaultAstVisitor {
    /// Extracted schemas
    schemas: Vec<ExtractedSchema>,

    /// Processing statistics
    statistics: AstParseStatistics,

    /// Warnings
    warnings: Vec<String>,

    /// Errors
    errors: Vec<String>,

    /// Current file path
    current_file: Option<String>,
}

impl DefaultAstVisitor {
    /// Create a new default AST visitor
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            statistics: AstParseStatistics {
                nodes_parsed: 0,
                functions_found: 0,
                types_found: 0,
                variables_found: 0,
                processing_time_ms: 0,
            },
            warnings: Vec::new(),
            errors: Vec::new(),
            current_file: None,
        }
    }

    /// Set current file path
    pub fn set_current_file(&mut self, file_path: String) {
        self.current_file = Some(file_path);
    }

    /// Extract schema from a node
    fn extract_schema_from_node(&self, node: &AstNode) -> Option<ExtractedSchema> {
        let source_file = self
            .current_file
            .as_ref()
            .map(|f| std::path::PathBuf::from(f))
            .unwrap_or_else(|| std::path::PathBuf::from("unknown"));

        match node.node_type {
            AstNodeType::Function => {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "node_type".to_string(),
                    serde_yaml::Value::String("function".to_string()),
                );
                metadata.insert(
                    "line".to_string(),
                    serde_yaml::Value::Number(serde_yaml::Number::from(node.line)),
                );
                metadata.insert(
                    "column".to_string(),
                    serde_yaml::Value::Number(serde_yaml::Number::from(node.column)),
                );

                Some(ExtractedSchema {
                    name: node.name.clone(),
                    schema_type: "function".to_string(),
                    content: serde_yaml::to_value(HashMap::from([
                        ("name".to_string(), node.name.clone()),
                        ("content".to_string(), node.content.clone()),
                        ("type".to_string(), "function".to_string()),
                    ]))
                    .unwrap(),
                    source_file,
                    metadata,
                })
            }
            AstNodeType::Type => {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "node_type".to_string(),
                    serde_yaml::Value::String("type".to_string()),
                );
                metadata.insert(
                    "line".to_string(),
                    serde_yaml::Value::Number(serde_yaml::Number::from(node.line)),
                );
                metadata.insert(
                    "column".to_string(),
                    serde_yaml::Value::Number(serde_yaml::Number::from(node.column)),
                );

                Some(ExtractedSchema {
                    name: node.name.clone(),
                    schema_type: "type".to_string(),
                    content: serde_yaml::to_value(HashMap::from([
                        ("name".to_string(), node.name.clone()),
                        ("content".to_string(), node.content.clone()),
                        ("type".to_string(), "type".to_string()),
                    ]))
                    .unwrap(),
                    source_file,
                    metadata,
                })
            }
            AstNodeType::Variable => {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "node_type".to_string(),
                    serde_yaml::Value::String("variable".to_string()),
                );
                metadata.insert(
                    "line".to_string(),
                    serde_yaml::Value::Number(serde_yaml::Number::from(node.line)),
                );
                metadata.insert(
                    "column".to_string(),
                    serde_yaml::Value::Number(serde_yaml::Number::from(node.column)),
                );

                Some(ExtractedSchema {
                    name: node.name.clone(),
                    schema_type: "variable".to_string(),
                    content: serde_yaml::to_value(HashMap::from([
                        ("name".to_string(), node.name.clone()),
                        ("content".to_string(), node.content.clone()),
                        ("type".to_string(), "variable".to_string()),
                    ]))
                    .unwrap(),
                    source_file,
                    metadata,
                })
            }
            _ => None,
        }
    }
}

impl Default for DefaultAstVisitor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AstVisitor for DefaultAstVisitor {
    async fn visit_node(&mut self, node: &AstNode) -> Result<()> {
        trace!("Visiting node: {} ({:?})", node.name, node.node_type);

        // Update statistics
        self.statistics.nodes_parsed += 1;

        match node.node_type {
            AstNodeType::Function => {
                self.statistics.functions_found += 1;
                self.visit_function(node).await?;
            }
            AstNodeType::Type => {
                self.statistics.types_found += 1;
                self.visit_type(node).await?;
            }
            AstNodeType::Variable => {
                self.statistics.variables_found += 1;
                self.visit_variable(node).await?;
            }
            AstNodeType::Import => {
                self.visit_import(node).await?;
            }
            _ => {
                // Handle other node types
                debug!("Visiting other node type: {:?}", node.node_type);
            }
        }

        // Visit child nodes
        for child in &node.children {
            self.visit_node(child).await?;
        }

        Ok(())
    }

    async fn visit_function(&mut self, node: &AstNode) -> Result<()> {
        debug!("Visiting function: {}", node.name);

        if let Some(schema) = self.extract_schema_from_node(node) {
            self.schemas.push(schema);
        }

        Ok(())
    }

    async fn visit_type(&mut self, node: &AstNode) -> Result<()> {
        debug!("Visiting type: {}", node.name);

        if let Some(schema) = self.extract_schema_from_node(node) {
            self.schemas.push(schema);
        }

        Ok(())
    }

    async fn visit_variable(&mut self, node: &AstNode) -> Result<()> {
        debug!("Visiting variable: {}", node.name);

        if let Some(schema) = self.extract_schema_from_node(node) {
            self.schemas.push(schema);
        }

        Ok(())
    }

    async fn visit_import(&mut self, node: &AstNode) -> Result<()> {
        debug!("Visiting import: {}", node.name);

        // Import nodes typically don't generate schemas, but we can track them
        self.warnings
            .push(format!("Import statement found: {}", node.content));

        Ok(())
    }

    fn get_results(&self) -> AstVisitorResult {
        AstVisitorResult {
            schemas: self.schemas.clone(),
            statistics: self.statistics.clone(),
            warnings: self.warnings.clone(),
            errors: self.errors.clone(),
        }
    }

    fn clone_box(&self) -> Box<dyn AstVisitor> {
        Box::new(DefaultAstVisitor {
            schemas: self.schemas.clone(),
            statistics: self.statistics.clone(),
            warnings: self.warnings.clone(),
            errors: self.errors.clone(),
            current_file: self.current_file.clone(),
        })
    }
}

/// AST visitor that can be cloned
pub trait CloneableAstVisitor: AstVisitor {
    /// Clone the visitor
    fn clone_box(&self) -> Box<dyn CloneableAstVisitor>;
}

impl CloneableAstVisitor for DefaultAstVisitor {
    fn clone_box(&self) -> Box<dyn CloneableAstVisitor> {
        Box::new(DefaultAstVisitor {
            schemas: self.schemas.clone(),
            statistics: self.statistics.clone(),
            warnings: self.warnings.clone(),
            errors: self.errors.clone(),
            current_file: self.current_file.clone(),
        })
    }
}

/// AST visitor builder
pub struct AstVisitorBuilder {
    /// Visitor name
    name: String,

    /// Visitor configuration
    config: HashMap<String, serde_yaml::Value>,
}

impl AstVisitorBuilder {
    /// Create a new visitor builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: HashMap::new(),
        }
    }

    /// Add configuration option
    pub fn with_config(mut self, key: String, value: serde_yaml::Value) -> Self {
        self.config.insert(key, value);
        self
    }

    /// Build the visitor
    pub fn build(self) -> DefaultAstVisitor {
        DefaultAstVisitor::new()
    }
}

/// AST visitor factory for creating visitors
pub struct DefaultAstVisitorFactory;

impl DefaultAstVisitorFactory {
    /// Create a new default visitor factory
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AstVisitorFactory for DefaultAstVisitorFactory {
    fn create_visitor(&self) -> Box<dyn AstVisitor> {
        Box::new(DefaultAstVisitor::new())
    }

    fn visitor_name(&self) -> &str {
        "Default"
    }

    fn clone_box(&self) -> Box<dyn AstVisitorFactory> {
        Box::new(DefaultAstVisitorFactory::new())
    }
}

impl Default for DefaultAstVisitorFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// AST visitor that filters nodes
pub struct FilteringAstVisitor {
    /// Base visitor
    base_visitor: Box<dyn AstVisitor>,

    /// Node filter
    filter: AstNodeFilter,
}

impl FilteringAstVisitor {
    /// Create a new filtering visitor
    pub fn new(base_visitor: Box<dyn AstVisitor>, filter: AstNodeFilter) -> Self {
        Self {
            base_visitor,
            filter,
        }
    }

    /// Check if a node matches the filter
    fn matches_filter(&self, node: &AstNode) -> bool {
        // Check node type
        if let Some(ref filter_type) = self.filter.node_type {
            if node.node_type != *filter_type {
                return false;
            }
        }

        // Check name pattern
        if let Some(ref name_pattern) = self.filter.name_pattern {
            if !node.name.contains(name_pattern) {
                return false;
            }
        }

        // Check content pattern
        if let Some(ref content_pattern) = self.filter.content_pattern {
            if !node.content.contains(content_pattern) {
                return false;
            }
        }

        true
    }
}

#[async_trait]
impl AstVisitor for FilteringAstVisitor {
    async fn visit_node(&mut self, node: &AstNode) -> Result<()> {
        if self.matches_filter(node) {
            self.base_visitor.visit_node(node).await?;
        }

        // Visit children if requested
        if self.filter.include_children {
            for child in &node.children {
                self.visit_node(child).await?;
            }
        }

        Ok(())
    }

    async fn visit_function(&mut self, node: &AstNode) -> Result<()> {
        if self.matches_filter(node) {
            self.base_visitor.visit_function(node).await?;
        }
        Ok(())
    }

    async fn visit_type(&mut self, node: &AstNode) -> Result<()> {
        if self.matches_filter(node) {
            self.base_visitor.visit_type(node).await?;
        }
        Ok(())
    }

    async fn visit_variable(&mut self, node: &AstNode) -> Result<()> {
        if self.matches_filter(node) {
            self.base_visitor.visit_variable(node).await?;
        }
        Ok(())
    }

    async fn visit_import(&mut self, node: &AstNode) -> Result<()> {
        if self.matches_filter(node) {
            self.base_visitor.visit_import(node).await?;
        }
        Ok(())
    }

    fn get_results(&self) -> AstVisitorResult {
        self.base_visitor.get_results()
    }

    fn clone_box(&self) -> Box<dyn AstVisitor> {
        Box::new(FilteringAstVisitor {
            base_visitor: self.base_visitor.clone_box(),
            filter: self.filter.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ast_visitor_creation() {
        let visitor = DefaultAstVisitor::new();
        let results = visitor.get_results();

        assert_eq!(results.schemas.len(), 0);
        assert_eq!(results.statistics.nodes_parsed, 0);
        assert_eq!(results.statistics.functions_found, 0);
    }

    #[tokio::test]
    async fn test_default_ast_visitor_visit_function() {
        let mut visitor = DefaultAstVisitor::new();
        visitor.set_current_file("test.go".to_string());

        let node = AstNode {
            node_type: AstNodeType::Function,
            name: "test_function".to_string(),
            content: "func test_function() {}".to_string(),
            line: 1,
            column: 1,
            metadata: HashMap::new(),
            children: Vec::new(),
        };

        visitor.visit_function(&node).await.unwrap();

        let results = visitor.get_results();
        assert_eq!(results.schemas.len(), 1);
        assert_eq!(results.statistics.functions_found, 0); // functions_found is only incremented in visit_node
        assert_eq!(results.schemas[0].name, "test_function");
    }

    #[tokio::test]
    async fn test_default_ast_visitor_visit_node() {
        let mut visitor = DefaultAstVisitor::new();
        visitor.set_current_file("test.go".to_string());

        let node = AstNode {
            node_type: AstNodeType::Type,
            name: "MyStruct".to_string(),
            content: "type MyStruct struct {}".to_string(),
            line: 1,
            column: 1,
            metadata: HashMap::new(),
            children: Vec::new(),
        };

        visitor.visit_node(&node).await.unwrap();

        let results = visitor.get_results();
        assert_eq!(results.schemas.len(), 1);
        assert_eq!(results.statistics.types_found, 1);
        assert_eq!(results.schemas[0].name, "MyStruct");
    }

    #[test]
    fn test_ast_visitor_builder() {
        let visitor = AstVisitorBuilder::new("TestVisitor".to_string())
            .with_config(
                "include_comments".to_string(),
                serde_yaml::Value::Bool(true),
            )
            .build();

        let results = visitor.get_results();
        assert_eq!(results.schemas.len(), 0);
    }
}
