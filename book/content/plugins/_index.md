+++
title = "Plugins"
description = "Learn about Gensonnet-rs plugins and how to create custom ones"
weight = 3
+++

# Plugins

Gensonnet-rs uses a plugin architecture to support different input sources and output formats. This page describes the available plugins and how to create custom ones.

## Built-in Plugins

### OpenAPI Plugin

The OpenAPI plugin generates Jsonnet code from OpenAPI/Swagger specifications.

**Configuration:**
```yaml
sources:
  - type: openapi
    name: "my-api"
    url: "https://api.example.com/openapi.json"
    options:
      include_examples: true
      generate_validation: true
```

**Features:**
- Generates Jsonnet functions for API endpoints
- Creates type definitions from schemas
- Supports request/response validation
- Handles authentication schemes

📖 **[Detailed OpenAPI Generator Documentation](/plugins/openapi-generator/)**

### CRD Plugin

The CRD plugin processes Kubernetes Custom Resource Definitions.

**Configuration:**
```yaml
sources:
  - type: crd
    name: "my-crds"
    files:
      - "crds/*.yaml"
    options:
      generate_validation: true
      include_status: false
```

**Features:**
- Converts CRD schemas to Jsonnet types
- Generates validation functions
- Handles nested object structures
- Supports CRD versioning

### Go AST Plugin

The Go AST plugin parses Go source code and generates corresponding Jsonnet code.

**Configuration:**
```yaml
sources:
  - type: go-ast
    name: "my-go-types"
    files:
      - "types/*.go"
    options:
      include_tags: true
      generate_constructors: true
```

**Features:**
- Parses Go structs and interfaces
- Converts Go types to Jsonnet
- Handles struct tags and annotations
- Generates constructor functions

📖 **[Detailed Go AST Generator Documentation](/plugins/go-ast-generator/)**

## Creating Custom Plugins

### Plugin Structure

Custom plugins should implement the following traits:

```rust
use gensonnet_rs::plugin::traits::{Generator, Lifecycle, Processor, Validator};

pub struct MyCustomPlugin {
    // Plugin state
}

impl Lifecycle for MyCustomPlugin {
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), Error> {
        // Initialize plugin with configuration
    }
    
    fn cleanup(&mut self) -> Result<(), Error> {
        // Clean up resources
    }
}

impl Processor for MyCustomPlugin {
    fn process(&self, input: &Input) -> Result<ProcessedData, Error> {
        // Process input data
    }
}

impl Generator for MyCustomPlugin {
    fn generate(&self, data: &ProcessedData) -> Result<GeneratedCode, Error> {
        // Generate output code
    }
}

impl Validator for MyCustomPlugin {
    fn validate(&self, code: &GeneratedCode) -> Result<ValidationResult, Error> {
        // Validate generated code
    }
}
```

### Plugin Manifest

Create a `plugin-manifest.yaml` file to describe your plugin:

```yaml
name: "my-custom-plugin"
version: "1.0.0"
description: "A custom plugin for Gensonnet-rs"
author: "Your Name"
license: "MIT"

capabilities:
  - "source-processing"
  - "code-generation"
  - "validation"

configuration:
  required:
    - "input_path"
  optional:
    - "output_format"
    - "options"

entry_point: "my_custom_plugin"
```

### Building and Installing

1. **Build your plugin:**
   ```bash
   cargo build --release
   ```

2. **Install the plugin:**
   ```bash
   gensonnet-rs plugin install ./target/release/my_custom_plugin
   ```

3. **Use in configuration:**
   ```yaml
   sources:
     - type: my-custom-plugin
       name: "my-data"
       input_path: "data/input.json"
       output_format: "jsonnet"
   ```

## Plugin Development Best Practices

### Error Handling

- Use descriptive error messages
- Provide context for debugging
- Handle edge cases gracefully

### Performance

- Process data incrementally when possible
- Cache expensive operations
- Use efficient data structures

### Testing

- Write unit tests for core functionality
- Create integration tests with sample data
- Test error conditions and edge cases

### Documentation

- Document all configuration options
- Provide usage examples
- Include troubleshooting guides

## Plugin Registry

The plugin registry allows you to discover and install plugins from external sources.

**List available plugins:**
```bash
gensonnet-rs plugin list
```

**Install from registry:**
```bash
gensonnet-rs plugin install my-custom-plugin
```

**Update plugins:**
```bash
gensonnet-rs plugin update
```

## External Plugin Development

For more detailed information about developing external plugins, see the [External Plugin System Documentation](/plugins/external-plugins/).

## Plugin API Reference

For complete API documentation, see the [Plugin API Reference](/api/plugins/).
