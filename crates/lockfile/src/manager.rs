//! Lockfile manager for handling lockfile operations

use crate::lockfile::Lockfile;
use crate::types::{FileChecksum, IncrementalPlan, LockfileEntry};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

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
        PathBuf::from("gensonnet.lock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
