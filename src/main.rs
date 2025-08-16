//! JsonnetGen CLI binary

use anyhow::Result;

use jsonnet_gen::cli::CliApp;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jsonnet_gen=info".into()),
        )
        .init();

    // Parse command line arguments
    let matches = CliApp::app().get_matches();

    // Run the CLI application
    CliApp::run(&matches).await
}
