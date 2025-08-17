# Gensonnet Plugins

This directory contains the modular plugin implementations for gensonnet. Each plugin is a separate crate that implements the plugin interface defined in `crates/plugin`.

## Plugin Structure

```
plugins/
├── go-ast/       # Go AST processing plugin
├── crd/          # Kubernetes CRD processing plugin
├── openapi/      # OpenAPI/Swagger processing plugin
└── README.md     # This file
```

## Plugin Architecture

The plugin system is designed with the following architecture:

1. **Core Infrastructure** (`crates/plugin`): Contains the common plugin traits, types, and interfaces that all plugins must implement.

2. **Abstract AST Infrastructure** (`crates/plugin-ast`): Contains the abstract AST processing infrastructure that can be extended for different languages.

3. **Plugin Registry** (`src/plugin/registry`): Manages plugin discovery, loading, and lifecycle. This stays in the root project.

4. **Plugin Implementations** (`plugins/`): Individual plugin crates that implement the plugin interface.

## Creating a New Plugin

To create a new plugin:

1. Create a new directory in `plugins/` for your plugin
2. Create a `Cargo.toml` with the following dependencies:
   ```toml
   [dependencies]
   gensonnet-plugin = { path = "../../crates/plugin" }
   # Add other dependencies as needed
   ```

3. Implement the required traits:
   - `Plugin`: Main plugin interface
   - `PluginFactory`: Factory for creating plugin instances

4. Register your plugin in the registry (see `src/plugin/registry/mod.rs`)

## Plugin Interface

All plugins must implement the `Plugin` trait:

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

## Plugin Capabilities

Plugins can support various capabilities:

- `Parse`: Can parse source files
- `SchemaExtraction`: Can extract schemas from sources
- `Validation`: Can validate schemas
- `CodeGeneration`: Can generate code from schemas
- `AstProcessing`: Can process abstract syntax trees
- `DependencyResolution`: Can handle dependencies

## Future Enhancements

In the future, this structure will support:

- Dynamic plugin loading from external crates
- Plugin hot-reloading
- Plugin versioning and compatibility
- Plugin marketplace/registry
- WASM-based plugins for sandboxed execution
