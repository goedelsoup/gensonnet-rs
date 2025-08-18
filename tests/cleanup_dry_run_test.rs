use gensonnet::{Config, JsonnetGen};
use std::path::PathBuf;

#[tokio::test]
async fn test_cleanup_dry_run_functionality() {
    // Create a configuration
    let mut config = Config::default();
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test-crd".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./test-output"),
        },
    ));

    // Create JsonnetGen instance
    let app = JsonnetGen::new(config).unwrap();

    // Perform cleanup dry run
    let result = app.cleanup_dry_run(168).unwrap(); // 1 week

    // Verify the dry run result structure
    assert_eq!(result.max_age_hours, 168);
    assert_eq!(result.total_sources_removed, 0); // Should be 0 for a new lockfile
    assert_eq!(result.total_files_removed, 0); // Should be 0 for a new lockfile
    assert_eq!(result.total_size_freed, 0); // Should be 0 for a new lockfile
    assert!(result.stale_sources.is_empty());
    assert!(result.stale_files.is_empty());
    assert_eq!(result.lockfile_path, PathBuf::from("gensonnet.lock"));
}

#[tokio::test]
async fn test_cleanup_dry_run_with_different_ages() {
    // Create a configuration
    let mut config = Config::default();
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test-crd".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./test-output"),
        },
    ));

    // Create JsonnetGen instance
    let app = JsonnetGen::new(config).unwrap();

    // Test with different max ages
    let result_1_hour = app.cleanup_dry_run(1).unwrap();
    let result_24_hours = app.cleanup_dry_run(24).unwrap();
    let result_168_hours = app.cleanup_dry_run(168).unwrap();

    // All should return 0 for a new lockfile
    assert_eq!(result_1_hour.max_age_hours, 1);
    assert_eq!(result_24_hours.max_age_hours, 24);
    assert_eq!(result_168_hours.max_age_hours, 168);

    assert_eq!(result_1_hour.total_sources_removed, 0);
    assert_eq!(result_24_hours.total_sources_removed, 0);
    assert_eq!(result_168_hours.total_sources_removed, 0);
}

#[tokio::test]
async fn test_cleanup_dry_run_result_structure() {
    // Create a configuration
    let mut config = Config::default();
    config.sources.push(gensonnet::config::Source::Crd(
        gensonnet::config::CrdSource {
            name: "test-crd".to_string(),
            git: gensonnet::config::GitSource {
                url: "https://github.com/test/repo.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["test.com/v1".to_string()],
            output_path: PathBuf::from("./test-output"),
        },
    ));

    // Create JsonnetGen instance
    let app = JsonnetGen::new(config).unwrap();

    // Perform cleanup dry run
    let result = app.cleanup_dry_run(168).unwrap();

    // Verify the result structure
    assert!(result.max_age_hours > 0);
    assert_eq!(result.stale_sources.len(), result.total_sources_removed);
    assert_eq!(result.stale_files.len(), result.total_files_removed);

    // Verify lockfile path
    assert_eq!(result.lockfile_path, PathBuf::from("gensonnet.lock"));
}
