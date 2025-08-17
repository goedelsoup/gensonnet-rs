+++
title = "OpenAPI Generator"
description = "Generate type-safe Jsonnet libraries from OpenAPI/Swagger specifications"
weight = 20
+++

# OpenAPI Generator

The OpenAPI Generator is a powerful feature that allows you to generate type-safe Jsonnet libraries directly from OpenAPI/Swagger specifications. It supports both OpenAPI 2.0 (Swagger) and OpenAPI 3.0+ specifications.

## Features

- **Multi-Format Support**: Handles both YAML and JSON OpenAPI specifications
- **Version Compatibility**: Supports OpenAPI 2.0 (Swagger) and OpenAPI 3.0+
- **Schema Extraction**: Extracts schemas from both `definitions` (v2) and `components.schemas` (v3)
- **Rich Metadata**: Preserves API information, descriptions, and examples
- **Validation Support**: Handles validation rules and constraints
- **Complex Types**: Supports objects, arrays, enums, and nested schemas

## Configuration

To use the OpenAPI generator, add an `openapi` source to your configuration:

```yaml
version: "1.0"

sources:
  - type: openapi
    name: "my-api"
    git:
      url: "https://github.com/example/my-api.git"
      ref: "main"
    include_patterns:
      - "**/*.yaml"
      - "**/*.yml"
      - "**/*.json"
    exclude_patterns:
      - "**/*_test.yaml"
      - "vendor/**"
      - "**/generated/**"
    output_path: "./generated/openapi"
    openapi_version: "3.0"
    include_examples: true
    include_descriptions: true
    base_url: "https://api.example.com/v1"

output:
  base_path: "./generated"
  organization: "flat"

generation:
  fail_fast: false
  deep_merge_strategy: "default"
```

### Configuration Options

- **name**: Unique identifier for this source
- **git**: Git repository configuration
  - **url**: Repository URL
  - **ref**: Git reference (branch, tag, or commit)
- **include_patterns**: Glob patterns for OpenAPI files to include
- **exclude_patterns**: Glob patterns for files to exclude
- **output_path**: Directory for generated Jsonnet files
- **openapi_version**: Target OpenAPI version (2.0, 3.0, 3.1)
- **include_examples**: Whether to include examples in generated code
- **include_descriptions**: Whether to include descriptions in generated code
- **base_url**: Custom base URL for the API

## Supported OpenAPI Features

The generator supports the following OpenAPI constructs:

### Schema Types
- **Objects**: Complex nested structures
- **Arrays**: Lists with item schemas
- **Primitives**: string, integer, number, boolean
- **Enums**: Enumerated values
- **References**: `$ref` to other schemas
- **Composition**: allOf, anyOf, oneOf, not

### Validation Rules
- **String**: minLength, maxLength, pattern, format
- **Numeric**: minimum, maximum, exclusiveMinimum, exclusiveMaximum
- **Arrays**: minItems, maxItems, uniqueItems
- **Objects**: required properties, additionalProperties

### Metadata
- **Descriptions**: Schema and property descriptions
- **Examples**: Example values for schemas
- **Defaults**: Default values for properties
- **Formats**: Data formats (email, uuid, date-time, etc.)

## OpenAPI 2.0 (Swagger) Support

For OpenAPI 2.0 specifications, the generator extracts schemas from the `definitions` section:

```yaml
swagger: "2.0"
info:
  title: My API
  version: 1.0.0
definitions:
  User:
    type: object
    properties:
      id:
        type: integer
        description: "User ID"
      name:
        type: string
        description: "User name"
      email:
        type: string
        format: email
    required:
      - id
      - name
```

### Generated Output

```jsonnet
// Generated from OpenAPI 2.0 specification
{
  User: {
    id: null,  // integer - User ID
    name: null,  // string - User name
    email: null,  // string (email format)
  },
  
  // Validation functions
  validateUser: function(user) {
    // Validates User object
    assert user.id != null : "User ID is required";
    assert user.name != null : "User name is required";
    assert std.isString(user.email) : "Email must be a string";
  },
  
  // Constructor functions
  newUser: function(id, name, email) {
    {
      id: id,
      name: name,
      email: email,
    }
  }
}
```

## OpenAPI 3.0+ Support

For OpenAPI 3.0+ specifications, the generator extracts schemas from the `components.schemas` section:

```yaml
openapi: 3.0.0
info:
  title: My API
  version: 1.0.0
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
          description: "User ID"
        name:
          type: string
          description: "User name"
        email:
          type: string
          format: email
        status:
          type: string
          enum: ["active", "inactive", "pending"]
      required:
        - id
        - name
    UserList:
      type: array
      items:
        $ref: "#/components/schemas/User"
```

### Generated Output

```jsonnet
// Generated from OpenAPI 3.0 specification
{
  User: {
    id: null,  // integer - User ID
    name: null,  // string - User name
    email: null,  // string (email format)
    status: null,  // string (enum: active, inactive, pending)
  },
  
  UserList: [],  // array of User
  
  // Validation functions
  validateUser: function(user) {
    // Validates User object
    assert user.id != null : "User ID is required";
    assert user.name != null : "User name is required";
    assert std.isString(user.email) : "Email must be a string";
    if user.status != null {
      assert std.member(user.status, ["active", "inactive", "pending"]) : "Invalid status";
    }
  },
  
  validateUserList: function(users) {
    // Validates UserList array
    assert std.isArray(users) : "UserList must be an array";
    std.mapWithIndex(function(i, user) {
      validateUser(user);
    }, users);
  },
  
  // Constructor functions
  newUser: function(id, name, email, status) {
    {
      id: id,
      name: name,
      email: email,
      status: status,
    }
  },
  
  newUserList: function(users) {
    users
  }
}
```

## Advanced Features

### Schema References

The generator handles `$ref` references within the same file and across files:

```yaml
components:
  schemas:
    Address:
      type: object
      properties:
        street: { type: string }
        city: { type: string }
        country: { type: string }
    
    User:
      type: object
      properties:
        name: { type: string }
        address: { $ref: "#/components/schemas/Address" }
```

### Complex Types

Support for complex nested structures:

```yaml
components:
  schemas:
    ComplexObject:
      type: object
      properties:
        simple_array:
          type: array
          items:
            type: string
        object_array:
          type: array
          items:
            type: object
            properties:
              key: { type: string }
              value: { type: number }
        nested_object:
          type: object
          properties:
            level1:
              type: object
              properties:
                level2:
                  type: object
                  properties:
                    final: { type: string }
```

### Validation Rules

Comprehensive validation rule support:

```yaml
components:
  schemas:
    ValidatedUser:
      type: object
      properties:
        username:
          type: string
          minLength: 3
          maxLength: 20
          pattern: "^[a-zA-Z0-9_]+$"
        age:
          type: integer
          minimum: 0
          maximum: 150
        email:
          type: string
          format: email
        tags:
          type: array
          items:
            type: string
          minItems: 1
          maxItems: 10
          uniqueItems: true
      required:
        - username
        - email
```

## Configuration Examples

### Basic Configuration

```yaml
version: "1.0"

sources:
  - type: openapi
    name: "petstore"
    git:
      url: "https://github.com/OAI/OpenAPI-Specification.git"
      ref: "main"
    include_patterns:
      - "examples/v3.0/petstore.yaml"
    output_path: "./generated/petstore"

output:
  base_path: "./generated"
  organization: "flat"
```

### Advanced Configuration

```yaml
version: "1.0"

sources:
  - type: openapi
    name: "complex-api"
    git:
      url: "https://github.com/example/complex-api.git"
      ref: "v2.1.0"
    include_patterns:
      - "schemas/**/*.yaml"
      - "schemas/**/*.json"
    exclude_patterns:
      - "schemas/**/*_test.yaml"
      - "schemas/deprecated/**"
    output_path: "./generated/complex-api"
    openapi_version: "3.1"
    include_examples: true
    include_descriptions: true
    base_url: "https://api.example.com/v2"
    validation:
      strict: true
      include_format_validation: true
    generation:
      include_constructors: true
      include_validators: true
      include_helpers: true

output:
  base_path: "./generated"
  organization: "api_version"
  validation:
    enabled: true
    strict: false
```

## Generated Code Structure

The OpenAPI generator creates a well-organized Jsonnet library:

```jsonnet
// Generated from OpenAPI specification
// Source: https://github.com/example/my-api
// Version: 3.0.0

{
  // Type definitions
  User: {
    id: null,  // integer
    name: null,  // string
    email: null,  // string (email)
  },
  
  // Validation functions
  validateUser: function(user) {
    // Validation logic
  },
  
  // Constructor functions
  newUser: function(id, name, email) {
    // Constructor logic
  },
  
  // Helper functions
  isUser: function(obj) {
    // Type checking
  },
  
  // Constants
  USER_STATUSES: ["active", "inactive", "pending"],
  
  // Metadata
  _metadata: {
    source: "https://github.com/example/my-api",
    version: "3.0.0",
    generated_at: "2024-01-01T12:00:00Z",
  }
}
```

## Best Practices

### Schema Organization

1. **Use descriptive names**: Choose clear, descriptive schema names
2. **Group related schemas**: Organize schemas logically
3. **Use references**: Avoid duplicating schema definitions
4. **Document schemas**: Include descriptions for all schemas

### Validation

1. **Define constraints**: Use validation rules appropriately
2. **Test validation**: Verify validation rules work as expected
3. **Handle edge cases**: Consider boundary conditions
4. **Provide defaults**: Use default values where appropriate

### Performance

1. **Optimize references**: Minimize circular references
2. **Limit complexity**: Avoid overly complex nested structures
3. **Use patterns**: Leverage pattern validation for efficiency
4. **Cache results**: Enable caching for large specifications

## Troubleshooting

### Common Issues

1. **Schema not found**: Check file paths and include patterns
2. **Reference errors**: Verify `$ref` paths are correct
3. **Validation failures**: Review validation rule syntax
4. **Generation errors**: Check OpenAPI version compatibility

### Debug Commands

```bash
# Validate OpenAPI specification
gensonnet-rs validate --source openapi

# Generate with verbose output
RUST_LOG=debug gensonnet-rs generate

# Check generated code
gensonnet-rs check --output ./generated/openapi
```

## Examples

### Pet Store API

```yaml
# petstore.yaml
openapi: 3.0.0
info:
  title: Pet Store API
  version: 1.0.0
components:
  schemas:
    Pet:
      type: object
      properties:
        id:
          type: integer
        name:
          type: string
        status:
          type: string
          enum: ["available", "pending", "sold"]
      required:
        - name
```

### Generated Pet Store Code

```jsonnet
// Generated from Pet Store API
{
  Pet: {
    id: null,  // integer
    name: null,  // string
    status: null,  // string (enum: available, pending, sold)
  },
  
  validatePet: function(pet) {
    assert pet.name != null : "Pet name is required";
    if pet.status != null {
      assert std.member(pet.status, ["available", "pending", "sold"]) : "Invalid status";
    }
  },
  
  newPet: function(name, id, status) {
    {
      id: id,
      name: name,
      status: status,
    }
  },
  
  PET_STATUSES: ["available", "pending", "sold"],
}
```

## Next Steps

- Explore the [Plugin API Reference](/api/plugins/) for advanced customization
- Check out [Examples](/examples/) for more OpenAPI integration patterns
- Learn about [External Plugins](/plugins/external-plugins/) for custom OpenAPI processing
