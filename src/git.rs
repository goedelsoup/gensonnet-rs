//! Git repository management for JsonnetGen

use crate::config::GitSource;
use anyhow::{anyhow, Result};
use dirs;
use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};
use hex;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub struct GitManager {
    cache_dir: PathBuf,
}

impl GitManager {
    /// Create a new GitManager instance
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Get the XDG cache directory for Git repositories
    fn get_cache_dir() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("Could not determine cache directory"))?
            .join("gensonnet")
            .join("git");

        Ok(cache_dir)
    }

    /// Ensure a repository is available locally, cloning if necessary
    pub async fn ensure_repository(&self, git_source: &GitSource) -> Result<PathBuf> {
        let repo_path = self.get_repo_path(git_source);

        if repo_path.exists() {
            info!("Repository already exists at {:?}", repo_path);
            self.update_repository(&repo_path, git_source).await?;
        } else {
            info!("Cloning repository from {}", git_source.url);
            self.clone_repository(git_source, &repo_path).await?;
        }

        // Checkout the specified reference
        self.checkout_reference(&repo_path, git_source)?;

        Ok(repo_path)
    }

    /// Get the local path for a repository
    fn get_repo_path(&self, git_source: &GitSource) -> PathBuf {
        let repo_hash = self.hash_repo_url(&git_source.url);
        self.cache_dir.join(repo_hash)
    }

    /// Hash the repository URL to create a unique directory name
    fn hash_repo_url(&self, url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Clone a repository
    async fn clone_repository(&self, git_source: &GitSource, repo_path: &Path) -> Result<()> {
        let mut callbacks = RemoteCallbacks::new();

        // Set up authentication callbacks if needed
        if let Some(auth) = &git_source.auth {
            self.setup_auth_callbacks(&mut callbacks, auth)?;
        }

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Perform the clone
        let _repo = Repository::clone(&git_source.url, repo_path)?;

        info!("Successfully cloned repository to {:?}", repo_path);
        Ok(())
    }

    /// Update an existing repository
    async fn update_repository(&self, repo_path: &Path, git_source: &GitSource) -> Result<()> {
        let repo = Repository::open(repo_path)?;

        // Fetch latest changes
        let mut callbacks = RemoteCallbacks::new();
        if let Some(auth) = &git_source.auth {
            self.setup_auth_callbacks(&mut callbacks, auth)?;
        }

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Fetch from origin
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )?;

        info!("Updated repository at {:?}", repo_path);
        Ok(())
    }

    /// Checkout a specific reference
    fn checkout_reference(&self, repo_path: &Path, git_source: &GitSource) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        let ref_name = git_source.ref_name();

        // Simplified checkout logic - just try to find the reference
        let reference = if ref_name == "main" || ref_name == "master" {
            // Try main first, then master
            repo.find_branch("main", git2::BranchType::Local)
                .or_else(|_| repo.find_branch("master", git2::BranchType::Local))
                .and_then(|branch| branch.get().peel_to_commit())
        } else if ref_name.starts_with("refs/") {
            repo.find_reference(ref_name)
                .and_then(|r| r.peel_to_commit())
        } else {
            // Try as a commit SHA first
            if let Ok(oid) = ref_name.parse::<git2::Oid>() {
                repo.find_commit(oid)
            } else {
                // Try as a branch
                repo.find_branch(ref_name, git2::BranchType::Local)
                    .or_else(|_| repo.find_branch(ref_name, git2::BranchType::Remote))
                    .and_then(|branch| branch.get().peel_to_commit())
            }
        }?;

        // Checkout the reference
        let tree = reference.tree()?;
        repo.checkout_tree(tree.as_object(), None)?;

        // Set HEAD to the reference
        repo.set_head_detached(reference.id())?;

        info!("Checked out reference: {}", ref_name);
        Ok(())
    }

    /// Set up authentication callbacks
    fn setup_auth_callbacks(
        &self,
        callbacks: &mut RemoteCallbacks,
        auth: &crate::config::GitAuth,
    ) -> Result<()> {
        let auth = auth.clone();
        callbacks.credentials(move |_url, username_from_url, _allowed_types| match &auth {
            crate::config::GitAuth::Ssh {
                key_path,
                passphrase,
            } => Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                key_path,
                passphrase.as_deref(),
            ),
            crate::config::GitAuth::Token { token } => {
                Cred::userpass_plaintext(username_from_url.unwrap_or("git"), token)
            }
            crate::config::GitAuth::Basic { username, password } => {
                Cred::userpass_plaintext(username, password)
            }
        });

        Ok(())
    }

    /// Get the current commit SHA of a repository
    pub fn get_current_commit(&self, repo_path: &Path) -> Result<String> {
        let repo = Repository::open(repo_path)?;
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Clean up old repositories (optional maintenance function)
    pub fn cleanup_old_repositories(&self, _max_age_days: u64) -> Result<()> {
        // Implementation for cleaning up old cached repositories
        // This would check modification times and remove old entries
        warn!("Repository cleanup not yet implemented");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_repo_url() {
        let manager = GitManager::new().unwrap();
        let hash1 = manager.hash_repo_url("https://github.com/test/repo.git");
        let hash2 = manager.hash_repo_url("https://github.com/test/repo.git");
        let hash3 = manager.hash_repo_url("https://github.com/other/repo.git");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_get_repo_path() {
        let manager = GitManager::new().unwrap();
        let git_source = crate::config::GitSource {
            url: "https://github.com/test/repo.git".to_string(),
            ref_name: Some("main".to_string()),
            auth: None,
        };

        let path = manager.get_repo_path(&git_source);
        assert!(path.to_string_lossy().contains("gensonnet"));
        assert!(path.to_string_lossy().contains("git"));
    }
}
