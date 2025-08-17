//! Lockfile types and data structures

use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Entry for a source in the lockfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileEntry {
    /// Git repository URL
    pub url: String,

    /// Git reference (branch, tag, or commit)
    pub ref_name: String,

    /// Exact commit SHA
    pub commit_sha: String,

    /// Last fetch timestamp
    pub fetched_at: DateTime<Utc>,

    /// Filters applied
    pub filters: Vec<String>,

    /// Source metadata
    pub metadata: SourceMetadata,
}

impl LockfileEntry {
    /// Create a new lockfile entry
    pub fn new(url: String, ref_name: String, commit_sha: String, filters: Vec<String>) -> Self {
        Self {
            url,
            ref_name,
            commit_sha,
            fetched_at: Utc::now(),
            filters,
            metadata: SourceMetadata::default(),
        }
    }

    /// Check if the entry is stale (older than specified duration)
    pub fn is_stale(&self, max_age_hours: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.fetched_at);
        age.num_hours() > max_age_hours as i64
    }
}

/// Source metadata for tracking additional information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceMetadata {
    /// Number of CRDs found
    pub crd_count: usize,

    /// Total file size
    pub total_size: u64,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Error count during processing
    pub error_count: usize,

    /// Warning count during processing
    pub warning_count: usize,
}

/// File checksum information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChecksum {
    /// SHA256 checksum
    pub sha256: String,

    /// File size in bytes
    pub size: u64,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,

    /// File metadata
    pub metadata: FileMetadata,
}

impl FileChecksum {
    /// Create a new file checksum
    pub fn new(sha256: String, size: u64, modified_at: DateTime<Utc>) -> Self {
        Self {
            sha256,
            size,
            modified_at,
            metadata: FileMetadata::default(),
        }
    }

    /// Calculate checksum from file content
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        let content = fs::read(path)?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let sha256 = hex::encode(hasher.finalize());

        let modified_at = DateTime::from(metadata.modified()?);

        Ok(Self {
            sha256,
            size: metadata.len(),
            modified_at,
            metadata: FileMetadata::default(),
        })
    }

    /// Check if file is stale (older than specified duration)
    pub fn is_stale(&self, max_age_hours: u64) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.modified_at);
        age.num_hours() > max_age_hours as i64
    }
}

/// File metadata for tracking additional information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMetadata {
    /// Source that generated this file
    pub source_id: Option<String>,

    /// Generation timestamp
    pub generated_at: Option<DateTime<Utc>>,

    /// File type (crd, validation, index, etc.)
    pub file_type: Option<String>,

    /// Line count
    pub line_count: Option<usize>,
}

/// Generation statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

/// Plan for incremental generation
#[derive(Debug, Clone)]
pub struct IncrementalPlan {
    /// Sources that have changed
    pub changed_sources: Vec<String>,

    /// Sources that depend on changed sources
    pub dependent_sources: Vec<String>,

    /// Files that need to be regenerated
    pub files_to_regenerate: Vec<PathBuf>,

    /// Whether incremental generation is possible
    pub can_incremental: bool,

    /// Estimated regeneration time in milliseconds
    pub estimated_time_ms: u64,
}

impl IncrementalPlan {
    /// Get total number of sources to process
    pub fn total_sources(&self) -> usize {
        self.changed_sources.len() + self.dependent_sources.len()
    }

    /// Get total number of files to regenerate
    pub fn total_files(&self) -> usize {
        self.files_to_regenerate.len()
    }

    /// Check if plan requires full regeneration
    pub fn requires_full_regeneration(&self) -> bool {
        !self.can_incremental
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_checksum() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "test content").unwrap();

        let checksum = FileChecksum::from_file(temp_file.path()).unwrap();
        assert_eq!(checksum.size, 12); // "test content" length
        assert!(!checksum.sha256.is_empty());
    }
}
