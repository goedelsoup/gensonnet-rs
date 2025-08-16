//! Validate command implementation

use crate::cli::utils;
use anyhow::Result;
use clap::{ArgMatches, Command};
use tracing::info;

pub fn command() -> Command {
    Command::new("validate")
        .about("Validate configuration file")
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .help("Configuration file path")
                .value_name("FILE"),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    info!("Validating configuration file");

    let config = utils::load_config(matches)?;

    println!("Configuration file is valid!");
    println!("Version: {}", config.version);
    println!("Sources: {}", config.sources.len());
    println!("Output path: {:?}", config.output.base_path);

    for source in &config.sources {
        println!(
            "  - {} ({})",
            source.name(),
            match source {
                crate::config::Source::Crd(_) => "CRD",
                crate::config::Source::GoAst(_) => "Go AST",
                crate::config::Source::OpenApi(_) => "OpenAPI",
            }
        );
    }

    Ok(())
}
