+++
title = "Gensonnet Documentation"
description = "Documentation for the Gensonnet project"
sort_by = "weight"
weight = 1
+++

# Welcome to Gensonnet

Gensonnet is a powerful tool for generating Jsonnet code from various sources including OpenAPI specifications, Go AST, and Custom Resource Definitions (CRDs).

## What is Gensonnet?

Gensonnet is a Rust-based code generator that helps you create Jsonnet code from:

- **OpenAPI Specifications**: Generate Jsonnet functions from API schemas
- **Go AST**: Convert Go structs and types to Jsonnet
- **Custom Resource Definitions (CRDs)**: Transform Kubernetes CRDs into Jsonnet

## Key Features

- **Plugin Architecture**: Extensible plugin system for different input sources
- **Type Safety**: Strong typing with Rust backend
- **Customizable Output**: Flexible configuration for generated code
- **Validation**: Built-in validation for generated Jsonnet code
- **Incremental Generation**: Support for incremental updates

## Quick Start

1. **Install Gensonnet**
   ```bash
   cargo install gensonnet
   ```

2. **Initialize a Project**
   ```bash
   gensonnet init
   ```

3. **Configure Your Sources**
   ```yaml
   # config.yaml
   sources:
     - type: openapi
       url: "https://api.example.com/openapi.json"
   ```

4. **Generate Jsonnet Code**
   ```bash
   gensonnet generate
   ```

## Getting Started

Check out our [Getting Started Guide](/getting-started/) to learn how to set up your first Gensonnet project.

## Plugins

Explore our [Plugin Documentation](/plugins/) to learn about the available plugins and how to create custom ones.

## Examples

Visit our [Examples Section](/examples/) to see real-world usage patterns and configurations.
