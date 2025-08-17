//! AST parser abstractions and utilities

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tracing::{debug, warn};

use super::*;

/// Base AST parser implementation
pub struct BaseAstParser {
    /// Parser name
    name: String,

    /// Supported file extensions
    supported_extensions: Vec<String>,
}

impl BaseAstParser {
    /// Create a new base AST parser
    pub fn new(name: String, supported_extensions: Vec<String>) -> Self {
        Self {
            name,
            supported_extensions,
        }
    }

    /// Get parser name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get supported extensions
    pub fn get_supported_extensions(&self) -> &[String] {
        &self.supported_extensions
    }
}

/// Default AST parser implementation
pub struct DefaultAstParser {
    base: BaseAstParser,
}

impl DefaultAstParser {
    /// Create a new default AST parser
    pub fn new() -> Self {
        Self {
            base: BaseAstParser::new(
                "Default".to_string(),
                vec!["txt".to_string(), "text".to_string()],
            ),
        }
    }
}

#[async_trait]
impl AstParser for DefaultAstParser {
    fn name(&self) -> &str {
        self.base.get_name()
    }

    fn supported_extensions(&self) -> Vec<&str> {
        self.base
            .get_supported_extensions()
            .iter()
            .map(|s| s.as_str())
            .collect()
    }

    async fn can_parse(&self, file_path: &Path) -> Result<bool> {
        if let Some(extension) = file_path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            Ok(self
                .base
                .get_supported_extensions()
                .contains(&ext.to_string()))
        } else {
            Ok(false)
        }
    }

    async fn parse_file(&self, file_path: &Path) -> Result<AstParseResult> {
        debug!("Parsing file: {:?}", file_path);

        let content = tokio::fs::read_to_string(file_path).await?;
        self.parse_source(&content, Some(file_path)).await
    }

    async fn parse_source(&self, source: &str, file_path: Option<&Path>) -> Result<AstParseResult> {
        let start_time = std::time::Instant::now();

        // Simple line-based parsing for default implementation
        let mut root_nodes = Vec::new();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut nodes_parsed = 0;
        let mut functions_found = 0;
        let mut types_found = 0;
        let mut variables_found = 0;

        for (line_num, line) in source.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            nodes_parsed += 1;

            // Simple heuristic parsing
            if line.starts_with("func ") || line.starts_with("function ") {
                functions_found += 1;
                root_nodes.push(AstNode {
                    node_type: AstNodeType::Function,
                    name: extract_name_from_line(line),
                    content: line.to_string(),
                    line: line_num + 1,
                    column: 1,
                    metadata: HashMap::new(),
                    children: Vec::new(),
                });
            } else if line.starts_with("type ")
                || line.starts_with("struct ")
                || line.starts_with("class ")
            {
                types_found += 1;
                root_nodes.push(AstNode {
                    node_type: AstNodeType::Type,
                    name: extract_name_from_line(line),
                    content: line.to_string(),
                    line: line_num + 1,
                    column: 1,
                    metadata: HashMap::new(),
                    children: Vec::new(),
                });
            } else if line.starts_with("var ")
                || line.starts_with("let ")
                || line.starts_with("const ")
            {
                variables_found += 1;
                root_nodes.push(AstNode {
                    node_type: AstNodeType::Variable,
                    name: extract_name_from_line(line),
                    content: line.to_string(),
                    line: line_num + 1,
                    column: 1,
                    metadata: HashMap::new(),
                    children: Vec::new(),
                });
            } else if line.starts_with("import ") || line.starts_with("use ") {
                root_nodes.push(AstNode {
                    node_type: AstNodeType::Import,
                    name: extract_name_from_line(line),
                    content: line.to_string(),
                    line: line_num + 1,
                    column: 1,
                    metadata: HashMap::new(),
                    children: Vec::new(),
                });
            } else if line.starts_with("//") || line.starts_with("/*") {
                root_nodes.push(AstNode {
                    node_type: AstNodeType::Comment,
                    name: "comment".to_string(),
                    content: line.to_string(),
                    line: line_num + 1,
                    column: 1,
                    metadata: HashMap::new(),
                    children: Vec::new(),
                });
            } else {
                // Unknown line type
                root_nodes.push(AstNode {
                    node_type: AstNodeType::Other("unknown".to_string()),
                    name: "unknown".to_string(),
                    content: line.to_string(),
                    line: line_num + 1,
                    column: 1,
                    metadata: HashMap::new(),
                    children: Vec::new(),
                });
            }
        }

        let processing_time = start_time.elapsed();

        Ok(AstParseResult {
            root_nodes,
            errors,
            warnings,
            statistics: AstParseStatistics {
                nodes_parsed,
                functions_found,
                types_found,
                variables_found,
                processing_time_ms: processing_time.as_millis() as u64,
            },
        })
    }

    fn clone_box(&self) -> Box<dyn AstParser> {
        Box::new(DefaultAstParser::new())
    }
}

impl Default for DefaultAstParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract name from a line of code
fn extract_name_from_line(line: &str) -> String {
    // Simple name extraction - can be overridden by specific implementations
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        // Remove any trailing parentheses or other syntax
        let name = parts[1];
        if let Some(open_paren) = name.find('(') {
            name[..open_paren].to_string()
        } else {
            name.to_string()
        }
    } else {
        "unknown".to_string()
    }
}

/// AST parser builder for creating custom parsers
pub struct AstParserBuilder {
    name: String,
    supported_extensions: Vec<String>,
}

impl AstParserBuilder {
    /// Create a new parser builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            supported_extensions: Vec::new(),
        }
    }

    /// Add supported extension
    pub fn with_extension(mut self, extension: String) -> Self {
        self.supported_extensions.push(extension);
        self
    }

    /// Add multiple supported extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.supported_extensions.extend(extensions);
        self
    }

    /// Build the parser
    pub fn build(self) -> BaseAstParser {
        BaseAstParser::new(self.name, self.supported_extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ast_parser_creation() {
        let parser = DefaultAstParser::new();
        assert_eq!(parser.name(), "Default");
        assert_eq!(parser.supported_extensions(), vec!["txt", "text"]);
    }

    #[test]
    fn test_extract_name_from_line() {
        assert_eq!(
            extract_name_from_line("func test_function()"),
            "test_function"
        );
        assert_eq!(extract_name_from_line("type MyStruct struct"), "MyStruct");
        assert_eq!(extract_name_from_line("var myVariable"), "myVariable");
    }

    #[tokio::test]
    async fn test_default_parser_can_parse() {
        let parser = DefaultAstParser::new();
        let path = Path::new("test.txt");

        assert!(parser.can_parse(path).await.unwrap());
    }

    #[tokio::test]
    async fn test_default_parser_parse_source() {
        let parser = DefaultAstParser::new();
        let source = "func test() {}\ntype MyType struct {}\nvar myVar = 1";

        let result = parser.parse_source(source, None).await.unwrap();

        assert_eq!(result.root_nodes.len(), 3);
        assert_eq!(result.statistics.functions_found, 1);
        assert_eq!(result.statistics.types_found, 1);
        assert_eq!(result.statistics.variables_found, 1);
    }
}
