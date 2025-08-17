//! CRD plugin tests

use super::*;
use crate::plugin::{Plugin, PluginCapability, PluginConfig, PluginFactory};
use tempfile::TempDir;

#[tokio::test]
async fn test_crd_plugin_creation() {
    let config = PluginConfig {
        plugin_id: "crd:test".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
    };

    let plugin = CrdPlugin::new(config);
    let metadata = plugin.metadata();

    assert_eq!(metadata.id, "crd:builtin");
    assert_eq!(metadata.name, "CRD Plugin");
    assert!(metadata.supported_types.contains(&"yaml".to_string()));
}

#[tokio::test]
async fn test_crd_plugin_factory() {
    let factory = CrdPluginFactory;
    let config = PluginConfig {
        plugin_id: "crd:test".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
    };

    let plugin = factory.create_plugin(config).await.unwrap();
    let metadata = plugin.metadata();

    assert_eq!(metadata.id, "crd:builtin");
    assert!(factory.supported_types().contains(&"crd".to_string()));
}

#[tokio::test]
async fn test_crd_plugin_can_handle() {
    let config = PluginConfig {
        plugin_id: "crd:test".to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![PluginCapability::Parse, PluginCapability::SchemaExtraction],
    };

    let plugin = CrdPlugin::new(config);
    let temp_dir = TempDir::new().unwrap();

    // Test with non-CRD file
    let non_crd_file = temp_dir.path().join("test.txt");
    tokio::fs::write(&non_crd_file, "This is not a CRD")
        .await
        .unwrap();
    assert!(!plugin.can_handle(&non_crd_file).await.unwrap());

    // Test with CRD file
    let crd_content = r#"
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: examples.test.com
spec:
  group: test.com
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
"#;
    let crd_file = temp_dir.path().join("test-crd.yaml");
    tokio::fs::write(&crd_file, crd_content).await.unwrap();
    assert!(plugin.can_handle(&crd_file).await.unwrap());
}
