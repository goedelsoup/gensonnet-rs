+++
title = "Examples"
description = "Real-world examples and usage patterns for Gensonnet-rs"
weight = 5
+++

# Examples

This section provides real-world examples and usage patterns for Gensonnet-rs.

## Basic Examples

### OpenAPI to Jsonnet

Generate Jsonnet functions from an OpenAPI specification.

**Input (OpenAPI):**
```yaml
openapi: 3.0.0
info:
  title: User API
  version: 1.0.0
paths:
  /users:
    get:
      summary: List users
      responses:
        '200':
          description: List of users
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/User'
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
        email:
          type: string
```

**Configuration:**
```yaml
sources:
  - type: openapi
    name: "user-api"
    file: "user-api.yaml"
    output:
      path: "generated/user-api.jsonnet"
```

**Generated Output:**
```jsonnet
// Generated from OpenAPI specification
{
  // User type definition
  User: {
    id: null,  // integer
    name: null,  // string
    email: null,  // string
  },
  
  // API functions
  listUsers: function() {
    // GET /users
    // Returns: array of User
  },
  
  // Validation functions
  validateUser: function(user) {
    // Validates User object
  }
}
```

### CRD to Jsonnet

Convert Kubernetes Custom Resource Definitions to Jsonnet types.

**Input (CRD):**
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: applications.example.com
spec:
  group: example.com
  names:
    kind: Application
    listKind: ApplicationList
    plural: applications
    singular: application
  scope: Namespaced
  versions:
    - name: v1
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                name:
                  type: string
                replicas:
                  type: integer
                  minimum: 1
```

**Configuration:**
```yaml
sources:
  - type: crd
    name: "applications"
    files:
      - "crds/application.yaml"
    output:
      path: "generated/applications.jsonnet"
```

**Generated Output:**
```jsonnet
// Generated from CRD specification
{
  Application: {
    apiVersion: "example.com/v1",
    kind: "Application",
    metadata: {
      name: null,  // string
      namespace: null,  // string
    },
    spec: {
      name: null,  // string
      replicas: null,  // integer (min: 1)
    },
  },
  
  // Validation functions
  validateApplication: function(app) {
    // Validates Application object
  },
  
  // Constructor functions
  newApplication: function(name, replicas) {
    {
      apiVersion: "example.com/v1",
      kind: "Application",
      metadata: {
        name: name,
      },
      spec: {
        name: name,
        replicas: replicas,
      },
    }
  }
}
```

### Go AST to Jsonnet

Parse Go structs and generate corresponding Jsonnet code.

**Input (Go):**
```go
package types

type User struct {
    ID    int    `json:"id"`
    Name  string `json:"name"`
    Email string `json:"email"`
}

type Config struct {
    Host     string `json:"host"`
    Port     int    `json:"port"`
    Database string `json:"database"`
}
```

**Configuration:**
```yaml
sources:
  - type: go-ast
    name: "go-types"
    files:
      - "types/*.go"
    output:
      path: "generated/go-types.jsonnet"
```

**Generated Output:**
```jsonnet
// Generated from Go AST
{
  User: {
    id: null,  // int
    name: null,  // string
    email: null,  // string
  },
  
  Config: {
    host: null,  // string
    port: null,  // int
    database: null,  // string
  },
  
  // Constructor functions
  newUser: function(id, name, email) {
    {
      id: id,
      name: name,
      email: email,
    }
  },
  
  newConfig: function(host, port, database) {
    {
      host: host,
      port: port,
      database: database,
    }
  }
}
```

## Advanced Examples

### Multi-Source Project

Combine multiple sources in a single project.

**Configuration:**
```yaml
project:
  name: "my-kubernetes-project"
  description: "Kubernetes configuration with custom resources"

sources:
  # OpenAPI for external APIs
  - type: openapi
    name: "external-api"
    url: "https://api.example.com/openapi.json"
    output:
      path: "generated/external-api.jsonnet"
  
  # CRDs for custom resources
  - type: crd
    name: "custom-resources"
    files:
      - "crds/*.yaml"
    output:
      path: "generated/custom-resources.jsonnet"
  
  # Go types for internal structures
  - type: go-ast
    name: "internal-types"
    files:
      - "internal/types/*.go"
    output:
      path: "generated/internal-types.jsonnet"

output:
  base_path: "generated"
  format: "jsonnet"
  validation:
    enabled: true
```

**Main Jsonnet File:**
```jsonnet
// main.jsonnet
local externalApi = import "generated/external-api.jsonnet";
local customResources = import "generated/custom-resources.jsonnet";
local internalTypes = import "generated/internal-types.jsonnet";

{
  // Kubernetes resources
  resources: [
    // Custom application
    customResources.newApplication("my-app", 3),
    
    // Service using external API
    {
      apiVersion: "v1",
      kind: "Service",
      metadata: {
        name: "my-app-service",
      },
      spec: {
        ports: [
          {
            port: 80,
            targetPort: 8080,
          },
        ],
        selector: {
          app: "my-app",
        },
      },
    },
  ],
  
  // Configuration using internal types
  config: internalTypes.newConfig("localhost", 8080, "myapp"),
}
```

### Custom Plugin Example

Create a custom plugin for a specific data format.

**Plugin Implementation:**
```rust
use gensonnet_rs::plugin::traits::{Generator, Lifecycle, Processor, Validator};

pub struct CustomPlugin {
    config: PluginConfig,
}

impl Lifecycle for CustomPlugin {
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), Error> {
        self.config = config.clone();
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl Processor for CustomPlugin {
    fn process(&self, input: &Input) -> Result<ProcessedData, Error> {
        // Parse custom format
        let data = parse_custom_format(&input.content)?;
        
        Ok(ProcessedData {
            schema: data.schema,
            types: data.types,
            functions: data.functions,
        })
    }
}

impl Generator for CustomPlugin {
    fn generate(&self, data: &ProcessedData) -> Result<GeneratedCode, Error> {
        // Generate Jsonnet code
        let code = generate_jsonnet(data)?;
        
        Ok(GeneratedCode {
            content: code,
            format: OutputFormat::Jsonnet,
            metadata: HashMap::new(),
        })
    }
}

impl Validator for CustomPlugin {
    fn validate(&self, code: &GeneratedCode) -> Result<ValidationResult, Error> {
        // Validate generated code
        let result = validate_jsonnet(&code.content)?;
        
        Ok(ValidationResult {
            is_valid: result.is_valid,
            errors: result.errors,
            warnings: result.warnings,
        })
    }
}
```

**Plugin Configuration:**
```yaml
plugins:
  custom:
    - name: "custom-plugin"
      path: "./plugins/custom-plugin"
      config:
        format: "custom"
        options:
          include_metadata: true
```

**Usage:**
```yaml
sources:
  - type: custom-plugin
    name: "custom-data"
    file: "data/custom-format.dat"
    output:
      path: "generated/custom-data.jsonnet"
```

## Integration Examples

### CI/CD Pipeline

Integrate Gensonnet-rs into a CI/CD pipeline.

**GitHub Actions:**
```yaml
name: Generate Jsonnet

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install Gensonnet-rs
        run: cargo install gensonnet-rs
      
      - name: Generate Jsonnet
        run: gensonnet-rs generate
      
      - name: Validate Generated Code
        run: gensonnet-rs validate
      
      - name: Commit Changes
        run: |
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add generated/
          git commit -m "Update generated Jsonnet code" || exit 0
          git push
```

### Kubernetes Operator

Use Gensonnet-rs in a Kubernetes operator.

**Operator Configuration:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: gensonnet-config
data:
  config.yaml: |
    sources:
      - type: crd
        name: "operator-crds"
        files:
          - "/crds/*.yaml"
        output:
          path: "/generated/crds.jsonnet"
```

**Operator Code:**
```rust
use gensonnet_rs::Generator;

pub struct Operator {
    generator: Generator,
}

impl Operator {
    pub fn new() -> Result<Self, Error> {
        let generator = Generator::from_config("config.yaml")?;
        Ok(Self { generator })
    }
    
    pub fn reconcile(&self, cr: &CustomResource) -> Result<(), Error> {
        // Generate Jsonnet code
        self.generator.generate()?;
        
        // Apply generated configuration
        self.apply_configuration()?;
        
        Ok(())
    }
}
```

## Best Practices

### Project Structure

```
my-project/
├── config.yaml
├── sources/
│   ├── openapi/
│   │   └── api.yaml
│   ├── crds/
│   │   └── *.yaml
│   └── go/
│       └── types.go
├── generated/
│   ├── api.jsonnet
│   ├── crds.jsonnet
│   └── types.jsonnet
├── templates/
│   └── custom-template.jsonnet
└── main.jsonnet
```

### Configuration Management

- Use environment variables for sensitive data
- Version control your configuration files
- Use separate configs for different environments
- Document all configuration options

### Code Organization

- Keep generated code separate from source code
- Use meaningful names for generated files
- Include validation in your workflows
- Test generated code thoroughly

### Performance Optimization

- Use incremental generation for large projects
- Cache processed data when possible
- Parallelize generation for multiple sources
- Monitor generation performance
