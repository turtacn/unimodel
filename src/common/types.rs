//! 通用类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 模型ID类型
pub type ModelId = String;

/// 请求ID类型
pub type RequestId = String;

/// 节点ID类型
pub type NodeId = String;

/// 插件ID类型
pub type PluginId = String;

/// 生成新的请求ID
pub fn new_request_id() -> RequestId {
    Uuid::new_v4().to_string()
}

/// 生成新的模型ID
pub fn new_model_id() -> ModelId {
    Uuid::new_v4().to_string()
}

/// 推理输入数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum InputData {
    /// 文本输入
    Text(String),
    /// 二进制数据（如图像、音频）
    Binary(Vec<u8>),
    /// JSON数据
    Json(serde_json::Value),
    /// 多模态输入
    Multimodal(HashMap<String, InputData>),
}

/// 推理输出数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum OutputData {
    /// 文本输出
    Text(String),
    /// 二进制数据
    Binary(Vec<u8>),
    /// JSON数据
    Json(serde_json::Value),
    /// 多模态输出
    Multimodal(HashMap<String, OutputData>),
}

/// 推理参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PredictionParameters {
    /// 最大生成token数（针对LLM）
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// top_p参数
    pub top_p: Option<f32>,
    /// top_k参数
    pub top_k: Option<u32>,
    /// 是否流式输出
    pub stream: Option<bool>,
    /// 自定义参数
    pub custom: HashMap<String, serde_json::Value>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 请求ID
    pub request_id: RequestId,
    /// 处理开始时间
    pub start_time: DateTime<Utc>,
    /// 处理结束时间
    pub end_time: DateTime<Utc>,
    /// 总延迟（毫秒）
    pub total_latency_ms: u64,
    /// 推理延迟（毫秒）
    pub inference_latency_ms: u64,
    /// 队列等待时间（毫秒）
    pub queue_wait_ms: u64,
    /// 预处理时间（毫秒）
    pub preprocessing_ms: u64,
    /// 后处理时间（毫秒）
    pub postprocessing_ms: u64,
    /// 生成的token数量（针对LLM）
    pub tokens_generated: Option<u32>,
    /// 输入token数量（针对LLM）
    pub tokens_input: Option<u32>,
    /// 吞吐量（tokens/sec）
    pub throughput_tokens_per_sec: Option<f64>,
    /// 批处理大小
    pub batch_size: u32,
    /// GPU使用率
    pub gpu_utilization: Option<f32>,
    /// 内存使用量（MB）
    pub memory_usage_mb: Option<u64>,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知状态
    Unknown,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU使用率（0.0-1.0）
    pub cpu_usage: f32,
    /// 内存使用量（字节）
    pub memory_usage_bytes: u64,
    /// 总内存量（字节）
    pub total_memory_bytes: u64,
    /// GPU使用情况
    pub gpu_usage: Vec<GpuUsage>,
    /// 磁盘使用量（字节）
    pub disk_usage_bytes: u64,
    /// 网络IO
    pub network_io: NetworkIO,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// GPU使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUsage {
    /// GPU设备ID
    pub device_id: u32,
    /// GPU使用率（0.0-1.0）
    pub utilization: f32,
    /// 显存使用量（字节）
    pub memory_used_bytes: u64,
    /// 总显存量（字节）
    pub memory_total_bytes: u64,
    /// GPU温度（摄氏度）
    pub temperature_celsius: Option<f32>,
    /// 功耗（瓦特）
    pub power_usage_watts: Option<f32>,
}

/// 网络IO统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 发送包数
    pub packets_sent: u64,
}

/// 批处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// 最大批处理大小
    pub max_batch_size: u32,
    /// 最大等待时间（毫秒）
    pub max_wait_time_ms: u64,
    /// 是否启用动态填充
    pub dynamic_padding: bool,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            max_wait_time_ms: 50,
            dynamic_padding: true,
            timeout_ms: 30000,
        }
    }
}