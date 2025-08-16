# Plugin Registry

The plugin registry manages the discovery, loading, and lifecycle of plugins in the JsonnetGen system.

## Overview

The plugin registry consists of several key components:

- **PluginRegistry**: Main registry that manages plugin discovery and loading
- **BuiltinPluginLoader**: Handles loading of built-in plugins
- **PluginDiscoveryService**: Discovers and loads external plugins
- **PluginManager**: Manages plugin instances and factories

## Adding Built-in Plugin Factories

To add a new built-in plugin factory to the registry, follow these steps:

### 1. Create the Plugin Factory

First, ensure your plugin factory implements the `PluginFactory` trait:

```rust
use crate::plugin::{Plugin, PluginFactory, PluginConfig};
use async_trait::async_trait;

pub struct MyPluginFactory;

#[async_trait]
impl PluginFactory for MyPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(MyPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["my-type".to_string(), "another-type".to_string()]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(MyPluginFactory)
    }
}
```

### 2. Register the Factory

Add the factory registration to `BuiltinPluginLoader::load_builtin_plugins()` in `src/plugin/registry/mod.rs`:

```rust
// Register MyPlugin factory
let my_plugin_factory = Box::new(crate::plugin::my_plugin::MyPluginFactory);
plugin_manager
    .register_factory("my-plugin".to_string(), my_plugin_factory)
    .await;
```

### 3. Add Plugin Configuration

Add the plugin configuration to `get_builtin_plugin_configs()`:

```rust
PluginConfig {
    plugin_id: "my-plugin:builtin".to_string(),
    config: serde_yaml::Value::Null,
    enabled_capabilities: vec![
        PluginCapability::Parse,
        PluginCapability::SchemaExtraction,
        // Add other capabilities as needed
    ],
},
```

### 4. Add Plugin Creation Support

Add a case for your plugin type in `create_plugin_from_entry()`:

```rust
"my-plugin" => {
    let factory = Box::new(crate::plugin::my_plugin::MyPluginFactory);
    self.plugin_manager
        .register_factory("my-plugin".to_string(), factory)
        .await;
    self.plugin_manager
        .create_plugin("my-plugin", entry.config.clone())
        .await?;
}
```

### 5. Update Plugin Information (Optional)

If you want your plugin to appear in the plugin listing with custom information, update the `get_plugin_info()` method in `src/lib.rs`:

```rust
plugin::PluginMetadata {
    id: "my-plugin:builtin".to_string(),
    name: "My Plugin".to_string(),
    version: "1.0.0".to_string(),
    description: "Description of what my plugin does".to_string(),
    supported_types: vec!["my-type".to_string(), "another-type".to_string()],
    capabilities: vec![
        plugin::PluginCapability::Parse,
        plugin::PluginCapability::SchemaExtraction,
    ],
},
```

## Plugin Capabilities

Available plugin capabilities:

- **Parse**: Can parse source files
- **SchemaExtraction**: Can extract schemas from source files
- **Validation**: Can validate source files
- **AstProcessing**: Can process abstract syntax trees
- **CodeGeneration**: Can generate code

## Plugin Lifecycle

1. **Discovery**: Plugins are discovered during initialization
2. **Registration**: Plugin factories are registered with the plugin manager
3. **Loading**: Plugin instances are created from configurations
4. **Execution**: Plugins are used to process source files
5. **Cleanup**: Plugins are cleaned up when no longer needed

## Testing

After adding a new plugin factory, test the integration:

```bash
# List all plugins
cargo run -- plugins list

# List with detailed information
cargo run -- plugins list --detailed

# Filter by capability
cargo run -- plugins list --capability Parse

# Filter by source type
cargo run -- plugins list --source_type my-type
```

## Example: OpenAPI Plugin

The OpenAPI plugin serves as a complete example of a built-in plugin:

### Factory Registration
```rust
// Register OpenAPI plugin factory
let openapi_factory = Box::new(crate::plugin::openapi::OpenApiPluginFactory);
plugin_manager
    .register_factory("openapi".to_string(), openapi_factory)
    .await;
```

### Configuration
```rust
PluginConfig {
    plugin_id: "openapi:builtin".to_string(),
    config: serde_yaml::Value::Null,
    enabled_capabilities: vec![
        PluginCapability::Parse,
        PluginCapability::SchemaExtraction,
        PluginCapability::Validation,
    ],
},
```

### Plugin Creation
```rust
"openapi" => {
    let factory = Box::new(crate::plugin::openapi::OpenApiPluginFactory);
    self.plugin_manager
        .register_factory("openapi".to_string(), factory)
        .await;
    self.plugin_manager
        .create_plugin("openapi", entry.config.clone())
        .await?;
}
```

## Troubleshooting

### Common Issues

1. **Plugin not appearing in list**: Ensure the factory is registered and configuration is added
2. **Plugin creation fails**: Check that the plugin type case is added to `create_plugin_from_entry`
3. **Capabilities not working**: Verify capabilities are correctly defined in the configuration

### Debugging

Enable debug logging to see plugin loading details:

```bash
RUST_LOG=debug cargo run -- plugins list
```

## Future Enhancements

- Dynamic plugin loading from external sources
- Plugin version management
- Plugin dependency resolution
- Plugin hot-reloading
- Plugin marketplace integration
