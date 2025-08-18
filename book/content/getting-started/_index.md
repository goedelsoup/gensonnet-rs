+++
title = "Getting Started"
description = "Learn how to get started with Gensonnet"
weight = 2
+++

# Getting Started with Gensonnet

This guide will walk you through setting up your first Gensonnet project and generating your first Jsonnet code.

## Prerequisites

Before you begin, make sure you have the following installed:

- **Rust**: Gensonnet is written in Rust and requires Rust 1.70 or later
- **Cargo**: Rust's package manager (comes with Rust)
- **Git**: For version control (optional but recommended)

## Installation

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/goedelsoup/gensonnet-rs.git
   cd gensonnet-rs
   ```

2. Build and install:
   ```bash
   cargo install --path .
   ```

### From Cargo

```bash
cargo install gensonnet
```

## Your First Project

### 1. Initialize a New Project

Create a new directory for your project and initialize it:

```bash
mkdir my-gensonnet-project
cd my-gensonnet-project
gensonnet init
```

This will create a basic project structure with a configuration file.

### 2. Configure Your Sources

Edit the generated `config.yaml` file to define your input sources:

```yaml
# config.yaml
sources:
  - type: openapi
    name: "my-api"
    url: "https://api.example.com/openapi.json"
    output:
      path: "generated/api.jsonnet"
      format: "jsonnet"

  - type: crd
    name: "my-crds"
    files:
      - "crds/*.yaml"
    output:
      path: "generated/crds.jsonnet"
      format: "jsonnet"
```

### 3. Generate Code

Run the generator to create your Jsonnet code:

```bash
gensonnet generate
```

This will process your sources and generate the specified output files.

### 4. Use Generated Code

The generated Jsonnet files can now be imported and used in your Jsonnet projects:

```jsonnet
// main.jsonnet
local api = import "generated/api.jsonnet";
local crds = import "generated/crds.jsonnet";

{
  api_resources: api.getResources(),
  custom_resources: crds.getCustomResources(),
}
```

## Configuration Options

### Source Types

Gensonnet supports several source types:

- **OpenAPI**: Generate from OpenAPI/Swagger specifications
- **CRD**: Generate from Kubernetes Custom Resource Definitions
- **Go AST**: Generate from Go source code

### Output Formats

- **Jsonnet**: Standard Jsonnet code
- **JSON**: JSON format for integration with other tools

### Advanced Configuration

For more advanced configuration options, see the [Configuration Reference](/api/configuration/).

## Next Steps

- Explore the [Plugin System](/plugins/) to understand how to extend Gensonnet
- Check out [Examples](/examples/) for real-world usage patterns
- Read the [API Reference](/api/) for detailed documentation
