//! Init command implementation

use crate::Config;
use anyhow::Result;
use clap::{ArgMatches, Command};
use std::path::PathBuf;
use tracing::info;

pub fn command() -> Command {
    Command::new("init")
        .about("Initialize a new configuration file")
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .value_name("FILE")
                .default_value(".jsonnet-gen.yaml"),
        )
        .arg(
            clap::Arg::new("example")
                .short('e')
                .long("example")
                .help("Create example configuration")
                .action(clap::ArgAction::SetTrue),
        )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    let output_path = PathBuf::from(matches.get_one::<String>("output").unwrap());
    let example = matches.get_flag("example");

    info!("Initializing configuration file: {:?}", output_path);

    let config = if example {
        create_example_config()
    } else {
        Config::default()
    };

    config.save_to_file(&output_path)?;

    info!("Configuration file created: {:?}", output_path);

    if example {
        println!("Example configuration created with sample CRD sources.");
        println!("Edit the file to customize your sources and settings.");
    } else {
        println!("Empty configuration file created.");
        println!("Add sources and settings to get started.");
    }

    Ok(())
}

fn create_example_config() -> Config {
    let mut config = Config::default();

    // Add example CRD source
    config
        .sources
        .push(crate::config::Source::Crd(crate::config::CrdSource {
            name: "example-crds".to_string(),
            git: crate::config::GitSource {
                url: "https://github.com/example/k8s-manifests.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec!["example.com/v1".to_string()],
            output_path: PathBuf::from("./generated/example"),
        }));

    config
}
