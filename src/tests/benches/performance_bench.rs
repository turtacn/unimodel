//! 性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use std::time::Duration;

use unimodel::prelude::*;
use unimodel::infrastructure::configuration::Config;
use unimodel::domain::service::batch_processor::BatchProcessor;

fn benchmark_batch_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = Config::default();
    let batch_processor = rt.block_on(async {
        let processor = BatchProcessor::new(&config).await.unwrap();
        processor.start().await.unwrap();
        processor
    });

    let mut group = c.benchmark_group("batch_processing");

    for batch_size in [1, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("submit_requests", batch_size),
            batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let mut tasks = Vec::new();

                    for i in 0..*batch_size {
                        let processor = batch_processor.clone();
                        let model_id = format!("test-model-{}", i % 4);
                        let input = InputData::Text(format!("Test input {}", i));
                        let parameters = PredictionParameters::default();

                        let task = tokio::spawn(async move {
                            processor.submit_request(model_id, input, parameters).await
                        });
                        tasks.push(task);
                    }

                    // 等待所有任务完成
                    for task in tasks {
                        let _ = task.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // 测试输入数据序列化
    let text_input = InputData::Text("Hello, world!".repeat(100));
    let json_input = InputData::Json(serde_json::json!({
        "text": "Hello, world!".repeat(100),
        "metadata": {
            "timestamp": chrono::Utc::now(),
            "version": "1.0.0"
        }
    }));

    group.bench_function("text_serialize", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&text_input).unwrap();
            black_box(serialized);
        });
    });

    group.bench_function("text_deserialize", |b| {
        let serialized = serde_json::to_string(&text_input).unwrap();
        b.iter(|| {
            let deserialized: InputData = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized);
        });
    });

    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&json_input).unwrap();
            black_box(serialized);
        });
    });

    group.bench_function("json_deserialize", |b| {
        let serialized = serde_json::to_string(&json_input).unwrap();
        b.iter(|| {
            let deserialized: InputData = serde_json::from_str(&serialized).unwrap();
            black_box(deserialized);
        });
    });

    group.finish();
}

fn benchmark_model_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = Config::default();
    let model_manager = rt.block_on(async {
        ModelManager::new(&config).await.unwrap()
    });

    let mut group = c.benchmark_group("model_operations");

    // 基准测试模型注册
    group.bench_function("register_model", |b| {
        b.to_async(&rt).iter(|| async {
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

            let model_id = model_manager.register_model(
                format!("test-model-{}", rand::random::<u32>()),
                ModelType::TextGeneration,
                model_config,
            ).await.unwrap();

            // 清理
            let _ = model_manager.unregister_model(&model_id).await;

            black_box(model_id);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_batch_processing,
    benchmark_serialization,
    benchmark_model_operations
);
criterion_main!(benches);