//! Plugin architecture for extensible source processing

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod ast;
pub mod crd;
pub mod openapi;
pub mod registry;
pub mod testing;
pub mod traits;

pub use ast::*;
pub use crd::*;
pub use openapi::*;
pub use registry::*;
pub use testing::*;
pub use traits::*;

/// Plugin identifier
pub type PluginId = String;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin identifier
    pub id: PluginId,

    /// Plugin name
    pub name: String,

    /// Plugin version
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Supported source types
    pub supported_types: Vec<String>,

    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginCapability {
    /// Can parse source files
    Parse,

    /// Can extract schemas
    SchemaExtraction,

    /// Can validate schemas
    Validation,

    /// Can generate code
    CodeGeneration,

    /// Can process AST
    AstProcessing,

    /// Can handle dependencies
    DependencyResolution,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin identifier
    pub plugin_id: PluginId,

    /// Plugin-specific configuration
    pub config: serde_yaml::Value,

    /// Enabled capabilities
    pub enabled_capabilities: Vec<PluginCapability>,
}

/// Plugin context for processing
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Working directory
    pub working_dir: PathBuf,

    /// Output directory
    pub output_dir: PathBuf,

    /// Plugin configuration
    pub config: PluginConfig,

    /// Shared state between plugins
    pub shared_state: Arc<RwLock<HashMap<String, serde_yaml::Value>>>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(working_dir: PathBuf, output_dir: PathBuf, config: PluginConfig) -> Self {
        Self {
            working_dir,
            output_dir,
            config,
            shared_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a value from shared state
    pub async fn get_shared_value(&self, key: &str) -> Option<serde_yaml::Value> {
        self.shared_state.read().await.get(key).cloned()
    }

    /// Set a value in shared state
    pub async fn set_shared_value(&self, key: String, value: serde_yaml::Value) {
        self.shared_state.write().await.insert(key, value);
    }
}

/// Plugin result
#[derive(Debug, Clone)]
pub struct PluginResult {
    /// Extracted schemas
    pub schemas: Vec<ExtractedSchema>,

    /// Generated files
    pub generated_files: Vec<PathBuf>,

    /// Processing statistics
    pub statistics: PluginStatistics,

    /// Warnings
    pub warnings: Vec<String>,

    /// Errors
    pub errors: Vec<String>,
}

/// Extracted schema from plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedSchema {
    /// Schema name
    pub name: String,

    /// Schema type
    pub schema_type: String,

    /// Schema content
    pub content: serde_yaml::Value,

    /// Source file
    pub source_file: PathBuf,

    /// Metadata
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Plugin processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatistics {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Number of files processed
    pub files_processed: usize,

    /// Number of schemas extracted
    pub schemas_extracted: usize,

    /// Number of files generated
    pub files_generated: usize,
}

/// Plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin
    async fn initialize(&self, context: &PluginContext) -> Result<()>;

    /// Check if the plugin can handle the given source
    async fn can_handle(&self, source_path: &Path) -> Result<bool>;

    /// Process a source and extract schemas
    async fn process_source(
        &self,
        source_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult>;

    /// Generate code from extracted schemas
    async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &PluginContext,
    ) -> Result<Vec<PathBuf>>;

    /// Clean up plugin resources
    async fn cleanup(&self, context: &PluginContext) -> Result<()>;

    /// Clone the plugin as a boxed trait object
    fn clone_box(&self) -> Box<dyn Plugin>;
}

/// Plugin factory trait for creating plugins
#[async_trait]
pub trait PluginFactory: Send + Sync {
    /// Create a new plugin instance
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>>;

    /// Get supported plugin types
    fn supported_types(&self) -> Vec<String>;

    /// Clone the factory as a boxed trait object
    fn clone_box(&self) -> Box<dyn PluginFactory>;
}

/// Plugin manager for coordinating multiple plugins
pub struct PluginManager {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<PluginId, Box<dyn Plugin>>>>,

    /// Plugin factories
    factories: Arc<RwLock<HashMap<String, Box<dyn PluginFactory>>>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a plugin factory
    pub async fn register_factory(&self, factory_type: String, factory: Box<dyn PluginFactory>) {
        self.factories.write().await.insert(factory_type, factory);
    }

    /// Create and register a plugin
    pub async fn create_plugin(&self, plugin_type: &str, config: PluginConfig) -> Result<()> {
        let factory = self.get_factory(plugin_type).await?;
        let plugin = factory.create_plugin(config.clone()).await?;
        let plugin_id = config.plugin_id.clone();

        self.plugins.write().await.insert(plugin_id, plugin);
        Ok(())
    }

    /// Get a factory by type
    async fn get_factory(&self, plugin_type: &str) -> Result<Box<dyn PluginFactory>> {
        let factories = self.factories.read().await;
        let factory = factories
            .get(plugin_type)
            .ok_or_else(|| anyhow::anyhow!("No factory found for plugin type: {}", plugin_type))?;
        Ok(factory.clone_box())
    }

    /// Get a plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<Box<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|p| p.as_ref().clone_box())
    }

    /// Process a source with the appropriate plugin
    pub async fn process_source(
        &self,
        source_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        let plugins = self.plugins.read().await;

        for plugin in plugins.values() {
            if plugin.can_handle(source_path).await? {
                return plugin.process_source(source_path, context).await;
            }
        }

        Err(anyhow::anyhow!(
            "No plugin found that can handle source: {:?}",
            source_path
        ))
    }

    /// Generate code for all plugins
    pub async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &PluginContext,
    ) -> Result<Vec<PathBuf>> {
        let mut all_generated_files = Vec::new();
        let plugins = self.plugins.read().await;

        for plugin in plugins.values() {
            let files = plugin.generate_code(schemas, context).await?;
            all_generated_files.extend(files);
        }

        Ok(all_generated_files)
    }

    /// Clean up all plugins
    pub async fn cleanup(&self, context: &PluginContext) -> Result<()> {
        let plugins = self.plugins.read().await;

        for plugin in plugins.values() {
            plugin.cleanup(context).await?;
        }

        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.plugins.read().await.is_empty());
        assert!(manager.factories.read().await.is_empty());
    }

    #[test]
    fn test_plugin_metadata() {
        let metadata = PluginMetadata {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            supported_types: vec!["test".to_string()],
            capabilities: vec![PluginCapability::Parse],
        };

        assert_eq!(metadata.id, "test-plugin");
        assert_eq!(metadata.name, "Test Plugin");
    }

    #[tokio::test]
    async fn test_plugin_context() {
        let temp_dir = TempDir::new().unwrap();
        let config = PluginConfig {
            plugin_id: "test".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse],
        };

        let context = PluginContext::new(
            temp_dir.path().to_path_buf(),
            temp_dir.path().join("output"),
            config,
        );

        // Test shared state
        context
            .set_shared_value(
                "test_key".to_string(),
                serde_yaml::Value::String("test_value".to_string()),
            )
            .await;
        let value = context.get_shared_value("test_key").await;
        assert_eq!(
            value,
            Some(serde_yaml::Value::String("test_value".to_string()))
        );
    }
}
