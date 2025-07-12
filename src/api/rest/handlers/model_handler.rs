//! 模型管理API处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::application::services::ModelService;
use crate::common::error::*;
use crate::common::types::*;
use crate::domain::model::*;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub model_service: Arc<ModelService>,
}

/// 模型注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterModelRequest {
    pub name: String,
    pub model_type: ModelType,
    pub backend: String,
    pub model_path: String,
    pub config: Option<serde_json::Value>,
}

/// 模型注册响应
#[derive(Debug, Serialize)]
pub struct RegisterModelResponse {
    pub model_id: ModelId,
    pub status: String,
    pub message: String,
}

/// 模型列表响应
#[derive(Debug, Serialize)]
pub struct ListModelsResponse {
    pub models: Vec<ModelInfo>,
    pub total: usize,
}

/// 创建模型路由
pub fn create_model_routes() -> Router<AppState> {
    Router::new()
        .route("/models", post(register_model))
        .route("/models", get(list_models))
        .route("/models/:model_id", get(get_model))
        .route("/models/:model_id", delete(unregister_model))
}

/// 注册模型
pub async fn register_model(
    State(state): State<AppState>,
    Json(request): Json<RegisterModelRequest>,
) -> Result<Json<RegisterModelResponse>, (StatusCode, Json<serde_json::Value>)> {
    info!("Registering model: {}", request.name);

    let model_config = ModelConfig {
        model_path: request.model_path,
        config_path: None,
        tokenizer_path: None,
        backend: request.backend,
        device: DeviceConfig {
            device_type: DeviceType::CUDA,
            device_ids: vec![0],
            memory_limit_mb: None,
            mixed_precision: false,
        },
        optimization: OptimizationConfig {
            kv_cache: true,
            quantization: None,
            graph_optimization: true,
            inference_parallelism: 1,
            memory_optimization: MemoryOptimization::Medium,
        },
        batch_config: BatchConfig::default(),
        custom_params: request
            .config
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
            .into_iter()
            .collect(),
    };

    match state
        .model_service
        .register_model(request.name.clone(), request.model_type, model_config)
        .await
    {
        Ok(model_id) => {
            let response = RegisterModelResponse {
                model_id,
                status: "success".to_string(),
                message: format!("Model '{}' registered successfully", request.name),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to register model: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({
                    "error": e.error_code(),
                    "message": e.to_string()
                })),
            ))
        }
    }
}

/// 获取模型列表
pub async fn list_models(
    State(state): State<AppState>,
) -> Result<Json<ListModelsResponse>, (StatusCode, Json<serde_json::Value>)> {
    match state.model_service.list_models().await {
        Ok(models) => {
            let response = ListModelsResponse {
                total: models.len(),
                models,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to list models: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({
                    "error": e.error_code(),
                    "message": e.to_string()
                })),
            ))
        }
    }
}

/// 获取单个模型信息
pub async fn get_model(
    State(state): State<AppState>,
    Path(model_id): Path<ModelId>,
) -> Result<Json<ModelInfo>, (StatusCode, Json<serde_json::Value>)> {
    match state.model_service.get_model_info(&model_id).await {
        Ok(model_info) => Ok(Json(model_info)),
        Err(e) => {
            error!("Failed to get model {}: {}", model_id, e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({
                    "error": e.error_code(),
                    "message": e.to_string()
                })),
            ))
        }
    }
}

/// 注销模型
pub async fn unregister_model(
    State(state): State<AppState>,
    Path(model_id): Path<ModelId>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    info!("Unregistering model: {}", model_id);

    match state.model_service.unregister_model(&model_id).await {
        Ok(()) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": format!("Model '{}' unregistered successfully", model_id)
        }))),
        Err(e) => {
            error!("Failed to unregister model {}: {}", model_id, e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({
                    "error": e.error_code(),
                    "message": e.to_string()
                })),
            ))
        }
    }
}
