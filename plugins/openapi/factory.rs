//! OpenAPI plugin factory

use anyhow::Result;
use async_trait::async_trait;

use gensonnet_plugin::*;
use super::plugin::OpenApiPlugin;

/// OpenAPI plugin factory
pub struct OpenApiPluginFactory;

#[async_trait]
impl PluginFactory for OpenApiPluginFactory {
    async fn create_plugin(&self, config: PluginConfig) -> Result<Box<dyn Plugin>> {
        Ok(Box::new(OpenApiPlugin::new(config)))
    }

    fn supported_types(&self) -> Vec<String> {
        vec![
            "openapi".to_string(),
            "swagger".to_string(),
            "yaml".to_string(),
            "json".to_string(),
        ]
    }

    fn clone_box(&self) -> Box<dyn PluginFactory> {
        Box::new(OpenApiPluginFactory)
    }
}
