//! Plugin registry for managing plugin discovery and registration

use anyhow::Result;
use tracing::{info, warn};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

use gensonnet_plugin::*;

/// Plugin registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Plugin configuration
    pub config: PluginConfig,

    /// Plugin file path
    pub plugin_path: PathBuf,

    /// Plugin status
    pub status: RegistryPluginStatus,

    /// Last loaded timestamp
    pub last_loaded: Option<chrono::DateTime<chrono::Utc>>,
}

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryPluginStatus {
    /// Plugin is available
    Available,

    /// Plugin is loaded
    Loaded,

    /// Plugin has errors
    Error(String),

    /// Plugin is disabled
    Disabled,
}

/// Plugin registry for managing plugins
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<PluginId, RegistryEntry>>>,

    /// Plugin directories
    plugin_dirs: Arc<RwLock<Vec<PathBuf>>>,

    /// Plugin manager
    plugin_manager: Arc<PluginManager>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_dirs: Arc::new(RwLock::new(Vec::new())),
            plugin_manager,
        }
    }

    /// Add a plugin directory
    pub async fn add_plugin_directory(&self, dir: PathBuf) {
        self.plugin_dirs.write().await.push(dir);
    }

    /// Discover plugins in registered directories
    pub async fn discover_plugins(&self) -> Result<()> {
        let plugin_dirs = self.plugin_dirs.read().await;
        for plugin_dir in &*plugin_dirs {
            self.scan_plugin_directory(plugin_dir).await?;
        }
        Ok(())
    }

    /// Scan a plugin directory for plugins
    async fn scan_plugin_directory(&self, plugin_dir: &Path) -> Result<()> {
        if !plugin_dir.exists() {
            return Ok(());
        }

        info!("Scanning plugin directory: {:?}", plugin_dir);

        for entry in WalkDir::new(plugin_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Check for plugin manifest files
            if let Some(file_name) = path.file_name() {
                if file_name == "plugin.yaml" || file_name == "plugin.yml" {
                    info!("Found plugin manifest: {:?}", path);
                    match self.load_plugin_manifest(path).await {
                        Ok(_) => {
                            info!("Successfully loaded plugin manifest: {:?}", path);
                        }
                        Err(e) => {
                            warn!("Failed to load plugin manifest {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a plugin manifest
    async fn load_plugin_manifest(&self, manifest_path: &Path) -> Result<()> {
        let content = tokio::fs::read_to_string(manifest_path).await?;
        let manifest: PluginManifest = serde_yaml::from_str(&content)?;

        let entry = RegistryEntry {
            metadata: manifest.metadata,
            config: manifest.config,
            plugin_path: manifest_path.to_path_buf(),
            status: RegistryPluginStatus::Available,
            last_loaded: None,
        };

        let plugin_id = entry.metadata.id.clone();
        self.plugins.write().await.insert(plugin_id, entry);

        Ok(())
    }

    /// Load a plugin
    pub async fn load_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(entry) = plugins.get_mut(plugin_id) {
            match self.create_plugin_from_entry(entry).await {
                Ok(_) => {
                    entry.status = RegistryPluginStatus::Loaded;
                    entry.last_loaded = Some(chrono::Utc::now());
                }
                Err(e) => {
                    entry.status = RegistryPluginStatus::Error(e.to_string());
                }
            }
        }

        Ok(())
    }

    /// Create a plugin from registry entry
    async fn create_plugin_from_entry(&self, entry: &RegistryEntry) -> Result<()> {
        // For now, we'll create built-in plugins based on the plugin type
        // In the future, this could load dynamic libraries or WASM modules

        let plugin_type = entry.metadata.id.split(':').next().unwrap_or("unknown");

        match plugin_type {
            "go-ast" => {
                // Note: In a real implementation, this would dynamically load the plugin
                // For now, we'll keep the built-in registration
                let factory = Box::new(crate::plugin::ast::GoAstPluginFactory);
                self.plugin_manager
                    .register_factory("go-ast".to_string(), factory)
                    .await;
                self.plugin_manager
                    .create_plugin("go-ast", entry.config.clone())
                    .await?;
            }
            "crd" => {
                // Note: In a real implementation, this would dynamically load the plugin
                // For now, we'll keep the built-in registration
                let factory = Box::new(crate::plugin::crd::CrdPluginFactory);
                self.plugin_manager
                    .register_factory("crd".to_string(), factory)
                    .await;
                self.plugin_manager
                    .create_plugin("crd", entry.config.clone())
                    .await?;
            }
            "openapi" => {
                // Note: In a real implementation, this would dynamically load the plugin
                // For now, we'll keep the built-in registration
                let factory = Box::new(crate::plugin::openapi::OpenApiPluginFactory);
                self.plugin_manager
                    .register_factory("openapi".to_string(), factory)
                    .await;
                self.plugin_manager
                    .create_plugin("openapi", entry.config.clone())
                    .await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown plugin type: {}", plugin_type));
            }
        }

        Ok(())
    }

    /// Get all registered plugins
    pub async fn get_plugins(&self) -> Vec<RegistryEntry> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<RegistryEntry> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).cloned()
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(entry) = plugins.get_mut(plugin_id) {
            entry.status = RegistryPluginStatus::Available;
        }

        Ok(())
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(entry) = plugins.get_mut(plugin_id) {
            entry.status = RegistryPluginStatus::Disabled;
        }

        Ok(())
    }

    /// Get plugins by capability
    pub async fn get_plugins_by_capability(
        &self,
        capability: &PluginCapability,
    ) -> Vec<RegistryEntry> {
        let plugins = self.plugins.read().await;

        plugins
            .values()
            .filter(|entry| entry.metadata.capabilities.contains(capability))
            .cloned()
            .collect()
    }

    /// Get plugins by source type
    pub async fn get_plugins_by_source_type(&self, source_type: &str) -> Vec<RegistryEntry> {
        let plugins = self.plugins.read().await;

        plugins
            .values()
            .filter(|entry| {
                entry
                    .metadata
                    .supported_types
                    .contains(&source_type.to_string())
            })
            .cloned()
            .collect()
    }
}

/// Plugin manifest file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub metadata: PluginMetadata,

    /// Plugin configuration
    pub config: PluginConfig,

    /// Plugin dependencies
    pub dependencies: Option<Vec<String>>,

    /// Plugin requirements
    pub requirements: Option<PluginRequirements>,
}

/// Plugin requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequirements {
    /// Minimum tool version
    pub min_tool_version: Option<String>,

    /// Required capabilities
    pub required_capabilities: Option<Vec<PluginCapability>>,

    /// Required dependencies
    pub required_dependencies: Option<Vec<String>>,
}

/// Built-in plugin loader
pub struct BuiltinPluginLoader;

impl BuiltinPluginLoader {
    /// Load built-in plugins
    pub async fn load_builtin_plugins(plugin_manager: &Arc<PluginManager>) -> Result<()> {
        // Register Go AST plugin factory
        let go_ast_factory = Box::new(crate::plugin::ast::GoAstPluginFactory);
        plugin_manager
            .register_factory("go-ast".to_string(), go_ast_factory)
            .await;

        // Register CRD plugin factory
        let crd_factory = Box::new(crate::plugin::crd::CrdPluginFactory);
        plugin_manager
            .register_factory("crd".to_string(), crd_factory)
            .await;

        // Register OpenAPI plugin factory
        let openapi_factory = Box::new(crate::plugin::openapi::OpenApiPluginFactory);
        plugin_manager
            .register_factory("openapi".to_string(), openapi_factory)
            .await;

        // - Additional plugin factories can be added here as needed

        Ok(())
    }

    /// Create built-in plugin configurations
    pub fn get_builtin_plugin_configs() -> Vec<PluginConfig> {
        vec![
            PluginConfig {
                plugin_id: "go-ast:builtin".to_string(),
                config: serde_yaml::Value::Null,
                enabled_capabilities: vec![
                    PluginCapability::Parse,
                    PluginCapability::SchemaExtraction,
                    PluginCapability::AstProcessing,
                ],
            },
            PluginConfig {
                plugin_id: "crd:builtin".to_string(),
                config: serde_yaml::Value::Null,
                enabled_capabilities: vec![
                    PluginCapability::Parse,
                    PluginCapability::SchemaExtraction,
                    PluginCapability::Validation,
                ],
            },
            PluginConfig {
                plugin_id: "openapi:builtin".to_string(),
                config: serde_yaml::Value::Null,
                enabled_capabilities: vec![
                    PluginCapability::Parse,
                    PluginCapability::SchemaExtraction,
                    PluginCapability::Validation,
                ],
            },
        ]
    }
}

/// Plugin discovery service
pub struct PluginDiscoveryService {
    /// Registry instance
    registry: Arc<PluginRegistry>,
}

impl PluginDiscoveryService {
    /// Create a new plugin discovery service
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self { registry }
    }

    /// Discover and load all available plugins
    pub async fn discover_and_load(&self) -> Result<()> {
        info!("Starting plugin discovery and loading process");

        // Discover plugins
        info!("Discovering plugins from configured directories");
        self.registry.discover_plugins().await?;

        // Load built-in plugins
        info!("Loading built-in plugins");
        let plugin_manager = Arc::clone(&self.registry.plugin_manager);
        BuiltinPluginLoader::load_builtin_plugins(&plugin_manager).await?;

        // Load discovered plugins
        info!("Loading discovered external plugins");
        let plugins = self.registry.get_plugins().await;
        let mut loaded_count = 0;
        let mut error_count = 0;

        for plugin in plugins {
            if matches!(plugin.status, RegistryPluginStatus::Available) {
                match self.registry.load_plugin(&plugin.metadata.id).await {
                    Ok(_) => {
                        info!("Successfully loaded plugin: {}", plugin.metadata.id);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load plugin {}: {}", plugin.metadata.id, e);
                        error_count += 1;
                    }
                }
            }
        }

        info!(
            "Plugin discovery completed: {} loaded, {} errors",
            loaded_count, error_count
        );
        Ok(())
    }

    /// Get available plugins for a source type
    pub async fn get_plugins_for_source(&self, source_path: &Path) -> Vec<RegistryEntry> {
        let mut available_plugins = Vec::new();

        // Get plugins by file extension
        if let Some(extension) = source_path.extension() {
            let source_type = extension.to_string_lossy();
            let plugins = self.registry.get_plugins_by_source_type(&source_type).await;
            available_plugins.extend(plugins);
        }

        // Get plugins by capability
        let parse_plugins = self
            .registry
            .get_plugins_by_capability(&PluginCapability::Parse)
            .await;
        available_plugins.extend(parse_plugins);

        // Remove duplicates
        available_plugins.sort_by(|a, b| a.metadata.id.cmp(&b.metadata.id));
        available_plugins.dedup_by(|a, b| a.metadata.id == b.metadata.id);

        available_plugins
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_plugin_registry() {
        let plugin_manager = Arc::new(PluginManager::new());
        let registry = PluginRegistry::new(plugin_manager);

        let plugins = registry.get_plugins().await;
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_builtin_plugin_loader() {
        let plugin_manager = Arc::new(PluginManager::new());
        BuiltinPluginLoader::load_builtin_plugins(&plugin_manager)
            .await
            .unwrap();

        let configs = BuiltinPluginLoader::get_builtin_plugin_configs();
        assert!(!configs.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_discovery_service() {
        let _temp_dir = TempDir::new().unwrap();
        let plugin_manager = Arc::new(PluginManager::new());
        let registry = Arc::new(PluginRegistry::new(plugin_manager));
        let discovery_service = PluginDiscoveryService::new(registry);

        // Test with a non-existent directory (should not fail)
        discovery_service.discover_and_load().await.unwrap();
    }
}
