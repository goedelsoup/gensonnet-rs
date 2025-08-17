//! Incremental generation command implementation

use crate::cli::utils;
use anyhow::Result;
use clap::{ArgMatches, Command};
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info};

pub fn command() -> Command {
    Command::new("incremental")
        .about("Perform incremental generation with advanced features")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file path")
                .value_name("FILE"),
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
                .help("Show what would be generated without actually doing it")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("parallel")
                .short('p')
                .long("parallel")
                .help("Process sources in parallel")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("max-workers")
                .long("max-workers")
                .help("Maximum number of parallel workers")
                .value_name("NUM")
                .default_value("4"),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    info!("Starting incremental generation");

    let force = matches.get_flag("force");
    let dry_run = matches.get_flag("dry-run");
    let parallel = matches.get_flag("parallel");
    let max_workers: usize = matches
        .get_one::<String>("max-workers")
        .unwrap()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid max-workers value"))?;

    let config = utils::load_config(matches)?;
    let app = utils::create_app(config)?;

    if dry_run {
        info!("Dry run mode - no files will be generated");
        println!("Dry run mode - no files will be generated");

        match app.get_status().await {
            Ok(status) => {
                println!(
                    "Would process {} changed sources",
                    status.changed_sources.len()
                );
                println!(
                    "Would process {} dependent sources",
                    status.dependent_sources.len()
                );
                println!("Estimated time: {}ms", status.estimated_time_ms);

                if parallel {
                    println!("Parallel processing would be enabled with {max_workers} workers");
                }
            }
            Err(e) => {
                println!("Could not determine status due to error: {e}");
                println!("This might be due to repository access issues.");
                println!(
                    "Total sources in configuration: {}",
                    app.config.sources.len()
                );
            }
        }

        return Ok(());
    }

    app.initialize().await?;

    if force {
        info!("Force flag set - performing full generation");
        println!("Force flag set - performing full generation");
    }

    if parallel {
        info!("Parallel processing enabled with {} workers", max_workers);
        println!("Parallel processing enabled with {max_workers} workers");

        // Get sources that need to be processed
        let sources_to_process = match app.get_status().await {
            Ok(status) => {
                if force {
                    // Force mode: process all sources
                    app.config.sources.iter().collect::<Vec<_>>()
                } else {
                    // Incremental mode: process only changed and dependent sources
                    let mut sources = Vec::new();
                    for source_id in &status.changed_sources {
                        if let Some(source) = app.find_source_by_id(source_id) {
                            sources.push(source);
                        }
                    }
                    for source_id in &status.dependent_sources {
                        if let Some(source) = app.find_source_by_id(source_id) {
                            sources.push(source);
                        }
                    }
                    sources
                }
            }
            Err(e) => {
                println!("Could not determine incremental status: {e}");
                println!("Falling back to processing all sources");
                app.config.sources.iter().collect::<Vec<_>>()
            }
        };

        if sources_to_process.is_empty() {
            println!("No sources to process");
            return Ok(());
        }

        println!(
            "Processing {} sources in parallel",
            sources_to_process.len()
        );

        // Process sources in parallel
        let results = process_sources_parallel(&app, &sources_to_process, max_workers).await?;

        // Calculate cache hit rate
        let cache_hit_rate = if force {
            // Force mode: no cache hits since all sources are processed
            0.0
        } else {
            // Try to get the incremental plan to calculate cache hit rate
            match app.lockfile_manager.get_incremental_plan(
                &sources_to_process
                    .iter()
                    .map(|s| s.name().to_string())
                    .collect::<Vec<_>>(),
            ) {
                Ok(plan) => app.calculate_cache_hit_rate(&plan),
                Err(_) => {
                    // Fallback calculation based on sources processed
                    let total_sources = app.config.sources.len();
                    let processed_sources = sources_to_process.len();
                    if total_sources > 0 {
                        (total_sources - processed_sources) as f64 / total_sources as f64
                    } else {
                        0.0
                    }
                }
            }
        };

        // Create generation result
        let total_errors: usize = results.iter().map(|r| r.errors.len()).sum();
        let total_warnings: usize = results.iter().map(|r| r.warnings.len()).sum();
        let files_generated: usize = results.iter().map(|r| r.files_generated).sum();

        let result = jsonnet_generator::GenerationResult {
            sources_processed: results.len(),
            total_sources: app.config.sources.len(),
            results: results.clone(),
            statistics: jsonnet_generator::result::GenerationStatistics {
                total_processing_time_ms: results.iter().map(|r| r.processing_time_ms).sum(),
                sources_processed: results.len(),
                files_generated,
                error_count: total_errors,
                warning_count: total_warnings,
                cache_hit_rate,
            },
        };

        // Display results
        display_generation_results(&result);

        return Ok(());
    }

    let result = app.generate().await?;

    println!("Incremental generation completed successfully!");
    println!(
        "Sources processed: {}/{}",
        result.sources_processed, result.total_sources
    );
    println!("Files generated: {}", result.statistics.files_generated);
    println!(
        "Processing time: {}ms",
        result.statistics.total_processing_time_ms
    );
    println!(
        "Cache hit rate: {:.1}%",
        result.statistics.cache_hit_rate * 100.0
    );

    if result.statistics.error_count > 0 {
        println!("Errors encountered: {}", result.statistics.error_count);
    }

    if result.statistics.warning_count > 0 {
        println!("Warnings encountered: {}", result.statistics.warning_count);
    }

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
        if !source_result.warnings.is_empty() {
            for warning in source_result.warnings {
                println!("    Warning: {warning}");
            }
        }
    }

    Ok(())
}

/// Process sources in parallel with a configurable number of workers
async fn process_sources_parallel(
    app: &crate::JsonnetGen,
    sources: &[&crate::config::Source],
    max_workers: usize,
) -> Result<Vec<jsonnet_generator::SourceResult>> {
    let semaphore = Arc::new(Semaphore::new(max_workers));
    let app = Arc::new(app);

    let futures = sources.iter().map(|source| {
        let semaphore = Arc::clone(&semaphore);
        let app = Arc::clone(&app);
        let source = *source;

        async move {
            let _permit = semaphore.acquire().await.unwrap();
            let source_name = source.name().to_string();

            info!("Processing source in parallel: {}", source_name);
            println!("Processing: {source_name}");

            match app.process_source_with_recovery(source).await {
                Ok(result) => {
                    info!("Successfully processed source in parallel: {}", source_name);
                    println!(
                        "Completed: {} ({} files generated)",
                        source_name, result.files_generated
                    );
                    Ok(result)
                }
                Err(e) => {
                    error!(
                        "Failed to process source in parallel {}: {}",
                        source_name, e
                    );
                    println!("Failed: {source_name} - {e}");
                    Err(e)
                }
            }
        }
    });

    // Collect results, allowing some failures
    let results = stream::iter(futures)
        .buffer_unordered(max_workers)
        .collect::<Vec<_>>()
        .await;

    // Separate successful and failed results
    let mut successful_results = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(source_result) => successful_results.push(source_result),
            Err(e) => errors.push(e),
        }
    }

    // Report any errors
    if !errors.is_empty() {
        println!("{} sources failed to process:", errors.len());
        for error in &errors {
            eprintln!("  Error: {error}");
        }
    }

    Ok(successful_results)
}

/// Display generation results in a formatted way
fn display_generation_results(result: &jsonnet_generator::GenerationResult) {
    println!("Incremental generation completed successfully!");
    println!(
        "Sources processed: {}/{}",
        result.sources_processed, result.total_sources
    );
    println!("Files generated: {}", result.statistics.files_generated);
    println!(
        "Processing time: {}ms",
        result.statistics.total_processing_time_ms
    );
    println!(
        "Cache hit rate: {:.1}%",
        result.statistics.cache_hit_rate * 100.0
    );

    if result.statistics.error_count > 0 {
        println!("Errors encountered: {}", result.statistics.error_count);
    }

    if result.statistics.warning_count > 0 {
        println!("Warnings encountered: {}", result.statistics.warning_count);
    }

    for source_result in &result.results {
        println!(
            "  {}: {} files generated",
            source_result.source_type, source_result.files_generated
        );
        if !source_result.errors.is_empty() {
            for error in &source_result.errors {
                eprintln!("    Error: {error}");
            }
        }
        if !source_result.warnings.is_empty() {
            for warning in &source_result.warnings {
                println!("    Warning: {warning}");
            }
        }
    }
}
