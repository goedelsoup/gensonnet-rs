//! CLI command implementations

use anyhow::Result;
use clap::{ArgMatches, Command};

pub mod commands;

/// Main CLI application
pub struct CliApp;

impl CliApp {
    /// Create the CLI application
    pub fn app() -> Command {
        Command::new("jsonnet-gen")
            .version(env!("CARGO_PKG_VERSION"))
            .about("Generate type-safe Jsonnet libraries from schema sources")
            .subcommand_negates_reqs(true)
            .subcommand(commands::init::command())
            .subcommand(commands::generate::command())
            .subcommand(commands::validate::command())
            .subcommand(commands::lock::command())
            .subcommand(commands::info::command())
            .subcommand(commands::status::command())
            .subcommand(commands::cleanup::command())
            .subcommand(commands::incremental::command())
            .subcommand(commands::plugins::command())
            .subcommand(commands::test::command())
    }

    /// Run the CLI application
    pub async fn run(matches: &ArgMatches) -> Result<()> {
        match matches.subcommand() {
            Some(("init", sub_matches)) => commands::init::run(sub_matches).await,
            Some(("generate", sub_matches)) => commands::generate::run(sub_matches).await,
            Some(("validate", sub_matches)) => commands::validate::run(sub_matches).await,
            Some(("lock", sub_matches)) => commands::lock::run(sub_matches).await,
            Some(("info", sub_matches)) => commands::info::run(sub_matches).await,
            Some(("status", sub_matches)) => commands::status::run(sub_matches).await,
            Some(("cleanup", sub_matches)) => commands::cleanup::run(sub_matches).await,
            Some(("incremental", sub_matches)) => commands::incremental::run(sub_matches).await,
            Some(("plugins", sub_matches)) => commands::plugins::run(sub_matches).await,
            Some(("test", sub_matches)) => commands::test::run(sub_matches).await,
            _ => {
                // No subcommand provided, show help
                let _ = Self::app().print_help();
                Ok(())
            }
        }
    }
}

/// Common CLI utilities
pub mod utils {
    use anyhow::{anyhow, Result};
    use std::path::PathBuf;

    /// Get configuration file path from arguments or use default
    pub fn get_config_path(matches: &clap::ArgMatches) -> Result<PathBuf> {
        if let Some(config_path) = matches.get_one::<String>("config") {
            Ok(PathBuf::from(config_path))
        } else {
            // Look for default config files
            let default_paths = [
                PathBuf::from(".jsonnet-gen.yaml"),
                PathBuf::from(".jsonnet-gen.yml"),
                PathBuf::from("jsonnet-gen.yaml"),
                PathBuf::from("jsonnet-gen.yml"),
            ];

            for path in &default_paths {
                if path.exists() {
                    return Ok(path.clone());
                }
            }

            Err(anyhow!("No configuration file found. Use --config to specify a file or create one with 'jsonnet-gen init'"))
        }
    }

    /// Load configuration from file
    pub fn load_config(matches: &clap::ArgMatches) -> Result<crate::Config> {
        let config_path = get_config_path(matches)?;
        crate::Config::from_file(&config_path)
    }

    /// Create JsonnetGen instance
    pub fn create_app(config: crate::Config) -> Result<crate::JsonnetGen> {
        crate::JsonnetGen::new(config)
    }
}
