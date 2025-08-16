//! Source configuration definitions

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Source types that can be processed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    /// Kubernetes CustomResourceDefinition source
    Crd(CrdSource),

    /// Go AST source for processing Go source files
    GoAst(GoAstSource),

    /// OpenAPI specification source for processing OpenAPI/Swagger files
    OpenApi(OpenApiSource),
}

impl Source {
    /// Get the name of the source
    pub fn name(&self) -> &str {
        match self {
            Source::Crd(crd) => &crd.name,
            Source::GoAst(go_ast) => &go_ast.name,
            Source::OpenApi(openapi) => &openapi.name,
        }
    }

    /// Validate the source configuration
    pub fn validate(&self) -> Result<()> {
        match self {
            Source::Crd(crd) => crd.validate(),
            Source::GoAst(go_ast) => go_ast.validate(),
            Source::OpenApi(openapi) => openapi.validate(),
        }
    }
}

/// CRD source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdSource {
    /// Name of the source
    pub name: String,

    /// Git repository configuration
    pub git: GitSource,

    /// Filters for CRDs (API group patterns)
    pub filters: Vec<String>,

    /// Output path for generated files
    pub output_path: PathBuf,
}

impl CrdSource {
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("CRD source name cannot be empty"));
        }

        self.git.validate()?;

        if self.output_path.to_string_lossy().is_empty() {
            return Err(anyhow!("CRD output path cannot be empty"));
        }

        Ok(())
    }
}

/// Git source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    /// Git repository URL
    pub url: String,

    /// Git reference (branch, tag, or commit SHA)
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,

    /// Authentication configuration (future)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<GitAuth>,
}

impl GitSource {
    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(anyhow!("Git URL cannot be empty"));
        }

        // Basic URL validation
        if !self.url.starts_with("http") && !self.url.starts_with("git@") {
            return Err(anyhow!("Invalid Git URL format: {}", self.url));
        }

        Ok(())
    }

    /// Get the reference name, defaulting to "main" if not specified
    pub fn ref_name(&self) -> &str {
        self.ref_name.as_deref().unwrap_or("main")
    }

    /// Get a unique identifier for this source
    pub fn identifier(&self) -> String {
        let ref_name = self.ref_name();
        format!("{}@{}", self.url, ref_name)
    }
}

/// Git authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GitAuth {
    /// SSH key authentication
    Ssh {
        /// Path to SSH private key
        key_path: PathBuf,
        /// SSH key passphrase (optional)
        passphrase: Option<String>,
    },

    /// Personal access token authentication
    Token {
        /// The access token
        token: String,
    },

    /// Username/password authentication
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
}

/// Go AST source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoAstSource {
    /// Name of the source
    pub name: String,

    /// Git repository configuration
    pub git: GitSource,

    /// File patterns to include (e.g., ["**/*.go"])
    pub include_patterns: Vec<String>,

    /// File patterns to exclude (e.g., ["**/*_test.go", "vendor/**"])
    pub exclude_patterns: Vec<String>,

    /// Output path for generated files
    pub output_path: PathBuf,

    /// Package filters (optional, for specific packages)
    pub package_filters: Option<Vec<String>>,
}

impl GoAstSource {
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Go AST source name cannot be empty"));
        }

        self.git.validate()?;

        if self.output_path.to_string_lossy().is_empty() {
            return Err(anyhow!("Go AST output path cannot be empty"));
        }

        if self.include_patterns.is_empty() {
            return Err(anyhow!(
                "Go AST source must have at least one include pattern"
            ));
        }

        Ok(())
    }
}

/// OpenAPI source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSource {
    /// Name of the source
    pub name: String,

    /// Git repository configuration
    pub git: GitSource,

    /// File patterns to include (e.g., ["**/*.yaml", "**/*.json"])
    pub include_patterns: Vec<String>,

    /// File patterns to exclude (e.g., ["**/*_test.yaml", "vendor/**"])
    pub exclude_patterns: Vec<String>,

    /// Output path for generated files
    pub output_path: PathBuf,

    /// OpenAPI version to target (2.0, 3.0, etc.)
    pub openapi_version: Option<String>,

    /// Whether to include examples in generated code
    pub include_examples: Option<bool>,

    /// Whether to include descriptions in generated code
    pub include_descriptions: Option<bool>,

    /// Custom base URL for the API
    pub base_url: Option<String>,
}

impl OpenApiSource {
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("OpenAPI source name cannot be empty"));
        }

        self.git.validate()?;

        if self.output_path.to_string_lossy().is_empty() {
            return Err(anyhow!("OpenAPI output path cannot be empty"));
        }

        if self.include_patterns.is_empty() {
            return Err(anyhow!(
                "OpenAPI source must have at least one include pattern"
            ));
        }

        // Validate OpenAPI version if specified
        if let Some(version) = &self.openapi_version {
            if !matches!(version.as_str(), "2.0" | "3.0" | "3.1") {
                return Err(anyhow!("Unsupported OpenAPI version: {}", version));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crd_source_validation() {
        let valid_source = CrdSource {
            name: "test".to_string(),
            git: GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./output"),
        };

        assert!(valid_source.validate().is_ok());
    }

    #[test]
    fn test_git_source_validation() {
        let valid_git = GitSource {
            url: "https://github.com/test/repo.git".to_string(),
            ref_name: Some("main".to_string()),
            auth: None,
        };

        assert!(valid_git.validate().is_ok());
        assert_eq!(valid_git.ref_name(), "main");
    }

    #[test]
    fn test_git_source_default_ref() {
        let git = GitSource {
            url: "https://github.com/test/repo.git".to_string(),
            ref_name: None,
            auth: None,
        };

        assert_eq!(git.ref_name(), "main");
    }

    #[test]
    fn test_invalid_git_url() {
        let invalid_git = GitSource {
            url: "invalid-url".to_string(),
            ref_name: None,
            auth: None,
        };

        assert!(invalid_git.validate().is_err());
    }
}
