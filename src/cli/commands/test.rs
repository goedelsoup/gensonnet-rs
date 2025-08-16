//! Plugin testing CLI commands

use anyhow::Result;
use clap::{Args, Subcommand, FromArgMatches};

use crate::plugin::testing::*;

#[derive(Subcommand)]
pub enum TestCommands {
    /// Run plugin tests
    Run(RunArgs),

    /// List available test suites
    List(ListArgs),

    /// Show test suite information
    Info(InfoArgs),

    /// Generate test report
    Report(ReportArgs),
}

#[derive(Args)]
pub struct RunArgs {
    /// Plugin ID to test
    #[arg(long)]
    plugin_id: Option<String>,

    /// Test suite file
    #[arg(long)]
    suite_file: Option<std::path::PathBuf>,

    /// Test case filter (comma-separated)
    #[arg(long)]
    filter: Option<String>,

    /// Test tags to include
    #[arg(long)]
    tags: Option<String>,

    /// Output format (json, yaml, text)
    #[arg(long, default_value = "text")]
    format: String,

    /// Output file for results
    #[arg(long)]
    output: Option<std::path::PathBuf>,

    /// Run tests in parallel
    #[arg(long)]
    parallel: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
pub struct ListArgs {
    /// Show detailed information
    #[arg(short, long)]
    detailed: bool,
}

#[derive(Args)]
pub struct InfoArgs {
    /// Test suite name or file
    suite: String,
}

#[derive(Args)]
pub struct ReportArgs {
    /// Test results file
    results_file: std::path::PathBuf,

    /// Output format (html, json, yaml)
    #[arg(long, default_value = "html")]
    format: String,

    /// Output file
    #[arg(long)]
    output: Option<std::path::PathBuf>,
}

/// Create the test command
pub fn command() -> clap::Command {
    clap::Command::new("test")
        .about("Run plugin tests")
        .subcommand_negates_reqs(true)
        .subcommand(RunArgs::augment_args(
            clap::Command::new("run").about("Run plugin tests")
        ))
        .subcommand(ListArgs::augment_args(
            clap::Command::new("list").about("List available test suites")
        ))
        .subcommand(InfoArgs::augment_args(
            clap::Command::new("info").about("Show test suite information")
        ))
        .subcommand(ReportArgs::augment_args(
            clap::Command::new("report").about("Generate test report")
        ))
}

/// Run test command
pub async fn run(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let args = RunArgs::from_arg_matches(sub_matches)?;
            run_tests(args).await
        }
        Some(("list", sub_matches)) => {
            let args = ListArgs::from_arg_matches(sub_matches)?;
            list_test_suites(args).await
        }
        Some(("info", sub_matches)) => {
            let args = InfoArgs::from_arg_matches(sub_matches)?;
            show_test_suite_info(args).await
        }
        Some(("report", sub_matches)) => {
            let args = ReportArgs::from_arg_matches(sub_matches)?;
            generate_test_report(args).await
        }
        _ => {
            let _ = command().print_help();
            Ok(())
        }
    }
}

async fn run_tests(args: RunArgs) -> Result<()> {
    println!("Running plugin tests...");

    // Load test suite
    let test_suite = if let Some(suite_file) = args.suite_file {
        load_test_suite_from_file(&suite_file).await?
    } else if let Some(plugin_id) = args.plugin_id {
        load_default_test_suite(&plugin_id).await?
    } else {
        return Err(anyhow::anyhow!(
            "Must specify either --suite-file or --plugin-id"
        ));
    };

    // Filter test cases
    let filtered_test_cases = filter_test_cases(&test_suite.test_cases, &args.filter, &args.tags)?;
    let mut filtered_suite = test_suite.clone();
    filtered_suite.test_cases = filtered_test_cases;

    // Create test runner
    let mut runner = PluginTestRunner::new(filtered_suite)?;

    // Run tests
    let summary = runner.run_all_tests().await?;

    // Output results
    output_test_results(&summary, &args.format, &args.output, args.verbose).await?;

    // Exit with appropriate code
    if summary.failed_tests > 0 {
        std::process::exit(1);
    }

    Ok(())
}

async fn list_test_suites(args: ListArgs) -> Result<()> {
    println!("Available test suites:");
    println!();

    // Discover test suites from multiple sources
    let mut test_suites = Vec::new();

    // 1. Built-in test suites
    let builtin_suites = discover_builtin_test_suites().await?;
    test_suites.extend(builtin_suites);

    // 2. Test suite files in current directory and subdirectories
    let file_suites = discover_test_suite_files().await?;
    test_suites.extend(file_suites);

    // 3. Test suites from plugin directories
    let plugin_suites = discover_plugin_test_suites().await?;
    test_suites.extend(plugin_suites);

    if test_suites.is_empty() {
        println!("No test suites found.");
        println!("To create a test suite, create a YAML file with the following structure:");
        println!("  name: my-test-suite");
        println!("  description: Description of the test suite");
        println!("  plugin_config:");
        println!("    plugin_id: my-plugin");
        println!("    config: {{}}");
        println!("    enabled_capabilities: [Parse, SchemaExtraction]");
        println!("  test_cases:");
        println!("    - name: test_case_1");
        println!("      description: Description of test case");
        println!("      type: Initialization");
        println!("      input_files: []");
        println!("      expected_result: true");
        return Ok(());
    }

    for suite in test_suites {
        println!("  {}", suite.name);
        if args.detailed {
            println!("    Description: {}", suite.description);
            println!("    Type: {}", suite.source_type);
            println!("    Test Cases: {}", suite.test_cases_count);
            if let Some(path) = suite.file_path {
                println!("    File: {}", path.display());
            }
            println!();
        }
    }

    Ok(())
}

async fn show_test_suite_info(args: InfoArgs) -> Result<()> {
    println!("Test Suite Information:");
    println!("  Name: {}", args.suite);
    println!();

    // Try to load the test suite from different sources
    let test_suite = if std::path::Path::new(&args.suite).exists() {
        // Load from file
        load_test_suite_from_file(std::path::Path::new(&args.suite)).await?
    } else {
        // Try to load as a built-in test suite
        load_default_test_suite(&args.suite).await?
    };

    println!("  Description: {}", test_suite.description);
    println!("  Test Cases: {}", test_suite.test_cases.len());
    println!("  Plugin ID: {}", test_suite.plugin_config.plugin_id);
    println!("  Enabled Capabilities: {:?}", test_suite.plugin_config.enabled_capabilities);
    println!();

    if !test_suite.test_cases.is_empty() {
        println!("  Test Cases:");
        for (i, test_case) in test_suite.test_cases.iter().enumerate() {
            println!("    {}. {}", i + 1, test_case.name);
            println!("       Description: {}", test_case.description);
            println!("       Type: {:?}", test_case.test_type);
            println!("       Expected Success: {}", test_case.expected.success);
            if !test_case.tags.is_empty() {
                println!("       Tags: {}", test_case.tags.join(", "));
            }
            if !test_case.input.files.is_empty() {
                println!("       Input Files: {}", test_case.input.files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "));
            }
            println!();
        }
    }

    Ok(())
}

async fn generate_test_report(args: ReportArgs) -> Result<()> {
    println!("Generating test report...");

    // Load test results from file
    let content = tokio::fs::read_to_string(&args.results_file).await?;
    let test_results: TestRunSummary = match args.results_file.extension().and_then(|s| s.to_str()) {
        Some("json") => serde_json::from_str(&content)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
        _ => return Err(anyhow::anyhow!("Unsupported results file format")),
    };

    // Generate report in specified format
    let report_content = match args.format.as_str() {
        "html" => generate_html_report(&test_results)?,
        "json" => serde_json::to_string_pretty(&test_results)?,
        "yaml" => serde_yaml::to_string(&test_results)?,
        _ => return Err(anyhow::anyhow!("Unsupported report format: {}", args.format)),
    };

    // Write report to output file or stdout
    if let Some(output_path) = args.output {
        tokio::fs::write(output_path, report_content).await?;
        println!("Report generated successfully!");
    } else {
        println!("{}", report_content);
    }

    Ok(())
}

/// Load test suite from file
async fn load_test_suite_from_file(file_path: &std::path::Path) -> Result<PluginTestSuite> {
    let content = tokio::fs::read_to_string(file_path).await?;
    let test_suite: PluginTestSuite = serde_yaml::from_str(&content)?;
    Ok(test_suite)
}

/// Load default test suite for plugin
async fn load_default_test_suite(plugin_id: &str) -> Result<PluginTestSuite> {
    let plugin_config = crate::plugin::PluginConfig {
        plugin_id: plugin_id.to_string(),
        config: serde_yaml::Value::Null,
        enabled_capabilities: vec![
            crate::plugin::PluginCapability::Parse,
            crate::plugin::PluginCapability::SchemaExtraction,
        ],
    };

    let test_cases = match plugin_id {
        "go-ast:builtin" => create_go_ast_test_cases(),
        "crd:builtin" => create_crd_test_cases(),
        "openapi:builtin" => create_openapi_test_cases(),
        _ => create_generic_test_cases(plugin_id),
    };

    Ok(utils::create_test_suite(
        &format!("{plugin_id}-tests"),
        &format!("Default test suite for {plugin_id}"),
        plugin_config,
        test_cases,
    ))
}

/// Create test cases for Go AST plugin
fn create_go_ast_test_cases() -> Vec<PluginTestCase> {
    vec![
        utils::create_test_case(
            "basic_initialization",
            "Test basic Go AST plugin initialization",
            TestCaseType::Initialization,
            vec![],
            true,
        ),
        utils::create_test_case(
            "go_file_parsing",
            "Test parsing of Go source files",
            TestCaseType::SourceProcessing,
            vec![std::path::PathBuf::from("test.go")],
            true,
        ),
        utils::create_test_case(
            "struct_extraction",
            "Test extraction of Go structs",
            TestCaseType::SchemaExtraction,
            vec![std::path::PathBuf::from("types.go")],
            true,
        ),
        utils::create_test_case(
            "interface_processing",
            "Test processing of Go interfaces",
            TestCaseType::SchemaExtraction,
            vec![std::path::PathBuf::from("interfaces.go")],
            true,
        ),
    ]
}

/// Create test cases for CRD plugin
fn create_crd_test_cases() -> Vec<PluginTestCase> {
    vec![
        utils::create_test_case(
            "basic_initialization",
            "Test basic CRD plugin initialization",
            TestCaseType::Initialization,
            vec![],
            true,
        ),
        utils::create_test_case(
            "crd_yaml_parsing",
            "Test parsing of CRD YAML files",
            TestCaseType::SourceProcessing,
            vec![std::path::PathBuf::from("crd.yaml")],
            true,
        ),
        utils::create_test_case(
            "schema_validation",
            "Test CRD schema validation",
            TestCaseType::Validation,
            vec![std::path::PathBuf::from("crd.yaml")],
            true,
        ),
        utils::create_test_case(
            "version_handling",
            "Test handling of multiple CRD versions",
            TestCaseType::SchemaExtraction,
            vec![std::path::PathBuf::from("crd.yaml")],
            true,
        ),
    ]
}

/// Create test cases for OpenAPI plugin
fn create_openapi_test_cases() -> Vec<PluginTestCase> {
    vec![
        utils::create_test_case(
            "basic_initialization",
            "Test basic OpenAPI plugin initialization",
            TestCaseType::Initialization,
            vec![],
            true,
        ),
        utils::create_test_case(
            "openapi_json_parsing",
            "Test parsing of OpenAPI JSON files",
            TestCaseType::SourceProcessing,
            vec![std::path::PathBuf::from("openapi.json")],
            true,
        ),
        utils::create_test_case(
            "openapi_yaml_parsing",
            "Test parsing of OpenAPI YAML files",
            TestCaseType::SourceProcessing,
            vec![std::path::PathBuf::from("openapi.yaml")],
            true,
        ),
        utils::create_test_case(
            "schema_extraction",
            "Test extraction of OpenAPI schemas",
            TestCaseType::SchemaExtraction,
            vec![std::path::PathBuf::from("openapi.yaml")],
            true,
        ),
    ]
}

/// Create generic test cases for unknown plugins
fn create_generic_test_cases(plugin_id: &str) -> Vec<PluginTestCase> {
    vec![
        utils::create_test_case(
            "basic_initialization",
            &format!("Test basic {} plugin initialization", plugin_id),
            TestCaseType::Initialization,
            vec![],
            true,
        ),
        utils::create_test_case(
            "source_processing",
            &format!("Test {} source processing", plugin_id),
            TestCaseType::SourceProcessing,
            vec![std::path::PathBuf::from("test.txt")],
            true,
        ),
        utils::create_test_case(
            "schema_extraction",
            &format!("Test {} schema extraction", plugin_id),
            TestCaseType::SchemaExtraction,
            vec![std::path::PathBuf::from("test.txt")],
            true,
        ),
    ]
}

/// Filter test cases based on filter and tags
fn filter_test_cases(
    test_cases: &[PluginTestCase],
    filter: &Option<String>,
    tags: &Option<String>,
) -> Result<Vec<PluginTestCase>> {
    let mut filtered = test_cases.to_vec();

    // Apply name filter
    if let Some(filter_str) = filter {
        let filters: Vec<&str> = filter_str.split(',').collect();
        filtered.retain(|tc| filters.iter().any(|f| tc.name.contains(f)));
    }

    // Apply tag filter
    if let Some(tags_str) = tags {
        let required_tags: Vec<&str> = tags_str.split(',').collect();
        filtered.retain(|tc| {
            required_tags
                .iter()
                .all(|tag| tc.tags.contains(&tag.to_string()))
        });
    }

    Ok(filtered)
}

/// Output test results in specified format
async fn output_test_results(
    summary: &TestRunSummary,
    format: &str,
    output_file: &Option<std::path::PathBuf>,
    verbose: bool,
) -> Result<()> {
    let output = match format {
        "json" => serde_json::to_string_pretty(summary)?,
        "yaml" => serde_yaml::to_string(summary)?,
        "text" => format_test_results_text(summary, verbose),
        _ => return Err(anyhow::anyhow!("Unsupported format: {}", format)),
    };

    if let Some(file_path) = output_file {
        tokio::fs::write(file_path, output).await?;
    } else {
        println!("{output}");
    }

    Ok(())
}

/// Format test results as text
fn format_test_results_text(summary: &TestRunSummary, verbose: bool) -> String {
    let mut output = String::new();

    output.push_str(&format!("Test Suite: {}\n", summary.test_suite_name));
    output.push_str(&format!("Total Tests: {}\n", summary.total_tests));
    output.push_str(&format!("Passed: {}\n", summary.passed_tests));
    output.push_str(&format!("Failed: {}\n", summary.failed_tests));
    output.push_str(&format!("Total Time: {}ms\n", summary.total_time_ms));
    output.push('\n');

    if verbose {
        for result in &summary.results {
            let status = if result.passed { "PASS" } else { "FAIL" };
            output.push_str(&format!(
                "{} {} ({}ms)\n",
                status, result.test_name, result.execution_time_ms
            ));

            if !result.passed {
                if let Some(error) = &result.error {
                    output.push_str(&format!("  Error: {error}\n"));
                }
            }
        }
    }

    output
}

/// Discover built-in test suites
async fn discover_builtin_test_suites() -> Result<Vec<TestSuiteInfo>> {
    let mut suites = Vec::new();

    // Built-in plugin test suites
    let builtin_plugins = vec!["go-ast:builtin", "crd:builtin", "openapi:builtin"];

    for plugin_id in builtin_plugins {
        match load_default_test_suite(plugin_id).await {
            Ok(test_suite) => {
                suites.push(TestSuiteInfo {
                    name: test_suite.name.clone(),
                    description: test_suite.description.clone(),
                    source_type: "Built-in".to_string(),
                    file_path: None,
                    test_cases_count: test_suite.test_cases.len(),
                });
            }
            Err(_) => {
                // Skip if default test suite cannot be loaded
                continue;
            }
        }
    }

    Ok(suites)
}

/// Discover test suite files in current directory and subdirectories
async fn discover_test_suite_files() -> Result<Vec<TestSuiteInfo>> {
    let mut suites = Vec::new();

    // Look for test suite files with common patterns
    let patterns = vec!["**/*test*.yaml", "**/*test*.yml", "**/*suite*.yaml", "**/*suite*.yml"];

    for pattern in patterns {
        if let Ok(entries) = glob::glob(pattern) {
            for entry in entries {
                if let Ok(path) = entry {
                    if let Ok(test_suite) = load_test_suite_from_file(&path).await {
                        suites.push(TestSuiteInfo {
                            name: test_suite.name.clone(),
                            description: test_suite.description.clone(),
                            source_type: "File".to_string(),
                            file_path: Some(path),
                            test_cases_count: test_suite.test_cases.len(),
                        });
                    }
                }
            }
        }
    }

    Ok(suites)
}

/// Discover test suites from plugin directories
async fn discover_plugin_test_suites() -> Result<Vec<TestSuiteInfo>> {
    let mut suites = Vec::new();

    // Look in common plugin directories
    let plugin_dirs = vec![
        std::path::PathBuf::from("./plugins"),
        std::path::PathBuf::from("~/.config/gensonnet/plugins"),
    ];

    for plugin_dir in plugin_dirs {
        if plugin_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(plugin_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let test_suite_path = entry.path().join("test-suite.yaml");
                        if test_suite_path.exists() {
                            if let Ok(test_suite) = load_test_suite_from_file(&test_suite_path).await {
                                suites.push(TestSuiteInfo {
                                    name: test_suite.name.clone(),
                                    description: test_suite.description.clone(),
                                    source_type: "Plugin".to_string(),
                                    file_path: Some(test_suite_path),
                                    test_cases_count: test_suite.test_cases.len(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(suites)
}

/// Test suite information for discovery
#[derive(Debug, Clone)]
struct TestSuiteInfo {
    name: String,
    description: String,
    source_type: String,
    file_path: Option<std::path::PathBuf>,
    test_cases_count: usize,
}

/// Generate HTML report from test results
fn generate_html_report(summary: &TestRunSummary) -> Result<String> {
    let mut html = String::new();

    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n");
    html.push_str("<head>\n");
    html.push_str("    <title>Test Report - ");
    html.push_str(&summary.test_suite_name);
    html.push_str("</title>\n");
    html.push_str("    <style>\n");
    html.push_str("        body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }\n");
    html.push_str("        .summary { margin: 20px 0; }\n");
    html.push_str("        .summary-item { display: inline-block; margin: 10px; padding: 10px; border-radius: 5px; }\n");
    html.push_str("        .passed { background-color: #d4edda; color: #155724; }\n");
    html.push_str("        .failed { background-color: #f8d7da; color: #721c24; }\n");
    html.push_str("        .total { background-color: #d1ecf1; color: #0c5460; }\n");
    html.push_str("        .test-result { margin: 10px 0; padding: 10px; border-radius: 5px; }\n");
    html.push_str("        .test-pass { background-color: #d4edda; border-left: 5px solid #28a745; }\n");
    html.push_str("        .test-fail { background-color: #f8d7da; border-left: 5px solid #dc3545; }\n");
    html.push_str("        .error { color: #721c24; font-family: monospace; }\n");
    html.push_str("    </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");

    // Header
    html.push_str("    <div class=\"header\">\n");
    html.push_str("        <h1>Test Report</h1>\n");
    html.push_str("        <h2>");
    html.push_str(&summary.test_suite_name);
    html.push_str("</h2>\n");
    html.push_str("        <p>Generated on: ");
    html.push_str(&chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
    html.push_str("</p>\n");
    html.push_str("    </div>\n");

    // Summary
    html.push_str("    <div class=\"summary\">\n");
    html.push_str("        <h3>Summary</h3>\n");
    html.push_str("        <div class=\"summary-item total\">\n");
    html.push_str("            <strong>Total Tests:</strong> ");
    html.push_str(&summary.total_tests.to_string());
    html.push_str("        </div>\n");
    html.push_str("        <div class=\"summary-item passed\">\n");
    html.push_str("            <strong>Passed:</strong> ");
    html.push_str(&summary.passed_tests.to_string());
    html.push_str("        </div>\n");
    html.push_str("        <div class=\"summary-item failed\">\n");
    html.push_str("            <strong>Failed:</strong> ");
    html.push_str(&summary.failed_tests.to_string());
    html.push_str("        </div>\n");
    html.push_str("        <div class=\"summary-item total\">\n");
    html.push_str("            <strong>Total Time:</strong> ");
    html.push_str(&summary.total_time_ms.to_string());
    html.push_str("ms\n");
    html.push_str("        </div>\n");
    html.push_str("    </div>\n");

    // Test Results
    html.push_str("    <div class=\"test-results\">\n");
    html.push_str("        <h3>Test Results</h3>\n");

    for result in &summary.results {
        let css_class = if result.passed { "test-pass" } else { "test-fail" };
        html.push_str("        <div class=\"test-result ");
        html.push_str(css_class);
        html.push_str("\">\n");
        html.push_str("            <h4>");
        html.push_str(&result.test_name);
        html.push_str("</h4>\n");
        html.push_str("            <p><strong>Status:</strong> ");
        html.push_str(if result.passed { "PASS" } else { "FAIL" });
        html.push_str("</p>\n");
        html.push_str("            <p><strong>Execution Time:</strong> ");
        html.push_str(&result.execution_time_ms.to_string());
        html.push_str("ms</p>\n");

        if !result.passed {
            if let Some(error) = &result.error {
                html.push_str("            <div class=\"error\">\n");
                html.push_str("                <strong>Error:</strong><br>\n");
                html.push_str("                <pre>");
                html.push_str(&html_escape::encode_text(error));
                html.push_str("</pre>\n");
                html.push_str("            </div>\n");
            }
        }

        html.push_str("        </div>\n");
    }

    html.push_str("    </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    Ok(html)
}
