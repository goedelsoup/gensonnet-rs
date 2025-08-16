# Go AST Generator

The Go AST Generator is a powerful feature that allows you to generate type-safe Jsonnet libraries directly from Go source code. It uses tree-sitter for robust parsing of Go syntax and can handle complex Go type definitions.

## Features

- **Robust Parsing**: Uses tree-sitter for accurate Go syntax parsing
- **Type Extraction**: Extracts structs, interfaces, and other type definitions
- **Documentation Support**: Preserves Go comments and documentation
- **Tag Processing**: Handles JSON tags and validation tags
- **Complex Types**: Supports pointers, slices, maps, arrays, and embedded types
- **Package Management**: Processes imports and package declarations

## Configuration

To use the Go AST generator, add a `go_ast` source to your configuration:

```yaml
version: "1.0"

sources:
  - type: go_ast
    name: "my-go-types"
    git:
      url: "https://github.com/example/my-go-repo.git"
      ref: "main"
    include_patterns:
      - "**/*.go"
    exclude_patterns:
      - "**/*_test.go"
      - "vendor/**"
      - "**/generated/**"
    output_path: "./generated/go-ast"
    package_filters:
      - "main"
      - "api"

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
- **include_patterns**: Glob patterns for Go files to include
- **exclude_patterns**: Glob patterns for files to exclude
- **output_path**: Directory for generated Jsonnet files
- **package_filters**: Optional list of package names to filter

## Supported Go Types

The generator supports the following Go type constructs:

### Struct Types
```go
type User struct {
    Name string `json:"name" validate:"required"`
    Age  int    `json:"age,omitempty"`
    Email string `json:"email"`
}
```

### Interface Types
```go
type UserService interface {
    CreateUser(user *User) error
    GetUser(name string) (*User, error)
}
```

### Complex Types
- **Pointers**: `*User`
- **Slices**: `[]string`
- **Maps**: `map[string]string`
- **Arrays**: `[10]int`
- **Embedded Types**: `User` in `type Admin struct { User }`

### Type Aliases
```go
type UserID string
type UserStatus string
```

## Generated Output

For each Go type, the generator creates a Jsonnet library file:

```jsonnet
// Generated from Go AST: User
// Source: /path/to/user.go

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

### Basic Struct
```go
// user.go
type User struct {
    Name  string `json:"name"`
    Email string `json:"email"`
    Age   int    `json:"age,omitempty"`
}
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

### Complex Type with Validation
```go
type CreateUserRequest struct {
    User User `json:"user"`
    ValidateEmail bool `json:"validateEmail" default:"true"`
}
```

The generator will extract the nested `User` type and create appropriate Jsonnet schemas.

## Advanced Features

### Documentation Preservation
Go comments are preserved and can be used for generating documentation:

```go
// User represents a user in the system
type User struct {
    // Name is the unique identifier for the user
    Name string `json:"name" validate:"required"`
}
```

### Tag Processing
The generator processes various Go tags:
- `json`: Field name mapping
- `validate`: Validation rules
- `default`: Default values
- `omitempty`: Optional fields

### Package Filtering
You can filter by specific packages:

```yaml
package_filters:
  - "main"
  - "api"
  - "types"
```

## Error Handling

The generator provides detailed error reporting:
- Syntax errors in Go files
- Missing dependencies
- Invalid type definitions
- File access issues

## Performance

The tree-sitter based parser is highly efficient:
- Fast parsing of large Go codebases
- Incremental parsing support
- Memory efficient AST representation
- Parallel processing of multiple files

## Limitations

- Currently supports Go 1.x syntax
- Some advanced Go features may not be fully supported
- Generated Jsonnet may require manual customization for complex use cases

## Future Enhancements

- Support for Go modules and dependency resolution
- Enhanced validation rule extraction
- Custom Jsonnet template support
- Integration with Go toolchain
- Support for Go generics (Go 1.18+)
