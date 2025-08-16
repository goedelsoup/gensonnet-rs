//! Plugin management CLI commands

use crate::JsonnetGen;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List available plugins
    List(ListArgs),

    /// Show plugin information
    Info(InfoArgs),

    /// Enable a plugin
    Enable(EnableArgs),

    /// Disable a plugin
    Disable(DisableArgs),

    /// Install a plugin
    Install(InstallArgs),

    /// Uninstall a plugin
    Uninstall(UninstallArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Show detailed information
    #[arg(short, long)]
    detailed: bool,

    /// Filter by capability
    #[arg(long)]
    capability: Option<String>,

    /// Filter by source type
    #[arg(long)]
    source_type: Option<String>,
}

#[derive(Args)]
pub struct InfoArgs {
    /// Plugin ID
    plugin_id: String,
}

#[derive(Args)]
pub struct EnableArgs {
    /// Plugin ID
    plugin_id: String,
}

#[derive(Args)]
pub struct DisableArgs {
    /// Plugin ID
    plugin_id: String,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Plugin source (file path, URL, or registry name)
    source: String,

    /// Plugin version
    #[arg(long)]
    version: Option<String>,

    /// Install to specific directory
    #[arg(long)]
    target_dir: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct UninstallArgs {
    /// Plugin ID
    plugin_id: String,

    /// Remove plugin files
    #[arg(long)]
    remove_files: bool,
}

/// Create the plugins command
pub fn command() -> clap::Command {
    clap::Command::new("plugins")
        .about("Manage plugins")
        .subcommand_negates_reqs(true)
        .subcommand(
            clap::Command::new("list")
                .about("List available plugins")
                .arg(clap::Arg::new("detailed").short('d').long("detailed").help("Show detailed information").action(clap::ArgAction::SetTrue))
                .arg(clap::arg!(--capability <CAPABILITY> "Filter by capability"))
                .arg(clap::arg!(--source_type <SOURCE_TYPE> "Filter by source type")),
        )
        .subcommand(
            clap::Command::new("info")
                .about("Show plugin information")
                .arg(clap::arg!(<PLUGIN_ID> "Plugin ID")),
        )
        .subcommand(
            clap::Command::new("enable")
                .about("Enable a plugin")
                .arg(clap::arg!(<PLUGIN_ID> "Plugin ID")),
        )
        .subcommand(
            clap::Command::new("disable")
                .about("Disable a plugin")
                .arg(clap::arg!(<PLUGIN_ID> "Plugin ID")),
        )
        .subcommand(
            clap::Command::new("install")
                .about("Install a plugin")
                .arg(clap::arg!(<SOURCE> "Plugin source"))
                .arg(clap::arg!(--version <VERSION> "Plugin version"))
                .arg(clap::arg!(--target_dir <TARGET_DIR> "Install to specific directory")),
        )
        .subcommand(
            clap::Command::new("uninstall")
                .about("Uninstall a plugin")
                .arg(clap::arg!(<PLUGIN_ID> "Plugin ID"))
                .arg(clap::Arg::new("remove_files").long("remove_files").help("Remove plugin files").action(clap::ArgAction::SetTrue)),
        )
}

/// Run plugin command
pub async fn run(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            let args = ListArgs {
                detailed: sub_matches.get_flag("detailed"),
                capability: sub_matches.get_one::<String>("capability").cloned(),
                source_type: sub_matches.get_one::<String>("source_type").cloned(),
            };
            run_list(args).await
        }
        Some(("info", sub_matches)) => {
            let plugin_id = sub_matches.get_one::<String>("PLUGIN_ID").unwrap().clone();
            let args = InfoArgs { plugin_id };
            run_info(args).await
        }
        Some(("enable", sub_matches)) => {
            let plugin_id = sub_matches.get_one::<String>("PLUGIN_ID").unwrap().clone();
            let args = EnableArgs { plugin_id };
            run_enable(args).await
        }
        Some(("disable", sub_matches)) => {
            let plugin_id = sub_matches.get_one::<String>("PLUGIN_ID").unwrap().clone();
            let args = DisableArgs { plugin_id };
            run_disable(args).await
        }
        Some(("install", sub_matches)) => {
            let source = sub_matches.get_one::<String>("SOURCE").unwrap().clone();
            let args = InstallArgs {
                source,
                version: sub_matches.get_one::<String>("version").cloned(),
                target_dir: sub_matches.get_one::<std::path::PathBuf>("target_dir").cloned(),
            };
            run_install(args).await
        }
        Some(("uninstall", sub_matches)) => {
            let plugin_id = sub_matches.get_one::<String>("PLUGIN_ID").unwrap().clone();
            let args = UninstallArgs {
                plugin_id,
                remove_files: sub_matches.get_flag("remove_files"),
            };
            run_uninstall(args).await
        }
        _ => {
            let _ = command().print_help();
            Ok(())
        }
    }
}

async fn run_list(args: ListArgs) -> Result<()> {
    let app = create_app().await?;
    let mut plugins = app.get_plugin_info().await?;

    // Apply filters
    if let Some(capability) = &args.capability {
        plugins.retain(|plugin| {
            plugin.capabilities.iter().any(|cap| {
                format!("{:?}", cap).to_lowercase() == capability.to_lowercase()
            })
        });
    }

    if let Some(source_type) = &args.source_type {
        plugins.retain(|plugin| {
            plugin.supported_types.iter().any(|t| {
                t.to_lowercase() == source_type.to_lowercase()
            })
        });
    }

    if plugins.is_empty() {
        println!("No plugins found.");
        return Ok(());
    }

    println!("Available plugins:");
    println!();

    for plugin in plugins {
        println!("  {} v{}", plugin.name, plugin.version);
        println!("    ID: {}", plugin.id);
        println!("    Description: {}", plugin.description);
        println!("    Supported types: {}", plugin.supported_types.join(", "));

        if args.detailed {
            println!("    Capabilities:");
            for capability in &plugin.capabilities {
                println!("      - {capability:?}");
            }
        }

        println!();
    }

    Ok(())
}

async fn run_info(args: InfoArgs) -> Result<()> {
    let app = create_app().await?;
    let plugins = app.get_plugin_info().await?;

    if let Some(plugin) = plugins.iter().find(|p| p.id == args.plugin_id) {
        println!("Plugin Information:");
        println!("  Name: {}", plugin.name);
        println!("  ID: {}", plugin.id);
        println!("  Version: {}", plugin.version);
        println!("  Description: {}", plugin.description);
        println!("  Supported types: {}", plugin.supported_types.join(", "));
        println!("  Capabilities:");
        for capability in &plugin.capabilities {
            println!("    - {capability:?}");
        }
    } else {
        println!("Plugin '{}' not found.", args.plugin_id);
    }

    Ok(())
}

async fn run_enable(args: EnableArgs) -> Result<()> {
    let app = create_app().await?;
    
    match app.enable_plugin(&args.plugin_id).await {
        Ok(()) => {
            println!("Plugin '{}' enabled successfully.", args.plugin_id);
        }
        Err(e) => {
            eprintln!("Failed to enable plugin '{}': {}", args.plugin_id, e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn run_disable(args: DisableArgs) -> Result<()> {
    let app = create_app().await?;
    
    match app.disable_plugin(&args.plugin_id).await {
        Ok(()) => {
            println!("Plugin '{}' disabled successfully.", args.plugin_id);
        }
        Err(e) => {
            eprintln!("Failed to disable plugin '{}': {}", args.plugin_id, e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn run_install(args: InstallArgs) -> Result<()> {
    let app = create_app().await?;
    
    match app.install_plugin(&args.source, args.version.as_deref(), args.target_dir.as_ref().map(|v| &**v)).await {
        Ok(()) => {
            println!("Plugin installed successfully from: {}", args.source);
            if let Some(version) = args.version {
                println!("Version: {}", version);
            }
            if let Some(target_dir) = args.target_dir {
                println!("Installed to: {}", target_dir.display());
            }
        }
        Err(e) => {
            eprintln!("Failed to install plugin from '{}': {}", args.source, e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn run_uninstall(args: UninstallArgs) -> Result<()> {
    let app = create_app().await?;
    
    match app.uninstall_plugin(&args.plugin_id, args.remove_files).await {
        Ok(()) => {
            println!("Plugin '{}' uninstalled successfully.", args.plugin_id);
            if args.remove_files {
                println!("Plugin files have been removed.");
            }
        }
        Err(e) => {
            eprintln!("Failed to uninstall plugin '{}': {}", args.plugin_id, e);
            return Err(e);
        }
    }
    
    Ok(())
}

/// Create JsonnetGen app instance
async fn create_app() -> Result<JsonnetGen> {
    // For now, we'll use a default config since we don't have access to CLI args
    // In a real implementation, we'd need to pass the config through
    let mut config = crate::Config::default();
    // Add a dummy source to satisfy the validation
    config
        .sources
        .push(crate::config::Source::Crd(crate::config::CrdSource {
            name: "dummy".to_string(),
            git: crate::config::GitSource {
                url: "https://github.com/example/dummy.git".to_string(),
                ref_name: Some("main".to_string()),
                auth: None,
            },
            filters: vec![],
            output_path: std::path::PathBuf::from("./dummy"),
        }));
    let app = crate::JsonnetGen::new(config)?;
    app.initialize().await?;
    Ok(app)
}
