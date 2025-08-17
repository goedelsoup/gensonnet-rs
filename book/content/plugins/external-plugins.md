+++
title = "External Plugin System"
description = "Learn how to create and use external plugins to extend Gensonnet-rs functionality"
weight = 10
+++

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

### Step 1: Create Plugin Directory

```bash
mkdir my-custom-plugin
cd my-custom-plugin
```

### Step 2: Create Plugin Manifest

Create a `plugin.yaml` file:

```yaml
metadata:
  id: "my-custom-plugin:1.0"
  name: "My Custom Plugin"
  version: "1.0.0"
  description: "A custom plugin for processing special formats"
  author: "Your Name"
  license: "MIT"
  supported_types:
    - "custom"
    - "yaml"
  capabilities:
    - "Parse"
    - "SchemaExtraction"
    - "Validation"

config:
  plugin_id: "my-custom-plugin:1.0"
  config:
    custom_setting: "default_value"
  enabled_capabilities:
    - "Parse"
    - "SchemaExtraction"

requirements:
  min_tool_version: "0.1.0"
  required_capabilities:
    - "Parse"
```

### Step 3: Implement Plugin Logic

Create your plugin implementation. The exact structure depends on the plugin type, but typically includes:

```rust
use gensonnet_rs::plugin::traits::{Generator, Lifecycle, Processor, Validator};

pub struct MyCustomPlugin {
    config: PluginConfig,
}

impl Lifecycle for MyCustomPlugin {
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), Error> {
        self.config = config.clone();
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl Processor for MyCustomPlugin {
    fn process(&self, input: &Input) -> Result<ProcessedData, Error> {
        // Implement your custom processing logic
        let data = parse_custom_format(&input.content)?;
        
        Ok(ProcessedData {
            schema: data.schema,
            types: data.types,
            functions: data.functions,
        })
    }
}

impl Generator for MyCustomPlugin {
    fn generate(&self, data: &ProcessedData) -> Result<GeneratedCode, Error> {
        // Implement your custom code generation
        let code = generate_custom_jsonnet(data)?;
        
        Ok(GeneratedCode {
            content: code,
            format: OutputFormat::Jsonnet,
            metadata: HashMap::new(),
        })
    }
}

impl Validator for MyCustomPlugin {
    fn validate(&self, code: &GeneratedCode) -> Result<ValidationResult, Error> {
        // Implement your custom validation
        let result = validate_custom_code(&code.content)?;
        
        Ok(ValidationResult {
            is_valid: result.is_valid,
            errors: result.errors,
            warnings: result.warnings,
        })
    }
}
```

### Step 4: Build and Install

```bash
# Build your plugin
cargo build --release

# Install the plugin
gensonnet-rs plugin install ./target/release/my_custom_plugin
```

## Using External Plugins

### Configuration

To use an external plugin, add it to your configuration:

```yaml
version: "1.0"

sources:
  - type: my-custom-plugin
    name: "my-data"
    file: "data/custom-format.dat"
    config:
      custom_setting: "my_value"
    output_path: "./generated/custom"

plugins:
  external:
    - name: "my-custom-plugin"
      path: "./plugins/my-custom-plugin"
      config:
        custom_setting: "value"
```

### Runtime Usage

```bash
# Generate using the plugin
gensonnet-rs generate

# Validate with the plugin
gensonnet-rs validate

# Check plugin status
gensonnet-rs plugin list
```

## Plugin Development Best Practices

### Error Handling

- Use descriptive error messages
- Provide context for debugging
- Handle edge cases gracefully
- Return structured error information

### Performance

- Process data incrementally when possible
- Cache expensive operations
- Use efficient data structures
- Profile your plugin for bottlenecks

### Testing

- Write unit tests for core functionality
- Create integration tests with sample data
- Test error conditions and edge cases
- Use the plugin testing framework

### Documentation

- Document all configuration options
- Provide usage examples
- Include troubleshooting guides
- Maintain changelog

## Plugin Registry

The plugin registry allows you to discover and install plugins from external sources.

### Registry Configuration

```yaml
plugins:
  registry:
    url: "https://plugins.gensonnet.dev"
    auth:
      token: "${PLUGIN_REGISTRY_TOKEN}"
    cache:
      enabled: true
      directory: "~/.cache/gensonnet/plugins"
```

### Registry Commands

```bash
# List available plugins
gensonnet-rs plugin list --registry

# Install from registry
gensonnet-rs plugin install my-custom-plugin

# Update plugins
gensonnet-rs plugin update

# Search plugins
gensonnet-rs plugin search "custom format"
```

## Security Considerations

### Plugin Validation

- Validate plugin signatures
- Check plugin compatibility
- Verify plugin sources
- Sandbox plugin execution

### Configuration

```yaml
plugins:
  security:
    validate_signatures: true
    allowed_sources:
      - "local"
      - "registry"
    sandbox_execution: true
    max_execution_time: 300
```

## Troubleshooting

### Common Issues

1. **Plugin not found**: Check plugin discovery paths
2. **Version incompatibility**: Update plugin or tool version
3. **Configuration errors**: Validate plugin configuration
4. **Performance issues**: Profile plugin execution

### Debug Commands

```bash
# Enable debug logging
RUST_LOG=debug gensonnet-rs generate

# Check plugin status
gensonnet-rs plugin status

# Validate plugin configuration
gensonnet-rs plugin validate my-custom-plugin
```

## Examples

### Simple Custom Format Plugin

```rust
use gensonnet_rs::plugin::traits::Processor;

pub struct SimpleCustomPlugin;

impl Processor for SimpleCustomPlugin {
    fn process(&self, input: &Input) -> Result<ProcessedData, Error> {
        // Parse simple key-value format
        let lines: Vec<&str> = std::str::from_utf8(&input.content)?
            .lines()
            .collect();
        
        let mut properties = HashMap::new();
        
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                properties.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        
        let schema = Schema::Object(ObjectSchema {
            properties,
            required: vec![],
            additional_properties: false,
        });
        
        Ok(ProcessedData {
            schema,
            types: vec![],
            functions: vec![],
        })
    }
}
```

### Advanced Plugin with Dependencies

```rust
use gensonnet_rs::plugin::traits::{Generator, Processor};
use gensonnet_rs::plugin::dependencies::DependencyManager;

pub struct AdvancedPlugin {
    dependencies: DependencyManager,
}

impl Processor for AdvancedPlugin {
    fn process(&self, input: &Input) -> Result<ProcessedData, Error> {
        // Process with dependency resolution
        let resolved = self.dependencies.resolve(&input.content)?;
        let processed = self.process_with_dependencies(resolved)?;
        
        Ok(processed)
    }
}

impl Generator for AdvancedPlugin {
    fn generate(&self, data: &ProcessedData) -> Result<GeneratedCode, Error> {
        // Generate with dependency imports
        let imports = self.dependencies.generate_imports()?;
        let code = self.generate_with_imports(data, &imports)?;
        
        Ok(code)
    }
}
```

## Next Steps

- Explore the [Plugin API Reference](/api/plugins/) for detailed technical documentation
- Check out [Examples](/examples/) for real-world plugin implementations
- Join the community to share and discover plugins
