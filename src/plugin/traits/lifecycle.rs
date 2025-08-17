//! Plugin lifecycle management traits

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Plugin lifecycle manager trait
#[async_trait]
pub trait PluginLifecycleManager: Send + Sync {
    /// Initialize plugins
    async fn initialize_plugins(&self) -> Result<()>;

    /// Start plugins
    async fn start_plugins(&self) -> Result<()>;

    /// Stop plugins
    async fn stop_plugins(&self) -> Result<()>;

    /// Reload plugins
    async fn reload_plugins(&self) -> Result<()>;

    /// Get plugin status
    async fn get_plugin_status(&self) -> Result<Vec<PluginStatus>>;
}

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    /// Plugin ID
    pub plugin_id: String,

    /// Plugin name
    pub name: String,

    /// Plugin status
    pub status: String,

    /// Plugin version
    pub version: String,

    /// Last activity
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,

    /// Error message (if any)
    pub error: Option<String>,
}
