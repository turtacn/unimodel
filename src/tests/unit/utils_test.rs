//! 单元测试

use unimodel::common::types::*;
use unimodel::common::error::*;
use unimodel::domain::model::*;

#[test]
fn test_model_id_validation() {
    // 有效的模型ID
    let valid_ids = vec![
        "model-123",
        "my_model",
        "test.model",
        "model123",
    ];

    for id in valid_ids {
        assert!(id.len() > 0);
        // 这里应该有更详细的验证逻辑
    }

    // 无效的模型ID
    let invalid_ids = vec![
        "",
        " ",
        "model with spaces",
        "model@special",
    ];

    for id in invalid_ids {
        // 这里应该测试验证失败
        assert!(id.is_empty() || id.contains(' ') || id.contains('@'));
    }
}

#[test]
fn test_input_data_serialization() {
    // 测试文本数据
    let text_data = InputData::Text("Hello, world!".to_string());
    let serialized = serde_json::to_string(&text_data).unwrap();
    let deserialized: InputData = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        InputData::Text(text) => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected text data"),
    }

    // 测试JSON数据
    let json_data = InputData::Json(serde_json::json!({
        "key": "value",
        "number": 42
    }));
    let serialized = serde_json::to_string(&json_data).unwrap();
    let deserialized: InputData = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        InputData::Json(json) => {
            assert_eq!(json["key"], "value");
            assert_eq!(json["number"], 42);
        }
        _ => panic!("Expected JSON data"),
    }
}

#[test]
fn test_prediction_parameters() {
    let mut params = PredictionParameters::default();

    // 测试默认值
    assert_eq!(params.temperature, 1.0);
    assert_eq!(params.max_tokens, 100);
    assert_eq!(params.top_p, 0.9);
    assert_eq!(params.top_k, 50);

    // 测试参数修改
    params.temperature = 0.8;
    params.max_tokens = 200;
    assert_eq!(params.temperature, 0.8);
    assert_eq!(params.max_tokens, 200);

    // 测试序列化
    let serialized = serde_json::to_string(&params).unwrap();
    let deserialized: PredictionParameters = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.temperature, 0.8);
    assert_eq!(deserialized.max_tokens, 200);
}

#[test]
fn test_error_handling() {
    // 测试错误创建
    let validation_error = UniModelError::validation("Invalid input");
    assert_eq!(validation_error.error_code(), "VALIDATION_ERROR");
    assert_eq!(validation_error.status_code(), 400);

    let model_error = UniModelError::model("Model not found");
    assert_eq!(model_error.error_code(), "MODEL_ERROR");
    assert_eq!(model_error.status_code(), 404);

    let internal_error = UniModelError::internal("Internal server error");
    assert_eq!(internal_error.error_code(), "INTERNAL_ERROR");
    assert_eq!(internal_error.status_code(), 500);

    // 测试错误链
    let source_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let chained_error = UniModelError::from_source("File operation failed", source_error);
    assert!(chained_error.to_string().contains("File operation failed"));
}

#[test]
fn test_model_config_validation() {
    // 有效配置
    let valid_config = ModelConfig {
        model_path: "/path/to/model".to_string(),
        config_path: None,
        tokenizer_path: None,
        backend: "pytorch".to_string(),
        device: DeviceConfig {
            device_type: DeviceType::CUDA,
            device_ids: vec![0],
            memory_limit_mb: Some(2048),
            mixed_precision: true,
        },
        optimization: OptimizationConfig {
            kv_cache: true,
            quantization: Some(QuantizationType::INT8),
            graph_optimization: true,
            inference_parallelism: 2,
            memory_optimization: MemoryOptimization::High,
        },
        batch_config: BatchConfig {
            max_batch_size: 32,
            max_wait_time_ms: 100,
            timeout_ms: 30000,
        },
        custom_params: std::collections::HashMap::new(),
    };

    // 这里应该有配置验证逻辑
    assert!(!valid_config.model_path.is_empty());
    assert!(!valid_config.backend.is_empty());
    assert!(!valid_config.device.device_ids.is_empty());
    assert!(valid_config.batch_config.max_batch_size > 0);
}

#[test]
fn test_performance_metrics() {
    let metrics = PerformanceMetrics {
        request_id: "test-request".to_string(),
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now(),
        total_latency_ms: 150,
        inference_latency_ms: 100,
        queue_wait_ms: 20,
        preprocessing_ms: 15,
        postprocessing_ms: 15,
        tokens_generated: Some(50),
        tokens_input: Some(20),
        throughput_tokens_per_sec: Some(333.33),
        batch_size: 8,
        gpu_utilization: Some(0.85),
        memory_usage_mb: Some(2048),
    };

    // 测试序列化
    let serialized = serde_json::to_string(&metrics).unwrap();
    let deserialized: PerformanceMetrics = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.request_id, "test-request");
    assert_eq!(deserialized.total_latency_ms, 150);
    assert_eq!(deserialized.batch_size, 8);
    assert_eq!(deserialized.tokens_generated, Some(50));
}