//! 批处理器服务

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

use crate::common::error::*;
use crate::common::types::*;
use crate::domain::model::*;
use crate::infrastructure::configuration::Config;

/// 批处理请求
#[derive(Debug)]
pub struct BatchRequest {
    pub request_id:      RequestId,                  // 请求 ID
    pub model_id:        ModelId,                    // 模型 ID
    pub input:           InputData,                  // 输入数据
    pub parameters:      PredictionParameters,       // 预测参数
    pub response_sender: oneshot::Sender<Result<PredictionResponse>>, // 响应通道
    pub submitted_at:    Instant,                    // 提交时间
}

/// 批处理组
#[derive(Debug)]
pub struct BatchGroup {
    pub model_id:   ModelId,         // 模型 ID
    pub requests:   Vec<BatchRequest>, // 请求列表
    pub created_at: Instant,         // 创建时间
}

/// 批处理器
#[derive(Debug)]
pub struct BatchProcessor {
    config:           Arc<Config>,
    pending_requests: Arc<Mutex<VecDeque<BatchRequest>>>,
    request_sender:   mpsc::UnboundedSender<BatchRequest>,
    request_receiver: Arc<Mutex<mpsc::UnboundedReceiver<BatchRequest>>>,
    running:          Arc<RwLock<bool>>,
}

impl BatchProcessor {
    /// 创建新的批处理器
    pub async fn new(config: &Config) -> Result<Self> {
        let (request_sender, request_receiver) = mpsc::unbounded_channel();
        Ok(Self {
            config: Arc::new(config.clone()),
            pending_requests: Arc::new(Mutex::new(VecDeque::new())),
            request_sender,
            request_receiver: Arc::new(Mutex::new(request_receiver)),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 启动批处理器
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(UniModelError::internal("BatchProcessor already running"));
            }
            *running = true;
        }

        info!("Starting batch processor");

        let processor = self.clone();
        tokio::spawn(async move {
            processor.run_batch_loop().await;
        });

        Ok(())
    }

    /// 停止批处理器
    pub async fn stop(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        info!("Stopping batch processor");
        Ok(())
    }

    /// 提交批处理请求
    pub async fn submit_request(
        &self,
        model_id: ModelId,
        input: InputData,
        parameters: PredictionParameters,
    ) -> Result<PredictionResponse> {
        let request_id = new_request_id();
        let (response_sender, response_receiver) = oneshot::channel();

        let batch_request = BatchRequest {
            request_id: request_id.clone(),
            model_id,
            input,
            parameters,
            response_sender,
            submitted_at: Instant::now(),
        };

        self.request_sender
            .send(batch_request)
            .map_err(|_| UniModelError::internal("Failed to send batch request"))?;

        let timeout_duration = Duration::from_millis(
            self.config.engine.batch_config.timeout_ms,
        );

        match timeout(timeout_duration, response_receiver).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => Err(UniModelError::internal("Response channel closed")),
            Err(_) => Err(UniModelError::internal("Request timeout")),
        }
    }

    /// 批处理主循环
    async fn run_batch_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_millis(10));

        while *self.running.read().await {
            interval.tick().await;

            self.collect_new_requests().await;

            if let Err(e) = self.process_batches().await {
                error!("Error processing batches: {}", e);
            }
        }

        info!("Batch processing loop stopped");
    }

    /// 收集新请求
    async fn collect_new_requests(&self) {
        let mut receiver = self.request_receiver.lock().await;
        let mut pending = self.pending_requests.lock().await;

        while let Ok(request) = receiver.try_recv() {
            pending.push_back(request);
        }
    }

    /// 处理所有批次
    async fn process_batches(&self) -> Result<()> {
        let mut pending = self.pending_requests.lock().await;

        if pending.is_empty() {
            return Ok(());
        }

        let mut groups = std::collections::HashMap::new();
        let mut expired_requests = Vec::new();

        let now = Instant::now();
        let max_wait_time = Duration::from_millis(
            self.config.engine.batch_config.max_wait_time_ms,
        );

        while let Some(request) = pending.pop_front() {
            if now.duration_since(request.submitted_at) > max_wait_time {
                expired_requests.push(request);
                continue;
            }

            groups.entry(request.model_id.clone())
                .or_insert_with(Vec::new)
                .push(request);
        }

        for request in expired_requests {
            let _ = request
                .response_sender
                .send(Err(UniModelError::internal("Request expired")));
        }

        for (model_id, requests) in groups {
            if let Err(e) = self.process_model_group(model_id, requests).await {
                error!("Error processing model group: {}", e);
            }
        }

        Ok(())
    }

    /// 处理模型分组
    async fn process_model_group(
        &self,
        model_id: ModelId,
        mut requests: Vec<BatchRequest>,
    ) -> Result<()> {
        let max_batch_size = self.config.engine.batch_config.max_batch_size as usize;

        while !requests.is_empty() {
            let batch_size = std::cmp::min(requests.len(), max_batch_size);
            let batch_requests = requests.drain(0..batch_size).collect();

            let batch_group = BatchGroup {
                model_id: model_id.clone(),
                requests: batch_requests,
                created_at: Instant::now(),
            };

            let processor = self.clone();
            tokio::spawn(async move {
                if let Err(e) = processor.execute_batch(batch_group).await {
                    error!("Error executing batch: {}", e);
                }
            });
        }

        Ok(())
    }

    /// 执行批次推理
    async fn execute_batch(&self, batch_group: BatchGroup) -> Result<()> {
        debug!(
            "Executing batch for model {} with {} requests",
            batch_group.model_id,
            batch_group.requests.len()
        );

        let start_time = Instant::now();

        let batch_inputs: Vec<InputData> = batch_group
            .requests
            .iter()
            .map(|req| req.input.clone())
            .collect();

        sleep(Duration::from_millis(50)).await;

        let batch_results = self.simulate_batch_inference(&batch_inputs).await?;
        let end_time = Instant::now();
        let total_latency = end_time.duration_since(start_time);

        for (i, request) in batch_group.requests.into_iter().enumerate() {
            let response = PredictionResponse {
                request_id: request.request_id.clone(),
                model_id: batch_group.model_id.clone(),
                output: batch_results
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| OutputData::Text("Error".to_string())),
                metadata: ResponseMetadata {
                    model_version: "1.0.0".to_string(),
                    backend: "simulated".to_string(),
                    custom_metadata: std::collections::HashMap::new(),
                },
                metrics: PerformanceMetrics {
                    request_id: request.request_id.clone(),
                    start_time: chrono::Utc::now()
                        - chrono::Duration::milliseconds(total_latency.as_millis() as i64),
                    end_time: chrono::Utc::now(),
                    total_latency_ms: total_latency.as_millis() as u64,
                    inference_latency_ms: total_latency.as_millis() as u64,
                    queue_wait_ms: request.submitted_at.elapsed().as_millis() as u64,
                    preprocessing_ms: 5,
                    postprocessing_ms: 5,
                    tokens_generated: None,
                    tokens_input: None,
                    throughput_tokens_per_sec: None,
                    batch_size: batch_inputs.len() as u32,
                    gpu_utilization: Some(0.75),
                    memory_usage_mb: Some(1024),
                },
                timestamp: chrono::Utc::now(),
            };

            let _ = request.response_sender.send(Ok(response));
        }

        debug!("Batch execution completed in {:?}", total_latency);
        Ok(())
    }

    /// 模拟推理逻辑
    async fn simulate_batch_inference(&self, inputs: &[InputData]) -> Result<Vec<OutputData>> {
        let mut results = Vec::new();

        for input in inputs {
            let output = match input {
                InputData::Text(text) => OutputData::Text(format!("Processed: {}", text)),
                InputData::Binary(data) => OutputData::Binary(data.clone()),
                InputData::Json(json) => OutputData::Json(json.clone()),
                InputData::Multimodal(map) => OutputData::Multimodal(map.clone()),
            };
            results.push(output);
        }

        Ok(results)
    }

    /// 获取状态信息
    pub async fn get_batch_stats(&self) -> BatchStats {
        let pending = self.pending_requests.lock().await;

        BatchStats {
            pending_requests: pending.len(),
            is_running: *self.running.read().await,
            total_processed: 0,
            avg_batch_size: 0.0,
            avg_wait_time_ms: 0.0,
        }
    }
}

// 为 BatchProcessor 实现 Clone
impl Clone for BatchProcessor {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            pending_requests: Arc::clone(&self.pending_requests),
            request_sender: self.request_sender.clone(),
            request_receiver: Arc::clone(&self.request_receiver),
            running: Arc::clone(&self.running),
        }
    }
}

/// 批处理统计信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchStats {
    pub pending_requests: usize,
    pub is_running: bool,
    pub total_processed: u64,
    pub avg_batch_size: f64,
    pub avg_wait_time_ms: f64,
}

/// 响应元数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseMetadata {
    pub model_version: String,
    pub backend: String,
    pub custom_metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 推理响应
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PredictionResponse {
    pub request_id: RequestId,
    pub model_id: ModelId,
    pub output: OutputData,
    pub metadata: ResponseMetadata,
    pub metrics: PerformanceMetrics,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
