//! 推理API处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

use crate::common::types::*;
use crate::common::error::*;
use crate::application::services::PredictionService;
use crate::domain::service::batch_processor::PredictionResponse;
use crate::api::rest::handlers::AppState;

/// 推理请求
#[derive(Debug, Deserialize)]
pub struct PredictRequest {
    pub input: InputData,
    pub parameters: Option<PredictionParameters>,
}

/// 推理响应
#[derive(Debug, Serialize)]
pub struct PredictResponse {
    pub request_id: RequestId,
    pub model_id: ModelId,
    pub output: OutputData,
    pub metadata: ResponseMetadata,
    pub metrics: PerformanceMetrics,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 批量推理请求
#[derive(Debug, Deserialize)]
pub struct BatchPredictRequest {
    pub inputs: Vec<InputData>,
    pub parameters: Option<PredictionParameters>,
}

/// 批量推理响应
#[derive(Debug, Serialize)]
pub struct BatchPredictResponse {
    pub request_id: RequestId,
    pub model_id: ModelId,
    pub outputs: Vec<OutputData>,
    pub metadata: ResponseMetadata,
    pub metrics: PerformanceMetrics,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 创建推理路由
pub fn create_predict_routes() -> Router<AppState> {
    Router::new()
        .route("/models/:model_id/predict", post(predict))
        .route("/models/:model_id/predict/batch", post(batch_predict))
}

/// 单个推理处理
pub async fn predict(
    State(state): State<AppState>,
    Path(model_id): Path<ModelId>,
    Json(request): Json<PredictRequest>,
) -> Result<Json<PredictResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Processing prediction request for model: {}", model_id);

    let parameters = request.parameters.unwrap_or_default();

    match state.prediction_service.predict(
        model_id.clone(),
        request.input,
        parameters,
    ).await {
        Ok(response) => {
            let predict_response = PredictResponse {
                request_id: response.request_id,
                model_id: response.model_id,
                output: response.output,
                metadata: response.metadata,
                metrics: response.metrics,
                timestamp: response.timestamp,
            };
            Ok(Json(predict_response))
        }
        Err(e) => {
            error!("Prediction failed for model {}: {}", model_id, e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({
                    "error": e.error_code(),
                    "message": e.to_string()
                }))
            ))
        }
    }
}

/// 批量推理处理
pub async fn batch_predict(
    State(state): State<AppState>,
    Path(model_id): Path<ModelId>,
    Json(request): Json<BatchPredictRequest>,
) -> Result<Json<BatchPredictResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Processing batch prediction request for model: {} with {} inputs",
          model_id, request.inputs.len());

    let parameters = request.parameters.unwrap_or_default();

    match state.prediction_service.batch_predict(
        model_id.clone(),
        request.inputs,
        parameters,
    ).await {
        Ok(responses) => {
            // 合并批量响应
            let request_id = new_request_id();
            let outputs: Vec<OutputData> = responses.iter()
                .map(|r| r.output.clone())
                .collect();

            let batch_response = BatchPredictResponse {
                request_id,
                model_id: model_id.clone(),
                outputs,
                metadata: responses.first()
                    .map(|r| r.metadata.clone())
                    .unwrap_or_else(|| ResponseMetadata {
                        model_version: "unknown".to_string(),
                        backend: "unknown".to_string(),
                        custom_metadata: std::collections::HashMap::new(),
                    }),
                metrics: merge_batch_metrics(&responses),
                timestamp: chrono::Utc::now(),
            };

            Ok(Json(batch_response))
        }
        Err(e) => {
            error!("Batch prediction failed for model {}: {}", model_id, e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({
                    "error": e.error_code(),
                    "message": e.to_string()
                }))
            ))
        }
    }
}

/// 合并批量推理的性能指标
fn merge_batch_metrics(responses: &[PredictionResponse]) -> PerformanceMetrics {
    if responses.is_empty() {
        return PerformanceMetrics {
            request_id: new_request_id(),
            start_time: chrono::Utc::now(),
            end_time: chrono::Utc::now(),
            total_latency_ms: 0,
            inference_latency_ms: 0,
            queue_wait_ms: 0,
            preprocessing_ms: 0,
            postprocessing_ms: 0,
            tokens_generated: None,
            tokens_input: None,
            throughput_tokens_per_sec: None,
            batch_size: 0,
            gpu_utilization: None,
            memory_usage_mb: None,
        };
    }

    let first_response = &responses[0];
    let total_tokens_generated = responses.iter()
        .filter_map(|r| r.metrics.tokens_generated)
        .sum::<u32>();
    let total_tokens_input = responses.iter()
        .filter_map(|r| r.metrics.tokens_input)
        .sum::<u32>();

    PerformanceMetrics {
        request_id: new_request_id(),
        start_time: first_response.metrics.start_time,
        end_time: first_response.metrics.end_time,
        total_latency_ms: first_response.metrics.total_latency_ms,
        inference_latency_ms: first_response.metrics.inference_latency_ms,
        queue_wait_ms: first_response.metrics.queue_wait_ms,
        preprocessing_ms: first_response.metrics.preprocessing_ms,
        postprocessing_ms: first_response.metrics.postprocessing_ms,
        tokens_generated: if total_tokens_generated > 0 { Some(total_tokens_generated) } else { None },
        tokens_input: if total_tokens_input > 0 { Some(total_tokens_input) } else { None },
        throughput_tokens_per_sec: first_response.metrics.throughput_tokens_per_sec,
        batch_size: responses.len() as u32,
        gpu_utilization: first_response.metrics.gpu_utilization,
        memory_usage_mb: first_response.metrics.memory_usage_mb,
    }
}

/// 响应元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub model_version: String,
    pub backend: String,
    pub custom_metadata: std::collections::HashMap<String, serde_json::Value>,
}