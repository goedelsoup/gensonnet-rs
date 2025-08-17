//! Configuration tests

use super::*;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(config.version, parsed.version);
}

#[test]
fn test_config_from_file() {
    let mut config = Config::default();
    config
        .sources
        .push(crate::config::Source::Crd(crate::config::CrdSource {
            name: "test".to_string(),
            git: crate::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./output"),
        }));

    let temp_file = NamedTempFile::new().unwrap();
    config
        .save_to_file(&temp_file.path().to_path_buf())
        .unwrap();

    let loaded = Config::from_file(&temp_file.path().to_path_buf()).unwrap();
    assert_eq!(config.version, loaded.version);
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();
    config.sources.push(Source::Crd(CrdSource {
        name: "test".to_string(),
        git: GitSource {
            url: "https://github.com/test/repo.git".to_string(),
            ref_name: Some("main".to_string()),
            auth: None,
        },
        filters: vec!["test.com/v1".to_string()],
        output_path: PathBuf::from("./output"),
    }));

    assert!(config.validate().is_ok());
}
