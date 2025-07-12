//! 模型应用服务

use std::sync::Arc;
use tracing::{info, error};

use crate::common::types::*;
use crate::common::error::*;
use crate::domain::model::*;
use crate::domain::service::ModelManager;

/// 模型应用服务
#[derive(Debug)]
pub struct ModelService {
    model_manager: Arc<ModelManager>,
}

impl ModelService {
    /// 创建新的模型服务
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self {
            model_manager,
        }
    }

    /// 注册模型
    pub async fn register_model(
        &self,
        name: String,
        model_type: ModelType,
        config: ModelConfig,
    ) -> Result<ModelId> {
        info!("Registering model: {} (type: {:?})", name, model_type);

        // 验证模型配置
        self.validate_model_config(&config)?;

        // 委托给领域服务
        self.model_manager.register_model(name, model_type, config).await
    }

    /// 注销模型
    pub async fn unregister_model(&self, model_id: &ModelId) -> Result<()> {
        info!("Unregistering model: {}", model_id);

        // 委托给领域服务
        self.model_manager.unregister_model(model_id).await
    }

    /// 获取模型信息
    pub async fn get_model_info(&self, model_id: &ModelId) -> Result<ModelInfo> {
        self.model_manager.get_model_info(model_id).await
    }

    /// 获取模型列表
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        self.model_manager.list_models().await
    }

    /// 验证模型配置
    fn validate_model_config(&self, config: &ModelConfig) -> Result<()> {
        // 检查模型路径
        if config.model_path.is_empty() {
            return Err(UniModelError::validation("Model path cannot be empty"));
        }

        // 检查后端
        if config.backend.is_empty() {
            return Err(UniModelError::validation("Backend cannot be empty"));
        }

        // 检查设备配置
        if config.device.device_ids.is_empty() {
            return Err(UniModelError::validation("At least one device ID must be specified"));
        }

        // 检查批处理配置
        if config.batch_config.max_batch_size == 0 {
            return Err(UniModelError::validation("Max batch size must be greater than 0"));
        }

        Ok(())
    }
}