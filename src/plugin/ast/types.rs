//! AST type definitions

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Go AST node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoAstNode {
    /// Package declaration
    Package(PackageNode),

    /// Import declaration
    Import(ImportNode),

    /// Type declaration
    TypeDecl(TypeDeclNode),

    /// Struct type
    StructType(StructTypeNode),

    /// Interface type
    InterfaceType(InterfaceTypeNode),

    /// Field declaration
    Field(FieldNode),

    /// Method declaration
    Method(MethodNode),

    /// Comment
    Comment(CommentNode),
}

/// Package node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageNode {
    /// Package name
    pub name: String,

    /// Package path
    pub path: String,

    /// Position information
    pub position: Position,
}

/// Import node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportNode {
    /// Import path
    pub path: String,

    /// Alias (if any)
    pub alias: Option<String>,

    /// Position information
    pub position: Position,
}

/// Type declaration node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDeclNode {
    /// Type name
    pub name: String,

    /// Type definition
    pub type_def: TypeDefinition,

    /// Position information
    pub position: Position,

    /// Documentation comments
    pub docs: Vec<String>,
}

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDefinition {
    /// Struct type
    Struct(StructTypeNode),

    /// Interface type
    Interface(InterfaceTypeNode),

    /// Alias type
    Alias(String),

    /// Array type
    Array(Box<TypeDefinition>),

    /// Pointer type
    Pointer(Box<TypeDefinition>),

    /// Map type
    Map(Box<TypeDefinition>, Box<TypeDefinition>),

    /// Slice type
    Slice(Box<TypeDefinition>),

    /// Basic type
    Basic(String),
}

/// Struct type node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructTypeNode {
    /// Struct fields
    pub fields: Vec<FieldNode>,

    /// Embedded types
    pub embedded: Vec<String>,

    /// Position information
    pub position: Position,
}

/// Interface type node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceTypeNode {
    /// Interface methods
    pub methods: Vec<MethodNode>,

    /// Embedded interfaces
    pub embedded: Vec<String>,

    /// Position information
    pub position: Position,
}

/// Field node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldNode {
    /// Field names (can be multiple for embedded fields)
    pub names: Vec<String>,

    /// Field type
    pub field_type: TypeDefinition,

    /// Field tags
    pub tags: Option<String>,

    /// Documentation comments
    pub docs: Vec<String>,

    /// Position information
    pub position: Position,
}

/// Method node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodNode {
    /// Method name
    pub name: String,

    /// Receiver type
    pub receiver: Option<TypeDefinition>,

    /// Method parameters
    pub params: Vec<FieldNode>,

    /// Method results
    pub results: Vec<FieldNode>,

    /// Documentation comments
    pub docs: Vec<String>,

    /// Position information
    pub position: Position,
}

/// Comment node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentNode {
    /// Comment text
    pub text: String,

    /// Comment type
    pub comment_type: CommentType,

    /// Position information
    pub position: Position,
}

/// Comment type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentType {
    /// Line comment
    Line,

    /// Block comment
    Block,

    /// Documentation comment
    Doc,
}

/// Position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// File path
    pub file: PathBuf,

    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,

    /// Offset in file
    pub offset: usize,
}
