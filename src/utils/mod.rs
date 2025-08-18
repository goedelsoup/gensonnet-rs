//! Utility functions for JsonnetGen

use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    } else if !path.is_dir() {
        return Err(anyhow!("Path exists but is not a directory: {:?}", path));
    }
    Ok(())
}

/// Get the XDG config directory for JsonnetGen
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not determine config directory"))?
        .join("gensonnet");

    Ok(config_dir)
}

/// Get the XDG cache directory for JsonnetGen
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow!("Could not determine cache directory"))?
        .join("gensonnet");

    Ok(cache_dir)
}

/// Find all YAML files in a directory recursively
pub fn find_yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut yaml_files = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "yaml" || ext == "yml" {
                yaml_files.push(path.to_path_buf());
            }
        }
    }

    Ok(yaml_files)
}

/// Calculate SHA256 hash of a file
pub fn calculate_file_hash(path: &Path) -> Result<String> {
    use hex;
    use sha2::{Digest, Sha256};

    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(hex::encode(hasher.finalize()))
}

/// Calculate SHA256 hash of a string
pub fn calculate_string_hash(content: &str) -> String {
    use hex;
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Convert a string to a valid filename
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Convert API version to directory name
pub fn api_version_to_dirname(api_version: &str) -> String {
    api_version.replace('/', "_")
}

/// Convert directory name back to API version
pub fn dirname_to_api_version(dirname: &str) -> String {
    dirname.replace('_', "/")
}

/// Check if a path is within a base directory
pub fn is_within_base(path: &Path, base: &Path) -> bool {
    path.canonicalize()
        .and_then(|p| base.canonicalize().map(|b| p.starts_with(b)))
        .unwrap_or(false)
}

/// Get relative path from base
pub fn get_relative_path(path: &Path, base: &Path) -> Result<PathBuf> {
    let canonical_path = path.canonicalize()?;
    let canonical_base = base.canonicalize()?;

    canonical_path
        .strip_prefix(canonical_base)
        .map(|p| p.to_path_buf())
        .map_err(|e| anyhow!("Failed to get relative path: {}", e))
}

/// Copy directory recursively
pub fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        return Err(anyhow!("Source is not a directory: {:?}", src));
    }

    ensure_directory(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Remove directory recursively
pub fn remove_directory(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Format bytes as human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Format duration as human readable string
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs > 0 {
        format!("{secs}.{millis:03}s")
    } else {
        format!("{millis}ms")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.yaml"), "test.yaml");
        assert_eq!(sanitize_filename("test file.yaml"), "test_file.yaml");
        assert_eq!(sanitize_filename("test@file.yaml"), "test_file.yaml");
    }

    #[test]
    fn test_api_version_conversion() {
        assert_eq!(api_version_to_dirname("apps/v1"), "apps_v1");
        assert_eq!(dirname_to_api_version("apps_v1"), "apps/v1");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_copy_directory() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        let dst_dir = temp_dir.path().join("dst");

        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("test.txt"), "test content").unwrap();

        copy_directory(&src_dir, &dst_dir).unwrap();

        assert!(dst_dir.exists());
        assert!(dst_dir.join("test.txt").exists());
    }
}
