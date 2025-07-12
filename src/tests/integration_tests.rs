//! 集成测试

use std::time::Duration;
use tokio::time::sleep;
use serde_json::json;

use unimodel::prelude::*;
use unimodel::infrastructure::configuration::Config;
use unimodel::domain::service::ModelManager;
use unimodel::application::services::{ModelService, PredictionService};

#[tokio::test]
async fn test_model_lifecycle() {
    // 创建测试配置
    let config = Config::default();
    let model_manager = Arc::new(ModelManager::new(&config).await.unwrap());
    let model_service = ModelService::new(model_manager.clone());

    // 注册模型
    let model_config = ModelConfig {
        model_path: "test_model.onnx".to_string(),
        config_path: None,
        tokenizer_path: None,
        backend: "onnx".to_string(),
        device: DeviceConfig {
            device_type: DeviceType::CPU,
            device_ids: vec![0],
            memory_limit_mb: Some(1024),
            mixed_precision: false,
        },
        optimization: OptimizationConfig {
            kv_cache: false,
            quantization: None,
            graph_optimization: true,
            inference_parallelism: 1,
            memory_optimization: MemoryOptimization::Low,
        },
        batch_config: BatchConfig::default(),
        custom_params: std::collections::HashMap::new(),
    };

    let model_id = model_service.register_model(
        "test-model".to_string(),
        ModelType::ImageClassification,
        model_config,
    ).await.unwrap();

    // 等待模型加载
    sleep(Duration::from_millis(100)).await;

    // 获取模型信息
    let model_info = model_service.get_model_info(&model_id).await.unwrap();
    assert_eq!(model_info.name, "test-model");
    assert_eq!(model_info.model_type, ModelType::ImageClassification);

    // 获取模型列表
    let models = model_service.list_models().await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].model_id, model_id);

    // 注销模型
    model_service.unregister_model(&model_id).await.unwrap();

    // 确认模型已注销
    let models = model_service.list_models().await.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_prediction_service() {
    // 创建测试服务
    let config = Config::default();
    let model_manager = Arc::new(ModelManager::new(&config).await.unwrap());
    let batch_processor = Arc::new(BatchProcessor::new(&config).await.unwrap());
    batch_processor.start().await.unwrap();

    let model_service = ModelService::new(model_manager.clone());
    let prediction_service = PredictionService::new(model_manager, batch_processor);

    // 注册测试模型
    let model_config = ModelConfig {
        model_path: "test_model.onnx".to_string(),
        config_path: None,
        tokenizer_path: None,
        backend: "onnx".to_string(),
        device: DeviceConfig {
            device_type: DeviceType::CPU,
            device_ids: vec![0],
            memory_limit_mb: Some(1024),
            mixed_precision: false,
        },
        optimization: OptimizationConfig {
            kv_cache: false,
            quantization: None,
            graph_optimization: true,
            inference_parallelism: 1,
            memory_optimization: MemoryOptimization::Low,
        },
        batch_config: BatchConfig::default(),
        custom_params: std::collections::HashMap::new(),
    };

    let model_id = model_service.register_model(
        "test-model".to_string(),
        ModelType::TextGeneration,
        model_config,
    ).await.unwrap();

    // 等待模型加载
    sleep(Duration::from_millis(100)).await;

    // 执行单次推理
    let input = InputData::Text("Hello, world!".to_string());
    let parameters = PredictionParameters::default();

    let response = prediction_service.predict(
        model_id.clone(),
        input,
        parameters,
    ).await.unwrap();

    assert_eq!(response.model_id, model_id);
    assert!(response.metrics.total_latency_ms > 0);

    // 执行批量推理
    let inputs = vec![
        InputData::Text("Hello".to_string()),
        InputData::Text("World".to_string()),
    ];
    let parameters = PredictionParameters::default();

    let responses = prediction_service.batch_predict(
        model_id.clone(),
        inputs,
        parameters,
    ).await.unwrap();

    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0].model_id, model_id);
    assert_eq!(responses[1].model_id, model_id);
}

#[tokio::test]
async fn test_batch_processing() {
    let config = Config::default();
    let batch_processor = BatchProcessor::new(&config).await.unwrap();
    batch_processor.start().await.unwrap();

    let model_id = "test-model".to_string();
    let input = InputData::Text("Test input".to_string());
    let parameters = PredictionParameters::default();

    // 提交多个请求
    let mut tasks = Vec::new();
    for i in 0..5 {
        let processor = batch_processor.clone();
        let model_id = model_id.clone();
        let input = InputData::Text(format!("Test input {}", i));
        let parameters = parameters.clone();

        let task = tokio::spawn(async move {
            processor.submit_request(model_id, input, parameters).await
        });
        tasks.push(task);
    }

    // 等待所有请求完成
    for task in tasks {
        let response = task.await.unwrap().unwrap();
        assert_eq!(response.model_id, model_id);
        assert!(response.metrics.total_latency_ms > 0);
    }

    // 获取批处理统计
    let stats = batch_processor.get_batch_stats().await;
    assert!(stats.is_running);

    batch_processor.stop().await.unwrap();
}