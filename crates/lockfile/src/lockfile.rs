//! Main lockfile implementation

use crate::types::{FileChecksum, GenerationStatistics, LockfileEntry};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_creation() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, "1.0");
        assert_eq!(lockfile.tool_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = Lockfile::new();
        let entry = crate::types::LockfileEntry::new(
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
    fn test_source_changed() {
        let mut lockfile = Lockfile::new();
        let entry = crate::types::LockfileEntry::new(
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
        let entry1 = crate::types::LockfileEntry::new(
            "https://github.com/test/repo1.git".to_string(),
            "main".to_string(),
            "abc123".to_string(),
            vec![],
        );
        let entry2 = crate::types::LockfileEntry::new(
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
}
