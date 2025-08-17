# JsonnetGen - Type-safe Jsonnet Library Generator

[![CI](https://github.com/goedelsoup/gensonnet-rs/workflows/CI/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3ACI)
[![Release](https://github.com/goedelsoup/gensonnet-rs/workflows/Release/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3ARelease)
[![Test Matrix](https://github.com/goedelsoup/gensonnet-rs/workflows/Test%20Matrix/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3A%22Test+Matrix%22)
[![Documentation](https://github.com/goedelsoup/gensonnet-rs/workflows/Documentation/badge.svg)](https://github.com/goedelsoup/gensonnet-rs/actions?query=workflow%3ADocumentation)

A production-ready Rust CLI tool that generates type-safe Jsonnet libraries from various schema sources, starting with Kubernetes CustomResourceDefinitions (CRDs).

## Features

### Phase 1: Foundation ✅
- **CRD Support**: Parse Kubernetes CustomResourceDefinitions and generate type-safe Jsonnet libraries
- **Git Integration**: Clone and cache Git repositories with support for branches, tags, and commit SHAs
- **XDG Compliance**: Uses XDG-compliant configuration and cache directories
- **Lockfile System**: Reproducible builds with exact commit SHAs and file checksums
- **Flexible Output**: Multiple organization strategies (API version, flat, hierarchical)
- **Validation**: Convert OpenAPI schema constraints to Jsonnet assertions
- **Extensible**: Plugin architecture for future source types (Go AST, OpenAPI)

### Phase 2: Advanced CRD Processing ✅
- **Advanced OpenAPI Schema Processing**: Deep schema analysis and type inference
- **Validation Rule Interpretation**: Convert OpenAPI constraints to Jsonnet validation functions
- **Complex Type Handling**: Support for arrays, objects, enums, patterns, and references
- **Schema Analysis**: Automatic field type detection and nested property analysis
- **Enhanced Code Generation**: Generate validation utilities and helper functions
- **Error Recovery**: Partial generation when some sources fail

### Phase 3: Advanced Features ✅
- **Incremental Generation**: Only regenerate changed sources and their dependencies
- **Dependency Tracking**: Track relationships between sources and files
- **Topological Sorting**: Ensure correct generation order based on dependencies
- **Cache Management**: Intelligent caching with staleness detection
- **Parallel Processing**: Process multiple sources concurrently
- **Advanced CLI**: Status monitoring, cleanup, and incremental generation commands
- **Statistics Tracking**: Monitor performance and cache hit rates

## Installation

### From Source

```bash
git clone https://github.com/goedelsoup/gensonnet-rs.git
cd gensonnet-rs
cargo build --release
cargo install --path .
```

### Requirements

- Rust 1.70+
- Git (for repository operations)

## Quick Start

1. **Initialize a configuration file**:
   ```bash
   jsonnet-gen init --example
   ```

## Documentation

📚 **Complete documentation is available at: [https://goedelsoup.github.io/gensonnet-rs](https://goedelsoup.github.io/gensonnet-rs)**

The documentation site includes:
- Getting started guide
- Plugin documentation
- API reference
- Examples and tutorials
- Best practices

**Local Development:**
```bash
just docs-dev  # Start local development server
```

2. **Edit the configuration** (`.jsonnet-gen.yaml`):
   ```yaml
   version: "1.0"
   sources:
     - type: "crd"
       name: "istio-crds"
       git:
         url: "https://github.com/istio/istio.git"
         ref: "1.20.0"
       filters:
         - "networking.istio.io/*"
       output_path: "./generated/istio"
   
   output:
     base_path: "./generated"
     organization: "api_version"
   
   generation:
     fail_fast: false
     deep_merge_strategy: "default"
   ```

3. **Generate Jsonnet libraries**:
   ```bash
   jsonnet-gen generate
   ```

4. **Use the generated libraries**:
   ```jsonnet
   local istio = import "./generated/istio/index.libsonnet";
   
   istio.networking_istio_io_v1beta1.virtualService({
     metadata: {
       name: "example",
       namespace: "default",
     },
     spec: {
       hosts: ["example.com"],
       http: [{
         route: [{
           destination: {
             host: "example-service",
             port: { number: 8080 },
           },
         }],
       }],
     },
   })
   ```

## Plugin Architecture

The tool now supports a comprehensive plugin architecture that enables extensibility and new source type processing.

### Plugin System Overview

The plugin system provides:
- **Extensible Architecture**: Add new source types and processing capabilities
- **Trait-Based Interfaces**: Clean, type-safe plugin interfaces
- **Plugin Registry**: Discover and manage plugins
- **Lifecycle Management**: Initialize, start, stop, and reload plugins
- **Built-in Plugins**: Go AST processing and CRD handling

### Available Plugins

#### Go AST Plugin
Processes Go source code and extracts type information:
```bash
# List available plugins
jsonnet-gen plugins list

# Show plugin information
jsonnet-gen plugins info go-ast:builtin
```

#### OpenAPI Plugin
Processes OpenAPI/Swagger specifications and extracts schema information:
```bash
# Show plugin information
jsonnet-gen plugins info openapi:builtin
```

Configuration example:
```yaml
sources:
  - type: openapi
    name: "user-management-api"
    git:
      url: "https://github.com/example/user-management-api.git"
      ref: "main"
    include_patterns:
      - "**/*.yaml"
      - "**/*.yml"
      - "**/*.json"
    exclude_patterns:
      - "**/*_test.yaml"
      - "vendor/**"
    output_path: "./generated/openapi"
    openapi_version: "3.0"
    include_examples: true
    include_descriptions: true
    base_url: "https://api.example.com/v1"
```

Features:
- **Multi-Format Support**: YAML and JSON OpenAPI specifications
- **Version Compatibility**: OpenAPI 2.0 (Swagger) and 3.0+
- **Schema Extraction**: From `definitions` (v2) and `components.schemas` (v3)
- **Rich Metadata**: Preserves descriptions, examples, and validation rules
- **Complex Types**: Objects, arrays, enums, and nested schemas

#### Plugin Management
```bash
# List all plugins
jsonnet-gen plugins list --detailed

# Enable/disable plugins
jsonnet-gen plugins enable <plugin-id>
jsonnet-gen plugins disable <plugin-id>

# Install/uninstall plugins
jsonnet-gen plugins install <source>
jsonnet-gen plugins uninstall <plugin-id>
```

### Creating Custom Plugins

Plugins are defined using manifest files (`plugin.yaml`):

```yaml
metadata:
  id: "my-plugin:1.0"
  name: "My Custom Plugin"
  version: "1.0.0"
  description: "Processes custom source types"
  supported_types: ["custom"]
  capabilities:
    - Parse
    - SchemaExtraction

config:
  plugin_id: "my-plugin:1.0"
  config:
    # Plugin-specific configuration
    custom_option: true
  enabled_capabilities:
    - Parse
    - SchemaExtraction
```

### Plugin Development

To create a custom plugin:

1. Implement the `Plugin` trait
2. Create a plugin factory implementing `PluginFactory`
3. Register your plugin with the system
4. Create a manifest file

See the [plugin examples](examples/) for complete working examples.

## Advanced Usage

### Incremental Generation

```bash
# Check generation status
jsonnet-gen status

# Perform incremental generation
jsonnet-gen incremental

# Force full regeneration
jsonnet-gen incremental --force

# Parallel processing
jsonnet-gen incremental --parallel --max-workers 8
```

### Cache Management

```bash
# Clean up stale entries (older than 1 week)
jsonnet-gen cleanup

# Clean up entries older than 24 hours
jsonnet-gen cleanup --max-age 24

# Dry run to see what would be cleaned
jsonnet-gen cleanup --dry-run
```

### Advanced Generation Options

```bash
# Generate with parallel processing
jsonnet-gen generate --parallel --max-workers 4

# Dry run to see what would be generated
jsonnet-gen generate --dry-run

# Force regeneration
jsonnet-gen generate --force

# Fail fast on errors
jsonnet-gen generate --fail-fast
```

## Configuration

### Source Types

#### CRD Source

```yaml
- type: "crd"
  name: "my-crds"
  git:
    url: "https://github.com/example/k8s-manifests.git"
    ref: "main"  # branch, tag, or commit SHA
  filters:
    - "example.com/v1"
    - "apps.example.com/*"
  output_path: "./generated/my-crds"
```

#### Authentication

```yaml
git:
  url: "https://github.com/private/repo.git"
  ref: "main"
  auth:
    type: "token"
    token: "${GITHUB_TOKEN}"
```

Supported authentication types:
- `token`: Personal access token
- `ssh`: SSH key authentication
- `basic`: Username/password

### Output Organization

- `api_version`: Organize by API version (e.g., `apps/v1/`, `networking.k8s.io/v1/`)
- `flat`: All files in one directory
- `hierarchical`: Nested directories matching schema organization

## CLI Commands

### `init`

Initialize a new configuration file.

```bash
jsonnet-gen init                    # Create empty config
jsonnet-gen init --example          # Create example config
jsonnet-gen init -o custom.yaml     # Specify output file
```

### `generate`

Generate Jsonnet libraries from configured sources.

```bash
jsonnet-gen generate                # Use default config
jsonnet-gen generate -c custom.yaml # Use custom config
jsonnet-gen generate --fail-fast    # Stop on first error
jsonnet-gen generate --dry-run      # Don't write files
jsonnet-gen generate -o ./output    # Override output directory
```

### `incremental`

Perform incremental generation with advanced features.

```bash
jsonnet-gen incremental             # Incremental generation
jsonnet-gen incremental --force     # Force full regeneration
jsonnet-gen incremental --parallel  # Parallel processing
jsonnet-gen incremental --dry-run   # Show what would be generated
```

### `status`

Show generation status and incremental generation information.

```bash
jsonnet-gen status                  # Basic status
jsonnet-gen status --detailed       # Detailed information
```

### `cleanup`

Clean up stale entries from lockfile and cache.

```bash
jsonnet-gen cleanup                 # Clean up entries older than 1 week
jsonnet-gen cleanup --max-age 24    # Clean up entries older than 24 hours
jsonnet-gen cleanup --dry-run       # Show what would be cleaned
```

### `validate`

Validate configuration file.

```bash
jsonnet-gen validate
jsonnet-gen validate -c custom.yaml
```

### `lock`

Manage lockfile for reproducible builds.

```bash
jsonnet-gen lock --status           # Show lockfile status
jsonnet-gen lock --update           # Update lockfile
```

### `info`

Show tool information.

```bash
jsonnet-gen info
jsonnet-gen info --detailed
```

## Generated Code Structure

The tool generates Jsonnet libraries with the following structure:

```
generated/
├── index.libsonnet              # Main index file
├── _meta.libsonnet              # Generation metadata
├── _validation.libsonnet        # Validation utilities
└── apps_v1/                     # API version directory
    ├── _index.libsonnet         # Version index
    ├── deployment.libsonnet     # Generated CRD library
    └── service.libsonnet        # Generated CRD library
```

### Generated Functions

For each CRD, the tool generates:

1. **Main resource function**:
   ```jsonnet
   function(metadata, spec={}) {
     local validated = validate.deployment(metadata, spec);
     apiVersion: "apps/v1",
     kind: "Deployment",
     metadata: validated.metadata,
     spec: validated.spec,
   }
   ```

2. **Validation functions** (from OpenAPI constraints):
   ```jsonnet
   function validateDeployment(metadata, spec) {
     // Validate metadata
     assert metadata != null : "metadata is required";
     assert metadata.name != null : "metadata.name is required";
     
     // Validate spec
     local validated_spec = spec + {
       replicas: spec.replicas || 1,
     };
     
     if spec.replicas != null then
       assert spec.replicas >= 0 : "replicas must be non-negative";
     
     {
       metadata: metadata,
       spec: validated_spec,
     }
   }
   ```

3. **Field-specific functions**:
   ```jsonnet
   function withReplicas(replicas) {
     spec +: {
       replicas: replicas,
     },
   }
   ```

4. **Helper functions**:
   ```jsonnet
   local deployment = {
     new: deployment,
     withReplicas: withReplicas,
     withImage: withImage,
   };
   ```

### Validation Utilities

The tool generates comprehensive validation utilities:

```jsonnet
{
  assertRequired: function(field, value, fieldName) {
    assert value != null : fieldName + " is required";
    value
  },
  
  assertString: function(value, fieldName) {
    assert std.type(value) == "string" : fieldName + " must be a string";
    value
  },
  
  assertPattern: function(value, pattern, fieldName) {
    assert std.regexMatch(pattern, value) : fieldName + " must match pattern " + pattern;
    value
  },
  
  assertMinLength: function(value, minLength, fieldName) {
    assert std.length(value) >= minLength : fieldName + " must be at least " + minLength + " characters";
    value
  },
  
  // ... and many more validation functions
}
```

## Development

### Using Just (Recommended)

This project uses [Just](https://github.com/casey/just) for development tasks. Install it with:

```bash
# macOS
brew install just

# Linux
cargo install just

# Windows
scoop install just
```

Then use the justfile for common tasks:

```bash
# Show all available tasks
just --list

# Run all checks (formatting, linting, tests)
just check-all

# Run tests
just test-all

# Build release version
just build-release

# Format code
just fmt-fix

# Run clippy with fixes
just clippy-fix

# Show project status
just status

# Create test CRD file
just create-test-crd
```

### Manual Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --release

# Format
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Documentation
cargo doc --no-deps --open
```

### Project Structure

```
src/
├── cli/          # Command-line interface
│   └── commands/ # Individual command implementations
├── config/       # Configuration management
├── crd/          # CRD parsing and schema extraction
├── generator/    # Jsonnet code generation
├── git/          # Git repository management
├── lockfile/     # Lockfile management
├── utils/        # Utility functions
├── lib.rs        # Library entry point
└── main.rs       # Binary entry point
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite: `just test-all`
6. Submit a pull request

### Code Style

- Follow Rust conventions
- Use `just fmt-fix` for formatting
- Use `just clippy-fix` for linting
- Write comprehensive tests
- Run `just check-all` before submitting PRs

### GitHub Actions

This project uses comprehensive GitHub Actions workflows for CI/CD:

- **CI**: Automated testing, linting, and building
- **Release**: Automated releases with multi-platform binaries
- **Security**: Dependency scanning and CodeQL analysis
- **Documentation**: Automated doc generation and deployment

See [.github/README.md](.github/README.md) for detailed workflow documentation.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Examples

See the [examples/](examples/) directory for complete working examples.

## Support

- **Issues**: [GitHub Issues](https://github.com/goedelsoup/gensonnet-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/goedelsoup/gensonnet-rs/discussions)
- **Documentation**: [GitHub Wiki](https://github.com/goedelsoup/gensonnet-rs/wiki)
