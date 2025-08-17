//! Go AST plugin factory

use anyhow::Result;
use async_trait::async_trait;

use super::plugin::GoAstPlugin;
use crate::plugin::*;

/// Go AST plugin factory
pub struct GoAstPluginFactory;

#[async_trait]
impl PluginFactory for GoAstPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(GoAstPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec!["go".to_string(), "golang".to_string()]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(GoAstPluginFactory)
    }
}
