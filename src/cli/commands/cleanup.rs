//! Cleanup command implementation

use crate::cli::utils;
use anyhow::Result;
use clap::{ArgMatches, Command};
use tracing::info;

pub fn command() -> Command {
    Command::new("cleanup")
        .about("Clean up stale entries from lockfile and cache")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file path")
                .value_name("FILE"),
        )
        .arg(
            clap::Arg::new("max-age")
                .short('a')
                .long("max-age")
                .help("Maximum age in hours for entries to keep")
                .value_name("HOURS")
                .default_value("168"), // 1 week
        )
        .arg(
            clap::Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be cleaned up without actually doing it")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    let max_age: u64 = matches
        .get_one::<String>("max-age")
        .unwrap()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid max-age value"))?;

    let dry_run = matches.get_flag("dry-run");

    if dry_run {
        info!(
            "Dry run: Would clean up entries older than {} hours",
            max_age
        );
        println!("Dry run mode - no changes will be made");
        println!("Would clean up entries older than {max_age} hours");
        
        let config = utils::load_config(matches)?;
        let app = utils::create_app(config)?;
        
        let result = app.cleanup_dry_run(max_age)?;
        
        println!("Cleanup dry run completed!");
        println!("Lockfile: {:?}", result.lockfile_path);
        println!("Max age: {} hours", result.max_age_hours);
        println!();
        
        if result.total_sources_removed == 0 && result.total_files_removed == 0 {
            println!("No stale entries found - nothing would be cleaned up");
        } else {
            println!("Would remove {} source entries and {} file entries", 
                result.total_sources_removed, result.total_files_removed);
            println!("Total space that would be freed: {} bytes ({:.2} MB)", 
                result.total_size_freed, result.total_size_freed as f64 / 1024.0 / 1024.0);
            println!();
            
            if !result.stale_sources.is_empty() {
                println!("Stale source entries that would be removed:");
                for source in &result.stale_sources {
                    println!("  - {} ({}@{}) - {} hours old", 
                        source.source_id, source.git_url, source.git_ref, source.age_hours);
                }
                println!();
            }
            
            if !result.stale_files.is_empty() {
                println!("Stale file entries that would be removed:");
                for file in &result.stale_files {
                    println!("  - {:?} ({} bytes, {} hours old)", 
                        file.file_path, file.size, file.age_hours);
                }
            }
        }
        
        return Ok(());
    }

    info!("Cleaning up entries older than {} hours", max_age);

    let config = utils::load_config(matches)?;
    let app = utils::create_app(config)?;

    app.cleanup(max_age)?;

    println!("Cleanup completed successfully");
    println!("Removed entries older than {max_age} hours");

    Ok(())
}
