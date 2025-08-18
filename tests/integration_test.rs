use gensonnet::LockfileManager;
use gensonnet::{Config, CrdParser};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_generation_workflow() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path();

    // Create a test CRD file
    let crd_content = r#"
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: tests.test.com
spec:
  group: test.com
  names:
    kind: Test
    plural: tests
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                name:
                  type: string
                count:
                  type: integer
"#;

    let crd_file = test_dir.join("test-crd.yaml");
    std::fs::write(&crd_file, crd_content).unwrap();

    // Create a minimal configuration
    let mut config = Config::default();
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test-crds".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: test_dir.join("generated"),
        },
    ));

    // Test configuration validation
    match config.validate() {
        Ok(_) => {}
        Err(e) => {
            println!("Configuration validation failed: {e}");
            panic!("Configuration validation failed");
        }
    }

    // Test CRD parsing
    let parser = CrdParser::new();
    let schemas = parser.parse_from_directory(test_dir, &[]).unwrap();

    // The parser should find the CRD we created
    assert_eq!(schemas.len(), 1);
    let schema = &schemas[0];
    assert_eq!(schema.name, "tests.test.com");
    assert_eq!(schema.group, "test.com");
    assert_eq!(schema.version, "v1");
    assert_eq!(schema.api_version, "test.com/v1");
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: Config = serde_yaml::from_str(&yaml).unwrap();

    assert_eq!(config.version, parsed.version);
    assert_eq!(config.sources.len(), parsed.sources.len());
}

#[test]
fn test_crd_schema_validation() {
    let schema = gensonnet::CrdSchema {
        name: "test".to_string(),
        group: "test.com".to_string(),
        version: "v1".to_string(),
        api_version: "test.com/v1".to_string(),
        kind: "test".to_string(),
        schema: serde_yaml::Value::Null,
        source_path: PathBuf::from("test.yaml"),
        validation_rules: gensonnet::ValidationRules::default(),
        schema_analysis: gensonnet::SchemaAnalysis::default(),
    };

    assert_eq!(schema.kind(), "test");
    assert_eq!(schema.resource_name(), "tests");
    assert_eq!(schema.api_version, "test.com/v1");
}

#[tokio::test]
async fn test_git_manager_creation() {
    let git_manager = gensonnet::GitManager::new();
    assert!(git_manager.is_ok());
}

#[test]
fn test_lockfile_operations() {
    let temp_dir = TempDir::new().unwrap();
    let lockfile_path = temp_dir.path().join("test.lock");

    let lockfile_manager = LockfileManager::new(lockfile_path.clone());
    let lockfile = lockfile_manager.load_or_create().unwrap();

    assert_eq!(lockfile.version, "1.0");
    assert_eq!(lockfile.tool_version, env!("CARGO_PKG_VERSION"));
}
