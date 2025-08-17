//! CRD plugin factory

use anyhow::Result;
use async_trait::async_trait;

use super::plugin::CrdPlugin;
use crate::plugin::*;

/// CRD plugin factory
pub struct CrdPluginFactory;

#[async_trait]
impl PluginFactory for CrdPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(CrdPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["crd".to_string(), "yaml".to_string(), "yml".to_string()]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(CrdPluginFactory)
    }
}
