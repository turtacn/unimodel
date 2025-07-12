//! 模型管理器服务

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::common::types::*;
use crate::common::error::*;
use crate::domain::model::*;
use crate::infrastructure::configuration::Config;
use crate::plugins::manager::PluginManager;

/// 模型管理器
#[derive(Debug)]
pub struct ModelManager {
    /// 已加载的模型
    models: Arc<RwLock<HashMap<ModelId, Model>>>,
    /// 插件管理器
    plugin_manager: Arc<PluginManager>,
    /// 配置
    config: Arc<Config>,
    /// 最大模型数量
    max_models: usize,
}

impl ModelManager {
    /// 创建新的模型管理器
    pub async fn new(config: &Config) -> Result<Self> {
        let plugin_manager = Arc::new(PluginManager::new(config).await?);
        let max_models = config.engine.max_models as usize;

        Ok(Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            plugin_manager,
            config: Arc::new(config.clone()),
            max_models,
        })
    }

    /// 注册模型
    pub async fn register_model(
        &self,
        name: String,
        model_type: ModelType,
        config: ModelConfig,
    ) -> Result<ModelId> {
        let model_id = new_model_id();
        let mut model = Model::new(model_id.clone(), name, model_type, config);

        // 检查是否达到最大模型数量
        {
            let models = self.models.read().await;
            if models.len() >= self.max_models {
                return Err(UniModelError::model("Maximum number of models reached"));
            }
        }

        // 更新模型状态为加载中
        model.update_status(ModelStatus::Loading);

        // 插入模型
        {
            let mut models = self.models.write().await;
            models.insert(model_id.clone(), model);
        }

        info!("Model registered: {}", model_id);

        // 异步加载模型
        let manager = Arc::clone(&self.plugin_manager);
        let models = Arc::clone(&self.models);
        let id = model_id.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::load_model_async(manager, models, id).await {
                error!("Failed to load model: {}", e);
            }
        });

        Ok(model_id)
    }

    /// 异步加载模型
    async fn load_model_async(
        plugin_manager: Arc<PluginManager>,
        models: Arc<RwLock<HashMap<ModelId, Model>>>,
        model_id: ModelId,
    ) -> Result<()> {
        // 获取模型配置
        let config = {
            let models = models.read().await;
            let model = models.get(&model_id)
                .ok_or_else(|| UniModelError::model("Model not found"))?;
            model.info.config.clone()
        };

        // 通过插件管理器加载模型
        match plugin_manager.load_model(&model_id, &config).await {
            Ok(instance) => {
                // 更新模型状态为就绪
                let mut models = models.write().await;
                if let Some(model) = models.get_mut(&model_id) {
                    model.instance = Some(instance);
                    model.update_status(ModelStatus::Ready);
                    model.info.health_status = HealthStatus::Healthy;
                    info!("Model loaded successfully: {}", model_id);
                }
            }
            Err(e) => {
                // 更新模型状态为错误
                let mut models = models.write().await;
                if let Some(model) = models.get_mut(&model_id) {
                    model.update_status(ModelStatus::Error(e.to_string()));
                    model.info.health_status = HealthStatus::Unhealthy;
                }
                error!("Failed to load model {}: {}", model_id, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// 卸载模型
    pub async fn unregister_model(&self, model_id: &ModelId) -> Result<()> {
        let mut models = self.models.write().await;

        if let Some(mut model) = models.remove(model_id) {
            // 通过插件管理器卸载模型
            if let Some(instance) = &model.instance {
                if let Err(e) = self.plugin_manager.unload_model(&instance.plugin_id, &instance.handle).await {
                    warn!("Failed to unload model from plugin: {}", e);
                }
            }

            model.update_status(ModelStatus::Unloaded);
            info!("Model unregistered: {}", model_id);
            Ok(())
        } else {
            Err(UniModelError::model("Model not found"))
        }
    }

    /// 获取模型信息
    pub async fn get_model_info(&self, model_id: &ModelId) -> Result<ModelInfo> {
        let models = self.models.read().await;
        let model = models.get(model_id)
            .ok_or_else(|| UniModelError::model("Model not found"))?;
        Ok(model.info.clone())
    }

    /// 获取所有模型列表
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let models = self.models.read().await;
        Ok(models.values().map(|m| m.info.clone()).collect())
    }

    /// 获取模型用于推理
    pub async fn get_model_for_inference(&self, model_id: &ModelId) -> Result<Model> {
        let mut models = self.models.write().await;

        match models.get_mut(model_id) {
            Some(model) => {
                if !model.is_loaded() {
                    return Err(UniModelError::model("Model not loaded"));
                }
                if !model.is_healthy() {
                    return Err(UniModelError::model("Model is unhealthy"));
                }

                model.touch();
                Ok(model.clone())
            }
            None => Err(UniModelError::model("Model not found")),
        }
    }

    /// 更新模型性能统计
    pub async fn update_model_performance(
        &self,
        model_id: &ModelId,
        latency_ms: u64,
        success: bool,
    ) -> Result<()> {
        let mut models = self.models.write().await;

        if let Some(model) = models.get_mut(model_id) {
            model.update_performance_stats(latency_ms, success);
            Ok(())
        } else {
            Err(UniModelError::model("Model not found"))
        }
    }

    /// 健康检查
    pub async fn health_check(&self) -> HealthStatus {
        let models = self.models.read().await;

        if models.is_empty() {
            return HealthStatus::Unknown;
        }

        let healthy_count = models.values()
            .filter(|m| m.is_healthy())
            .count();

        if healthy_count == models.len() {
            HealthStatus::Healthy
        } else if healthy_count > 0 {
            HealthStatus::Healthy // 至少有一个健康的模型
        } else {
            HealthStatus::Unhealthy
        }
    }

    /// 获取资源使用情况
    pub async fn get_resource_usage(&self) -> Result<ResourceUsage> {
        // 这里应该从系统监控组件获取实际的资源使用情况
        // 为了简化，返回一个默认值
        Ok(ResourceUsage {
            cpu_usage: 0.0,
            memory_usage_bytes: 0,
            total_memory_bytes: 0,
            gpu_usage: vec![],
            disk_usage_bytes: 0,
            network_io: NetworkIO {
                bytes_received: 0,
                bytes_sent: 0,
                packets_received: 0,
                packets_sent: 0,
            },
            timestamp: chrono::Utc::now(),
        })
    }
}