+++
title = "Go AST Generator"
description = "Generate type-safe Jsonnet libraries from Go source code using tree-sitter parsing"
weight = 30
+++

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

{
  User: {
    name: null,  // string (required)
    age: null,   // int
    email: null, // string
  },
  
  // Validation functions
  validateUser: function(user) {
    // Validates User object based on Go tags
    assert user.name != null : "Name is required";
    assert std.isString(user.name) : "Name must be a string";
    assert std.isNumber(user.age) : "Age must be a number";
    assert std.isString(user.email) : "Email must be a string";
  },
  
  // Constructor functions
  newUser: function(name, age, email) {
    {
      name: name,
      age: age,
      email: email,
    }
  },
  
  // Type checking functions
  isUser: function(obj) {
    std.isObject(obj) &&
    std.isString(obj.name) &&
    std.isNumber(obj.age) &&
    std.isString(obj.email)
  }
}
```

## Advanced Type Support

### Embedded Structs

```go
type Base struct {
    ID   string `json:"id"`
    Name string `json:"name"`
}

type User struct {
    Base
    Email string `json:"email"`
}
```

Generated output:
```jsonnet
{
  Base: {
    id: null,   // string
    name: null, // string
  },
  
  User: {
    id: null,    // string (inherited from Base)
    name: null,  // string (inherited from Base)
    email: null, // string
  },
  
  validateUser: function(user) {
    validateBase(user);
    assert std.isString(user.email) : "Email must be a string";
  },
  
  newUser: function(id, name, email) {
    {
      id: id,
      name: name,
      email: email,
    }
  }
}
```

### Complex Nested Types

```go
type Config struct {
    Database DatabaseConfig `json:"database"`
    Cache    CacheConfig    `json:"cache"`
}

type DatabaseConfig struct {
    Host     string `json:"host"`
    Port     int    `json:"port"`
    Username string `json:"username"`
    Password string `json:"password"`
}

type CacheConfig struct {
    Enabled bool   `json:"enabled"`
    TTL     int    `json:"ttl"`
    Type    string `json:"type"`
}
```

Generated output:
```jsonnet
{
  DatabaseConfig: {
    host: null,     // string
    port: null,     // int
    username: null, // string
    password: null, // string
  },
  
  CacheConfig: {
    enabled: null, // bool
    ttl: null,     // int
    type: null,    // string
  },
  
  Config: {
    database: null, // DatabaseConfig
    cache: null,    // CacheConfig
  },
  
  validateDatabaseConfig: function(db) {
    assert std.isString(db.host) : "Host must be a string";
    assert std.isNumber(db.port) : "Port must be a number";
    assert std.isString(db.username) : "Username must be a string";
    assert std.isString(db.password) : "Password must be a string";
  },
  
  validateCacheConfig: function(cache) {
    assert std.isBoolean(cache.enabled) : "Enabled must be a boolean";
    assert std.isNumber(cache.ttl) : "TTL must be a number";
    assert std.isString(cache.type) : "Type must be a string";
  },
  
  validateConfig: function(config) {
    validateDatabaseConfig(config.database);
    validateCacheConfig(config.cache);
  },
  
  newDatabaseConfig: function(host, port, username, password) {
    {
      host: host,
      port: port,
      username: username,
      password: password,
    }
  },
  
  newCacheConfig: function(enabled, ttl, type) {
    {
      enabled: enabled,
      ttl: ttl,
      type: type,
    }
  },
  
  newConfig: function(database, cache) {
    {
      database: database,
      cache: cache,
    }
  }
}
```

## Tag Processing

The generator processes various Go struct tags:

### JSON Tags
```go
type User struct {
    Name     string `json:"name"`
    Age      int    `json:"age,omitempty"`
    Email    string `json:"email,omitempty"`
    Password string `json:"-"`  // Ignored
}
```

### Validation Tags
```go
type User struct {
    Name     string `json:"name" validate:"required"`
    Age      int    `json:"age" validate:"min=0,max=150"`
    Email    string `json:"email" validate:"email"`
    Username string `json:"username" validate:"min=3,max=20"`
}
```

### Custom Tags
```go
type User struct {
    Name     string `json:"name" description:"User's full name"`
    Age      int    `json:"age" example:"25"`
    Email    string `json:"email" format:"email"`
}
```

## Package Management

The generator handles Go package structures:

### Package Filtering
```yaml
sources:
  - type: go_ast
    name: "api-types"
    include_patterns:
      - "**/*.go"
    package_filters:
      - "api"
      - "models"
      - "types"
```

### Import Processing
```go
package api

import (
    "time"
    "github.com/example/types"
)

type User struct {
    ID        string    `json:"id"`
    CreatedAt time.Time `json:"created_at"`
    Profile   types.Profile `json:"profile"`
}
```

## Configuration Examples

### Basic Configuration
```yaml
version: "1.0"

sources:
  - type: go_ast
    name: "user-types"
    git:
      url: "https://github.com/example/user-service.git"
      ref: "main"
    include_patterns:
      - "types/*.go"
      - "models/*.go"
    exclude_patterns:
      - "**/*_test.go"
      - "vendor/**"
    output_path: "./generated/user-types"

output:
  base_path: "./generated"
  organization: "flat"
```

### Advanced Configuration
```yaml
version: "1.0"

sources:
  - type: go_ast
    name: "complex-api"
    git:
      url: "https://github.com/example/complex-api.git"
      ref: "v2.1.0"
    include_patterns:
      - "pkg/**/*.go"
      - "internal/**/*.go"
    exclude_patterns:
      - "**/*_test.go"
      - "**/*_mock.go"
      - "vendor/**"
      - "cmd/**"
    output_path: "./generated/complex-api"
    package_filters:
      - "api"
      - "models"
      - "types"
    generation:
      include_constructors: true
      include_validators: true
      include_helpers: true
      preserve_comments: true
      include_examples: true

output:
  base_path: "./generated"
  organization: "package"
  validation:
    enabled: true
    strict: false
```

## Generated Code Structure

The Go AST generator creates well-organized Jsonnet libraries:

```jsonnet
// Generated from Go AST
// Source: https://github.com/example/my-go-repo
// Package: api

{
  // Type definitions
  User: {
    id: null,    // string
    name: null,  // string
    email: null, // string
  },
  
  // Validation functions
  validateUser: function(user) {
    // Validation logic based on Go tags
  },
  
  // Constructor functions
  newUser: function(id, name, email) {
    // Constructor logic
  },
  
  // Helper functions
  isUser: function(obj) {
    // Type checking
  },
  
  // Constants (from const declarations)
  USER_STATUS_ACTIVE: "active",
  USER_STATUS_INACTIVE: "inactive",
  
  // Metadata
  _metadata: {
    source: "https://github.com/example/my-go-repo",
    package: "api",
    generated_at: "2024-01-01T12:00:00Z",
  }
}
```

## Best Practices

### Go Code Organization

1. **Use clear type names**: Choose descriptive struct and interface names
2. **Add documentation**: Include comments for complex types
3. **Use tags consistently**: Apply JSON and validation tags uniformly
4. **Group related types**: Organize types in logical packages

### Tag Usage

1. **JSON tags**: Always include for serialization
2. **Validation tags**: Use for runtime validation
3. **Custom tags**: Add for additional metadata
4. **Omitempty**: Use for optional fields

### Type Design

1. **Keep it simple**: Avoid overly complex nested structures
2. **Use composition**: Prefer embedding over inheritance
3. **Document constraints**: Add validation tags for business rules
4. **Consider defaults**: Provide sensible default values

## Troubleshooting

### Common Issues

1. **Type not found**: Check include patterns and package filters
2. **Parse errors**: Verify Go syntax is valid
3. **Tag issues**: Review struct tag syntax
4. **Import problems**: Check package dependencies

### Debug Commands

```bash
# Validate Go source
gensonnet-rs validate --source go_ast

# Generate with verbose output
RUST_LOG=debug gensonnet-rs generate

# Check generated code
gensonnet-rs check --output ./generated/go-ast
```

## Examples

### User Management System

```go
// user.go
package models

import "time"

type User struct {
    ID        string    `json:"id" validate:"required"`
    Username  string    `json:"username" validate:"required,min=3,max=20"`
    Email     string    `json:"email" validate:"required,email"`
    Age       int       `json:"age" validate:"min=0,max=150"`
    CreatedAt time.Time `json:"created_at"`
    UpdatedAt time.Time `json:"updated_at"`
    Profile   Profile   `json:"profile"`
}

type Profile struct {
    FirstName string `json:"first_name"`
    LastName  string `json:"last_name"`
    Bio       string `json:"bio"`
    Avatar    string `json:"avatar"`
}
```

### Generated User Management Code

```jsonnet
// Generated from Go AST: User Management System
{
  Profile: {
    first_name: null, // string
    last_name: null,  // string
    bio: null,        // string
    avatar: null,     // string
  },
  
  User: {
    id: null,         // string (required)
    username: null,   // string (required, min=3, max=20)
    email: null,      // string (required, email)
    age: null,        // int (min=0, max=150)
    created_at: null, // string (time.Time)
    updated_at: null, // string (time.Time)
    profile: null,    // Profile
  },
  
  validateProfile: function(profile) {
    // Profile validation logic
  },
  
  validateUser: function(user) {
    // User validation logic with all constraints
    assert user.id != null : "ID is required";
    assert std.isString(user.username) : "Username must be a string";
    assert std.length(user.username) >= 3 : "Username must be at least 3 characters";
    assert std.length(user.username) <= 20 : "Username must be at most 20 characters";
    assert std.isString(user.email) : "Email must be a string";
    // Email validation logic
    assert std.isNumber(user.age) : "Age must be a number";
    assert user.age >= 0 : "Age must be at least 0";
    assert user.age <= 150 : "Age must be at most 150";
    if user.profile != null {
      validateProfile(user.profile);
    }
  },
  
  newProfile: function(firstName, lastName, bio, avatar) {
    {
      first_name: firstName,
      last_name: lastName,
      bio: bio,
      avatar: avatar,
    }
  },
  
  newUser: function(id, username, email, age, profile) {
    {
      id: id,
      username: username,
      email: email,
      age: age,
      profile: profile,
    }
  }
}
```

## Next Steps

- Explore the [Plugin API Reference](/api/plugins/) for advanced customization
- Check out [Examples](/examples/) for more Go AST integration patterns
- Learn about [External Plugins](/plugins/external-plugins/) for custom Go processing
