# External Plugin System

The JsonnetGen tool supports external plugins that can be loaded at runtime to extend its functionality. This document explains how to create and use external plugins.

## Overview

External plugins allow you to:
- Add support for new schema formats
- Implement custom validation rules
- Extend code generation capabilities
- Add new processing capabilities

## Plugin Structure

An external plugin consists of:
1. A plugin manifest file (`plugin.yaml` or `plugin.yml`)
2. Plugin implementation files
3. Optional configuration files

### Plugin Manifest

The plugin manifest defines the plugin's metadata and configuration:

```yaml
metadata:
  id: "my-plugin:1.0"
  name: "My Custom Plugin"
  version: "1.0.0"
  description: "Description of what this plugin does"
  supported_types:
    - "custom"
    - "yaml"
  capabilities:
    - "Parse"
    - "SchemaExtraction"
    - "Validation"

config:
  plugin_id: "my-plugin:1.0"
  config:
    # Plugin-specific configuration
    my_setting: "value"
  enabled_capabilities:
    - "Parse"
    - "SchemaExtraction"

dependencies:
  - "base-plugin:1.0"

requirements:
  min_tool_version: "0.1.0"
  required_capabilities:
    - "Parse"
```

## Plugin Capabilities

Plugins can implement the following capabilities:

- **Parse**: Can parse source files
- **SchemaExtraction**: Can extract schemas from parsed content
- **Validation**: Can validate schemas
- **CodeGeneration**: Can generate code from schemas
- **AstProcessing**: Can process abstract syntax trees
- **DependencyResolution**: Can handle dependencies

## Plugin Discovery

Plugins are discovered from the following directories (configurable):

1. `./plugins` (local project plugins)
2. `~/.config/gensonnet/plugins` (user plugins)
3. Custom directories specified in configuration

### Configuration

Plugin discovery can be configured in your `gensonnet.yaml` file:

```yaml
version: "1.0"
sources:
  # ... your sources

plugins:
  plugin_directories:
    - "./plugins"
    - "~/.config/gensonnet/plugins"
    - "$XDG_CONFIG_HOME/gensonnet/plugins"
  enable_external_discovery: true
  registry_url: "https://plugins.gensonnet.dev"
  cache_directory: "~/.cache/gensonnet/plugins"
  validation:
    validate_signatures: false
    check_compatibility: true
    allowed_sources:
      - "local"
      - "registry"
```

## Creating a Plugin

### 1. Create Plugin Directory

```bash
mkdir -p plugins/my-plugin
cd plugins/my-plugin
```

### 2. Create Plugin Manifest

Create `plugin.yaml`:

```yaml
metadata:
  id: "my-plugin:1.0"
  name: "My Custom Plugin"
  version: "1.0.0"
  description: "Processes custom schema format"
  supported_types:
    - "custom"
  capabilities:
    - "Parse"
    - "SchemaExtraction"

config:
  plugin_id: "my-plugin:1.0"
  config:
    custom_setting: "value"
  enabled_capabilities:
    - "Parse"
    - "SchemaExtraction"
```

### 3. Implement Plugin Logic

Currently, external plugins are loaded as built-in plugins. The plugin system will look for the plugin type in the manifest ID and create the appropriate built-in plugin.

For a fully external plugin system, you would need to implement:
- Dynamic library loading
- WASM module support
- Plugin sandboxing

## Plugin Management

### Listing Plugins

```bash
jsonnet-gen plugins list
```

### Plugin Information

```bash
jsonnet-gen plugins info <plugin-id>
```

### Enabling/Disabling Plugins

```bash
jsonnet-gen plugins enable <plugin-id>
jsonnet-gen plugins disable <plugin-id>
```

### Installing Plugins

```bash
jsonnet-gen plugins install <source>
```

## Plugin Development

### Built-in Plugin Types

The following plugin types are currently supported:

1. **go-ast**: Processes Go source code and extracts type information
2. **crd**: Processes Kubernetes CustomResourceDefinitions
3. **openapi**: Processes OpenAPI/Swagger specifications

### Adding New Plugin Types

To add a new plugin type:

1. Implement the `Plugin` trait
2. Implement the `PluginFactory` trait
3. Register the factory in the plugin manager
4. Update the plugin registry to handle the new type

### Example Plugin Implementation

```rust
use crate::plugin::*;

pub struct MyCustomPlugin {
    config: PluginConfig,
}

impl MyCustomPlugin {
    pub fn new(config: PluginConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Plugin for MyCustomPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: self.config.plugin_id.clone(),
            name: "My Custom Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Custom plugin for processing schemas".to_string(),
            supported_types: vec!["custom".to_string()],
            capabilities: vec![
                PluginCapability::Parse,
                PluginCapability::SchemaExtraction,
            ],
        }
    }

    async fn initialize(&self, _context: &PluginContext) -> Result<()> {
        // Initialize plugin
        Ok(())
    }

    async fn can_handle(&self, source_path: &Path) -> Result<bool> {
        // Check if this plugin can handle the given source
        Ok(source_path.extension().map_or(false, |ext| ext == "custom"))
    }

    async fn process_source(
        &self,
        source_path: &Path,
        context: &PluginContext,
    ) -> Result<PluginResult> {
        // Process the source file
        // Extract schemas
        // Return results
        Ok(PluginResult {
            schemas: vec![],
            generated_files: vec![],
            statistics: PluginStatistics {
                processing_time_ms: 0,
                files_processed: 1,
                schemas_extracted: 0,
                files_generated: 0,
            },
            warnings: vec![],
            errors: vec![],
        })
    }

    async fn generate_code(
        &self,
        schemas: &[ExtractedSchema],
        context: &PluginContext,
    ) -> Result<Vec<PathBuf>> {
        // Generate code from schemas
        Ok(vec![])
    }

    async fn cleanup(&self, _context: &PluginContext) -> Result<()> {
        // Clean up resources
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(MyCustomPlugin {
            config: self.config.clone(),
        })
    }
}

pub struct MyCustomPluginFactory;

#[async_trait]
impl PluginFactory for MyCustomPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(MyCustomPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["custom".to_string()]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(MyCustomPluginFactory)
    }
}
```

## Security Considerations

- Plugins run in the same process as the main application
- Validate plugin manifests before loading
- Consider implementing plugin sandboxing for untrusted plugins
- Use signature verification for plugins from external sources

## Future Enhancements

- Dynamic library loading for external plugins
- WASM module support for better isolation
- Plugin marketplace/registry
- Plugin versioning and dependency management
- Plugin sandboxing and security
- Plugin hot-reloading
