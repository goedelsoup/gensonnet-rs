//! Lockfile management for reproducible builds

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Lockfile structure for tracking generation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile version
    pub version: String,

    /// Generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Tool version used
    pub tool_version: String,

    /// Source entries
    pub sources: HashMap<String, LockfileEntry>,

    /// Generated files checksums
    pub files: HashMap<PathBuf, FileChecksum>,

    /// Dependencies between sources
    pub dependencies: HashMap<String, Vec<String>>,

    /// Generation statistics
    pub statistics: GenerationStatistics,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

impl Lockfile {
    /// Create a new lockfile
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            sources: HashMap::new(),
            files: HashMap::new(),
            dependencies: HashMap::new(),
            statistics: GenerationStatistics::default(),
        }
    }

    /// Load lockfile from disk
    pub fn from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(anyhow!("Lockfile does not exist: {:?}", path));
        }

        let content = std::fs::read_to_string(path)?;
        let lockfile: Lockfile = serde_yaml::from_str(&content)?;
        Ok(lockfile)
    }

    /// Save lockfile to disk
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a source entry
    pub fn add_source(&mut self, source_id: String, entry: LockfileEntry) {
        self.sources.insert(source_id, entry);
    }

    /// Add a file checksum
    pub fn add_file(&mut self, file_path: PathBuf, checksum: FileChecksum) {
        self.files.insert(file_path, checksum);
    }

    /// Add a dependency relationship
    pub fn add_dependency(&mut self, source_id: String, depends_on: String) {
        self.dependencies
            .entry(source_id)
            .or_default()
            .push(depends_on);
    }

    /// Check if a source has changed
    pub fn source_changed(&self, source_id: &str, current_commit: &str) -> bool {
        if let Some(entry) = self.sources.get(source_id) {
            entry.commit_sha != current_commit
        } else {
            true // New source
        }
    }

    /// Check if a file has changed
    pub fn file_changed(&self, file_path: &Path, current_checksum: &str) -> bool {
        if let Some(checksum) = self.files.get(file_path) {
            checksum.sha256 != current_checksum
        } else {
            true // New file
        }
    }

    /// Get all changed sources
    pub fn get_changed_sources(&self, current_sources: &HashMap<String, String>) -> Vec<String> {
        let mut changed = Vec::new();

        for (source_id, current_commit) in current_sources {
            if self.source_changed(source_id, current_commit) {
                changed.push(source_id.clone());
            }
        }

        changed
    }

    /// Get sources that need regeneration due to dependencies
    pub fn get_dependent_sources(&self, changed_sources: &[String]) -> Vec<String> {
        let mut dependent_sources = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for changed_source in changed_sources {
            self.collect_dependents(changed_source, &mut dependent_sources, &mut visited);
        }

        dependent_sources
    }

    /// Recursively collect dependent sources
    fn collect_dependents(
        &self,
        source_id: &str,
        dependents: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if visited.contains(source_id) {
            return;
        }
        visited.insert(source_id.to_string());

        for (dependent_id, dependencies) in &self.dependencies {
            if dependencies.contains(&source_id.to_string()) {
                dependents.push(dependent_id.clone());
                self.collect_dependents(dependent_id, dependents, visited);
            }
        }
    }

    /// Update the lockfile with new generation data
    pub fn update(
        &mut self,
        sources: HashMap<String, LockfileEntry>,
        files: HashMap<PathBuf, FileChecksum>,
    ) {
        self.generated_at = Utc::now();
        self.sources = sources;
        self.files = files;
    }

    /// Get generation order based on dependencies
    pub fn get_generation_order(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        for source_id in self.sources.keys() {
            if !visited.contains(source_id) {
                self.topological_sort(source_id, &mut order, &mut visited, &mut temp_visited)?;
            }
        }

        Ok(order)
    }

    /// Topological sort for dependency resolution
    fn topological_sort(
        &self,
        source_id: &str,
        order: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if temp_visited.contains(source_id) {
            return Err(anyhow!(
                "Circular dependency detected involving {}",
                source_id
            ));
        }

        if visited.contains(source_id) {
            return Ok(());
        }

        temp_visited.insert(source_id.to_string());

        if let Some(dependencies) = self.dependencies.get(source_id) {
            for dependency in dependencies {
                self.topological_sort(dependency, order, visited, temp_visited)?;
            }
        }

        temp_visited.remove(source_id);
        visited.insert(source_id.to_string());
        order.push(source_id.to_string());

        Ok(())
    }

    /// Check if incremental generation is possible
    pub fn can_incremental_generate(&self, changed_sources: &[String]) -> bool {
        // Check if any changed sources have dependencies that would require full regeneration
        let dependent_sources = self.get_dependent_sources(changed_sources);
        dependent_sources.len() <= changed_sources.len() * 2 // Allow some dependency overhead
    }

    /// Get files that need regeneration
    pub fn get_files_to_regenerate(&self, changed_sources: &[String]) -> Vec<PathBuf> {
        let mut files_to_regenerate = Vec::new();

        for file_path in self.files.keys() {
            // Check if file is related to changed sources
            if self.is_file_related_to_sources(file_path, changed_sources) {
                files_to_regenerate.push(file_path.clone());
            }
        }

        files_to_regenerate
    }

    /// Check if a file is related to specific sources
    fn is_file_related_to_sources(&self, file_path: &Path, sources: &[String]) -> bool {
        // This is a simplified implementation
        // In practice, you'd track which source generated which file
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        sources
            .iter()
            .any(|source_id| file_name.contains(&source_id.to_lowercase()))
    }
}

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
    pub fn from_file(path: &Path) -> Result<Self> {
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

/// Lockfile manager for handling lockfile operations
pub struct LockfileManager {
    lockfile_path: PathBuf,
}

impl LockfileManager {
    /// Create a new lockfile manager
    pub fn new(lockfile_path: PathBuf) -> Self {
        Self { lockfile_path }
    }

    /// Load or create lockfile
    pub fn load_or_create(&self) -> Result<Lockfile> {
        if self.lockfile_path.exists() {
            Lockfile::from_file(&self.lockfile_path)
        } else {
            Ok(Lockfile::new())
        }
    }

    /// Get the lockfile path
    pub fn path(&self) -> &PathBuf {
        &self.lockfile_path
    }

    /// Save lockfile
    pub fn save(&self, lockfile: &Lockfile) -> Result<()> {
        lockfile.save_to_file(&self.lockfile_path)
    }

    /// Update lockfile with new data
    pub fn update(
        &self,
        sources: HashMap<String, LockfileEntry>,
        files: HashMap<PathBuf, FileChecksum>,
    ) -> Result<()> {
        let mut lockfile = self.load_or_create()?;
        lockfile.update(sources, files);
        self.save(&lockfile)
    }

    /// Check if regeneration is needed
    pub fn needs_regeneration(&self, current_sources: &HashMap<String, String>) -> Result<bool> {
        if !self.lockfile_path.exists() {
            return Ok(true); // No lockfile, need to regenerate
        }

        let lockfile = Lockfile::from_file(&self.lockfile_path)?;

        // Check if any sources have changed
        for (source_id, current_commit) in current_sources {
            if lockfile.source_changed(source_id, current_commit) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get incremental generation plan
    pub fn get_incremental_plan(&self, changed_sources: &[String]) -> Result<IncrementalPlan> {
        let lockfile = self.load_or_create()?;

        let dependent_sources = lockfile.get_dependent_sources(changed_sources);
        let files_to_regenerate = lockfile.get_files_to_regenerate(changed_sources);
        let can_incremental = lockfile.can_incremental_generate(changed_sources);

        Ok(IncrementalPlan {
            changed_sources: changed_sources.to_vec(),
            dependent_sources,
            files_to_regenerate: files_to_regenerate.clone(),
            can_incremental,
            estimated_time_ms: self.estimate_regeneration_time(&lockfile, &files_to_regenerate),
        })
    }

    /// Estimate regeneration time based on file sizes and previous statistics
    fn estimate_regeneration_time(&self, lockfile: &Lockfile, files: &[PathBuf]) -> u64 {
        let total_size: u64 = files
            .iter()
            .filter_map(|path| lockfile.files.get(path))
            .map(|checksum| checksum.size)
            .sum();

        // Rough estimate: 1ms per KB
        total_size / 1024
    }

    /// Clean up stale entries
    pub fn cleanup_stale_entries(&self, max_age_hours: u64) -> Result<()> {
        let mut lockfile = self.load_or_create()?;
        let mut cleaned = false;

        // Clean up stale sources
        lockfile.sources.retain(|_, entry| {
            if entry.is_stale(max_age_hours) {
                cleaned = true;
                false
            } else {
                true
            }
        });

        // Clean up stale files
        lockfile.files.retain(|_, checksum| {
            if checksum.is_stale(max_age_hours) {
                cleaned = true;
                false
            } else {
                true
            }
        });

        if cleaned {
            self.save(&lockfile)?;
        }

        Ok(())
    }

    /// Get the default lockfile path
    pub fn default_path() -> PathBuf {
        PathBuf::from("jsonnet-gen.lock")
    }
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
    fn test_lockfile_creation() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, "1.0");
        assert_eq!(lockfile.tool_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = Lockfile::new();
        let entry = LockfileEntry::new(
            "https://github.com/test/repo.git".to_string(),
            "main".to_string(),
            "abc123".to_string(),
            vec!["test.com/v1".to_string()],
        );
        lockfile.add_source("test".to_string(), entry);

        let yaml = serde_yaml::to_string(&lockfile).unwrap();
        let parsed: Lockfile = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.sources.len(), 1);
        assert!(parsed.sources.contains_key("test"));
    }

    #[test]
    fn test_file_checksum() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "test content").unwrap();

        let checksum = FileChecksum::from_file(temp_file.path()).unwrap();
        assert_eq!(checksum.size, 12); // "test content" length
        assert!(!checksum.sha256.is_empty());
    }

    #[test]
    fn test_source_changed() {
        let mut lockfile = Lockfile::new();
        let entry = LockfileEntry::new(
            "https://github.com/test/repo.git".to_string(),
            "main".to_string(),
            "abc123".to_string(),
            vec![],
        );
        lockfile.add_source("test".to_string(), entry);

        // Same commit
        assert!(!lockfile.source_changed("test", "abc123"));

        // Different commit
        assert!(lockfile.source_changed("test", "def456"));

        // New source
        assert!(lockfile.source_changed("new", "abc123"));
    }

    #[test]
    fn test_dependency_tracking() {
        let mut lockfile = Lockfile::new();

        // Add dependencies
        lockfile.add_dependency("source2".to_string(), "source1".to_string());
        lockfile.add_dependency("source3".to_string(), "source1".to_string());

        // Test dependency collection
        let dependents = lockfile.get_dependent_sources(&["source1".to_string()]);
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"source2".to_string()));
        assert!(dependents.contains(&"source3".to_string()));
    }

    #[test]
    fn test_generation_order() {
        let mut lockfile = Lockfile::new();

        // Add dependencies: source2 depends on source1
        lockfile.add_dependency("source2".to_string(), "source1".to_string());

        // Add sources
        let entry1 = LockfileEntry::new(
            "https://github.com/test/repo1.git".to_string(),
            "main".to_string(),
            "abc123".to_string(),
            vec![],
        );
        let entry2 = LockfileEntry::new(
            "https://github.com/test/repo2.git".to_string(),
            "main".to_string(),
            "def456".to_string(),
            vec![],
        );

        lockfile.add_source("source1".to_string(), entry1);
        lockfile.add_source("source2".to_string(), entry2);

        let order = lockfile.get_generation_order().unwrap();

        // source1 should come before source2
        let source1_index = order.iter().position(|s| s == "source1").unwrap();
        let source2_index = order.iter().position(|s| s == "source2").unwrap();
        assert!(source1_index < source2_index);
    }

    #[test]
    fn test_incremental_plan() {
        let manager = LockfileManager::new(PathBuf::from("test.lock"));
        let plan = manager
            .get_incremental_plan(&["source1".to_string()])
            .unwrap();

        assert_eq!(plan.changed_sources, vec!["source1"]);
        assert_eq!(plan.total_sources(), 1);
    }
}
