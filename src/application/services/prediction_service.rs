//! 推理应用服务

use std::sync::Arc;
use tracing::{info, error};

use crate::common::types::*;
use crate::common::error::*;
use crate::domain::service::{ModelManager, BatchProcessor};
use crate::domain::service::batch_processor::PredictionResponse;

/// 推理应用服务
#[derive(Debug)]
pub struct PredictionService {
    model_manager: Arc<ModelManager>,
    batch_processor: Arc<BatchProcessor>,
}

impl PredictionService {
    /// 创建新的推理服务
    pub fn new(
        model_manager: Arc<ModelManager>,
        batch_processor: Arc<BatchProcessor>,
    ) -> Self {
        Self {
            model_manager,
            batch_processor,
        }
    }

    /// 执行推理
    pub async fn predict(
        &self,
        model_id: ModelId,
        input: InputData,
        parameters: PredictionParameters,
    ) -> Result<PredictionResponse> {
        info!("Processing prediction request for model: {}", model_id);

        // 验证模型是否存在且可用
        self.validate_model_availability(&model_id).await?;

        // 验证输入数据
        self.validate_input_data(&input)?;

        // 通过批处理器执行推理
        let response = self.batch_processor.submit_request(
            model_id.clone(),
            input,
            parameters,
        ).await?;

        // 更新模型性能统计
        self.model_manager.update_model_performance(
            &model_id,
            response.metrics.total_latency_ms,
            true,
        ).await?;

        info!("Prediction completed for model: {} in {}ms",
              model_id, response.metrics.total_latency_ms);

        Ok(response)
    }

    /// 批量推理
    pub async fn batch_predict(
        &self,
        model_id: ModelId,
        inputs: Vec<InputData>,
        parameters: PredictionParameters,
    ) -> Result<Vec<PredictionResponse>> {
        info!("Processing batch prediction request for model: {} with {} inputs",
              model_id, inputs.len());

        // 验证模型是否存在且可用
        self.validate_model_availability(&model_id).await?;

        // 验证输入数据
        for input in &inputs {
            self.validate_input_data(input)?;
        }

        // 并行处理多个推理请求
        let mut tasks = Vec::new();

        for input in inputs {
            let batch_processor = Arc::clone(&self.batch_processor);
            let model_id = model_id.clone();
            let parameters = parameters.clone();

            let task = tokio::spawn(async move {
                batch_processor.submit_request(model_id, input, parameters).await
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        let mut responses = Vec::new();
        let mut total_latency = 0u64;
        let mut success_count = 0;

        for task in tasks {
            match task.await {
                Ok(Ok(response)) => {
                    total_latency += response.metrics.total_latency_ms;
                    success_count += 1;
                    responses.push(response);
                }
                Ok(Err(e)) => {
                    error!("Batch prediction task failed: {}", e);
                    return Err(e);
                }
                Err(e) => {
                    error!("Batch prediction task panicked: {}", e);
                    return Err(UniModelError::internal("Task panicked"));
                }
            }
        }

        // 更新模型性能统计
        let avg_latency = if success_count > 0 { total_latency / success_count } else { 0 };
        self.model_manager.update_model_performance(
            &model_id,
            avg_latency,
            success_count == responses.len() as u64,
        ).await?;

        info!("Batch prediction completed for model: {} with {} successful responses",
              model_id, success_count);

        Ok(responses)
    }

    /// 验证模型可用性
    async fn validate_model_availability(&self, model_id: &ModelId) -> Result<()> {
        let model_info = self.model_manager.get_model_info(model_id).await?;

        match model_info.status {
            ModelStatus::Ready | ModelStatus::Running => Ok(()),
            ModelStatus::Initializing | ModelStatus::Loading => {
                Err(UniModelError::model("Model is not ready yet"))
            }
            ModelStatus::Error(ref msg) => {
                Err(UniModelError::model(format!("Model is in error state: {}", msg)))
            }
            ModelStatus::Unloaded => {
                Err(UniModelError::model("Model is unloaded"))
            }
        }
    }

    /// 验证输入数据
    fn validate_input_data(&self, input: &InputData) -> Result<()> {
        match input {
            InputData::Text(text) => {
                if text.is_empty() {
                    return Err(UniModelError::validation("Text input cannot be empty"));
                }
                if text.len() > 1_000_000 { // 1MB limit
                    return Err(UniModelError::validation("Text input too large"));
                }
            }
            InputData::Binary(data) => {
                if data.is_empty() {
                    return Err(UniModelError::validation("Binary input cannot be empty"));
                }
                if data.len() > 100_000_000 { // 100MB limit
                    return Err(UniModelError::validation("Binary input too large"));
                }
            }
            InputData::Json(json) => {
                if json.is_null() {
                    return Err(UniModelError::validation("JSON input cannot be null"));
                }
            }
            InputData::Multimodal(map) => {
                if map.is_empty() {
                    return Err(UniModelError::validation("Multimodal input cannot be empty"));
                }
                for (key, value) in map {
                    if key.is_empty() {
                        return Err(UniModelError::validation("Multimodal key cannot be empty"));
                    }
                    self.validate_input_data(value)?;
                }
            }
        }

        Ok(())
    }
}