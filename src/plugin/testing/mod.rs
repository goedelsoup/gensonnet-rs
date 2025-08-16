//! Plugin testing framework for standardized plugin testing

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use crate::plugin::*;

/// Plugin test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTestSuite {
    /// Test suite name
    pub name: String,

    /// Test suite description
    pub description: String,

    /// Plugin configuration for testing
    pub plugin_config: PluginConfig,

    /// Test cases
    pub test_cases: Vec<PluginTestCase>,

    /// Test environment setup
    pub setup: Option<TestSetup>,

    /// Test environment cleanup
    pub cleanup: Option<TestCleanup>,
}

/// Individual test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTestCase {
    /// Test case name
    pub name: String,

    /// Test case description
    pub description: String,

    /// Test case type
    pub test_type: TestCaseType,

    /// Input data for the test
    pub input: TestInput,

    /// Expected output
    pub expected: TestExpected,

    /// Test case timeout in seconds
    pub timeout_seconds: Option<u64>,

    /// Whether this test case is required
    pub required: bool,

    /// Test case tags for filtering
    pub tags: Vec<String>,
}

/// Test case types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestCaseType {
    /// Test plugin initialization
    Initialization,

    /// Test source processing
    SourceProcessing,

    /// Test schema extraction
    SchemaExtraction,

    /// Test code generation
    CodeGeneration,

    /// Test validation
    Validation,

    /// Test error handling
    ErrorHandling,

    /// Test performance
    Performance,

    /// Test integration
    Integration,

    /// Custom test type
    Custom(String),
}

/// Test input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    /// Input file paths
    pub files: Vec<PathBuf>,

    /// Input content (if not using files)
    pub content: Option<String>,

    /// Input configuration
    pub config: serde_yaml::Value,

    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

/// Expected test output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExpected {
    /// Expected success
    pub success: bool,

    /// Expected output files
    pub output_files: Vec<PathBuf>,

    /// Expected output content patterns
    pub content_patterns: Vec<String>,

    /// Expected error patterns (if success is false)
    pub error_patterns: Vec<String>,

    /// Expected performance metrics
    pub performance: Option<PerformanceExpectations>,

    /// Expected schemas
    pub schemas: Option<SchemaExpectations>,
}

/// Performance expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceExpectations {
    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: u64,

    /// Maximum memory usage in bytes
    pub max_memory_usage_bytes: usize,

    /// Maximum file size in bytes
    pub max_output_size_bytes: usize,
}

/// Schema expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaExpectations {
    /// Expected number of schemas
    pub schema_count: usize,

    /// Expected schema types
    pub schema_types: Vec<String>,

    /// Expected schema properties
    pub schema_properties: HashMap<String, serde_yaml::Value>,
}

/// Test setup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSetup {
    /// Setup script or commands
    pub commands: Vec<String>,

    /// Files to create
    pub files: HashMap<PathBuf, String>,

    /// Environment setup
    pub environment: HashMap<String, String>,
}

/// Test cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCleanup {
    /// Cleanup commands
    pub commands: Vec<String>,

    /// Files to remove
    pub remove_files: Vec<PathBuf>,

    /// Directories to remove
    pub remove_directories: Vec<PathBuf>,
}

/// Test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTestResult {
    /// Test case name
    pub test_name: String,

    /// Whether the test passed
    pub passed: bool,

    /// Test execution time in milliseconds
    pub execution_time_ms: u64,

    /// Test output
    pub output: TestOutput,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Test output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutput {
    /// Generated files
    pub files: Vec<PathBuf>,

    /// Generated content
    pub content: String,

    /// Extracted schemas
    pub schemas: Vec<ExtractedSchema>,

    /// Warnings
    pub warnings: Vec<String>,

    /// Errors
    pub errors: Vec<String>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Memory usage in bytes
    pub memory_usage_bytes: usize,

    /// Output size in bytes
    pub output_size_bytes: usize,
}

/// Plugin test runner
pub struct PluginTestRunner {
    /// Test suite configuration
    test_suite: PluginTestSuite,

    /// Temporary directory for test execution
    temp_dir: TempDir,

    /// Test results
    results: Vec<PluginTestResult>,
}

impl PluginTestRunner {
    /// Create a new plugin test runner
    pub fn new(test_suite: PluginTestSuite) -> Result<Self> {
        let temp_dir = TempDir::new()?;

        Ok(Self {
            test_suite,
            temp_dir,
            results: Vec::new(),
        })
    }

    /// Run all test cases
    pub async fn run_all_tests(&mut self) -> Result<TestRunSummary> {
        let start_time = std::time::Instant::now();

        // Setup test environment
        self.setup_test_environment().await?;

        // Run each test case
        for test_case in &self.test_suite.test_cases {
            let result = self.run_test_case(test_case).await;
            self.results.push(result);
        }

        // Cleanup test environment
        self.cleanup_test_environment().await?;

        let total_time = start_time.elapsed();

        Ok(TestRunSummary {
            test_suite_name: self.test_suite.name.clone(),
            total_tests: self.results.len(),
            passed_tests: self.results.iter().filter(|r| r.passed).count(),
            failed_tests: self.results.iter().filter(|r| !r.passed).count(),
            total_time_ms: total_time.as_millis() as u64,
            results: self.results.clone(),
        })
    }

    /// Run a single test case
    async fn run_test_case(&self, test_case: &PluginTestCase) -> PluginTestResult {
        let start_time = std::time::Instant::now();
        let initial_memory = self.get_memory_usage();

        match self.execute_test_case(test_case).await {
            Ok(output) => {
                let execution_time = start_time.elapsed();
                let final_memory = self.get_memory_usage();
                let memory_usage = final_memory.saturating_sub(initial_memory);
                let output_size = self.calculate_output_size(&output);
                let passed = self.validate_test_output(test_case, &output);

                PluginTestResult {
                    test_name: test_case.name.clone(),
                    passed,
                    execution_time_ms: execution_time.as_millis() as u64,
                    output,
                    error: None,
                    performance: PerformanceMetrics {
                        processing_time_ms: execution_time.as_millis() as u64,
                        memory_usage_bytes: memory_usage,
                        output_size_bytes: output_size,
                    },
                }
            }
            Err(e) => {
                let execution_time = start_time.elapsed();
                let final_memory = self.get_memory_usage();
                let memory_usage = final_memory.saturating_sub(initial_memory);

                PluginTestResult {
                    test_name: test_case.name.clone(),
                    passed: false,
                    execution_time_ms: execution_time.as_millis() as u64,
                    output: TestOutput {
                        files: Vec::new(),
                        content: String::new(),
                        schemas: Vec::new(),
                        warnings: Vec::new(),
                        errors: vec![e.to_string()],
                    },
                    error: Some(e.to_string()),
                    performance: PerformanceMetrics {
                        processing_time_ms: execution_time.as_millis() as u64,
                        memory_usage_bytes: memory_usage,
                        output_size_bytes: 0,
                    },
                }
            }
        }
    }

    /// Execute a test case
    async fn execute_test_case(&self, test_case: &PluginTestCase) -> Result<TestOutput> {
        // Create plugin context
        let context = PluginContext::new(
            self.temp_dir.path().to_path_buf(),
            self.temp_dir.path().join("output"),
            self.test_suite.plugin_config.clone(),
        );

        // Create plugin manager and register the plugin
        let plugin_manager = Arc::new(PluginManager::new());

        // Execute based on test type
        match &test_case.test_type {
            TestCaseType::Initialization => {
                self.test_initialization(&context, &plugin_manager).await
            }
            TestCaseType::SourceProcessing => {
                self.test_source_processing(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::SchemaExtraction => {
                self.test_schema_extraction(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::CodeGeneration => {
                self.test_code_generation(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::Validation => {
                self.test_validation(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::ErrorHandling => {
                self.test_error_handling(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::Performance => {
                self.test_performance(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::Integration => {
                self.test_integration(test_case, &context, &plugin_manager)
                    .await
            }
            TestCaseType::Custom(custom_type) => {
                self.test_custom(test_case, custom_type, &context, &plugin_manager)
                    .await
            }
        }
    }

    /// Test plugin initialization
    async fn test_initialization(
        &self,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Try to get the plugin from the manager
        if let Some(plugin) = plugin_manager.get_plugin(&context.config.plugin_id).await {
            // Test plugin initialization
            match plugin.initialize(context).await {
                Ok(()) => {
                    // Check if plugin can handle any test files
                    let test_files = vec![
                        PathBuf::from("test.go"),
                        PathBuf::from("test.yaml"),
                        PathBuf::from("test.json"),
                    ];

                    let mut can_handle_results = Vec::new();
                    for test_file in test_files {
                        match plugin.can_handle(&test_file).await {
                            Ok(can_handle) => {
                                can_handle_results.push((test_file, can_handle));
                            }
                            Err(e) => {
                                warnings.push(format!(
                                    "Failed to check if plugin can handle {:?}: {}",
                                    test_file, e
                                ));
                            }
                        }
                    }

                    // Create a summary of initialization results
                    let content = format!(
                        "Plugin '{}' initialized successfully.\nCan handle files: {:?}",
                        context.config.plugin_id, can_handle_results
                    );

                    Ok(TestOutput {
                        files: Vec::new(),
                        content,
                        schemas: Vec::new(),
                        warnings,
                        errors,
                    })
                }
                Err(e) => {
                    errors.push(format!("Failed to initialize plugin: {}", e));
                    Ok(TestOutput {
                        files: Vec::new(),
                        content: String::new(),
                        schemas: Vec::new(),
                        warnings,
                        errors,
                    })
                }
            }
        } else {
            errors.push(format!(
                "Plugin '{}' not found in plugin manager",
                context.config.plugin_id
            ));
            Ok(TestOutput {
                files: Vec::new(),
                content: String::new(),
                schemas: Vec::new(),
                warnings,
                errors,
            })
        }
    }

    /// Test source processing
    async fn test_source_processing(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        // Create test files
        for file_path in &test_case.input.files {
            let full_path = self.temp_dir.path().join(file_path);
            std::fs::create_dir_all(full_path.parent().unwrap())?;

            // Use content from test case if available, otherwise create empty file
            let content = test_case.input.content.as_deref().unwrap_or("");
            std::fs::write(&full_path, content)?;
        }

        // Process each input file
        let mut all_schemas = Vec::new();
        let mut all_files = Vec::new();
        let mut all_warnings = Vec::new();
        let mut all_errors = Vec::new();

        for file_path in &test_case.input.files {
            let full_path = self.temp_dir.path().join(file_path);

            match plugin_manager.process_source(&full_path, context).await {
                Ok(result) => {
                    all_schemas.extend(result.schemas);
                    all_files.extend(result.generated_files);
                    all_warnings.extend(result.warnings);
                    all_errors.extend(result.errors);
                }
                Err(e) => {
                    all_errors.push(e.to_string());
                }
            }
        }

        Ok(TestOutput {
            files: all_files,
            content: String::new(),
            schemas: all_schemas,
            warnings: all_warnings,
            errors: all_errors,
        })
    }

    /// Test schema extraction
    async fn test_schema_extraction(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        // Similar to source processing but focus on schema extraction
        self.test_source_processing(test_case, context, plugin_manager)
            .await
    }

    /// Test code generation
    async fn test_code_generation(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        // First extract schemas
        let schemas = if let Ok(output) = self
            .test_schema_extraction(test_case, context, plugin_manager)
            .await
        {
            output.schemas
        } else {
            return Err(anyhow::anyhow!(
                "Failed to extract schemas for code generation"
            ));
        };

        // Generate code from schemas
        let generated_files = plugin_manager.generate_code(&schemas, context).await?;

        Ok(TestOutput {
            files: generated_files,
            content: String::new(),
            schemas,
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }

    /// Test validation
    async fn test_validation(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let warnings = Vec::new();
        let mut errors = Vec::new();

        // First, extract schemas from the test case
        let schemas = if let Ok(output) = self
            .test_schema_extraction(test_case, context, plugin_manager)
            .await
        {
            output.schemas
        } else {
            errors.push("Failed to extract schemas for validation".to_string());
            return Ok(TestOutput {
                files: Vec::new(),
                content: String::new(),
                schemas: Vec::new(),
                warnings,
                errors,
            });
        };

        // Validate each schema
        let mut validation_results = Vec::new();
        for schema in &schemas {
            let validation_result = self.validate_schema(schema, context, plugin_manager).await;
            validation_results.push((schema.name.clone(), validation_result));
        }

        // Generate validation report
        let mut valid_schemas = 0;
        let mut invalid_schemas = 0;
        let mut validation_details = Vec::new();

        for (schema_name, result) in validation_results {
            match result {
                Ok(is_valid) => {
                    if is_valid {
                        valid_schemas += 1;
                        validation_details.push(format!("✓ {}: Valid", schema_name));
                    } else {
                        invalid_schemas += 1;
                        validation_details.push(format!("✗ {}: Invalid", schema_name));
                    }
                }
                Err(e) => {
                    invalid_schemas += 1;
                    validation_details.push(format!("✗ {}: Validation error - {}", schema_name, e));
                    errors.push(format!(
                        "Validation error for schema '{}': {}",
                        schema_name, e
                    ));
                }
            }
        }

        let content = format!(
            "Validation Results:\nValid schemas: {}\nInvalid schemas: {}\n\nDetails:\n{}",
            valid_schemas,
            invalid_schemas,
            validation_details.join("\n")
        );

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas,
            warnings,
            errors,
        })
    }

    /// Validate a single schema
    async fn validate_schema(
        &self,
        schema: &ExtractedSchema,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<bool> {
        // Check if the plugin supports validation capability
        if let Some(plugin) = plugin_manager.get_plugin(&context.config.plugin_id).await {
            let metadata = plugin.metadata();
            if !metadata
                .capabilities
                .contains(&PluginCapability::Validation)
            {
                return Ok(true); // Skip validation if plugin doesn't support it
            }

            // Basic schema validation checks
            let mut is_valid = true;

            // Check if schema has required fields
            if schema.name.is_empty() {
                is_valid = false;
            }

            if schema.schema_type.is_empty() {
                is_valid = false;
            }

            // Check if content is not null/empty
            if schema.content.is_null() {
                is_valid = false;
            }

            // Additional validation could be implemented here based on schema type
            match schema.schema_type.as_str() {
                "openapi" | "swagger" => {
                    // Validate OpenAPI schema structure
                    if let Some(openapi_version) = schema.content.get("openapi") {
                        if !openapi_version.is_string() {
                            is_valid = false;
                        }
                    } else if let Some(swagger_version) = schema.content.get("swagger") {
                        if !swagger_version.is_string() {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                }
                "crd" => {
                    // Validate CRD schema structure
                    if let Some(api_version) = schema.content.get("apiVersion") {
                        if !api_version.is_string() {
                            is_valid = false;
                        }
                    } else {
                        is_valid = false;
                    }
                }
                _ => {
                    // For other schema types, just check basic structure
                    if !schema.content.as_mapping().is_some()
                        && !schema.content.as_sequence().is_some()
                    {
                        is_valid = false;
                    }
                }
            }

            Ok(is_valid)
        } else {
            Err(anyhow::anyhow!("Plugin not found for validation"))
        }
    }

    /// Test error handling
    async fn test_error_handling(
        &self,
        _test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let warnings = Vec::new();
        let mut errors = Vec::new();
        let mut error_handling_results = Vec::new();

        // Test various error scenarios
        let error_scenarios = vec![
            ("invalid_file_path", PathBuf::from("/nonexistent/file.go")),
            ("empty_file", self.temp_dir.path().join("empty.go")),
            (
                "malformed_content",
                self.temp_dir.path().join("malformed.go"),
            ),
            (
                "unsupported_extension",
                self.temp_dir.path().join("test.xyz"),
            ),
        ];

        // Create test files for error scenarios
        std::fs::write(self.temp_dir.path().join("empty.go"), "")?;
        std::fs::write(
            self.temp_dir.path().join("malformed.go"),
            "package main\nfunc main() {\n    // Missing closing brace\n",
        )?;

        for (scenario_name, file_path) in error_scenarios {
            let result = self
                .test_error_scenario(scenario_name, &file_path, context, plugin_manager)
                .await;
            error_handling_results.push((scenario_name.to_string(), result));
        }

        // Analyze error handling results
        let mut handled_errors = 0;
        let mut unhandled_errors = 0;
        let mut error_details = Vec::new();

        for (scenario_name, result) in error_handling_results {
            match result {
                Ok(handled_properly) => {
                    if handled_properly {
                        handled_errors += 1;
                        error_details.push(format!("✓ {}: Error handled properly", scenario_name));
                    } else {
                        unhandled_errors += 1;
                        error_details
                            .push(format!("✗ {}: Error not handled properly", scenario_name));
                        errors.push(format!(
                            "Error scenario '{}' was not handled properly",
                            scenario_name
                        ));
                    }
                }
                Err(e) => {
                    unhandled_errors += 1;
                    error_details.push(format!("✗ {}: Test failed - {}", scenario_name, e));
                    errors.push(format!(
                        "Error handling test failed for '{}': {}",
                        scenario_name, e
                    ));
                }
            }
        }

        let content = format!(
            "Error Handling Test Results:\nProperly handled errors: {}\nUnhandled errors: {}\n\nDetails:\n{}",
            handled_errors,
            unhandled_errors,
            error_details.join("\n")
        );

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas: Vec::new(),
            warnings,
            errors,
        })
    }

    /// Test a specific error scenario
    async fn test_error_scenario(
        &self,
        scenario_name: &str,
        file_path: &PathBuf,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<bool> {
        // Try to process the problematic file
        let result = plugin_manager.process_source(file_path, context).await;

        match result {
            Ok(_) => {
                // If we get here, the error wasn't handled (should have failed)
                Ok(false)
            }
            Err(e) => {
                // Check if the error is appropriate for the scenario
                let error_message = e.to_string().to_lowercase();
                let handled_properly = match scenario_name {
                    "invalid_file_path" => {
                        error_message.contains("not found")
                            || error_message.contains("no such file")
                            || error_message.contains("cannot find")
                    }
                    "empty_file" => {
                        error_message.contains("empty")
                            || error_message.contains("no content")
                            || error_message.contains("invalid")
                    }
                    "malformed_content" => {
                        error_message.contains("syntax")
                            || error_message.contains("parse")
                            || error_message.contains("malformed")
                            || error_message.contains("invalid")
                    }
                    "unsupported_extension" => {
                        error_message.contains("unsupported")
                            || error_message.contains("cannot handle")
                            || error_message.contains("unknown")
                    }
                    _ => true, // Default to true for unknown scenarios
                };

                Ok(handled_properly)
            }
        }
    }

    /// Test performance
    async fn test_performance(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Run multiple iterations to get performance metrics
        let iterations = 5;
        let mut processing_times = Vec::new();
        let mut memory_usages = Vec::new();
        let mut output_sizes = Vec::new();

        for i in 0..iterations {
            let start_time = std::time::Instant::now();
            let initial_memory = self.get_memory_usage();

            let result = match &test_case.test_type {
                TestCaseType::SourceProcessing => {
                    self.test_source_processing(test_case, context, plugin_manager)
                        .await
                }
                TestCaseType::SchemaExtraction => {
                    self.test_schema_extraction(test_case, context, plugin_manager)
                        .await
                }
                TestCaseType::CodeGeneration => {
                    self.test_code_generation(test_case, context, plugin_manager)
                        .await
                }
                _ => {
                    // For other test types, use source processing as default
                    self.test_source_processing(test_case, context, plugin_manager)
                        .await
                }
            };

            let processing_time = start_time.elapsed();
            let final_memory = self.get_memory_usage();
            let memory_usage = final_memory.saturating_sub(initial_memory);

            match result {
                Ok(output) => {
                    let output_size = self.calculate_output_size(&output);

                    processing_times.push(processing_time.as_millis() as u64);
                    memory_usages.push(memory_usage);
                    output_sizes.push(output_size);
                }
                Err(e) => {
                    errors.push(format!(
                        "Performance test iteration {} failed: {}",
                        i + 1,
                        e
                    ));
                }
            }
        }

        // Calculate performance statistics
        let avg_processing_time = if !processing_times.is_empty() {
            processing_times.iter().sum::<u64>() / processing_times.len() as u64
        } else {
            0
        };

        let avg_memory_usage = if !memory_usages.is_empty() {
            memory_usages.iter().sum::<usize>() / memory_usages.len()
        } else {
            0
        };

        let avg_output_size = if !output_sizes.is_empty() {
            output_sizes.iter().sum::<usize>() / output_sizes.len()
        } else {
            0
        };

        let min_processing_time = processing_times.iter().min().copied().unwrap_or(0);
        let max_processing_time = processing_times.iter().max().copied().unwrap_or(0);

        // Check against performance expectations if provided
        let mut performance_issues = Vec::new();
        if let Some(expectations) = &test_case.expected.performance {
            if avg_processing_time > expectations.max_processing_time_ms {
                performance_issues.push(format!(
                    "Average processing time ({}ms) exceeds maximum ({}ms)",
                    avg_processing_time, expectations.max_processing_time_ms
                ));
            }

            if avg_memory_usage > expectations.max_memory_usage_bytes {
                performance_issues.push(format!(
                    "Average memory usage ({} bytes) exceeds maximum ({} bytes)",
                    avg_memory_usage, expectations.max_memory_usage_bytes
                ));
            }

            if avg_output_size > expectations.max_output_size_bytes {
                performance_issues.push(format!(
                    "Average output size ({} bytes) exceeds maximum ({} bytes)",
                    avg_output_size, expectations.max_output_size_bytes
                ));
            }
        }

        // Generate performance report
        let content = format!(
            "Performance Test Results ({} iterations):\n\n\
            Processing Time:\n\
            - Average: {}ms\n\
            - Min: {}ms\n\
            - Max: {}ms\n\n\
            Memory Usage:\n\
            - Average: {} bytes\n\
            - Min: {} bytes\n\
            - Max: {} bytes\n\n\
            Output Size:\n\
            - Average: {} bytes\n\
            - Min: {} bytes\n\
            - Max: {} bytes\n\n\
            Performance Issues:\n{}",
            iterations,
            avg_processing_time,
            min_processing_time,
            max_processing_time,
            avg_memory_usage,
            memory_usages.iter().min().copied().unwrap_or(0),
            memory_usages.iter().max().copied().unwrap_or(0),
            avg_output_size,
            output_sizes.iter().min().copied().unwrap_or(0),
            output_sizes.iter().max().copied().unwrap_or(0),
            if performance_issues.is_empty() {
                "None".to_string()
            } else {
                performance_issues.join("\n")
            }
        );

        // Add performance issues as warnings
        warnings.extend(performance_issues);

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas: Vec::new(),
            warnings,
            errors,
        })
    }

    /// Test integration
    async fn test_integration(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut integration_results = Vec::new();

        // Test the full integration workflow: initialization -> source processing -> schema extraction -> code generation
        let workflow_steps = vec![
            (
                "initialization",
                self.test_initialization(context, plugin_manager).await,
            ),
            (
                "source_processing",
                self.test_source_processing(test_case, context, plugin_manager)
                    .await,
            ),
            (
                "schema_extraction",
                self.test_schema_extraction(test_case, context, plugin_manager)
                    .await,
            ),
            (
                "code_generation",
                self.test_code_generation(test_case, context, plugin_manager)
                    .await,
            ),
        ];

        let mut all_schemas = Vec::new();
        let mut all_files = Vec::new();
        let mut step_results = Vec::new();

        for (step_name, step_result) in workflow_steps {
            match step_result {
                Ok(output) => {
                    step_results.push(format!("✓ {}: Success", step_name));

                    // Collect schemas and files from each step
                    all_schemas.extend(output.schemas.clone());
                    all_files.extend(output.files.clone());

                    // Add warnings and errors
                    warnings.extend(output.warnings.clone());
                    errors.extend(output.errors.clone());

                    integration_results.push((step_name.to_string(), Ok(output)));
                }
                Err(e) => {
                    step_results.push(format!("✗ {}: Failed - {}", step_name, e));
                    errors.push(format!("Integration step '{}' failed: {}", step_name, e));
                    integration_results.push((step_name.to_string(), Err(e)));
                }
            }
        }

        // Test plugin cleanup
        let cleanup_result = plugin_manager.cleanup(context).await;
        match cleanup_result {
            Ok(()) => {
                step_results.push("✓ cleanup: Success".to_string());
            }
            Err(e) => {
                step_results.push(format!("✗ cleanup: Failed - {}", e));
                errors.push(format!("Integration cleanup failed: {}", e));
            }
        }

        // Test shared state functionality
        let shared_state_test = self.test_shared_state_integration(context).await;
        match shared_state_test {
            Ok(()) => {
                step_results.push("✓ shared_state: Success".to_string());
            }
            Err(e) => {
                step_results.push(format!("✗ shared_state: Failed - {}", e));
                errors.push(format!("Shared state integration test failed: {}", e));
            }
        }

        // Generate integration report
        let content = format!(
            "Integration Test Results:\n\n\
            Workflow Steps:\n{}\n\n\
            Total Schemas Extracted: {}\n\
            Total Files Generated: {}\n\
            Total Warnings: {}\n\
            Total Errors: {}\n\n\
            Integration Status: {}",
            step_results.join("\n"),
            all_schemas.len(),
            all_files.len(),
            warnings.len(),
            errors.len(),
            if errors.is_empty() {
                "PASSED"
            } else {
                "FAILED"
            }
        );

        Ok(TestOutput {
            files: all_files,
            content,
            schemas: all_schemas,
            warnings,
            errors,
        })
    }

    /// Test shared state integration
    async fn test_shared_state_integration(&self, context: &PluginContext) -> Result<()> {
        // Test setting and getting shared state values
        let test_key = "integration_test_key";
        let test_value = serde_yaml::Value::String("integration_test_value".to_string());

        // Set a value
        context
            .set_shared_value(test_key.to_string(), test_value.clone())
            .await;

        // Get the value back
        let retrieved_value = context.get_shared_value(test_key).await;

        match retrieved_value {
            Some(value) => {
                if value == test_value {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Retrieved value doesn't match original value"
                    ))
                }
            }
            None => Err(anyhow::anyhow!("Failed to retrieve shared state value")),
        }
    }

    /// Test custom test type
    async fn test_custom(
        &self,
        test_case: &PluginTestCase,
        custom_type: &str,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Handle different custom test types
        let result = match custom_type.to_lowercase().as_str() {
            "concurrent" => {
                self.test_concurrent_execution(test_case, context, plugin_manager)
                    .await
            }
            "stress" => {
                self.test_stress_execution(test_case, context, plugin_manager)
                    .await
            }
            "memory_leak" => {
                self.test_memory_leak_detection(test_case, context, plugin_manager)
                    .await
            }
            "compatibility" => {
                self.test_compatibility(test_case, context, plugin_manager)
                    .await
            }
            "security" => {
                self.test_security_checks(test_case, context, plugin_manager)
                    .await
            }
            _ => {
                // For unknown custom types, try to execute as a combination of existing tests
                warnings.push(format!(
                    "Unknown custom test type '{}', running basic tests",
                    custom_type
                ));
                self.test_basic_custom_execution(test_case, context, plugin_manager)
                    .await
            }
        };

        match result {
            Ok(mut output) => {
                output.warnings.extend(warnings);
                output.errors.extend(errors);
                Ok(output)
            }
            Err(e) => {
                errors.push(format!("Custom test '{}' failed: {}", custom_type, e));
                Ok(TestOutput {
                    files: Vec::new(),
                    content: format!("Custom test '{}' failed: {}", custom_type, e),
                    schemas: Vec::new(),
                    warnings,
                    errors,
                })
            }
        }
    }

    /// Test concurrent execution
    async fn test_concurrent_execution(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut all_schemas = Vec::new();
        let mut all_files = Vec::new();
        let mut all_warnings = Vec::new();
        let mut all_errors = Vec::new();

        // Run multiple concurrent tasks
        let tasks: Vec<_> = (0..3)
            .map(|i| {
                let test_case = test_case.clone();
                let context = context.clone();
                let plugin_manager = plugin_manager.clone();
                let temp_dir = self.temp_dir.path().to_path_buf();

                tokio::spawn(async move {
                    // Create a unique context for each task
                    let task_context = PluginContext::new(
                        temp_dir.join(format!("task_{}", i)),
                        temp_dir.join(format!("task_{}_output", i)),
                        context.config.clone(),
                    );

                    // Process the test case
                    let runner = PluginTestRunner::new(PluginTestSuite {
                        name: format!("concurrent_task_{}", i),
                        description: "Concurrent test task".to_string(),
                        plugin_config: context.config.clone(),
                        test_cases: vec![test_case.clone()],
                        setup: None,
                        cleanup: None,
                    })?;

                    runner
                        .test_source_processing(&test_case, &task_context, &plugin_manager)
                        .await
                })
            })
            .collect();

        // Wait for all tasks to complete
        let task_count = tasks.len();
        for task in tasks {
            match task.await {
                Ok(Ok(output)) => {
                    all_schemas.extend(output.schemas);
                    all_files.extend(output.files);
                    all_warnings.extend(output.warnings);
                    all_errors.extend(output.errors);
                }
                Ok(Err(e)) => {
                    all_errors.push(format!("Concurrent task failed: {}", e));
                }
                Err(e) => {
                    all_errors.push(format!("Concurrent task panicked: {}", e));
                }
            }
        }

        let content = format!(
            "Concurrent Execution Test Results:\n\
            Tasks completed: {}\n\
            Total schemas: {}\n\
            Total files: {}\n\
            Total warnings: {}\n\
            Total errors: {}",
            task_count,
            all_schemas.len(),
            all_files.len(),
            all_warnings.len(),
            all_errors.len()
        );

        Ok(TestOutput {
            files: all_files,
            content,
            schemas: all_schemas,
            warnings: all_warnings,
            errors: all_errors,
        })
    }

    /// Test stress execution
    async fn test_stress_execution(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let iterations = 10;
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut total_processing_time = 0;

        for _i in 0..iterations {
            let start_time = std::time::Instant::now();

            let result = self
                .test_source_processing(test_case, context, plugin_manager)
                .await;
            let processing_time = start_time.elapsed().as_millis() as u64;
            total_processing_time += processing_time;

            match result {
                Ok(_) => success_count += 1,
                Err(_) => failure_count += 1,
            }
        }

        let avg_processing_time = total_processing_time / iterations;
        let success_rate = (success_count as f64 / iterations as f64) * 100.0;

        let content = format!(
            "Stress Test Results:\n\
            Iterations: {}\n\
            Successes: {}\n\
            Failures: {}\n\
            Success Rate: {:.1}%\n\
            Average Processing Time: {}ms",
            iterations, success_count, failure_count, success_rate, avg_processing_time
        );

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas: Vec::new(),
            warnings: Vec::new(),
            errors: if failure_count > 0 {
                vec![format!("{} stress test iterations failed", failure_count)]
            } else {
                Vec::new()
            },
        })
    }

    /// Test memory leak detection
    async fn test_memory_leak_detection(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let iterations = 5;
        let mut memory_readings = Vec::new();

        for _i in 0..iterations {
            let initial_memory = self.get_memory_usage();

            // Run the test
            let _result = self
                .test_source_processing(test_case, context, plugin_manager)
                .await;

            // Force garbage collection by dropping result
            drop(_result);

            let final_memory = self.get_memory_usage();
            memory_readings.push((initial_memory, final_memory));

            // Small delay to allow memory cleanup
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Analyze memory usage patterns
        let mut memory_growth = Vec::new();
        for (initial, final_mem) in memory_readings {
            let growth = final_mem.saturating_sub(initial);
            memory_growth.push(growth);
        }

        let avg_growth = memory_growth.iter().sum::<usize>() / memory_growth.len();
        let max_growth = memory_growth.iter().max().copied().unwrap_or(0);

        let content = format!(
            "Memory Leak Detection Results:\n\
            Iterations: {}\n\
            Average Memory Growth: {} bytes\n\
            Maximum Memory Growth: {} bytes\n\
            Memory Growth Pattern: {:?}",
            iterations, avg_growth, max_growth, memory_growth
        );

        let warnings = if avg_growth > 1024 * 1024 {
            // 1MB threshold
            vec!["Potential memory leak detected: average growth > 1MB".to_string()]
        } else {
            Vec::new()
        };

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas: Vec::new(),
            warnings,
            errors: Vec::new(),
        })
    }

    /// Test compatibility
    async fn test_compatibility(
        &self,
        _test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut compatibility_issues = Vec::new();
        let mut warnings = Vec::new();

        // Test different input formats
        let test_formats = vec![
            ("go", "package main\n\ntype Test struct {\n    Field string `json:\"field\"`\n}\n"),
            ("yaml", "apiVersion: v1\nkind: Test\nmetadata:\n  name: test\n"),
            ("json", "{\n  \"type\": \"object\",\n  \"properties\": {\n    \"field\": {\n      \"type\": \"string\"\n    }\n  }\n}"),
        ];

        for (format, content) in &test_formats {
            let temp_file = self.temp_dir.path().join(format!("test.{}", format));
            std::fs::write(&temp_file, content)?;

            let result = plugin_manager.process_source(&temp_file, context).await;
            match result {
                Ok(_) => {
                    // Format is supported
                }
                Err(e) => {
                    compatibility_issues.push(format!("{} format: {}", format, e));
                }
            }
        }

        let content = format!(
            "Compatibility Test Results:\n\
            Tested Formats: {}\n\
            Compatibility Issues: {}\n\n\
            Issues:\n{}",
            test_formats.len(),
            compatibility_issues.len(),
            if compatibility_issues.is_empty() {
                "None".to_string()
            } else {
                compatibility_issues.join("\n")
            }
        );

        warnings.extend(compatibility_issues);

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas: Vec::new(),
            warnings,
            errors: Vec::new(),
        })
    }

    /// Test security checks
    async fn test_security_checks(
        &self,
        _test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        let mut security_issues = Vec::new();
        let mut warnings = Vec::new();

        // Test for potential security issues
        let security_tests = vec![
            ("path_traversal", "../../../etc/passwd"),
            ("command_injection", "$(rm -rf /)"),
            ("script_injection", "<script>alert('xss')</script>"),
        ];

        for (test_name, malicious_input) in &security_tests {
            // Create a test file with malicious content
            let temp_file = self
                .temp_dir
                .path()
                .join(format!("security_test_{}.txt", test_name));
            std::fs::write(&temp_file, malicious_input)?;

            let result = plugin_manager.process_source(&temp_file, context).await;
            match result {
                Ok(_) => {
                    // This might be a security issue - the plugin processed malicious input
                    security_issues.push(format!(
                        "{}: Plugin processed potentially malicious input",
                        test_name
                    ));
                }
                Err(_) => {
                    // This is good - the plugin rejected malicious input
                }
            }
        }

        let content = format!(
            "Security Test Results:\n\
            Security Tests: {}\n\
            Security Issues Found: {}\n\n\
            Issues:\n{}",
            security_tests.len(),
            security_issues.len(),
            if security_issues.is_empty() {
                "None".to_string()
            } else {
                security_issues.join("\n")
            }
        );

        warnings.extend(security_issues);

        Ok(TestOutput {
            files: Vec::new(),
            content,
            schemas: Vec::new(),
            warnings,
            errors: Vec::new(),
        })
    }

    /// Test basic custom execution for unknown types
    async fn test_basic_custom_execution(
        &self,
        test_case: &PluginTestCase,
        context: &PluginContext,
        plugin_manager: &Arc<PluginManager>,
    ) -> Result<TestOutput> {
        // Run a combination of basic tests
        let source_result = self
            .test_source_processing(test_case, context, plugin_manager)
            .await;
        let schema_result = self
            .test_schema_extraction(test_case, context, plugin_manager)
            .await;

        let mut all_schemas = Vec::new();
        let mut all_files = Vec::new();
        let mut all_warnings = Vec::new();
        let mut all_errors = Vec::new();

        if let Ok(output) = source_result {
            all_schemas.extend(output.schemas);
            all_files.extend(output.files);
            all_warnings.extend(output.warnings);
            all_errors.extend(output.errors);
        }

        if let Ok(output) = schema_result {
            all_schemas.extend(output.schemas);
            all_files.extend(output.files);
            all_warnings.extend(output.warnings);
            all_errors.extend(output.errors);
        }

        let content = format!(
            "Basic Custom Test Results:\n\
            Total Schemas: {}\n\
            Total Files: {}\n\
            Total Warnings: {}\n\
            Total Errors: {}",
            all_schemas.len(),
            all_files.len(),
            all_warnings.len(),
            all_errors.len()
        );

        Ok(TestOutput {
            files: all_files,
            content,
            schemas: all_schemas,
            warnings: all_warnings,
            errors: all_errors,
        })
    }

    /// Validate test output against expectations
    fn validate_test_output(&self, test_case: &PluginTestCase, output: &TestOutput) -> bool {
        let expected = &test_case.expected;

        // Check success/failure
        if expected.success && !output.errors.is_empty() {
            return false;
        }

        if !expected.success && output.errors.is_empty() {
            return false;
        }

        // Check output files
        for expected_file in &expected.output_files {
            if !output.files.contains(expected_file) {
                return false;
            }
        }

        // Check content patterns
        for pattern in &expected.content_patterns {
            if !output.content.contains(pattern) {
                return false;
            }
        }

        // Check error patterns
        for pattern in &expected.error_patterns {
            let has_error = output.errors.iter().any(|e| e.contains(pattern));
            if !has_error {
                return false;
            }
        }

        // Check schema expectations
        if let Some(schema_expectations) = &expected.schemas {
            if output.schemas.len() != schema_expectations.schema_count {
                return false;
            }

            for expected_type in &schema_expectations.schema_types {
                let has_type = output
                    .schemas
                    .iter()
                    .any(|s| s.schema_type == *expected_type);
                if !has_type {
                    return false;
                }
            }
        }

        true
    }

    /// Setup test environment
    async fn setup_test_environment(&self) -> Result<()> {
        if let Some(setup) = &self.test_suite.setup {
            // Create files
            for (file_path, content) in &setup.files {
                let full_path = self.temp_dir.path().join(file_path);
                std::fs::create_dir_all(full_path.parent().unwrap())?;
                std::fs::write(&full_path, content)?;
            }

            // Set environment variables
            for (key, value) in &setup.environment {
                std::env::set_var(key, value);
            }

            // Run setup commands
            for command in &setup.commands {
                if let Err(e) = self.execute_command(command).await {
                    // Log the error but don't fail the setup
                    eprintln!("Setup command '{}' failed: {}", command, e);
                }
            }
        }

        Ok(())
    }

    /// Cleanup test environment
    async fn cleanup_test_environment(&self) -> Result<()> {
        if let Some(cleanup) = &self.test_suite.cleanup {
            // Remove files
            for file_path in &cleanup.remove_files {
                let full_path = self.temp_dir.path().join(file_path);
                if full_path.exists() {
                    std::fs::remove_file(full_path)?;
                }
            }

            // Remove directories
            for dir_path in &cleanup.remove_directories {
                let full_path = self.temp_dir.path().join(dir_path);
                if full_path.exists() {
                    std::fs::remove_dir_all(full_path)?;
                }
            }

            // Run cleanup commands
            for command in &cleanup.commands {
                if let Err(e) = self.execute_command(command).await {
                    // Log the error but don't fail the cleanup
                    eprintln!("Cleanup command '{}' failed: {}", command, e);
                }
            }
        }

        Ok(())
    }

    /// Get current memory usage in bytes
    fn get_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, we can use the task_info API, but for simplicity we'll use a fallback
            // This is a rough approximation based on process info
            if let Ok(output) = std::process::Command::new("ps")
                .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()
            {
                if let Ok(memory_str) = String::from_utf8(output.stdout) {
                    if let Ok(kb) = memory_str.trim().parse::<usize>() {
                        return kb * 1024; // Convert KB to bytes
                    }
                }
            }
        }

        // Fallback: return 0 if we can't determine memory usage
        0
    }

    /// Calculate output size in bytes
    fn calculate_output_size(&self, output: &TestOutput) -> usize {
        let mut total_size = 0;

        // Add content size
        total_size += output.content.len();

        // Add file sizes
        for file_path in &output.files {
            if let Ok(metadata) = std::fs::metadata(file_path) {
                total_size += metadata.len() as usize;
            }
        }

        // Add schema content size (approximate)
        for schema in &output.schemas {
            total_size += serde_yaml::to_string(&schema.content)
                .map(|s| s.len())
                .unwrap_or(0);
        }

        total_size
    }

    /// Execute a shell command
    async fn execute_command(&self, command: &str) -> Result<()> {
        // Split command into program and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let program = parts[0];
        let args = &parts[1..];

        // Execute the command
        let output = tokio::process::Command::new(program)
            .args(args)
            .current_dir(self.temp_dir.path())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Command '{}' failed with status {}: {}",
                command,
                output.status,
                stderr
            ));
        }

        Ok(())
    }
}

/// Test run summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunSummary {
    /// Test suite name
    pub test_suite_name: String,

    /// Total number of tests
    pub total_tests: usize,

    /// Number of passed tests
    pub passed_tests: usize,

    /// Number of failed tests
    pub failed_tests: usize,

    /// Total execution time in milliseconds
    pub total_time_ms: u64,

    /// Individual test results
    pub results: Vec<PluginTestResult>,
}

/// Plugin test trait for plugins to implement
#[async_trait]
pub trait PluginTestable: Plugin {
    /// Get test suite for this plugin
    fn get_test_suite(&self) -> PluginTestSuite;

    /// Run plugin-specific tests
    async fn run_plugin_tests(&self) -> Result<TestRunSummary>;
}

/// Test runner utilities
pub mod utils {
    use super::*;

    /// Create a simple test case
    pub fn create_test_case(
        name: &str,
        description: &str,
        test_type: TestCaseType,
        input_files: Vec<PathBuf>,
        expected_success: bool,
    ) -> PluginTestCase {
        PluginTestCase {
            name: name.to_string(),
            description: description.to_string(),
            test_type,
            input: TestInput {
                files: input_files,
                content: None,
                config: serde_yaml::Value::Null,
                env_vars: HashMap::new(),
            },
            expected: TestExpected {
                success: expected_success,
                output_files: Vec::new(),
                content_patterns: Vec::new(),
                error_patterns: Vec::new(),
                performance: None,
                schemas: None,
            },
            timeout_seconds: None,
            required: true,
            tags: Vec::new(),
        }
    }

    /// Create a test suite
    pub fn create_test_suite(
        name: &str,
        description: &str,
        plugin_config: PluginConfig,
        test_cases: Vec<PluginTestCase>,
    ) -> PluginTestSuite {
        PluginTestSuite {
            name: name.to_string(),
            description: description.to_string(),
            plugin_config,
            test_cases,
            setup: None,
            cleanup: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_case_creation() {
        let test_case = utils::create_test_case(
            "test_initialization",
            "Test plugin initialization",
            TestCaseType::Initialization,
            vec![PathBuf::from("test.go")],
            true,
        );

        assert_eq!(test_case.name, "test_initialization");
        assert!(test_case.required);
    }

    #[test]
    fn test_test_suite_creation() {
        let plugin_config = PluginConfig {
            plugin_id: "test-plugin".to_string(),
            config: serde_yaml::Value::Null,
            enabled_capabilities: vec![PluginCapability::Parse],
        };

        let test_cases = vec![utils::create_test_case(
            "test1",
            "Test 1",
            TestCaseType::Initialization,
            vec![],
            true,
        )];

        let test_suite = utils::create_test_suite(
            "test-suite",
            "Test suite description",
            plugin_config,
            test_cases,
        );

        assert_eq!(test_suite.name, "test-suite");
        assert_eq!(test_suite.test_cases.len(), 1);
    }
}
