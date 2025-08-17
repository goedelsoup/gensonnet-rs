//! OpenAPI type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenAPI specification version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenApiVersion {
    /// OpenAPI 2.0 (Swagger)
    V2,
    /// OpenAPI 3.0
    V3,
    /// OpenAPI 3.1
    V31,
}

/// OpenAPI specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// OpenAPI version
    #[serde(rename = "openapi")]
    pub version: Option<String>,

    /// Swagger version (v2)
    #[serde(rename = "swagger")]
    pub swagger_version: Option<String>,

    /// API information
    pub info: ApiInfo,

    /// Base path (v2)
    #[serde(rename = "basePath")]
    pub base_path: Option<String>,

    /// Servers (v3)
    pub servers: Option<Vec<Server>>,

    /// Paths/endpoints
    pub paths: HashMap<String, PathItem>,

    /// Definitions/schemas (v2)
    pub definitions: Option<HashMap<String, Schema>>,

    /// Components (v3)
    pub components: Option<Components>,
}

/// API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// API title
    pub title: String,

    /// API version
    pub version: String,

    /// API description
    pub description: Option<String>,

    /// Contact information
    pub contact: Option<Contact>,

    /// License information
    pub license: Option<License>,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// Contact name
    pub name: Option<String>,

    /// Contact email
    pub email: Option<String>,

    /// Contact URL
    pub url: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// License name
    pub name: String,

    /// License URL
    pub url: Option<String>,
}

/// Server information (v3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server URL
    pub url: String,

    /// Server description
    pub description: Option<String>,
}

/// Path item (endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    /// HTTP methods
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,

    /// Parameters
    pub parameters: Option<Vec<Parameter>>,
}

/// API operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation summary
    pub summary: Option<String>,

    /// Operation description
    pub description: Option<String>,

    /// Operation tags
    pub tags: Option<Vec<String>>,

    /// Operation parameters
    pub parameters: Option<Vec<Parameter>>,

    /// Request body
    pub request_body: Option<RequestBody>,

    /// Responses
    pub responses: HashMap<String, Response>,

    /// Operation ID
    pub operation_id: Option<String>,
}

/// Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,

    /// Parameter location
    pub r#in: String,

    /// Parameter description
    pub description: Option<String>,

    /// Whether parameter is required
    pub required: Option<bool>,

    /// Parameter schema
    pub schema: Option<Schema>,

    /// Parameter type (v2)
    pub r#type: Option<String>,

    /// Parameter format (v2)
    pub format: Option<String>,
}

/// Request body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Request body description
    pub description: Option<String>,

    /// Request body content
    pub content: HashMap<String, MediaType>,

    /// Whether request body is required
    pub required: Option<bool>,
}

/// Media type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    /// Media type schema
    pub schema: Option<Schema>,

    /// Media type examples
    pub examples: Option<HashMap<String, Example>>,
}

/// Example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example summary
    pub summary: Option<String>,

    /// Example description
    pub description: Option<String>,

    /// Example value
    pub value: Option<serde_json::Value>,
}

/// Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Response description
    pub description: String,

    /// Response content
    pub content: Option<HashMap<String, MediaType>>,

    /// Response headers
    pub headers: Option<HashMap<String, Header>>,
}

/// Header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// Header description
    pub description: Option<String>,

    /// Header schema
    pub schema: Option<Schema>,
}

/// Components (v3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    /// Component schemas
    pub schemas: Option<HashMap<String, Schema>>,

    /// Component responses
    pub responses: Option<HashMap<String, Response>>,

    /// Component parameters
    pub parameters: Option<HashMap<String, Parameter>>,

    /// Component examples
    pub examples: Option<HashMap<String, Example>>,
}

/// Schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema type
    pub r#type: Option<String>,

    /// Schema format
    pub format: Option<String>,

    /// Schema description
    pub description: Option<String>,

    /// Schema properties
    pub properties: Option<HashMap<String, Schema>>,

    /// Required properties
    pub required: Option<Vec<String>>,

    /// Schema items (for arrays)
    pub items: Option<Box<Schema>>,

    /// Schema reference
    pub r#ref: Option<String>,

    /// Schema allOf
    pub all_of: Option<Vec<Schema>>,

    /// Schema anyOf
    pub any_of: Option<Vec<Schema>>,

    /// Schema oneOf
    pub one_of: Option<Vec<Schema>>,

    /// Schema not
    pub not: Option<Box<Schema>>,

    /// Schema additional properties
    pub additional_properties: Option<Box<Schema>>,

    /// Schema enum values
    pub r#enum: Option<Vec<serde_json::Value>>,

    /// Schema example
    pub example: Option<serde_json::Value>,

    /// Schema examples
    pub examples: Option<HashMap<String, Example>>,

    /// Schema default value
    pub default: Option<serde_json::Value>,

    /// Schema minimum value
    pub minimum: Option<f64>,

    /// Schema maximum value
    pub maximum: Option<f64>,

    /// Schema min length
    pub min_length: Option<u64>,

    /// Schema max length
    pub max_length: Option<u64>,

    /// Schema pattern
    pub pattern: Option<String>,
}
