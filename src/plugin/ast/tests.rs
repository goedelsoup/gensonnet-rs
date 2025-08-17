//! AST plugin tests

use super::*;
use crate::plugin::{Plugin, PluginCapability, PluginConfig, PluginContext};
use tempfile::TempDir;

#[tokio::test]
async fn test_go_ast_parser_basic() {
    let mut parser = GoAstParser::new();

    let test_content = r#"
package main

// TestStruct is a test struct
type TestStruct struct {
    Name string `json:"name"`
    Age  int    `json:"age,omitempty"`
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.go");
    tokio::fs::write(&test_file, test_content).await.unwrap();

    parser
        .parse_content(test_content, &test_file)
        .await
        .unwrap();

    let schemas = parser.extract_schemas();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].name, "TestStruct");
}

#[tokio::test]
async fn test_go_ast_parser_complex() {
    let mut parser = GoAstParser::new();

    let test_content = r#"
package main

import (
    "time"
    "k8s.io/apimachinery/pkg/apis/meta/v1"
)

// User represents a user in the system
type User struct {
    // Name is the unique identifier for the user
    Name string `json:"name" validate:"required"`
    
    // Age of the user
    Age int `json:"age,omitempty"`
    
    // Email address
    Email string `json:"email" validate:"email"`
    
    // Created timestamp
    CreatedAt time.Time `json:"createdAt"`
    
    // Metadata for the user
    Metadata v1.ObjectMeta `json:"metadata"`
    
    // Optional settings
    Settings *UserSettings `json:"settings,omitempty"`
    
    // List of roles
    Roles []string `json:"roles"`
    
    // Map of attributes
    Attributes map[string]string `json:"attributes"`
}

// UserSettings contains user preferences
type UserSettings struct {
    // Theme preference
    Theme string `json:"theme" default:"light"`
    
    // Notification settings
    Notifications bool `json:"notifications" default:"true"`
    
    // Language preference
    Language string `json:"language" default:"en"`
}

// UserService defines the interface for user operations
type UserService interface {
    // CreateUser creates a new user
    CreateUser(user *User) error
    
    // GetUser retrieves a user by name
    GetUser(name string) (*User, error)
    
    // UpdateUser updates an existing user
    UpdateUser(user *User) error
    
    // DeleteUser removes a user
    DeleteUser(name string) error
    
    // ListUsers returns all users
    ListUsers() ([]*User, error)
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.go");
    tokio::fs::write(&test_file, test_content).await.unwrap();

    parser
        .parse_content(test_content, &test_file)
        .await
        .unwrap();

    let schemas = parser.extract_schemas();
    assert_eq!(schemas.len(), 3); // User, UserSettings, UserService

    // Check that we have the expected types
    let schema_names: Vec<&str> = schemas.iter().map(|s| s.name.as_str()).collect();
    assert!(schema_names.contains(&"User"));
    assert!(schema_names.contains(&"UserSettings"));
    assert!(schema_names.contains(&"UserService"));

    // Check package info
    let package_info = parser.get_package_info();
    assert!(package_info.is_some());
    assert_eq!(package_info.unwrap().name, "main");

    // Check imports
    let nodes = parser.get_nodes();
    let imports: Vec<&ImportNode> = nodes
        .iter()
        .filter_map(|node| {
            if let GoAstNode::Import(import) = node {
                Some(import)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(imports.len(), 2);
    assert!(imports.iter().any(|i| i.path == "time"));
    assert!(imports
        .iter()
        .any(|i| i.path == "k8s.io/apimachinery/pkg/apis/meta/v1"));
}

#[tokio::test]
async fn test_go_ast_parser_with_comments() {
    let mut parser = GoAstParser::new();

    let test_content = r#"
package main

// This is a line comment
/* This is a block comment */

// User represents a user
type User struct {
    Name string // Inline comment
    Age  int    /* Another inline comment */
}
"#;

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.go");
    tokio::fs::write(&test_file, test_content).await.unwrap();

    parser
        .parse_content(test_content, &test_file)
        .await
        .unwrap();

    let nodes = parser.get_nodes();
    let comments: Vec<&CommentNode> = nodes
        .iter()
        .filter_map(|node| {
            if let GoAstNode::Comment(comment) = node {
                Some(comment)
            } else {
                None
            }
        })
        .collect();

    assert!(!comments.is_empty());
    assert!(comments
        .iter()
        .any(|c| c.text.contains("This is a line comment")));
    assert!(comments
        .iter()
        .any(|c| c.text.contains("This is a block comment")));
}

#[tokio::test]
async fn test_go_ast_plugin() {
    let config = PluginConfig {
        plugin_id: "test-go-plugin".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
    };

    let plugin = GoAstPlugin::new(config);
    let metadata = plugin.metadata();

    assert_eq!(metadata.name, "Go AST Plugin");
    assert!(metadata.supported_types.contains(&"go".to_string()));
}

#[tokio::test]
async fn test_go_ast_plugin_processing() {
    let config = PluginConfig {
        plugin_id: "test-go-plugin".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
    };

    let plugin = GoAstPlugin::new(config.clone());
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.go");

    let test_content = r#"
package main

type TestStruct struct {
    Name string `json:"name"`
    Age  int    `json:"age,omitempty"`
}
"#;

    tokio::fs::write(&test_file, &test_content).await.unwrap();

    let context = PluginContext::new(
        temp_dir.path().to_path_buf(),
        temp_dir.path().join("output"),
        config,
    );

    let result = plugin.process_source(&test_file, &context).await.unwrap();

    assert_eq!(result.schemas.len(), 1);
    assert_eq!(result.schemas[0].name, "TestStruct");
    assert_eq!(result.statistics.files_processed, 1);
    assert_eq!(result.statistics.schemas_extracted, 1);
}
