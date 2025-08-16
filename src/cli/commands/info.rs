//! Info command implementation

use anyhow::Result;
use clap::{ArgMatches, Command};

pub fn command() -> Command {
    Command::new("info").about("Show tool information").arg(
        clap::Arg::new("detailed")
            .short('d')
            .long("detailed")
            .help("Show detailed information")
            .action(clap::ArgAction::SetTrue),
    )
}

pub async fn run(matches: &ArgMatches) -> Result<()> {
    let detailed = matches.get_flag("detailed");

    println!("JsonnetGen - Type-safe Jsonnet Library Generator");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));

    if detailed {
        println!("\nDetailed Information:");
        println!("  - Supports Kubernetes CRD parsing");
        println!("  - Advanced OpenAPI schema processing");
        println!("  - Validation rule interpretation");
        println!("  - XDG-compliant configuration and cache directories");
        println!("  - Git repository caching and management");
        println!("  - Lockfile for reproducible builds");
        println!("  - Incremental generation support");
        println!("  - Multiple output organization strategies");
        println!("  - Error recovery and partial generation");
        println!("  - Dependency tracking and topological sorting");
    }

    Ok(())
}
