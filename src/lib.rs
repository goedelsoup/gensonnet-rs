//! Jsonnet Generator Library
//!
//! A Rust library for generating type-safe Jsonnet libraries from various schema sources,
//! starting with Kubernetes CustomResourceDefinitions (CRDs).

pub mod cli;
pub mod config;
pub mod crd;
pub mod generator;
pub mod git;
pub mod lockfile;
pub mod plugin;
pub mod utils;

pub use config::{Config, GenerationConfig, OutputConfig, Source};
pub use crd::{CrdParser, CrdSchema, SchemaAnalysis, ValidationRules};
pub use generator::{GenerationResult, JsonnetGenerator, SourceResult};
pub use git::GitManager;
pub use lockfile::{IncrementalPlan, Lockfile, LockfileEntry, LockfileManager};
pub use plugin::{ExtractedSchema, PluginConfig, PluginContext, PluginManager, PluginResult};

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};
use chrono::Utc;

/// Main application context that coordinates all components
pub struct JsonnetGen {
    config: Config,
    git_manager: GitManager,
    crd_parser: CrdParser,
    generator: JsonnetGenerator,
    lockfile_manager: LockfileManager,
    plugin_manager: Arc<PluginManager>,
}

impl JsonnetGen {
    /// Create a new JsonnetGen instance with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        let git_manager = GitManager::new()?;
        let crd_parser = CrdParser::new();
        let generator = JsonnetGenerator::new(config.output.clone());
        let lockfile_manager = LockfileManager::new(LockfileManager::default_path());
        let plugin_manager = Arc::new(PluginManager::new());

        Ok(Self {
            config,
            git_manager,
            crd_parser,
            generator,
            lockfile_manager,
            plugin_manager,
        })
    }

    /// Initialize the plugin system
    pub async fn initialize_plugins(&self) -> Result<()> {
        info!("Initializing plugin system");

        // Load built-in plugins
        self.load_builtin_plugins().await?;

        // Discover and load external plugins
        self.discover_external_plugins().await?;

        info!("Plugin system initialized successfully");
        Ok(())
    }

    /// Load built-in plugins
    async fn load_builtin_plugins(&self) -> Result<()> {
        info!("Loading built-in plugins");

        // Register Go AST plugin factory
        let go_ast_factory = Box::new(plugin::ast::GoAstPluginFactory);
        self.plugin_manager
            .register_factory("go-ast".to_string(), go_ast_factory)
            .await;

        // Register CRD plugin factory
        let crd_factory = Box::new(plugin::crd::CrdPluginFactory);
        self.plugin_manager
            .register_factory("crd".to_string(), crd_factory)
            .await;

        // Register OpenAPI plugin factory
        let openapi_factory = Box::new(plugin::openapi::OpenApiPluginFactory);
        self.plugin_manager
            .register_factory("openapi".to_string(), openapi_factory)
            .await;

        // Create Go AST plugin
        let go_ast_config = PluginConfig {
            plugin_id: "go-ast:builtin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![
                plugin::PluginCapability::Parse,
                plugin::PluginCapability::SchemaExtraction,
                plugin::PluginCapability::AstProcessing,
            ],
        };

        self.plugin_manager
            .create_plugin("go-ast", go_ast_config)
            .await?;

        // Create CRD plugin
        let crd_config = PluginConfig {
            plugin_id: "crd:builtin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![
                plugin::PluginCapability::Parse,
                plugin::PluginCapability::SchemaExtraction,
                plugin::PluginCapability::Validation,
            ],
        };

        self.plugin_manager
            .create_plugin("crd", crd_config)
            .await?;

        // Create OpenAPI plugin
        let openapi_config = PluginConfig {
            plugin_id: "openapi:builtin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![
                plugin::PluginCapability::Parse,
                plugin::PluginCapability::SchemaExtraction,
                plugin::PluginCapability::Validation,
            ],
        };

        self.plugin_manager
            .create_plugin("openapi", openapi_config)
            .await?;

        info!("Built-in plugins loaded successfully");
        Ok(())
    }

    /// Discover and load external plugins
    async fn discover_external_plugins(&self) -> Result<()> {
        info!("Discovering external plugins");

        if !self.config.plugins.enable_external_discovery {
            info!("External plugin discovery is disabled");
            return Ok(());
        }

        // Create plugin registry
        let registry = Arc::new(plugin::registry::PluginRegistry::new(Arc::clone(&self.plugin_manager)));

        // Add plugin directories to registry
        for plugin_dir in &self.config.plugins.plugin_directories {
            let expanded_dir = self.expand_plugin_directory(plugin_dir)?;
            if expanded_dir.exists() {
                info!("Adding plugin directory: {:?}", expanded_dir);
                registry.add_plugin_directory(expanded_dir).await;
            } else {
                info!("Plugin directory does not exist, skipping: {:?}", expanded_dir);
            }
        }

        // Create discovery service
        let discovery_service = plugin::registry::PluginDiscoveryService::new(registry);

        // Discover and load plugins
        match discovery_service.discover_and_load().await {
            Ok(_) => {
                info!("External plugin discovery completed successfully");
            }
            Err(e) => {
                warn!("External plugin discovery failed: {}", e);
                // Don't fail the entire process if plugin discovery fails
            }
        }

        Ok(())
    }

    /// Expand plugin directory path (handle ~ and environment variables)
    fn expand_plugin_directory(&self, path: &PathBuf) -> Result<PathBuf> {
        let path_str = path.to_string_lossy();
        
        if path_str.starts_with("~/") {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| anyhow::anyhow!("Could not determine home directory"))?;
            return Ok(PathBuf::from(home).join(&path_str[2..]));
        }

        // Handle environment variables like $XDG_CONFIG_HOME
        if path_str.contains('$') {
            let expanded = shellexpand::env(&path_str)
                .map_err(|e| anyhow::anyhow!("Failed to expand environment variables: {}", e))?;
            return Ok(PathBuf::from(expanded.as_ref()));
        }

        Ok(path.clone())
    }



    /// Generate Jsonnet libraries from all configured sources
    pub async fn generate(&self) -> Result<GenerationResult> {
        info!("Starting Jsonnet library generation");

        let start_time = Instant::now();
        let mut total_errors = 0;
        let total_warnings = 0;

        // Check if incremental generation is possible
        let current_sources = self.get_current_source_commits().await?;
        let incremental_plan = self
            .lockfile_manager
            .get_incremental_plan(&current_sources.keys().cloned().collect::<Vec<_>>())?;

        let results =
            if incremental_plan.can_incremental && !incremental_plan.changed_sources.is_empty() {
                info!(
                    "Using incremental generation for {} changed sources",
                    incremental_plan.changed_sources.len()
                );
                self.generate_incremental(&incremental_plan).await?
            } else {
                info!(
                    "Performing full generation for {} sources",
                    self.config.sources.len()
                );
                self.generate_full().await?
            };

        // Calculate statistics
        for result in &results {
            total_errors += result.errors.len();
        }

        let generation_time = start_time.elapsed();
        info!("Generation completed in {:?}", generation_time);

        let result = GenerationResult {
            sources_processed: results.len(),
            total_sources: self.config.sources.len(),
            results: results.clone(),
            statistics: GenerationStatistics {
                total_processing_time_ms: generation_time.as_millis() as u64,
                sources_processed: results.len(),
                files_generated: results.iter().map(|r| r.files_generated).sum(),
                error_count: total_errors,
                warning_count: total_warnings,
                cache_hit_rate: self.calculate_cache_hit_rate(&incremental_plan),
            },
        };

        // Update lockfile with new generation data
        self.update_lockfile(&result).await?;

        Ok(result)
    }

    /// Generate libraries incrementally
    async fn generate_incremental(&self, plan: &IncrementalPlan) -> Result<Vec<SourceResult>> {
        let mut results = Vec::new();

        // Process changed sources first
        for source_id in &plan.changed_sources {
            if let Some(source) = self.find_source_by_id(source_id) {
                match self.process_source_with_recovery(source).await {
                    Ok(result) => {
                        info!("Successfully processed changed source: {}", source_id);
                        results.push(result);
                    }
                    Err(e) => {
                        error!("Failed to process changed source {}: {}", source_id, e);
                        if self.config.generation.fail_fast {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Process dependent sources
        for source_id in &plan.dependent_sources {
            if let Some(source) = self.find_source_by_id(source_id) {
                match self.process_source_with_recovery(source).await {
                    Ok(result) => {
                        info!("Successfully processed dependent source: {}", source_id);
                        results.push(result);
                    }
                    Err(e) => {
                        warn!("Failed to process dependent source {}: {}", source_id, e);
                        // Don't fail fast for dependent sources
                    }
                }
            }
        }

        Ok(results)
    }

    /// Generate libraries for all sources
    async fn generate_full(&self) -> Result<Vec<SourceResult>> {
        let mut results = Vec::new();

        for source in &self.config.sources {
            match self.process_source_with_recovery(source).await {
                Ok(result) => {
                    info!("Successfully processed source: {}", source.name());
                    results.push(result);
                }
                Err(e) => {
                    error!("Failed to process source {}: {}", source.name(), e);
                    if self.config.generation.fail_fast {
                        return Err(e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Process a single source with error recovery
    pub async fn process_source_with_recovery(&self, source: &Source) -> Result<SourceResult> {
        let start_time = Instant::now();

        match self.process_source(source).await {
            Ok(mut result) => {
                let processing_time = start_time.elapsed();
                result.processing_time_ms = processing_time.as_millis() as u64;
                Ok(result)
            }
            Err(e) => {
                // Try to recover by generating partial results
                warn!(
                    "Attempting error recovery for source {}: {}",
                    source.name(),
                    e
                );
                self.generate_partial_result(source, &e).await
            }
        }
    }

    /// Generate a partial result when processing fails
    async fn generate_partial_result(
        &self,
        source: &Source,
        error: &anyhow::Error,
    ) -> Result<SourceResult> {
        // Create a minimal result with error information
        Ok(SourceResult {
            source_type: source.source_type().to_string(),
            files_generated: 0,
            errors: vec![error.to_string()],
            output_path: source.output_path().to_path_buf(),
            processing_time_ms: 0,
            warnings: vec!["Partial generation due to processing error".to_string()],
        })
    }

    /// Process a single source
    async fn process_source(&self, source: &Source) -> Result<SourceResult> {
        match source {
            Source::Crd(crd_source) => {
                // Try to use plugin first, fall back to built-in CRD parser
                if let Ok(plugin_result) = self.process_with_plugins(crd_source).await {
                    return Ok(plugin_result);
                }

                // Fall back to built-in CRD processing
                let repo_path = self.git_manager.ensure_repository(&crd_source.git).await?;
                let schemas = self
                    .crd_parser
                    .parse_from_directory(&repo_path, &crd_source.filters)?;
                self.generator
                    .generate_crd_library(&schemas, &crd_source.output_path)
                    .await
            }
            Source::GoAst(go_ast_source) => {
                // Use Go AST plugin
                self.process_go_source(go_ast_source).await
            }
            Source::OpenApi(openapi_source) => {
                // Use OpenAPI plugin
                self.process_openapi_source(openapi_source).await
            }
        }
    }

    /// Process source with plugins
    async fn process_with_plugins(
        &self,
        crd_source: &crate::config::CrdSource,
    ) -> Result<SourceResult> {
        // Create plugin context
        let plugin_config = PluginConfig {
            plugin_id: "crd:builtin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![
                plugin::PluginCapability::Parse,
                plugin::PluginCapability::SchemaExtraction,
            ],
        };

        let context = PluginContext::new(
            crd_source.output_path.clone(),
            crd_source.output_path.clone(),
            plugin_config,
        );

        // Process with plugin manager
        let repo_path = self.git_manager.ensure_repository(&crd_source.git).await?;
        let plugin_result = self
            .plugin_manager
            .process_source(&repo_path, &context)
            .await?;

        // Convert plugin result to source result
        Ok(SourceResult {
            source_type: "crd".to_string(),
            files_generated: plugin_result.generated_files.len(),
            errors: plugin_result.errors,
            output_path: crd_source.output_path.clone(),
            processing_time_ms: plugin_result.statistics.processing_time_ms,
            warnings: plugin_result.warnings,
        })
    }

    /// Process Go source with AST plugin
    async fn process_go_source(&self, go_ast_source: &crate::config::GoAstSource) -> Result<SourceResult> {
        let start_time = std::time::Instant::now();
        
        // Ensure repository is available
        let repo_path = self.git_manager.ensure_repository(&go_ast_source.git).await?;
        
        // Find Go source files
        let go_files = self.find_go_files(&repo_path, &go_ast_source.include_patterns, &go_ast_source.exclude_patterns).await?;
        
        if go_files.is_empty() {
            return Err(anyhow::anyhow!("No Go source files found matching the patterns"));
        }
        
        // Process each Go file with the plugin
        let mut all_schemas = Vec::new();
        let mut total_errors = 0;
        let total_warnings = 0;
        
        for go_file in &go_files {
            match self.process_go_file_with_plugin(go_file, go_ast_source).await {
                Ok(schemas) => {
                    all_schemas.extend(schemas);
                }
                Err(e) => {
                    total_errors += 1;
                    tracing::warn!("Failed to process Go file {}: {}", go_file.display(), e);
                }
            }
        }
        
        // Generate Jsonnet code from schemas
        let generated_files = self.generate_jsonnet_from_schemas(&all_schemas, &go_ast_source.output_path).await?;
        
        let processing_time = start_time.elapsed();
        
        Ok(SourceResult {
            source_type: "go_ast".to_string(),
            files_generated: generated_files.len(),
            errors: if total_errors > 0 { vec![format!("{} files failed to process", total_errors)] } else { vec![] },
            output_path: go_ast_source.output_path.clone(),
            processing_time_ms: processing_time.as_millis() as u64,
            warnings: if total_warnings > 0 { vec![format!("{} warnings generated", total_warnings)] } else { vec![] },
        })
    }

    /// Find Go source files matching the patterns
    async fn find_go_files(
        &self,
        repo_path: &Path,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Vec<PathBuf>> {
        let mut go_files = Vec::new();
        
        for pattern in include_patterns {
            let glob_pattern = repo_path.join(pattern);
            let entries = glob::glob(&glob_pattern.to_string_lossy())?;
            
            for entry in entries {
                match entry {
                    Ok(path) => {
                        // Check if file should be excluded
                        let should_exclude = exclude_patterns.iter().any(|exclude_pattern| {
                            let exclude_glob = repo_path.join(exclude_pattern);
                            if let Ok(mut exclude_entries) = glob::glob(&exclude_glob.to_string_lossy()) {
                                exclude_entries.any(|exclude_entry| {
                                    exclude_entry.map_or(false, |exclude_path| exclude_path == path)
                                })
                            } else {
                                false
                            }
                        });
                        
                        if !should_exclude && path.is_file() {
                            go_files.push(path);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to match pattern {}: {}", pattern, e);
                    }
                }
            }
        }
        
        Ok(go_files)
    }

    /// Process a single Go file with the plugin
    async fn process_go_file_with_plugin(
        &self,
        go_file: &Path,
        go_ast_source: &crate::config::GoAstSource,
    ) -> Result<Vec<crate::plugin::ExtractedSchema>> {
        // Create plugin context
        let plugin_config = crate::plugin::PluginConfig {
            plugin_id: "go-ast:builtin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![
                crate::plugin::PluginCapability::Parse,
                crate::plugin::PluginCapability::SchemaExtraction,
                crate::plugin::PluginCapability::AstProcessing,
            ],
        };

        let context = crate::plugin::PluginContext::new(
            go_file.parent().unwrap_or(Path::new(".")).to_path_buf(),
            go_ast_source.output_path.clone(),
            plugin_config,
        );

        // Process with plugin manager
        let plugin_result = self
            .plugin_manager
            .process_source(go_file, &context)
            .await?;

        Ok(plugin_result.schemas)
    }

    /// Generate Jsonnet code from extracted schemas
    async fn generate_jsonnet_from_schemas(
        &self,
        schemas: &[crate::plugin::ExtractedSchema],
        output_path: &Path,
    ) -> Result<Vec<PathBuf>> {
        let mut generated_files = Vec::new();
        
        // Ensure output directory exists
        tokio::fs::create_dir_all(output_path).await?;
        
        for schema in schemas {
            let output_file = output_path.join(format!("{}.libsonnet", schema.name.to_lowercase()));
            
            // Generate Jsonnet code from the schema
            let jsonnet_code = self.generate_jsonnet_code(schema)?;
            tokio::fs::write(&output_file, jsonnet_code).await?;
            
            generated_files.push(output_file);
        }
        
        Ok(generated_files)
    }

    /// Generate Jsonnet code from schema
    fn generate_jsonnet_code(&self, schema: &crate::plugin::ExtractedSchema) -> Result<String> {
        let mut code = String::new();
        
        code.push_str(&format!("// Generated from Go AST: {}\n", schema.name));
        code.push_str(&format!("// Source: {}\n\n", schema.source_file.display()));
        
        // Add imports
        code.push_str("local k = import \"k.libsonnet\";\n");
        code.push_str("local validate = import \"_validation.libsonnet\";\n\n");
        
        // Generate the main function
        code.push_str(&format!("// Create a new {} resource\n", schema.name));
        code.push_str("function(metadata, spec={}) {\n");
        code.push_str(&format!(
            "  apiVersion: \"{}\",\n",
            schema.name.to_lowercase()
        ));
        code.push_str(&format!("  kind: \"{}\",\n", schema.name));
        code.push_str("  metadata: metadata,\n");
        code.push_str("  spec: spec,\n");
        code.push_str("}\n");
        
        Ok(code)
    }

    /// Process OpenAPI source with plugin
    async fn process_openapi_source(&self, openapi_source: &crate::config::OpenApiSource) -> Result<SourceResult> {
        let start_time = std::time::Instant::now();
        
        // Ensure repository is available
        let repo_path = self.git_manager.ensure_repository(&openapi_source.git).await?;
        
        // Find OpenAPI specification files
        let openapi_files = self.find_openapi_files(&repo_path, &openapi_source.include_patterns, &openapi_source.exclude_patterns).await?;
        
        if openapi_files.is_empty() {
            return Err(anyhow::anyhow!("No OpenAPI specification files found matching the patterns"));
        }
        
        // Process each OpenAPI file with the plugin
        let mut all_schemas = Vec::new();
        let mut total_errors = 0;
        let total_warnings = 0;
        
        for openapi_file in &openapi_files {
            match self.process_openapi_file_with_plugin(openapi_file, openapi_source).await {
                Ok(schemas) => {
                    all_schemas.extend(schemas);
                }
                Err(e) => {
                    total_errors += 1;
                    tracing::warn!("Failed to process OpenAPI file {}: {}", openapi_file.display(), e);
                }
            }
        }
        
        // Generate Jsonnet code from schemas
        let generated_files = self.generate_jsonnet_from_schemas(&all_schemas, &openapi_source.output_path).await?;
        
        let processing_time = start_time.elapsed();
        
        Ok(SourceResult {
            source_type: "openapi".to_string(),
            files_generated: generated_files.len(),
            errors: if total_errors > 0 { vec![format!("{} files failed to process", total_errors)] } else { vec![] },
            output_path: openapi_source.output_path.clone(),
            processing_time_ms: processing_time.as_millis() as u64,
            warnings: if total_warnings > 0 { vec![format!("{} warnings generated", total_warnings)] } else { vec![] },
        })
    }

    /// Get current source commit information
    async fn get_current_source_commits(&self) -> Result<HashMap<String, String>> {
        let mut commits = HashMap::new();

        for source in &self.config.sources {
            match source {
                Source::Crd(crd_source) => {
                    let repo_path = self.git_manager.ensure_repository(&crd_source.git).await?;
                    let commit_sha = self.git_manager.get_current_commit(&repo_path)?;
                    commits.insert(source.name().to_string(), commit_sha);
                }
                Source::GoAst(go_ast_source) => {
                    let repo_path = self.git_manager.ensure_repository(&go_ast_source.git).await?;
                    let commit_sha = self.git_manager.get_current_commit(&repo_path)?;
                    commits.insert(source.name().to_string(), commit_sha);
                }
                Source::OpenApi(openapi_source) => {
                    let repo_path = self.git_manager.ensure_repository(&openapi_source.git).await?;
                    let commit_sha = self.git_manager.get_current_commit(&repo_path)?;
                    commits.insert(source.name().to_string(), commit_sha);
                }
            }
        }

        Ok(commits)
    }

    /// Find source by ID
    fn find_source_by_id(&self, source_id: &str) -> Option<&Source> {
        self.config.sources.iter().find(|s| s.name() == source_id)
    }

    /// Calculate cache hit rate
    fn calculate_cache_hit_rate(&self, plan: &IncrementalPlan) -> f64 {
        if plan.requires_full_regeneration() {
            0.0
        } else {
            let total_sources = self.config.sources.len();
            let cached_sources = total_sources - plan.total_sources();
            cached_sources as f64 / total_sources as f64
        }
    }

    /// Update lockfile with generation results
    async fn update_lockfile(&self, result: &GenerationResult) -> Result<()> {
        let mut lockfile = self.lockfile_manager.load_or_create()?;

        // Update sources
        let current_sources = self.get_current_source_commits().await?;
        for (source_id, commit_sha) in current_sources {
            let source = self.find_source_by_id(&source_id).unwrap();
            let entry = LockfileEntry::new(
                source.git_url().to_string(),
                source.git_ref().unwrap_or("main").to_string(),
                commit_sha,
                source.filters().to_vec(),
            );
            lockfile.add_source(source_id, entry);
        }

        // Update files
        for source_result in &result.results {
            for file_path in self.get_generated_files(&source_result.output_path).await? {
                if let Ok(checksum) = lockfile::FileChecksum::from_file(&file_path) {
                    lockfile.add_file(file_path, checksum);
                }
            }
        }

        // Update statistics
        lockfile.statistics = lockfile::GenerationStatistics {
            total_processing_time_ms: result.statistics.total_processing_time_ms,
            sources_processed: result.statistics.sources_processed,
            files_generated: result.statistics.files_generated,
            error_count: result.statistics.error_count,
            warning_count: result.statistics.warning_count,
            cache_hit_rate: result.statistics.cache_hit_rate,
        };

        self.lockfile_manager.save(&lockfile)?;
        Ok(())
    }

    /// Get generated files from output directory
    async fn get_generated_files(
        &self,
        output_path: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        if output_path.exists() {
            for entry in walkdir::WalkDir::new(output_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                files.push(entry.path().to_path_buf());
            }
        }

        Ok(files)
    }

    /// Initialize the application (create directories, validate config, etc.)
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing JsonnetGen application");

        // Ensure output directories exist
        std::fs::create_dir_all(&self.config.output.base_path)?;

        // Validate configuration
        self.config.validate()?;

        // Initialize plugin system
        self.initialize_plugins().await?;

        // Initialize lockfile if it doesn't exist
        if !LockfileManager::default_path().exists() {
            let lockfile = lockfile::Lockfile::new();
            self.lockfile_manager.save(&lockfile)?;
        }

        info!("Initialization completed successfully");
        Ok(())
    }

    /// Clean up stale entries
    pub fn cleanup(&self, max_age_hours: u64) -> Result<()> {
        info!(
            "Cleaning up stale entries older than {} hours",
            max_age_hours
        );
        self.lockfile_manager.cleanup_stale_entries(max_age_hours)?;
        info!("Cleanup completed successfully");
        Ok(())
    }

    /// Perform a dry run of cleanup to show what would be cleaned
    pub fn cleanup_dry_run(&self, max_age_hours: u64) -> Result<CleanupDryRunResult> {
        info!(
            "Dry run: Checking for stale entries older than {} hours",
            max_age_hours
        );

        let lockfile = self.lockfile_manager.load_or_create()?;
        let mut stale_sources = Vec::new();
        let mut stale_files = Vec::new();
        let mut total_size_freed = 0u64;

        // Check for stale sources
        for (source_id, entry) in &lockfile.sources {
            if entry.is_stale(max_age_hours) {
                stale_sources.push(CleanupSourceEntry {
                    source_id: source_id.clone(),
                    git_url: entry.url.clone(),
                    git_ref: entry.ref_name.clone(),
                    fetched_at: entry.fetched_at,
                    age_hours: (Utc::now().signed_duration_since(entry.fetched_at).num_hours() as u64),
                });
            }
        }

        // Check for stale files
        for (file_path, checksum) in &lockfile.files {
            if checksum.is_stale(max_age_hours) {
                stale_files.push(CleanupFileEntry {
                    file_path: file_path.clone(),
                    size: checksum.size,
                    modified_at: checksum.modified_at,
                    age_hours: (Utc::now().signed_duration_since(checksum.modified_at).num_hours() as u64),
                });
                total_size_freed += checksum.size;
            }
        }

        let total_sources_removed = stale_sources.len();
        let total_files_removed = stale_files.len();
        
        let result = CleanupDryRunResult {
            max_age_hours,
            stale_sources,
            stale_files,
            total_sources_removed,
            total_files_removed,
            total_size_freed,
            lockfile_path: self.lockfile_manager.path().clone(),
        };

        info!(
            "Dry run: Would remove {} sources and {} files ({} bytes)",
            result.total_sources_removed,
            result.total_files_removed,
            result.total_size_freed
        );

        Ok(result)
    }

    /// Get generation status
    pub async fn get_status(&self) -> Result<GenerationStatus> {
        let lockfile = self.lockfile_manager.load_or_create()?;
        let current_sources = self.get_current_source_commits().await?;
        let incremental_plan = self
            .lockfile_manager
            .get_incremental_plan(&current_sources.keys().cloned().collect::<Vec<_>>())?;

        Ok(GenerationStatus {
            last_generation: lockfile.generated_at,
            tool_version: lockfile.tool_version,
            sources_count: self.config.sources.len(),
            changed_sources: incremental_plan.changed_sources,
            dependent_sources: incremental_plan.dependent_sources,
            can_incremental: incremental_plan.can_incremental,
            estimated_time_ms: incremental_plan.estimated_time_ms,
            statistics: lockfile.statistics,
        })
    }

    /// Perform a dry run of generation to show what would be generated
    pub async fn dry_run(&self) -> Result<DryRunResult> {
        info!("Starting dry run generation");

        let start_time = Instant::now();
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut results = Vec::new();

        // Check if incremental generation is possible
        let current_sources = self.get_current_source_commits().await?;
        let incremental_plan = self
            .lockfile_manager
            .get_incremental_plan(&current_sources.keys().cloned().collect::<Vec<_>>())?;

        let sources_to_process = if incremental_plan.can_incremental && !incremental_plan.changed_sources.is_empty() {
            info!(
                "Dry run: Would use incremental generation for {} changed sources",
                incremental_plan.changed_sources.len()
            );
            // Get changed sources
            let mut sources = Vec::new();
            for source_id in &incremental_plan.changed_sources {
                if let Some(source) = self.find_source_by_id(source_id) {
                    sources.push(source);
                }
            }
            // Get dependent sources
            for source_id in &incremental_plan.dependent_sources {
                if let Some(source) = self.find_source_by_id(source_id) {
                    sources.push(source);
                }
            }
            sources
        } else {
            info!(
                "Dry run: Would perform full generation for {} sources",
                self.config.sources.len()
            );
            self.config.sources.iter().collect::<Vec<_>>()
        };

        // Process each source in dry run mode
        for source in &sources_to_process {
            match self.process_source_dry_run(source).await {
                Ok(result) => {
                    info!("Dry run: Successfully processed source: {}", source.name());
                    results.push(result);
                }
                Err(e) => {
                    error!("Dry run: Failed to process source {}: {}", source.name(), e);
                    if self.config.generation.fail_fast {
                        return Err(e);
                    }
                    // Add error result
                    results.push(DryRunSourceResult {
                        source_name: source.name().to_string(),
                        source_type: source.source_type().to_string(),
                        files_would_generate: 0,
                        errors: vec![e.to_string()],
                        warnings: Vec::new(),
                        output_path: source.output_path().to_path_buf(),
                    });
                }
            }
        }

        // Calculate statistics
        for result in &results {
            total_errors += result.errors.len();
            total_warnings += result.warnings.len();
        }

        let generation_time = start_time.elapsed();
        info!("Dry run completed in {:?}", generation_time);

        let result = DryRunResult {
            sources_processed: results.len(),
            total_sources: self.config.sources.len(),
            results: results.clone(),
            statistics: DryRunStatistics {
                total_processing_time_ms: generation_time.as_millis() as u64,
                sources_processed: results.len(),
                files_would_generate: results.iter().map(|r| r.files_would_generate).sum(),
                error_count: total_errors,
                warning_count: total_warnings,
                cache_hit_rate: self.calculate_cache_hit_rate(&incremental_plan),
                incremental_mode: incremental_plan.can_incremental && !incremental_plan.changed_sources.is_empty(),
                changed_sources_count: incremental_plan.changed_sources.len(),
                dependent_sources_count: incremental_plan.dependent_sources.len(),
            },
        };

        Ok(result)
    }

    /// Process a single source in dry run mode
    async fn process_source_dry_run(&self, source: &Source) -> Result<DryRunSourceResult> {
        let start_time = Instant::now();
        let source_name = source.name();
        
        info!("Dry run: Processing source: {}", source_name);

        // Simulate the processing without actually writing files
        let mut files_would_generate = 0;
        let mut errors = Vec::new();
        let warnings = Vec::new();

        match source {
            Source::Crd(crd_source) => {
                // Simulate CRD processing
                match self.git_manager.ensure_repository(&crd_source.git).await {
                    Ok(repo_path) => {
                        // Parse CRDs from the repository
                        match self.crd_parser.parse_from_directory(&repo_path, &crd_source.filters) {
                            Ok(schemas) => {
                                // Calculate how many files would be generated
                                let grouped_schemas = self.group_schemas_by_version(&schemas);
                                files_would_generate = grouped_schemas.len() + 3; // +3 for index, metadata, and validation files
                                
                                info!("Dry run: Would generate {} files for CRD source {}", files_would_generate, source_name);
                            }
                            Err(e) => {
                                errors.push(format!("Failed to parse CRDs: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Failed to clone repository: {}", e));
                    }
                }
            }
            Source::GoAst(go_ast_source) => {
                // Simulate Go AST processing
                match self.git_manager.ensure_repository(&go_ast_source.git).await {
                    Ok(_) => {
                        // Estimate files based on Go files found
                        files_would_generate = 2; // At least lib.jsonnet and metadata
                        info!("Dry run: Would generate {} files for Go AST source {}", files_would_generate, source_name);
                    }
                    Err(e) => {
                        errors.push(format!("Failed to clone repository: {}", e));
                    }
                }
            }
            Source::OpenApi(openapi_source) => {
                // Simulate OpenAPI processing
                match self.git_manager.ensure_repository(&openapi_source.git).await {
                    Ok(_) => {
                        // Estimate files based on OpenAPI specs found
                        files_would_generate = 2; // At least lib.jsonnet and metadata
                        info!("Dry run: Would generate {} files for OpenAPI source {}", files_would_generate, source_name);
                    }
                    Err(e) => {
                        errors.push(format!("Failed to clone repository: {}", e));
                    }
                }
            }
        }

        let processing_time = start_time.elapsed();
        info!("Dry run: Processed source {} in {:?}", source_name, processing_time);

        Ok(DryRunSourceResult {
            source_name: source_name.to_string(),
            source_type: source.source_type().to_string(),
            files_would_generate,
            errors,
            warnings,
            output_path: source.output_path().to_path_buf(),
        })
    }

    /// Group schemas by API version (helper method for dry run)
    fn group_schemas_by_version<'a>(
        &self,
        schemas: &'a [crate::crd::CrdSchema],
    ) -> std::collections::HashMap<String, Vec<&'a crate::crd::CrdSchema>> {
        let mut grouped = std::collections::HashMap::new();

        for schema in schemas {
            let api_version = schema.api_version.clone();
            grouped
                .entry(api_version)
                .or_insert_with(Vec::new)
                .push(schema);
        }

        grouped
    }

    /// Get plugin information
    pub async fn get_plugin_info(&self) -> Result<Vec<plugin::PluginMetadata>> {
        // Return the built-in plugin metadata
        Ok(vec![
            plugin::PluginMetadata {
                id: "go-ast:builtin".to_string(),
                name: "Go AST Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Plugin for processing Go source code and extracting type information"
                    .to_string(),
                supported_types: vec!["go".to_string(), "golang".to_string()],
                capabilities: vec![
                    plugin::PluginCapability::Parse,
                    plugin::PluginCapability::SchemaExtraction,
                    plugin::PluginCapability::AstProcessing,
                ],
            },
            plugin::PluginMetadata {
                id: "crd:builtin".to_string(),
                name: "CRD Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Plugin for processing Kubernetes CustomResourceDefinitions"
                    .to_string(),
                supported_types: vec!["yaml".to_string(), "yml".to_string()],
                capabilities: vec![
                    plugin::PluginCapability::Parse,
                    plugin::PluginCapability::SchemaExtraction,
                    plugin::PluginCapability::Validation,
                ],
            },
            plugin::PluginMetadata {
                id: "openapi:builtin".to_string(),
                name: "OpenAPI Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Plugin for processing OpenAPI/Swagger specifications and extracting type information"
                    .to_string(),
                supported_types: vec!["openapi".to_string(), "swagger".to_string(), "yaml".to_string(), "json".to_string()],
                capabilities: vec![
                    plugin::PluginCapability::Parse,
                    plugin::PluginCapability::SchemaExtraction,
                    plugin::PluginCapability::Validation,
                ],
            },
        ])
    }

    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("Enabling plugin: {}", plugin_id);
        
        // For now, we only support built-in plugins
        // In the future, this would interact with a plugin registry
        match plugin_id {
            "go-ast:builtin" | "openapi:builtin" | "crd:builtin" => {
                info!("Plugin {} is already enabled (built-in)", plugin_id);
                Ok(())
            }
            _ => {
                warn!("Plugin {} not found or not supported", plugin_id);
                Err(anyhow::anyhow!("Plugin {} not found", plugin_id))
            }
        }
    }

    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("Disabling plugin: {}", plugin_id);
        
        // For now, we only support built-in plugins which cannot be disabled
        // In the future, this would interact with a plugin registry
        match plugin_id {
            "go-ast:builtin" | "openapi:builtin" | "crd:builtin" => {
                warn!("Cannot disable built-in plugin: {}", plugin_id);
                Err(anyhow::anyhow!("Cannot disable built-in plugin: {}", plugin_id))
            }
            _ => {
                warn!("Plugin {} not found", plugin_id);
                Err(anyhow::anyhow!("Plugin {} not found", plugin_id))
            }
        }
    }

    /// Install a plugin
    pub async fn install_plugin(&self, source: &str, _version: Option<&str>, _target_dir: Option<&Path>) -> Result<()> {
        info!("Installing plugin from: {}", source);
        
        // For now, we only support built-in plugins
        // In the future, this would:
        // 1. Parse the source (file path, URL, or registry name)
        // 2. Download/validate the plugin
        // 3. Install it to the target directory
        // 4. Register it with the plugin manager
        
        if source.starts_with("http") || source.starts_with("https") {
            return Err(anyhow::anyhow!("Plugin installation from URLs not yet implemented"));
        }
        
        if source.contains("://") {
            return Err(anyhow::anyhow!("Plugin installation from registry not yet implemented"));
        }
        
        // Check if it's a local file
        let source_path = Path::new(source);
        if source_path.exists() && source_path.is_file() {
            return Err(anyhow::anyhow!("Plugin installation from local files not yet implemented"));
        }
        
        // Check if it's a built-in plugin name
        match source {
            "go-ast" | "openapi" | "crd" => {
                info!("Plugin {} is already available as a built-in plugin", source);
                Ok(())
            }
            _ => {
                Err(anyhow::anyhow!("Plugin installation not yet implemented for: {}", source))
            }
        }
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, plugin_id: &str, _remove_files: bool) -> Result<()> {
        info!("Uninstalling plugin: {}", plugin_id);
        
        // For now, we only support built-in plugins which cannot be uninstalled
        // In the future, this would:
        // 1. Remove the plugin from the plugin manager
        // 2. Optionally remove plugin files
        // 3. Update the plugin registry
        
        match plugin_id {
            "go-ast:builtin" | "openapi:builtin" | "crd:builtin" => {
                warn!("Cannot uninstall built-in plugin: {}", plugin_id);
                Err(anyhow::anyhow!("Cannot uninstall built-in plugin: {}", plugin_id))
            }
            _ => {
                warn!("Plugin {} not found", plugin_id);
                Err(anyhow::anyhow!("Plugin {} not found", plugin_id))
            }
        }
    }

    /// Find OpenAPI specification files matching the patterns
    async fn find_openapi_files(
        &self,
        repo_path: &Path,
        include_patterns: &[String],
        exclude_patterns: &[String],
    ) -> Result<Vec<PathBuf>> {
        let mut openapi_files = Vec::new();
        
        for pattern in include_patterns {
            let glob_pattern = repo_path.join(pattern);
            let entries = glob::glob(&glob_pattern.to_string_lossy())?;
            
            for entry in entries {
                match entry {
                    Ok(path) => {
                        // Check if file should be excluded
                        let should_exclude = exclude_patterns.iter().any(|exclude_pattern| {
                            let exclude_glob = repo_path.join(exclude_pattern);
                            if let Ok(mut exclude_entries) = glob::glob(&exclude_glob.to_string_lossy()) {
                                exclude_entries.any(|exclude_entry| {
                                    exclude_entry.map_or(false, |exclude_path| exclude_path == path)
                                })
                            } else {
                                false
                            }
                        });
                        
                        if !should_exclude && path.is_file() {
                            openapi_files.push(path);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to match pattern {}: {}", pattern, e);
                    }
                }
            }
        }
        
        Ok(openapi_files)
    }

    /// Process a single OpenAPI file with the plugin
    async fn process_openapi_file_with_plugin(
        &self,
        openapi_file: &Path,
        openapi_source: &crate::config::OpenApiSource,
    ) -> Result<Vec<crate::plugin::ExtractedSchema>> {
        // Create plugin context
        let plugin_config = crate::plugin::PluginConfig {
            plugin_id: "openapi:builtin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![
                crate::plugin::PluginCapability::Parse,
                crate::plugin::PluginCapability::SchemaExtraction,
                crate::plugin::PluginCapability::Validation,
            ],
        };

        let context = crate::plugin::PluginContext::new(
            openapi_file.parent().unwrap_or(Path::new(".")).to_path_buf(),
            openapi_source.output_path.clone(),
            plugin_config,
        );

        // Process with plugin manager
        let plugin_result = self
            .plugin_manager
            .process_source(openapi_file, &context)
            .await?;

        Ok(plugin_result.schemas)
    }
}

/// Application error types
#[derive(thiserror::Error, Debug)]
pub enum JsonnetGenError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("CRD parsing failed: {0}")]
    CrdParsing(String),

    #[error("Generation failed: {0}")]
    Generation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_yaml::Error),

    #[error("Lockfile error: {0}")]
    Lockfile(String),

    #[error("Plugin error: {0}")]
    Plugin(String),
}

/// Result type for the main application
pub type JsonnetGenResult<T> = Result<T, JsonnetGenError>;

/// Generation status information
#[derive(Debug, Clone)]
pub struct GenerationStatus {
    pub last_generation: chrono::DateTime<chrono::Utc>,
    pub tool_version: String,
    pub sources_count: usize,
    pub changed_sources: Vec<String>,
    pub dependent_sources: Vec<String>,
    pub can_incremental: bool,
    pub estimated_time_ms: u64,
    pub statistics: lockfile::GenerationStatistics,
}

/// Enhanced generation statistics
#[derive(Debug, Clone)]
pub struct GenerationStatistics {
    pub total_processing_time_ms: u64,
    pub sources_processed: usize,
    pub files_generated: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub cache_hit_rate: f64,
}

/// Dry run result for a single source
#[derive(Debug, Clone)]
pub struct DryRunSourceResult {
    pub source_name: String,
    pub source_type: String,
    pub files_would_generate: usize,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub output_path: PathBuf,
}

/// Dry run statistics
#[derive(Debug, Clone)]
pub struct DryRunStatistics {
    pub total_processing_time_ms: u64,
    pub sources_processed: usize,
    pub files_would_generate: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub cache_hit_rate: f64,
    pub incremental_mode: bool,
    pub changed_sources_count: usize,
    pub dependent_sources_count: usize,
}

/// Dry run result
#[derive(Debug, Clone)]
pub struct DryRunResult {
    pub sources_processed: usize,
    pub total_sources: usize,
    pub results: Vec<DryRunSourceResult>,
    pub statistics: DryRunStatistics,
}

/// Cleanup dry run result for a single source entry
#[derive(Debug, Clone)]
pub struct CleanupSourceEntry {
    pub source_id: String,
    pub git_url: String,
    pub git_ref: String,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub age_hours: u64,
}

/// Cleanup dry run result for a single file entry
#[derive(Debug, Clone)]
pub struct CleanupFileEntry {
    pub file_path: PathBuf,
    pub size: u64,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub age_hours: u64,
}

/// Cleanup dry run result
#[derive(Debug, Clone)]
pub struct CleanupDryRunResult {
    pub max_age_hours: u64,
    pub stale_sources: Vec<CleanupSourceEntry>,
    pub stale_files: Vec<CleanupFileEntry>,
    pub total_sources_removed: usize,
    pub total_files_removed: usize,
    pub total_size_freed: u64,
    pub lockfile_path: PathBuf,
}

// Add missing methods to Source trait
impl Source {
    pub fn source_type(&self) -> &str {
        match self {
            Source::Crd(_) => "crd",
            Source::GoAst(_) => "go_ast",
            Source::OpenApi(_) => "openapi",
        }
    }

    pub fn git_url(&self) -> &str {
        match self {
            Source::Crd(crd) => &crd.git.url,
            Source::GoAst(go_ast) => &go_ast.git.url,
            Source::OpenApi(openapi) => &openapi.git.url,
        }
    }

    pub fn git_ref(&self) -> Option<&str> {
        match self {
            Source::Crd(crd) => crd.git.ref_name.as_deref(),
            Source::GoAst(go_ast) => go_ast.git.ref_name.as_deref(),
            Source::OpenApi(openapi) => openapi.git.ref_name.as_deref(),
        }
    }

    pub fn filters(&self) -> &[String] {
        match self {
            Source::Crd(crd) => &crd.filters,
            Source::GoAst(go_ast) => &go_ast.include_patterns,
            Source::OpenApi(openapi) => &openapi.include_patterns,
        }
    }

    pub fn output_path(&self) -> &std::path::Path {
        match self {
            Source::Crd(crd) => &crd.output_path,
            Source::GoAst(go_ast) => &go_ast.output_path,
            Source::OpenApi(openapi) => &openapi.output_path,
        }
    }
}

// Add missing fields to SourceResult
impl SourceResult {
    pub fn processing_time_ms(&self) -> u64 {
        self.processing_time_ms
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
