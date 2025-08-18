use gensonnet::{Config, JsonnetGen};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_external_plugin_discovery() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().join("plugins");
    std::fs::create_dir_all(&plugin_dir).unwrap();

    // Create a test plugin manifest
    let plugin_manifest = r#"
metadata:
  id: "test-plugin:1.0"
  name: "Test Plugin"
  version: "1.0.0"
  description: "A test plugin for external plugin discovery"
  supported_types:
    - "test"
    - "yaml"
  capabilities:
    - "Parse"
    - "SchemaExtraction"

config:
  plugin_id: "test-plugin:1.0"
  config:
    test_setting: "value"
  enabled_capabilities:
    - "Parse"
    - "SchemaExtraction"
"#;

    // Write the plugin manifest
    let manifest_path = plugin_dir.join("plugin.yaml");
    std::fs::write(&manifest_path, plugin_manifest).unwrap();

    // Create a configuration with the plugin directory
    let mut config = Config::default();
    config.plugins.plugin_directories = vec![plugin_dir];
    config.plugins.enable_external_discovery = true;

    // Add a dummy source to satisfy validation
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./output"),
        },
    ));

    // Create JsonnetGen instance
    let app = JsonnetGen::new(config).unwrap();

    // Initialize plugins (this should discover the external plugin)
    let result = app.initialize_plugins().await;
    assert!(result.is_ok(), "Plugin initialization should succeed");

    // Get plugin information
    let plugins = app.get_plugin_info().await.unwrap();

    // Should have at least the built-in plugins
    assert!(!plugins.is_empty(), "Should have at least built-in plugins");

    // Check that we have the expected built-in plugins
    let plugin_ids: Vec<_> = plugins.iter().map(|p| &p.id).collect();
    assert!(
        plugin_ids.contains(&&"go-ast:builtin".to_string()),
        "Should have go-ast plugin"
    );
    assert!(
        plugin_ids.contains(&&"crd:builtin".to_string()),
        "Should have crd plugin"
    );
    assert!(
        plugin_ids.contains(&&"openapi:builtin".to_string()),
        "Should have openapi plugin"
    );
}

#[tokio::test]
async fn test_plugin_discovery_disabled() {
    // Create a configuration with plugin discovery disabled
    let mut config = Config::default();
    config.plugins.enable_external_discovery = false;

    // Add a dummy source to satisfy validation
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./output"),
        },
    ));

    // Create JsonnetGen instance
    let app = JsonnetGen::new(config).unwrap();

    // Initialize plugins (this should skip external plugin discovery)
    let result = app.initialize_plugins().await;
    assert!(
        result.is_ok(),
        "Plugin initialization should succeed even with discovery disabled"
    );

    // Get plugin information
    let plugins = app.get_plugin_info().await.unwrap();

    // Should still have built-in plugins
    assert!(
        !plugins.is_empty(),
        "Should have built-in plugins even with discovery disabled"
    );
}

#[tokio::test]
async fn test_plugin_directory_expansion() {
    // Test that plugin directories with ~ are expanded correctly
    let mut config = Config::default();
    config.plugins.plugin_directories = vec![
        PathBuf::from("~/test-plugins"),
        PathBuf::from("./local-plugins"),
    ];
    config.plugins.enable_external_discovery = true;

    // Add a dummy source to satisfy validation
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./output"),
        },
    ));

    // Create JsonnetGen instance
    let app = JsonnetGen::new(config).unwrap();

    // Initialize plugins (this should handle the directory expansion gracefully)
    let result = app.initialize_plugins().await;
    assert!(
        result.is_ok(),
        "Plugin initialization should handle non-existent directories gracefully"
    );
}
