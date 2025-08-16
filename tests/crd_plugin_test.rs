use anyhow::Result;
use jsonnet_gen::plugin::*;
use jsonnet_gen::plugin::crd::{CrdPlugin, CrdPluginFactory};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_crd_plugin_integration() -> Result<()> {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new()?;
    let test_crd_file = temp_dir.path().join("test-crd.yaml");
    
    // Copy the test CRD file
    let source_crd = PathBuf::from("test-data/test-crd-plugin.yaml");
    tokio::fs::copy(&source_crd, &test_crd_file).await?;

    // Create plugin configuration
    let config = PluginConfig {
        plugin_id: "crd:test".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![
            PluginCapability::Parse,
            PluginCapability::SchemaExtraction,
            PluginCapability::Validation,
        ],
    };

    // Create plugin context
    let context = PluginContext::new(
        temp_dir.path().to_path_buf(),
        temp_dir.path().join("output"),
        config.clone(),
    );

    // Create CRD plugin
    let plugin = CrdPlugin::new(config);

    // Test that the plugin can handle the CRD file
    assert!(plugin.can_handle(&test_crd_file).await?);

    // Process the source
    let result = plugin.process_source(&test_crd_file, &context).await?;

    // Verify results
    assert_eq!(result.schemas.len(), 1);
    assert_eq!(result.schemas[0].name, "examples.test.com");
    assert_eq!(result.schemas[0].schema_type, "object");

    // Check metadata
    let metadata = &result.schemas[0].metadata;
    assert_eq!(metadata.get("group").and_then(|v| v.as_str()), Some("test.com"));
    assert_eq!(metadata.get("version").and_then(|v| v.as_str()), Some("v1"));
    assert_eq!(metadata.get("kind").and_then(|v| v.as_str()), Some("Example"));

    // Verify generated files
    assert!(!result.generated_files.is_empty());
    
    // Check that the output directory was created
    assert!(context.output_dir.exists());

    // Verify no errors
    assert!(result.errors.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_crd_plugin_factory() -> Result<()> {
    let factory = CrdPluginFactory;
    
    // Test supported types
    let supported_types = factory.supported_types();
    assert!(supported_types.contains(&"crd".to_string()));
    assert!(supported_types.contains(&"yaml".to_string()));
    assert!(supported_types.contains(&"yml".to_string()));

    // Test plugin creation
    let config = PluginConfig {
        plugin_id: "crd:test".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![
            PluginCapability::Parse,
            PluginCapability::SchemaExtraction,
        ],
    };

    let plugin = factory.create_plugin(config).await?;
    let metadata = plugin.metadata();
    
    assert_eq!(metadata.id, "crd:builtin");
    assert_eq!(metadata.name, "CRD Plugin");

    Ok(())
}
