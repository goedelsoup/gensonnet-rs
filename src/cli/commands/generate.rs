//! Generate command implementation

use crate::cli::utils;
use anyhow::Result;
use clap::{ArgMatches, Command};
use std::path::PathBuf;
use tracing::info;

pub fn command() -> Command {
    Command::new("generate")
        .about("Generate Jsonnet libraries from configured sources")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file path")
                .value_name("FILE"),
        )
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .help("Output directory")
                .value_name("DIR"),
        )
        .arg(
            clap::Arg::new("force")
                .short('f')
                .long("force")
                .help("Force regeneration even if sources haven't changed")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("dry-run")
                .long("dry-run")
                .help("Don't write files")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("fail-fast")
                .long("fail-fast")
                .help("Stop on first error")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    info!("Starting Jsonnet library generation");

    let mut config = utils::load_config(matches)?;

    // Override output path if specified
    if let Some(output_path) = matches.get_one::<String>("output") {
        config.output.base_path = PathBuf::from(output_path);
    }

    // Override fail_fast setting if specified
    if matches.get_flag("fail-fast") {
        config.generation.fail_fast = true;
    }

    let app = utils::create_app(config)?;
    app.initialize().await?;

    if matches.get_flag("dry-run") {
        info!("Dry run mode - no files will be written");
        println!("Dry run mode - no files will be written");
        
        let result = app.dry_run().await?;
        
        println!("Dry run completed successfully!");
        println!(
            "Sources that would be processed: {}/{}",
            result.sources_processed, result.total_sources
        );
        println!("Files that would be generated: {}", result.statistics.files_would_generate);
        println!(
            "Estimated processing time: {}ms",
            result.statistics.total_processing_time_ms
        );
        
        if result.statistics.incremental_mode {
            println!("Would use incremental generation:");
            println!("  Changed sources: {}", result.statistics.changed_sources_count);
            println!("  Dependent sources: {}", result.statistics.dependent_sources_count);
        } else {
            println!("Would perform full generation for all sources");
        }
        
        println!(
            "Estimated cache hit rate: {:.1}%",
            result.statistics.cache_hit_rate * 100.0
        );

        if result.statistics.error_count > 0 {
            println!("Errors that would occur: {}", result.statistics.error_count);
        }

        if result.statistics.warning_count > 0 {
            println!("Warnings that would occur: {}", result.statistics.warning_count);
        }

        for source_result in result.results {
            println!(
                "  {} ({}): {} files would be generated",
                source_result.source_name, source_result.source_type, source_result.files_would_generate
            );
            if !source_result.errors.is_empty() {
                for error in source_result.errors {
                    eprintln!("    Error: {error}");
                }
            }
            if !source_result.warnings.is_empty() {
                for warning in source_result.warnings {
                    println!("    Warning: {warning}");
                }
            }
        }
        
        return Ok(());
    }

    let result = app.generate().await?;

    println!("Generation completed successfully!");
    println!(
        "Sources processed: {}/{}",
        result.sources_processed, result.total_sources
    );
    println!("Files generated: {}", result.statistics.files_generated);
    println!(
        "Processing time: {}ms",
        result.statistics.total_processing_time_ms
    );

    for source_result in result.results {
        println!(
            "  {}: {} files generated",
            source_result.source_type, source_result.files_generated
        );
        if !source_result.errors.is_empty() {
            for error in source_result.errors {
                eprintln!("    Error: {error}");
            }
        }
    }

    Ok(())
}
