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
      name:
        type: string
    required:
      - id
      - name
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
        name:
          type: string
      required:
        - id
        - name
```

## Generated Output

For each OpenAPI schema, the generator creates a Jsonnet library file:

```jsonnet
// Generated from OpenAPI: User
// Source: /path/to/openapi.yaml

local k = import "k.libsonnet";
local validate = import "_validation.libsonnet";

// Create a new User resource
function(metadata, spec={}) {
  apiVersion: "user",
  kind: "User",
  metadata: metadata,
  spec: spec,
}
```

## Usage Examples

### Basic Object Schema
```yaml
# openapi.yaml
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
          format: email
      required:
        - id
        - name
        - email
```

Generated Jsonnet:
```jsonnet
// user.libsonnet
function(metadata, spec={}) {
  apiVersion: "user",
  kind: "User",
  metadata: metadata,
  spec: spec,
}
```

### Complex Schema with Validation
```yaml
# openapi.yaml
components:
  schemas:
    CreateUserRequest:
      type: object
      properties:
        username:
          type: string
          minLength: 3
          maxLength: 50
          pattern: '^[a-zA-Z0-9_-]+$'
        email:
          type: string
          format: email
        age:
          type: integer
          minimum: 0
          maximum: 150
        roles:
          type: array
          items:
            type: string
            enum: [admin, user, moderator]
          default: ["user"]
      required:
        - username
        - email
```

The generator will extract the validation rules and create appropriate Jsonnet schemas.

## Advanced Features

### Schema References
The generator handles `$ref` references within the same specification:

```yaml
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        profile:
          $ref: '#/components/schemas/UserProfile'
    UserProfile:
      type: object
      properties:
        bio:
          type: string
        avatar:
          type: string
          format: uri
```

### Enum Support
```yaml
components:
  schemas:
    UserStatus:
      type: string
      enum:
        - active
        - inactive
        - suspended
      default: active
```

### Array Types
```yaml
components:
  schemas:
    UserList:
      type: array
      items:
        $ref: '#/components/schemas/User'
      minItems: 1
      maxItems: 100
```

### Nested Objects
```yaml
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
        metadata:
          type: object
          additionalProperties: true
          properties:
            department:
              type: string
            location:
              type: string
```

## Error Handling

The generator provides detailed error reporting:
- Invalid OpenAPI specification format
- Missing required fields
- Unsupported OpenAPI features
- File access issues
- Schema parsing errors

## Performance

The OpenAPI parser is highly efficient:
- Fast parsing of large OpenAPI specifications
- Memory efficient schema representation
- Parallel processing of multiple files
- Incremental parsing support

## Limitations

- Currently supports OpenAPI 2.0 and 3.0+ specifications
- Some advanced OpenAPI features may not be fully supported
- Generated Jsonnet may require manual customization for complex use cases
- External references (`$ref` to other files) are not yet supported

## Future Enhancements

- Support for external schema references
- Enhanced validation rule extraction
- Custom Jsonnet template support
- Integration with OpenAPI tooling
- Support for OpenAPI 3.1 specific features
- Schema composition and inheritance
- API endpoint generation
