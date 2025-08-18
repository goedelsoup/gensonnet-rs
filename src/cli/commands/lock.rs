//! Lock command implementation

use anyhow::Result;
use clap::{ArgMatches, Command};
use jsonnet_lockfile::LockfileManager;
use std::path::PathBuf;
use tracing::{info, warn};

pub fn command() -> Command {
    Command::new("lock")
        .about("Manage lockfile for reproducible builds")
        .arg(
            clap::Arg::new("status")
                .long("status")
                .help("Show lockfile status")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("update")
                .long("update")
                .help("Update lockfile")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    let lockfile_manager = LockfileManager::new(LockfileManager::default_path());

    if matches.get_flag("status") {
        show_lock_status(&lockfile_manager).await?;
    } else if matches.get_flag("update") {
        update_lockfile(&lockfile_manager).await?;
    } else {
        println!("Use --status to show lockfile status or --update to update the lockfile");
    }

    Ok(())
}

async fn show_lock_status(lockfile_manager: &LockfileManager) -> Result<()> {
    let lockfile_path = LockfileManager::default_path();

    if !lockfile_path.exists() {
        println!("No lockfile found");
        return Ok(());
    }

    let lockfile = lockfile_manager.load_or_create()?;

    println!("Lockfile Status:");
    println!(
        "  Generated: {}",
        lockfile.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Tool version: {}", lockfile.tool_version);
    println!("  Sources: {}", lockfile.sources.len());
    println!("  Files: {}", lockfile.files.len());

    for (source_id, entry) in &lockfile.sources {
        println!(
            "  Source {}: {}@{} ({})",
            source_id, entry.url, entry.ref_name, entry.commit_sha
        );
    }

    Ok(())
}

/// Load configuration from file or create default
fn load_config() -> Result<crate::Config> {
    // Look for default config files
    let default_paths = [
        PathBuf::from("test-config.yaml"),
        PathBuf::from(".gensonnet.yaml"),
        PathBuf::from(".gensonnet.yml"),
        PathBuf::from("gensonnet.yaml"),
        PathBuf::from("gensonnet.yml"),
    ];

    for path in &default_paths {
        if path.exists() {
            println!("Loading configuration from: {path:?}");
            return crate::Config::from_file(path);
        }
    }

    println!("No configuration file found, using default configuration");
    // Return default config if no file found
    Ok(crate::Config::default())
}

async fn update_lockfile(lockfile_manager: &LockfileManager) -> Result<()> {
    info!("Updating lockfile");

    // Load configuration to get current sources
    let config = load_config()?;

    // Create GitManager for getting commit SHAs
    let git_manager = crate::GitManager::new()?;

    // Get current commit SHAs for all sources
    let mut current_sources = std::collections::HashMap::new();
    let mut source_entries = std::collections::HashMap::new();

    if config.sources.is_empty() {
        println!("No sources configured. Please create a configuration file with sources.");
        println!("Example configuration:");
        println!("  version: \"1.0\"");
        println!("  sources:");
        println!("    - type: crd");
        println!("      name: my-crds");
        println!("      git:");
        println!("        url: \"https://github.com/example/repo.git\"");
        println!("        ref: \"main\"");
        println!("      filters:");
        println!("        - \"example.com/v1\"");
        println!("      output_path: \"./generated/my-crds\"");
        return Ok(());
    }

    for source in &config.sources {
        let source_name = source.name().to_string();

        match source {
            crate::config::Source::Crd(crd_source) => {
                // Get repository path and current commit
                let repo_path = match git_manager.ensure_repository(&crd_source.git).await {
                    Ok(path) => path,
                    Err(e) => {
                        warn!("Failed to access repository {}: {}", crd_source.git.url, e);
                        println!("Skipping source '{source_name}' due to repository access error");
                        continue;
                    }
                };
                let commit_sha = match git_manager.get_current_commit(&repo_path) {
                    Ok(sha) => sha,
                    Err(e) => {
                        warn!("Failed to get commit SHA for {}: {}", crd_source.git.url, e);
                        println!("Skipping source '{source_name}' due to commit access error");
                        continue;
                    }
                };

                // Create lockfile entry
                let entry = jsonnet_lockfile::LockfileEntry::new(
                    crd_source.git.url.clone(),
                    crd_source
                        .git
                        .ref_name
                        .clone()
                        .unwrap_or_else(|| "main".to_string()),
                    commit_sha.clone(),
                    crd_source.filters.clone(),
                );

                current_sources.insert(source_name.clone(), commit_sha);
                source_entries.insert(source_name, entry);
            }
            crate::config::Source::GoAst(go_ast_source) => {
                // Get repository path and current commit
                let repo_path = match git_manager.ensure_repository(&go_ast_source.git).await {
                    Ok(path) => path,
                    Err(e) => {
                        warn!(
                            "Failed to access repository {}: {}",
                            go_ast_source.git.url, e
                        );
                        println!("Skipping source '{source_name}' due to repository access error");
                        continue;
                    }
                };
                let commit_sha = match git_manager.get_current_commit(&repo_path) {
                    Ok(sha) => sha,
                    Err(e) => {
                        warn!(
                            "Failed to get commit SHA for {}: {}",
                            go_ast_source.git.url, e
                        );
                        println!("Skipping source '{source_name}' due to commit access error");
                        continue;
                    }
                };

                // Create lockfile entry
                let entry = jsonnet_lockfile::LockfileEntry::new(
                    go_ast_source.git.url.clone(),
                    go_ast_source
                        .git
                        .ref_name
                        .clone()
                        .unwrap_or_else(|| "main".to_string()),
                    commit_sha.clone(),
                    go_ast_source.include_patterns.clone(),
                );

                current_sources.insert(source_name.clone(), commit_sha);
                source_entries.insert(source_name, entry);
            }
            crate::config::Source::OpenApi(openapi_source) => {
                // Get repository path and current commit
                let repo_path = match git_manager.ensure_repository(&openapi_source.git).await {
                    Ok(path) => path,
                    Err(e) => {
                        warn!(
                            "Failed to access repository {}: {}",
                            openapi_source.git.url, e
                        );
                        println!("Skipping source '{source_name}' due to repository access error");
                        continue;
                    }
                };
                let commit_sha = match git_manager.get_current_commit(&repo_path) {
                    Ok(sha) => sha,
                    Err(e) => {
                        warn!(
                            "Failed to get commit SHA for {}: {}",
                            openapi_source.git.url, e
                        );
                        println!("Skipping source '{source_name}' due to commit access error");
                        continue;
                    }
                };

                // Create lockfile entry
                let entry = jsonnet_lockfile::LockfileEntry::new(
                    openapi_source.git.url.clone(),
                    openapi_source
                        .git
                        .ref_name
                        .clone()
                        .unwrap_or_else(|| "main".to_string()),
                    commit_sha.clone(),
                    openapi_source.include_patterns.clone(),
                );

                current_sources.insert(source_name.clone(), commit_sha);
                source_entries.insert(source_name, entry);
            }
        }
    }

    // Calculate checksums for generated files
    let mut file_checksums = std::collections::HashMap::new();

    if config.output.base_path.exists() {
        for entry in walkdir::WalkDir::new(&config.output.base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative_path = file_path
                .strip_prefix(&config.output.base_path)
                .unwrap_or(file_path)
                .to_path_buf();

            match jsonnet_lockfile::FileChecksum::from_file(file_path) {
                Ok(checksum) => {
                    file_checksums.insert(relative_path, checksum);
                }
                Err(e) => {
                    warn!("Failed to calculate checksum for {:?}: {}", file_path, e);
                }
            }
        }
    }

    // Update the lockfile
    lockfile_manager.update(source_entries, file_checksums.clone())?;

    println!("Lockfile updated successfully");
    println!("  Sources: {}", current_sources.len());
    println!("  Files: {}", file_checksums.len());

    // Show what changed
    if let Ok(existing_lockfile) = lockfile_manager.load_or_create() {
        let changed_sources: Vec<_> = current_sources
            .iter()
            .filter(|(source_id, current_commit)| {
                existing_lockfile.source_changed(source_id, current_commit)
            })
            .collect();

        if !changed_sources.is_empty() {
            println!("  Changed sources:");
            for (source_id, commit_sha) in changed_sources {
                println!("    {source_id}: {commit_sha}");
            }
        }
    }

    Ok(())
}
