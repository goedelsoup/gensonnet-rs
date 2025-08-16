//! Status command implementation

use crate::cli::utils;
use anyhow::Result;
use clap::{ArgMatches, Command};
use tracing::info;

pub fn command() -> Command {
    Command::new("status")
        .about("Show generation status and incremental generation information")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file path")
                .value_name("FILE"),
        )
        .arg(
            clap::Arg::new("detailed")
                .short('d')
                .long("detailed")
                .help("Show detailed information")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    info!("Checking generation status");

    let config = utils::load_config(matches)?;
    let app = utils::create_app(config)?;

    let status = app.get_status().await?;

    println!("Generation Status:");
    println!(
        "  Last generation: {}",
        status.last_generation.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("  Tool version: {}", status.tool_version);
    println!("  Sources configured: {}", status.sources_count);

    if !status.changed_sources.is_empty() {
        println!("  Changed sources: {}", status.changed_sources.join(", "));
    }

    if !status.dependent_sources.is_empty() {
        println!(
            "  Dependent sources: {}",
            status.dependent_sources.join(", ")
        );
    }

    println!(
        "  Incremental generation: {}",
        if status.can_incremental {
            "possible"
        } else {
            "not possible"
        }
    );
    println!(
        "  Estimated regeneration time: {}ms",
        status.estimated_time_ms
    );

    if matches.get_flag("detailed") {
        println!("\nDetailed Statistics:");
        println!(
            "  Total processing time: {}ms",
            status.statistics.total_processing_time_ms
        );
        println!(
            "  Sources processed: {}",
            status.statistics.sources_processed
        );
        println!("  Files generated: {}", status.statistics.files_generated);
        println!("  Error count: {}", status.statistics.error_count);
        println!("  Warning count: {}", status.statistics.warning_count);
        println!(
            "  Cache hit rate: {:.1}%",
            status.statistics.cache_hit_rate * 100.0
        );
    }

    Ok(())
}
