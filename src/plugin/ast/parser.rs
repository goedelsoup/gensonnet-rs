//! Go AST parser implementation

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{Language, Node, Parser};

use super::types::*;
use crate::plugin::*;

/// Go AST parser using tree-sitter
pub struct GoAstParser {
    /// Tree-sitter parser
    parser: Parser,

    /// Go language
    language: Language,

    /// Parsed AST nodes
    nodes: Vec<GoAstNode>,

    /// Type definitions
    type_defs: HashMap<String, TypeDefinition>,

    /// Package information
    package_info: Option<PackageNode>,
}

impl Default for GoAstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GoAstParser {
    /// Create a new Go AST parser
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_go::language();
        parser.set_language(language).unwrap();

        Self {
            parser,
            language,
            nodes: Vec::new(),
            type_defs: HashMap::new(),
            package_info: None,
        }
    }

    /// Parse a Go source file
    pub async fn parse_file(&mut self, file_path: &Path) -> Result<()> {
        let content = tokio::fs::read_to_string(file_path).await?;
        self.parse_content(&content, file_path).await
    }

    /// Parse Go source content using tree-sitter
    pub async fn parse_content(&mut self, content: &str, file_path: &Path) -> Result<()> {
        // Clear previous state
        self.nodes.clear();
        self.type_defs.clear();
        self.package_info = None;

        // Parse with tree-sitter
        let tree = self.parser.parse(content, None).unwrap();
        let root_node = tree.root_node();

        // Extract package information
        self.extract_package_info(&root_node, file_path, content)?;

        // Extract type declarations
        self.extract_type_declarations(&root_node, file_path, content)?;

        // Extract function declarations (including methods)
        self.extract_function_declarations(&root_node, file_path, content)?;

        // Extract imports
        self.extract_imports(&root_node, file_path, content)?;

        // Extract comments
        self.extract_comments(&root_node, file_path, content)?;

        Ok(())
    }

    /// Extract package information from AST
    fn extract_package_info(
        &mut self,
        root_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut cursor = root_node.walk();

        for node in root_node.children(&mut cursor) {
            if node.kind() == "package_clause" {
                for child in node.children(&mut node.walk()) {
                    if child.kind() == "package_identifier" {
                        let package_name = self.get_node_text(child, content);

                        self.package_info = Some(PackageNode {
                            name: package_name,
                            path: file_path.to_string_lossy().to_string(),
                            position: self.node_to_position(child, file_path),
                        });
                        break;
                    }
                }
                break;
            }
        }

        Ok(())
    }

    /// Extract type declarations from AST
    fn extract_type_declarations(
        &mut self,
        root_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut cursor = root_node.walk();

        for node in root_node.children(&mut cursor) {
            if node.kind() == "type_declaration" {
                self.process_type_declaration(&node, file_path, content)?;
            }
        }

        Ok(())
    }

    /// Process a type declaration node
    fn process_type_declaration(
        &mut self,
        type_decl_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut cursor = type_decl_node.walk();

        // Process type specs directly
        for child in type_decl_node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                self.process_type_spec(&child, file_path, content)?;
            } else if child.kind() == "type_spec_list" {
                for spec in child.children(&mut child.walk()) {
                    if spec.kind() == "type_spec" {
                        self.process_type_spec(&spec, file_path, content)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process a type spec node
    fn process_type_spec(
        &mut self,
        type_spec: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut name_node = None;
        let mut type_node = None;
        let mut cursor = type_spec.walk();

        // Extract name and type
        for child in type_spec.children(&mut cursor) {
            match child.kind() {
                "type_identifier" => name_node = Some(child),
                "struct_type" | "interface_type" | "array_type" | "pointer_type" | "map_type"
                | "slice_type" | "channel_type" | "function_type" => {
                    type_node = Some(child);
                }
                _ => {}
            }
        }

        if let (Some(name), Some(type_def_node)) = (name_node, type_node) {
            let type_name = self.get_node_text(name, content);
            let type_definition = self.parse_type_definition(&type_def_node, content)?;

            let type_decl = TypeDeclNode {
                name: type_name.clone(),
                type_def: type_definition.clone(),
                position: self.node_to_position(name, file_path),
                docs: self.extract_documentation(type_spec, content),
            };

            self.nodes.push(GoAstNode::TypeDecl(type_decl));
            self.type_defs.insert(type_name.clone(), type_definition);
        }

        Ok(())
    }

    /// Parse type definition from AST node
    fn parse_type_definition(&self, type_node: &Node, content: &str) -> Result<TypeDefinition> {
        match type_node.kind() {
            "struct_type" => self.parse_struct_type(type_node, content),
            "interface_type" => self.parse_interface_type(type_node, content),
            "array_type" => self.parse_array_type(type_node, content),
            "pointer_type" => self.parse_pointer_type(type_node, content),
            "map_type" => self.parse_map_type(type_node, content),
            "slice_type" => self.parse_slice_type(type_node, content),
            "type_identifier" => Ok(TypeDefinition::Basic(
                self.get_node_text(*type_node, content),
            )),
            _ => Ok(TypeDefinition::Basic("unknown".to_string())),
        }
    }

    /// Parse struct type
    fn parse_struct_type(&self, struct_node: &Node, content: &str) -> Result<TypeDefinition> {
        let mut fields = Vec::new();
        let mut embedded = Vec::new();
        let mut cursor = struct_node.walk();

        // Find field list
        for child in struct_node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                for field_decl in child.children(&mut child.walk()) {
                    if field_decl.kind() == "field_declaration" {
                        let field = self.parse_field_declaration(&field_decl, content)?;
                        if field.names.is_empty() {
                            // This is an embedded field
                            if let TypeDefinition::Basic(type_name) = &field.field_type {
                                embedded.push(type_name.clone());
                            }
                        } else {
                            fields.push(field);
                        }
                    }
                }
            }
        }

        let struct_type = StructTypeNode {
            fields,
            embedded,
            position: self.node_to_position(*struct_node, &PathBuf::new()),
        };

        Ok(TypeDefinition::Struct(struct_type))
    }

    /// Parse interface type
    fn parse_interface_type(&self, interface_node: &Node, content: &str) -> Result<TypeDefinition> {
        let mut methods = Vec::new();
        let embedded = Vec::new();
        let mut cursor = interface_node.walk();

        // Find method list
        for child in interface_node.children(&mut cursor) {
            if child.kind() == "method_spec_list" {
                for method_spec in child.children(&mut child.walk()) {
                    if method_spec.kind() == "method_spec" {
                        let method = self.parse_method_spec(&method_spec, content)?;
                        methods.push(method);
                    }
                }
            }
        }

        let interface_type = InterfaceTypeNode {
            methods,
            embedded,
            position: self.node_to_position(*interface_node, &PathBuf::new()),
        };

        Ok(TypeDefinition::Interface(interface_type))
    }

    /// Parse array type
    fn parse_array_type(&self, array_node: &Node, content: &str) -> Result<TypeDefinition> {
        let mut cursor = array_node.walk();

        for child in array_node.children(&mut cursor) {
            if child.kind() == "element_type" {
                let element_type = self.parse_type_definition(&child, content)?;
                return Ok(TypeDefinition::Array(Box::new(element_type)));
            }
        }

        Ok(TypeDefinition::Array(Box::new(TypeDefinition::Basic(
            "unknown".to_string(),
        ))))
    }

    /// Parse pointer type
    fn parse_pointer_type(&self, pointer_node: &Node, content: &str) -> Result<TypeDefinition> {
        let mut cursor = pointer_node.walk();

        for child in pointer_node.children(&mut cursor) {
            if child.kind() == "base_type" {
                let base_type = self.parse_type_definition(&child, content)?;
                return Ok(TypeDefinition::Pointer(Box::new(base_type)));
            } else if child.kind() == "type_identifier" {
                let type_name = self.get_node_text(child, content);
                return Ok(TypeDefinition::Pointer(Box::new(TypeDefinition::Basic(
                    type_name,
                ))));
            }
        }

        Ok(TypeDefinition::Pointer(Box::new(TypeDefinition::Basic(
            "unknown".to_string(),
        ))))
    }

    /// Parse map type
    fn parse_map_type(&self, map_node: &Node, content: &str) -> Result<TypeDefinition> {
        let mut key_type = None;
        let mut value_type = None;
        let mut cursor = map_node.walk();

        for child in map_node.children(&mut cursor) {
            match child.kind() {
                "key_type" => key_type = Some(self.parse_type_definition(&child, content)?),
                "value_type" => value_type = Some(self.parse_type_definition(&child, content)?),
                _ => {}
            }
        }

        let key = key_type.unwrap_or(TypeDefinition::Basic("string".to_string()));
        let value = value_type.unwrap_or(TypeDefinition::Basic("unknown".to_string()));

        Ok(TypeDefinition::Map(Box::new(key), Box::new(value)))
    }

    /// Parse slice type
    fn parse_slice_type(&self, slice_node: &Node, content: &str) -> Result<TypeDefinition> {
        let mut cursor = slice_node.walk();

        for child in slice_node.children(&mut cursor) {
            if child.kind() == "element_type" {
                let element_type = self.parse_type_definition(&child, content)?;
                return Ok(TypeDefinition::Slice(Box::new(element_type)));
            }
        }

        Ok(TypeDefinition::Slice(Box::new(TypeDefinition::Basic(
            "unknown".to_string(),
        ))))
    }

    /// Parse field declaration
    fn parse_field_declaration(&self, field_decl: &Node, content: &str) -> Result<FieldNode> {
        let mut names = Vec::new();
        let mut field_type = TypeDefinition::Basic("unknown".to_string());
        let mut tags = None;
        let mut cursor = field_decl.walk();

        for child in field_decl.children(&mut cursor) {
            match child.kind() {
                "field_identifier_list" => {
                    for name_node in child.children(&mut child.walk()) {
                        if name_node.kind() == "field_identifier" {
                            names.push(self.get_node_text(name_node, content));
                        }
                    }
                }
                "type" => {
                    field_type = self.parse_type_definition(&child, content)?;
                }
                "raw_string_literal" | "interpreted_string_literal" => {
                    tags = Some(
                        self.get_node_text(child, content)
                            .trim_matches('"')
                            .to_string(),
                    );
                }
                _ => {}
            }
        }

        Ok(FieldNode {
            names,
            field_type,
            tags,
            docs: self.extract_documentation(field_decl, content),
            position: self.node_to_position(*field_decl, &PathBuf::new()),
        })
    }

    /// Parse method specification
    fn parse_method_spec(&self, method_spec: &Node, content: &str) -> Result<MethodNode> {
        let mut name = String::new();
        let mut params = Vec::new();
        let mut results = Vec::new();
        let mut cursor = method_spec.walk();

        for child in method_spec.children(&mut cursor) {
            match child.kind() {
                "field_identifier" => {
                    name = self.get_node_text(child, content);
                }
                "parameter_list" => {
                    params = self.parse_parameter_list(&child, content)?;
                }
                "result" => {
                    results = self.parse_result_list(&child, content)?;
                }
                _ => {}
            }
        }

        Ok(MethodNode {
            name,
            receiver: None, // Method specifications in interfaces don't have receivers
            params,
            results,
            docs: self.extract_documentation(method_spec, content),
            position: self.node_to_position(*method_spec, &PathBuf::new()),
        })
    }

    /// Parse parameter list
    fn parse_parameter_list(&self, param_list: &Node, content: &str) -> Result<Vec<FieldNode>> {
        let mut params = Vec::new();
        let mut cursor = param_list.walk();

        for child in param_list.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let param = self.parse_parameter_declaration(&child, content)?;
                params.push(param);
            }
        }

        Ok(params)
    }

    /// Parse parameter declaration
    fn parse_parameter_declaration(&self, param_decl: &Node, content: &str) -> Result<FieldNode> {
        let mut names = Vec::new();
        let mut param_type = TypeDefinition::Basic("unknown".to_string());
        let mut cursor = param_decl.walk();

        for child in param_decl.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    names.push(self.get_node_text(child, content));
                }
                "type" => {
                    param_type = self.parse_type_definition(&child, content)?;
                }
                _ => {}
            }
        }

        Ok(FieldNode {
            names,
            field_type: param_type,
            tags: None,
            docs: Vec::new(),
            position: self.node_to_position(*param_decl, &PathBuf::new()),
        })
    }

    /// Parse result list
    fn parse_result_list(&self, result_list: &Node, content: &str) -> Result<Vec<FieldNode>> {
        let mut results = Vec::new();
        let mut cursor = result_list.walk();

        for child in result_list.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let result = self.parse_parameter_declaration(&child, content)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Extract function declarations from AST
    fn extract_function_declarations(
        &mut self,
        root_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut cursor = root_node.walk();

        for node in root_node.children(&mut cursor) {
            if node.kind() == "function_declaration" {
                self.process_function_declaration(&node, file_path, content)?;
            } else if node.kind() == "method_declaration" {
                self.process_method_declaration(&node, file_path, content)?;
            }
        }

        Ok(())
    }

    /// Process a function declaration node
    fn process_function_declaration(
        &mut self,
        func_decl_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut name = String::new();
        let mut receiver = None;
        let mut params = Vec::new();
        let mut results = Vec::new();
        let mut cursor = func_decl_node.walk();

        for child in func_decl_node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    name = self.get_node_text(child, content);
                }
                "parameter_list" => {
                    let mut param_cursor = child.walk();
                    let mut first_param = true;

                    for param in child.children(&mut param_cursor) {
                        if param.kind() == "parameter_declaration" {
                            if first_param {
                                // Check if this is a receiver parameter
                                let receiver_info =
                                    self.parse_receiver_parameter(&param, content)?;
                                if receiver_info.is_some() {
                                    receiver = receiver_info;
                                    first_param = false;
                                    continue;
                                }
                            }

                            let param_node = self.parse_parameter_declaration(&param, content)?;
                            params.push(param_node);
                            first_param = false;
                        }
                    }
                }
                "result" => {
                    results = self.parse_result_list(&child, content)?;
                }
                _ => {}
            }
        }

        // Only create a method node if there's a receiver
        if receiver.is_some() {
            let method_node = MethodNode {
                name,
                receiver,
                params,
                results,
                docs: self.extract_documentation(func_decl_node, content),
                position: self.node_to_position(*func_decl_node, file_path),
            };

            self.nodes.push(GoAstNode::Method(method_node));
        }

        Ok(())
    }

    /// Process a method declaration node
    fn process_method_declaration(
        &mut self,
        method_decl_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut name = String::new();
        let mut receiver = None;
        let mut params = Vec::new();
        let mut results = Vec::new();
        let mut cursor = method_decl_node.walk();

        for child in method_decl_node.children(&mut cursor) {
            match child.kind() {
                "field_identifier" => {
                    name = self.get_node_text(child, content);
                }
                "parameter_list" => {
                    // Parse regular parameters (not receiver)
                    params = self.parse_parameter_list(&child, content)?;
                }
                "result" => {
                    results = self.parse_result_list(&child, content)?;
                }
                _ => {}
            }
        }

        // For method declarations, we need to parse the receiver from the parent context
        // The receiver is typically the first parameter in the method declaration
        // Let me check if there's a receiver parameter list
        let mut receiver_cursor = method_decl_node.walk();
        for child in method_decl_node.children(&mut receiver_cursor) {
            if child.kind() == "parameter_list" {
                // Check if this parameter list contains a receiver
                let mut param_cursor = child.walk();
                for param in child.children(&mut param_cursor) {
                    if param.kind() == "parameter_declaration" {
                        let receiver_info = self.parse_receiver_parameter(&param, content)?;
                        if receiver_info.is_some() {
                            receiver = receiver_info;
                            break;
                        }
                    }
                }
                break;
            }
        }

        // Create method node
        let method_node = MethodNode {
            name,
            receiver,
            params,
            results,
            docs: self.extract_documentation(method_decl_node, content),
            position: self.node_to_position(*method_decl_node, file_path),
        };

        self.nodes.push(GoAstNode::Method(method_node));

        Ok(())
    }

    /// Parse receiver parameter from a parameter declaration
    fn parse_receiver_parameter(
        &self,
        param_decl: &Node,
        content: &str,
    ) -> Result<Option<TypeDefinition>> {
        let mut names = Vec::new();
        let mut param_type = TypeDefinition::Basic("unknown".to_string());
        let mut cursor = param_decl.walk();

        for child in param_decl.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let name = self.get_node_text(child, content);
                    names.push(name);
                }
                "type" => {
                    param_type = self.parse_type_definition(&child, content)?;
                }
                "type_identifier" => {
                    let type_name = self.get_node_text(child, content);
                    param_type = TypeDefinition::Basic(type_name.clone());
                }
                "pointer_type" => {
                    param_type = self.parse_type_definition(&child, content)?;
                }
                _ => {}
            }
        }

        // In Go, receivers are written as (t Type) or (Type)
        // The receiver is always a single parameter with a type
        // We identify it by checking if it has a valid type (not "unknown")
        match &param_type {
            TypeDefinition::Basic(type_name) => {
                if !type_name.is_empty() && type_name != "unknown" {
                    return Ok(Some(param_type));
                }
            }
            TypeDefinition::Pointer(inner_type) => {
                if let TypeDefinition::Basic(type_name) = inner_type.as_ref() {
                    if !type_name.is_empty() && type_name != "unknown" {
                        return Ok(Some(param_type));
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Extract imports from AST
    fn extract_imports(&mut self, root_node: &Node, file_path: &Path, content: &str) -> Result<()> {
        let mut cursor = root_node.walk();

        for node in root_node.children(&mut cursor) {
            if node.kind() == "import_declaration" {
                self.process_import_declaration(&node, file_path, content)?;
            }
        }

        Ok(())
    }

    /// Process import declaration
    fn process_import_declaration(
        &mut self,
        import_decl: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        for child in import_decl.children(&mut import_decl.walk()) {
            if child.kind() == "import_spec_list" {
                for import_spec in child.children(&mut child.walk()) {
                    if import_spec.kind() == "import_spec" {
                        self.process_import_spec(&import_spec, file_path, content)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Process import specification
    fn process_import_spec(
        &mut self,
        import_spec: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        let mut path = String::new();
        let mut alias = None;
        let mut cursor = import_spec.walk();

        for child in import_spec.children(&mut cursor) {
            match child.kind() {
                "package_identifier" => {
                    alias = Some(self.get_node_text(child, content));
                }
                "interpreted_string_literal" => {
                    path = self
                        .get_node_text(child, content)
                        .trim_matches('"')
                        .to_string();
                }
                _ => {}
            }
        }

        let import_node = ImportNode {
            path,
            alias,
            position: self.node_to_position(*import_spec, file_path),
        };

        self.nodes.push(GoAstNode::Import(import_node));
        Ok(())
    }

    /// Extract comments from AST
    fn extract_comments(
        &mut self,
        _root_node: &Node,
        file_path: &Path,
        content: &str,
    ) -> Result<()> {
        // Tree-sitter doesn't include comments in the AST by default
        // We'll extract them from the source text
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("//") {
                let comment = CommentNode {
                    text: trimmed[2..].trim().to_string(),
                    comment_type: CommentType::Line,
                    position: Position {
                        file: file_path.to_path_buf(),
                        line: line_num + 1,
                        column: 1,
                        offset: 0,
                    },
                };
                self.nodes.push(GoAstNode::Comment(comment));
            } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
                let comment = CommentNode {
                    text: trimmed[2..trimmed.len() - 2].trim().to_string(),
                    comment_type: CommentType::Block,
                    position: Position {
                        file: file_path.to_path_buf(),
                        line: line_num + 1,
                        column: 1,
                        offset: 0,
                    },
                };
                self.nodes.push(GoAstNode::Comment(comment));
            }
        }

        Ok(())
    }

    /// Extract documentation comments for a node
    fn extract_documentation(&self, node: &Node, content: &str) -> Vec<String> {
        let mut docs = Vec::new();
        let node_start = node.start_byte();

        // Look for comments before the node
        let lines: Vec<&str> = content.lines().collect();
        let mut current_byte = 0;

        for line in lines.iter() {
            let line_bytes = line.len() + 1; // +1 for newline
            let next_byte = current_byte + line_bytes;

            if next_byte > node_start {
                break;
            }

            let trimmed = line.trim();
            if trimmed.starts_with("//") && !trimmed.starts_with("//go:") {
                docs.push(trimmed[2..].trim().to_string());
            }

            current_byte = next_byte;
        }

        docs
    }

    /// Get text content of a node
    fn get_node_text(&self, node: Node, content: &str) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        content[start..end].to_string()
    }

    /// Convert tree-sitter node to position
    fn node_to_position(&self, node: Node, file_path: &Path) -> Position {
        Position {
            file: file_path.to_path_buf(),
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
            offset: node.start_byte(),
        }
    }

    /// Get all parsed nodes
    pub fn get_nodes(&self) -> &[GoAstNode] {
        &self.nodes
    }

    /// Get type definitions
    pub fn get_type_defs(&self) -> &HashMap<String, TypeDefinition> {
        &self.type_defs
    }

    /// Get package information
    pub fn get_package_info(&self) -> Option<&PackageNode> {
        self.package_info.as_ref()
    }

    /// Extract schemas from AST
    pub fn extract_schemas(&self) -> Vec<ExtractedSchema> {
        let mut schemas = Vec::new();

        for node in &self.nodes {
            if let GoAstNode::TypeDecl(type_decl) = node {
                let schema = self.type_decl_to_schema(type_decl);
                schemas.push(schema);
            }
        }

        schemas
    }

    /// Convert type declaration to schema
    fn type_decl_to_schema(&self, type_decl: &TypeDeclNode) -> ExtractedSchema {
        let mut metadata = HashMap::new();
        metadata.insert(
            "package".to_string(),
            serde_yaml::Value::String(
                self.package_info
                    .as_ref()
                    .map(|p| p.name.clone())
                    .unwrap_or_default(),
            ),
        );
        metadata.insert(
            "docs".to_string(),
            serde_yaml::Value::Sequence(
                type_decl
                    .docs
                    .iter()
                    .map(|d| serde_yaml::Value::String(d.clone()))
                    .collect(),
            ),
        );

        let schema_content = match &type_decl.type_def {
            TypeDefinition::Struct(struct_type) => self.struct_to_schema(struct_type),
            TypeDefinition::Interface(interface_type) => self.interface_to_schema(interface_type),
            _ => serde_yaml::Value::Null,
        };

        ExtractedSchema {
            name: type_decl.name.clone(),
            schema_type: "go_struct".to_string(),
            content: schema_content,
            source_file: type_decl.position.file.clone(),
            metadata,
        }
    }

    /// Convert struct type to schema
    fn struct_to_schema(&self, struct_type: &StructTypeNode) -> serde_yaml::Value {
        let mut properties = serde_yaml::Mapping::new();
        let mut required = Vec::new();

        for field in &struct_type.fields {
            for name in &field.names {
                let field_schema = self.field_to_schema(field);
                properties.insert(serde_yaml::Value::String(name.clone()), field_schema);

                // Check if field is required (no pointer, no omitempty tag)
                if !self.field_is_optional(field) {
                    required.push(name.clone());
                }
            }
        }

        let mut schema = serde_yaml::Mapping::new();
        schema.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String("object".to_string()),
        );
        schema.insert(
            serde_yaml::Value::String("properties".to_string()),
            serde_yaml::Value::Mapping(properties),
        );

        if !required.is_empty() {
            schema.insert(
                serde_yaml::Value::String("required".to_string()),
                serde_yaml::Value::Sequence(
                    required
                        .iter()
                        .map(|r| serde_yaml::Value::String(r.clone()))
                        .collect(),
                ),
            );
        }

        serde_yaml::Value::Mapping(schema)
    }

    /// Convert interface type to schema
    fn interface_to_schema(&self, _interface_type: &InterfaceTypeNode) -> serde_yaml::Value {
        let mut schema = serde_yaml::Mapping::new();
        schema.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String("object".to_string()),
        );

        // For interfaces, we might want to generate different schemas
        // depending on the use case. For now, we'll create a basic object schema.

        serde_yaml::Value::Mapping(schema)
    }

    /// Convert field to schema
    fn field_to_schema(&self, field: &FieldNode) -> serde_yaml::Value {
        let mut schema = serde_yaml::Mapping::new();

        let field_type = self.type_def_to_schema_type(&field.field_type);
        schema.insert(
            serde_yaml::Value::String("type".to_string()),
            serde_yaml::Value::String(field_type),
        );

        // Add description from docs
        if !field.docs.is_empty() {
            schema.insert(
                serde_yaml::Value::String("description".to_string()),
                serde_yaml::Value::String(field.docs.join(" ")),
            );
        }

        serde_yaml::Value::Mapping(schema)
    }

    /// Convert type definition to schema type
    fn type_def_to_schema_type(&self, type_def: &TypeDefinition) -> String {
        match type_def {
            TypeDefinition::Basic(basic_type) => match basic_type.as_str() {
                "string" => "string".to_string(),
                "int" | "int8" | "int16" | "int32" | "int64" => "integer".to_string(),
                "uint" | "uint8" | "uint16" | "uint32" | "uint64" => "integer".to_string(),
                "float32" | "float64" => "number".to_string(),
                "bool" => "boolean".to_string(),
                _ => "string".to_string(),
            },
            TypeDefinition::Array(_) => "array".to_string(),
            TypeDefinition::Slice(_) => "array".to_string(),
            TypeDefinition::Map(_, _) => "object".to_string(),
            TypeDefinition::Pointer(_) => "object".to_string(),
            TypeDefinition::Struct(_) => "object".to_string(),
            TypeDefinition::Interface(_) => "object".to_string(),
            TypeDefinition::Alias(_) => "string".to_string(),
        }
    }

    /// Check if field is optional
    fn field_is_optional(&self, field: &FieldNode) -> bool {
        // Check for pointer type
        if let TypeDefinition::Pointer(_) = field.field_type {
            return true;
        }

        // Check for omitempty tag
        if let Some(tags) = &field.tags {
            return tags.contains("omitempty");
        }

        false
    }
}
