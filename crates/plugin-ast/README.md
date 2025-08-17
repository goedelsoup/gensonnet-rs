# Gensonnet Plugin AST Infrastructure

This crate provides the abstract AST (Abstract Syntax Tree) processing infrastructure for gensonnet plugins. It defines common interfaces and utilities that can be extended for different programming languages.

## Overview

The AST plugin infrastructure enables language-agnostic source code processing by providing:

- **Abstract AST Types**: Common data structures for representing AST nodes
- **Parser Traits**: Interfaces for language-specific parsers
- **Visitor Patterns**: Traversal and processing of AST nodes
- **Plugin Framework**: Integration with the main gensonnet plugin system

## Key Components

### AST Node Types

The crate defines common AST node types that can be extended:

```rust
pub enum AstNodeType {
    Function,    // Function or method definition
    Type,        // Type definition (struct, class, interface, etc.)
    Variable,    // Variable declaration
    Import,      // Import statement
    Package,     // Package/module declaration
    Comment,     // Comment
    Other(String), // Other/unknown node type
}
```

### AST Parser Trait

The `AstParser` trait defines the interface for language-specific parsers:

```rust
#[async_trait]
pub trait AstParser: Send + Sync {
    fn name(&self) -> &str;
    fn supported_extensions(&self) -> Vec<&str>;
    async fn can_parse(&self, file_path: &Path) -> Result<bool>;
    async fn parse_file(&self, file_path: &Path) -> Result<AstParseResult>;
    async fn parse_source(&self, source: &str, file_path: Option<&Path>) -> Result<AstParseResult>;
    fn clone_box(&self) -> Box<dyn AstParser>;
}
```

### AST Visitor Trait

The `AstVisitor` trait enables traversal and processing of AST nodes:

```rust
#[async_trait]
pub trait AstVisitor: Send + Sync {
    async fn visit_node(&mut self, node: &AstNode) -> Result<()>;
    async fn visit_function(&mut self, node: &AstNode) -> Result<()>;
    async fn visit_type(&mut self, node: &AstNode) -> Result<()>;
    async fn visit_variable(&mut self, node: &AstNode) -> Result<()>;
    async fn visit_import(&mut self, node: &AstNode) -> Result<()>;
    fn get_results(&self) -> AstVisitorResult;
    fn clone_box(&self) -> Box<dyn AstVisitor>;
}
```

### Abstract AST Plugin

The `AbstractAstPlugin` provides a complete plugin implementation that can be extended:

```rust
pub struct AbstractAstPlugin {
    parser: Box<dyn AstParser>,
    config: PluginConfig,
    visitor: Box<dyn AstVisitor>,
}
```

## Usage

### Creating a Language-Specific Parser

1. Implement the `AstParser` trait for your language:

```rust
use gensonnet_plugin_ast::*;

pub struct MyLanguageParser {
    // Language-specific parser implementation
}

#[async_trait]
impl AstParser for MyLanguageParser {
    fn name(&self) -> &str {
        "MyLanguage"
    }
    
    fn supported_extensions(&self) -> Vec<&str> {
        vec!["ml", "mylang"]
    }
    
    async fn can_parse(&self, file_path: &Path) -> Result<bool> {
        // Implementation
    }
    
    async fn parse_file(&self, file_path: &Path) -> Result<AstParseResult> {
        // Implementation
    }
    
    async fn parse_source(&self, source: &str, file_path: Option<&Path>) -> Result<AstParseResult> {
        // Implementation
    }
    
    fn clone_box(&self) -> Box<dyn AstParser> {
        Box::new(MyLanguageParser::new())
    }
}
```

### Creating a Custom Visitor

1. Implement the `AstVisitor` trait:

```rust
pub struct MyCustomVisitor {
    schemas: Vec<ExtractedSchema>,
    // Other visitor state
}

#[async_trait]
impl AstVisitor for MyCustomVisitor {
    async fn visit_node(&mut self, node: &AstNode) -> Result<()> {
        // Custom node processing
        Ok(())
    }
    
    // Implement other required methods...
    
    fn get_results(&self) -> AstVisitorResult {
        AstVisitorResult {
            schemas: self.schemas.clone(),
            // Other results
        }
    }
    
    fn clone_box(&self) -> Box<dyn AstVisitor> {
        Box::new(MyCustomVisitor::new())
    }
}
```

### Creating a Complete Plugin

1. Use the abstract plugin with your parser and visitor:

```rust
use gensonnet_plugin_ast::*;

pub struct MyLanguagePluginFactory;

#[async_trait]
impl PluginFactory for MyLanguagePluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        let parser = Box::new(MyLanguageParser::new());
        let visitor = Box::new(MyCustomVisitor::new());
        
        Ok(Box::new(AbstractAstPlugin::new(parser, visitor, config)))
    }
    
    fn supported_types(&self) -> Vec<String> {
        vec!["ml".to_string(), "mylang".to_string()]
    }
    
    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(MyLanguagePluginFactory)
    }
}
```

## Built-in Implementations

### Default Parser

The crate provides a `DefaultAstParser` that implements basic line-based parsing:

- Supports `.txt` and `.text` files
- Recognizes common patterns like `func`, `type`, `var`, `import`
- Extensible for custom patterns

### Default Visitor

The `DefaultAstVisitor` provides basic schema extraction:

- Extracts schemas from function, type, and variable nodes
- Tracks processing statistics
- Generates warnings for import statements

## Design Principles

- **Language Agnostic**: Core infrastructure works with any programming language
- **Extensible**: Easy to add new node types and processing logic
- **Async First**: All operations are async for better performance
- **Type Safe**: Strong typing throughout the AST processing pipeline
- **Composable**: Parsers and visitors can be combined and extended

## Future Enhancements

- Language-specific parser implementations
- Advanced AST analysis and transformation
- Code generation from AST nodes
- AST caching and incremental processing
- Cross-language AST comparison
