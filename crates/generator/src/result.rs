//! Result types for generation operations

use std::path::PathBuf;

/// Result of processing a source
#[derive(Debug, Clone)]
pub struct SourceResult {
    pub source_type: String,
    pub files_generated: usize,
    pub errors: Vec<String>,
    pub output_path: PathBuf,
    pub processing_time_ms: u64,
    pub warnings: Vec<String>,
}

/// Overall generation result
#[derive(Debug)]
pub struct GenerationResult {
    pub sources_processed: usize,
    pub total_sources: usize,
    pub results: Vec<SourceResult>,
    pub statistics: GenerationStatistics,
}

/// Generation statistics
#[derive(Debug, Clone)]
pub struct GenerationStatistics {
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,

    /// Number of sources processed
    pub sources_processed: usize,

    /// Number of files generated
    pub files_generated: usize,

    /// Number of errors encountered
    pub error_count: usize,

    /// Number of warnings encountered
    pub warning_count: usize,

    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

impl Default for GenerationStatistics {
    fn default() -> Self {
        Self {
            total_processing_time_ms: 0,
            sources_processed: 0,
            files_generated: 0,
            error_count: 0,
            warning_count: 0,
            cache_hit_rate: 0.0,
        }
    }
}
