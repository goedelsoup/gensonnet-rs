# Gensonnet Plugin Infrastructure

This crate provides the core plugin infrastructure for gensonnet. It defines the common traits, types, and interfaces that all gensonnet plugins must implement.

## Overview

The plugin infrastructure enables extensible source processing by providing:

- **Plugin Traits**: Common interfaces that all plugins must implement
- **Plugin Types**: Shared data structures for plugin communication
- **Plugin Manager**: Coordination and lifecycle management for multiple plugins
- **Plugin Context**: Shared state and configuration for plugin execution

## Key Components

### Plugin Trait

The main `Plugin` trait defines the interface that all plugins must implement:

```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    async fn initialize(&self, context: &PluginContext) -> Result<()>;
    async fn can_handle(&self, source_path: &Path) -> Result<bool>;
    async fn process_source(&self, source_path: &Path, context: &PluginContext) -> Result<PluginResult>;
    async fn generate_code(&self, schemas: &[ExtractedSchema], context: &PluginContext) -> Result<Vec<PathBuf>>;
    async fn cleanup(&self, context: &PluginContext) -> Result<()>;
    fn clone_box(&self) -> Box<dyn Plugin>;
}
```

### Plugin Factory

The `PluginFactory` trait enables dynamic plugin creation:

```rust
#[async_trait]
pub trait PluginFactory: Send + Sync {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>>;
    fn supported_types(&self) -> Vec<String>;
    fn clone_box(&self) -> Box<dyn PluginFactory>;
}
```

### Plugin Manager

The `PluginManager` coordinates multiple plugins:

```rust
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<PluginId, Box<dyn Plugin>>>>,
    factories: Arc<RwLock<HashMap<String, Box<dyn PluginFactory>>>>,
}
```

### Core Types

- `PluginMetadata`: Plugin identification and capabilities
- `PluginConfig`: Plugin-specific configuration
- `PluginContext`: Shared execution context
- `PluginResult`: Processing results and statistics
- `ExtractedSchema`: Schema data extracted from sources

## Usage

### For Plugin Implementers

1. Add this crate as a dependency:
   ```toml
   [dependencies]
   gensonnet-plugin = { path = "../../crates/plugin" }
   ```

2. Implement the required traits:
   ```rust
   use gensonnet_plugin::*;
   
   pub struct MyPlugin {
       // Plugin implementation
   }
   
   #[async_trait]
   impl Plugin for MyPlugin {
       // Implement required methods
   }
   ```

### For Plugin Consumers

1. Use the plugin manager to coordinate plugins:
   ```rust
   use gensonnet_plugin::*;
   
   let manager = PluginManager::new();
   // Register and use plugins
   ```

## Plugin Capabilities

Plugins can declare support for various capabilities:

- `Parse`: Can parse source files
- `SchemaExtraction`: Can extract schemas from sources
- `Validation`: Can validate schemas
- `CodeGeneration`: Can generate code from schemas
- `AstProcessing`: Can process abstract syntax trees
- `DependencyResolution`: Can handle dependencies

## Design Principles

- **Extensibility**: Easy to add new plugin types
- **Composability**: Plugins can work together
- **Async-first**: All operations are async for better performance
- **Type Safety**: Strong typing throughout the plugin system
- **Error Handling**: Comprehensive error handling with anyhow
- **Thread Safety**: All components are Send + Sync

## Future Enhancements

- Dynamic plugin loading from external crates
- Plugin hot-reloading capabilities
- Plugin versioning and compatibility checking
- Plugin marketplace/registry integration
- WASM-based plugin sandboxing
