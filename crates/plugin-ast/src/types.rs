//! AST type definitions and utilities

use crate::AstNodeType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AST node type with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNodeInfo {
    /// Node type
    pub node_type: super::AstNodeType,

    /// Node name
    pub name: String,

    /// Node content
    pub content: String,

    /// Source location
    pub location: SourceLocation,

    /// Node metadata
    pub metadata: HashMap<String, serde_yaml::Value>,

    /// Node attributes
    pub attributes: Vec<NodeAttribute>,
}

/// Source code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file_path: String,

    /// Line number (1-based)
    pub line: usize,

    /// Column number (1-based)
    pub column: usize,

    /// End line number (1-based)
    pub end_line: Option<usize>,

    /// End column number (1-based)
    pub end_column: Option<usize>,
}

/// Node attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAttribute {
    /// Attribute name
    pub name: String,

    /// Attribute value
    pub value: serde_yaml::Value,

    /// Attribute type
    pub attribute_type: AttributeType,
}

/// Attribute type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeType {
    /// String attribute
    String,

    /// Numeric attribute
    Numeric,

    /// Boolean attribute
    Boolean,

    /// Array attribute
    Array,

    /// Object attribute
    Object,

    /// Function attribute
    Function,

    /// Type attribute
    Type,
}

/// AST node filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNodeFilter {
    /// Filter by node type
    pub node_type: Option<super::AstNodeType>,

    /// Filter by name pattern
    pub name_pattern: Option<String>,

    /// Filter by content pattern
    pub content_pattern: Option<String>,

    /// Filter by metadata
    pub metadata_filters: HashMap<String, serde_yaml::Value>,

    /// Include child nodes
    pub include_children: bool,

    /// Maximum depth for traversal
    pub max_depth: Option<usize>,
}

/// AST node query result
#[derive(Debug, Clone)]
pub struct AstNodeQueryResult {
    /// Matching nodes
    pub nodes: Vec<AstNodeInfo>,

    /// Query statistics
    pub statistics: QueryStatistics,
}

/// Query statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStatistics {
    /// Number of nodes examined
    pub nodes_examined: usize,

    /// Number of nodes matched
    pub nodes_matched: usize,

    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
}

/// AST node transformer
pub trait AstNodeTransformer {
    /// Transform a node
    fn transform_node(&self, node: &AstNodeInfo) -> Result<AstNodeInfo, String>;

    /// Get transformer name
    fn name(&self) -> &str;
}

/// AST node validator
pub trait AstNodeValidator {
    /// Validate a node
    fn validate_node(&self, node: &AstNodeInfo) -> Result<ValidationResult, String>;

    /// Get validator name
    fn name(&self) -> &str;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Is valid
    pub is_valid: bool,

    /// Validation errors
    pub errors: Vec<String>,

    /// Validation warnings
    pub warnings: Vec<String>,

    /// Validation suggestions
    pub suggestions: Vec<String>,
}

/// AST node analyzer
pub trait AstNodeAnalyzer {
    /// Analyze a node
    fn analyze_node(&self, node: &AstNodeInfo) -> Result<AnalysisResult, String>;

    /// Get analyzer name
    fn name(&self) -> &str;
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Analysis metrics
    pub metrics: HashMap<String, f64>,

    /// Analysis insights
    pub insights: Vec<String>,

    /// Analysis recommendations
    pub recommendations: Vec<String>,

    /// Complexity score
    pub complexity_score: Option<f64>,
}

/// AST node formatter
pub trait AstNodeFormatter {
    /// Format a node
    fn format_node(&self, node: &AstNodeInfo) -> Result<String, String>;

    /// Get formatter name
    fn name(&self) -> &str;

    /// Get supported output formats
    fn supported_formats(&self) -> Vec<String>;
}

/// AST node exporter
pub trait AstNodeExporter {
    /// Export nodes
    fn export_nodes(&self, nodes: &[AstNodeInfo], format: &str) -> Result<Vec<u8>, String>;

    /// Get exporter name
    fn name(&self) -> &str;

    /// Get supported export formats
    fn supported_formats(&self) -> Vec<String>;
}

/// AST node importer
pub trait AstNodeImporter {
    /// Import nodes
    fn import_nodes(&self, data: &[u8], format: &str) -> Result<Vec<AstNodeInfo>, String>;

    /// Get importer name
    fn name(&self) -> &str;

    /// Get supported import formats
    fn supported_formats(&self) -> Vec<String>;
}

/// AST node cache
pub trait AstNodeCache {
    /// Store nodes
    fn store_nodes(&self, key: &str, nodes: &[AstNodeInfo]) -> Result<(), String>;

    /// Retrieve nodes
    fn retrieve_nodes(&self, key: &str) -> Result<Option<Vec<AstNodeInfo>>, String>;

    /// Clear cache
    fn clear_cache(&self) -> Result<(), String>;

    /// Get cache name
    fn name(&self) -> &str;
}

/// AST node index
pub trait AstNodeIndex {
    /// Index nodes
    fn index_nodes(&self, nodes: &[AstNodeInfo]) -> Result<(), String>;

    /// Search nodes
    fn search_nodes(&self, query: &str) -> Result<Vec<AstNodeInfo>, String>;

    /// Get index name
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_info_creation() {
        let location = SourceLocation {
            file_path: "test.go".to_string(),
            line: 1,
            column: 1,
            end_line: Some(1),
            end_column: Some(10),
        };

        let node_info = AstNodeInfo {
            node_type: AstNodeType::Function,
            name: "test_function".to_string(),
            content: "func test_function() {}".to_string(),
            location,
            metadata: HashMap::new(),
            attributes: Vec::new(),
        };

        assert_eq!(node_info.name, "test_function");
        assert_eq!(node_info.node_type, AstNodeType::Function);
    }

    #[test]
    fn test_source_location() {
        let location = SourceLocation {
            file_path: "test.go".to_string(),
            line: 10,
            column: 5,
            end_line: Some(15),
            end_column: Some(20),
        };

        assert_eq!(location.file_path, "test.go");
        assert_eq!(location.line, 10);
        assert_eq!(location.column, 5);
        assert_eq!(location.end_line, Some(15));
        assert_eq!(location.end_column, Some(20));
    }

    #[test]
    fn test_node_attribute() {
        let attribute = NodeAttribute {
            name: "visibility".to_string(),
            value: serde_yaml::Value::String("public".to_string()),
            attribute_type: AttributeType::String,
        };

        assert_eq!(attribute.name, "visibility");
        assert_eq!(attribute.attribute_type, AttributeType::String);
    }

    #[test]
    fn test_ast_node_filter() {
        let filter = AstNodeFilter {
            node_type: Some(AstNodeType::Function),
            name_pattern: Some("test.*".to_string()),
            content_pattern: None,
            metadata_filters: HashMap::new(),
            include_children: true,
            max_depth: Some(3),
        };

        assert_eq!(filter.node_type, Some(AstNodeType::Function));
        assert_eq!(filter.name_pattern, Some("test.*".to_string()));
        assert!(filter.include_children);
        assert_eq!(filter.max_depth, Some(3));
    }
}
