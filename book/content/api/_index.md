+++
title = "API Reference"
description = "Complete API reference for Gensonnet-rs"
weight = 4
+++

# API Reference

This section provides comprehensive API documentation for Gensonnet-rs.

## Command Line Interface

### Global Options

All commands support the following global options:

- `--config <FILE>`: Path to configuration file (default: `config.yaml`)
- `--verbose`: Enable verbose output
- `--quiet`: Suppress output
- `--log-level <LEVEL>`: Set log level (debug, info, warn, error)

### Commands

#### `init`

Initialize a new Gensonnet-rs project.

```bash
gensonnet-rs init [OPTIONS]
```

**Options:**
- `--force`: Overwrite existing files
- `--template <TEMPLATE>`: Use specific template

#### `generate`

Generate Jsonnet code from configured sources.

```bash
gensonnet-rs generate [OPTIONS]
```

**Options:**
- `--source <SOURCE>`: Generate from specific source only
- `--output <PATH>`: Override output path
- `--dry-run`: Show what would be generated without writing files

#### `validate`

Validate generated code and configuration.

```bash
gensonnet-rs validate [OPTIONS]
```

**Options:**
- `--strict`: Enable strict validation
- `--format <FORMAT>`: Output format (text, json)

#### `test`

Run tests for generated code.

```bash
gensonnet-rs test [OPTIONS]
```

**Options:**
- `--test-file <FILE>`: Specific test file to run
- `--timeout <SECONDS>`: Test timeout in seconds

#### `plugin`

Manage plugins.

```bash
gensonnet-rs plugin <SUBCOMMAND>
```

**Subcommands:**
- `list`: List installed plugins
- `install <PLUGIN>`: Install a plugin
- `uninstall <PLUGIN>`: Uninstall a plugin
- `update`: Update all plugins

## Configuration Reference

### Top-level Configuration

```yaml
# config.yaml
version: "1.0"
project:
  name: "my-project"
  description: "My Gensonnet-rs project"

sources:
  # Source definitions (see below)

output:
  # Output configuration (see below)

plugins:
  # Plugin configuration (see below)
```

### Source Configuration

#### OpenAPI Source

```yaml
sources:
  - type: openapi
    name: "my-api"
    url: "https://api.example.com/openapi.json"
    # or
    file: "path/to/openapi.yaml"
    
    options:
      include_examples: true
      generate_validation: true
      skip_deprecated: false
      
    output:
      path: "generated/api.jsonnet"
      format: "jsonnet"
      template: "custom-template.jsonnet"
```

#### CRD Source

```yaml
sources:
  - type: crd
    name: "my-crds"
    files:
      - "crds/*.yaml"
    # or
    url: "https://raw.githubusercontent.com/example/crds/main/crds.yaml"
    
    options:
      generate_validation: true
      include_status: false
      skip_versions: ["v1alpha1"]
      
    output:
      path: "generated/crds.jsonnet"
      format: "jsonnet"
```

#### Go AST Source

```yaml
sources:
  - type: go-ast
    name: "my-go-types"
    files:
      - "types/*.go"
    
    options:
      include_tags: true
      generate_constructors: true
      skip_unexported: true
      
    output:
      path: "generated/types.jsonnet"
      format: "jsonnet"
```

### Output Configuration

```yaml
output:
  base_path: "generated"
  format: "jsonnet"  # jsonnet, json
  template: "default"
  
  validation:
    enabled: true
    strict: false
    
  formatting:
    indent: 2
    max_line_length: 80
    sort_keys: true
```

### Plugin Configuration

```yaml
plugins:
  registry:
    url: "https://plugins.gensonnet-rs.com"
    auth:
      token: "${PLUGIN_TOKEN}"
  
  custom:
    - name: "my-plugin"
      path: "./plugins/my-plugin"
      config:
        option1: "value1"
        option2: "value2"
```

## Plugin API

### Core Traits

#### `Lifecycle`

```rust
pub trait Lifecycle {
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), Error>;
    fn cleanup(&mut self) -> Result<(), Error>;
}
```

#### `Processor`

```rust
pub trait Processor {
    fn process(&self, input: &Input) -> Result<ProcessedData, Error>;
}
```

#### `Generator`

```rust
pub trait Generator {
    fn generate(&self, data: &ProcessedData) -> Result<GeneratedCode, Error>;
}
```

#### `Validator`

```rust
pub trait Validator {
    fn validate(&self, code: &GeneratedCode) -> Result<ValidationResult, Error>;
}
```

### Data Types

#### `Input`

```rust
pub struct Input {
    pub source_type: String,
    pub content: Vec<u8>,
    pub metadata: HashMap<String, Value>,
}
```

#### `ProcessedData`

```rust
pub struct ProcessedData {
    pub schema: Schema,
    pub types: Vec<TypeDefinition>,
    pub functions: Vec<FunctionDefinition>,
}
```

#### `GeneratedCode`

```rust
pub struct GeneratedCode {
    pub content: String,
    pub format: OutputFormat,
    pub metadata: HashMap<String, Value>,
}
```

#### `ValidationResult`

```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum GensonnetError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Generation error: {0}")]
    Generation(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Error Context

All errors include context information to help with debugging:

- Source file and line number
- Configuration values that caused the error
- Suggested fixes when possible

## Logging

Gensonnet-rs uses structured logging with the following levels:

- **DEBUG**: Detailed debugging information
- **INFO**: General information about operations
- **WARN**: Warning messages for potential issues
- **ERROR**: Error messages for failed operations

### Log Format

```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "level": "INFO",
  "target": "gensonnet_rs::generator",
  "message": "Generated Jsonnet code",
  "fields": {
    "source": "openapi",
    "output_path": "generated/api.jsonnet",
    "duration_ms": 150
  }
}
```

## Performance

### Optimization Tips

1. **Use incremental generation** when possible
2. **Cache processed data** for large sources
3. **Parallel processing** for multiple sources
4. **Lazy loading** for large files

### Memory Usage

- Process sources one at a time for large projects
- Use streaming for very large files
- Clean up temporary files after generation

## Security

### Plugin Security

- Plugins run in isolated environments
- Sandboxed file system access
- Network access restrictions
- Code signing verification

### Configuration Security

- Environment variable substitution
- Secret management integration
- Configuration validation
- Audit logging
