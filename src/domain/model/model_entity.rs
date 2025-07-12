//! 模型实体定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::common::error::*;
use crate::common::types::*;

/// 模型状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelStatus {
    /// 初始化中
    Initializing,
    /// 加载中
    Loading,
    /// 就绪
    Ready,
    /// 运行中
    Running,
    /// 错误
    Error(String),
    /// 已卸载
    Unloaded,
}

/// 模型类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    /// 大语言模型
    LLM,
    /// 计算机视觉模型
    CV,
    /// 音频模型
    Audio,
    /// 多模态模型
    Multimodal,
    /// 传统机器学习模型
    ML,
    /// 自定义模型
    Custom(String),
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 模型文件路径
    pub model_path: String,
    /// 配置文件路径
    pub config_path: Option<String>,
    /// 分词器路径
    pub tokenizer_path: Option<String>,
    /// 推理后端
    pub backend: String,
    /// 设备配置
    pub device: DeviceConfig,
    /// 优化配置
    pub optimization: OptimizationConfig,
    /// 批处理配置
    pub batch_config: BatchConfig,
    /// 自定义参数
    pub custom_params: HashMap<String, serde_json::Value>,
}

/// 设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// 设备类型
    pub device_type: DeviceType,
    /// 设备ID列表
    pub device_ids: Vec<u32>,
    /// 内存限制（MB）
    pub memory_limit_mb: Option<u64>,
    /// 是否启用混合精度
    pub mixed_precision: bool,
}

/// 设备类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    /// CPU
    CPU,
    /// CUDA GPU
    CUDA,
    /// Metal GPU (Apple)
    Metal,
    /// OpenCL
    OpenCL,
    /// NPU
    NPU,
}

/// 优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// 是否启用KV缓存
    pub kv_cache: bool,
    /// 是否启用量化
    pub quantization: Option<QuantizationType>,
    /// 是否启用图优化
    pub graph_optimization: bool,
    /// 推理并行度
    pub inference_parallelism: u32,
    /// 内存优化级别
    pub memory_optimization: MemoryOptimization,
}

/// 量化类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantizationType {
    /// INT8量化
    INT8,
    /// INT4量化
    INT4,
    /// FP16量化
    FP16,
    /// 动态量化
    Dynamic,
}

/// 内存优化级别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryOptimization {
    /// 无优化
    None,
    /// 低内存优化
    Low,
    /// 中等内存优化
    Medium,
    /// 高内存优化
    High,
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// 模型作者
    pub author: Option<String>,
    /// 模型描述
    pub description: Option<String>,
    /// 模型许可证
    pub license: Option<String>,
    /// 模型标签
    pub tags: Vec<String>,
    /// 模型版本
    pub version: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 自定义元数据
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

/// 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型ID
    pub id: ModelId,
    /// 模型名称
    pub name: String,
    /// 模型类型
    pub model_type: ModelType,
    /// 模型状态
    pub status: ModelStatus,
    /// 模型配置
    pub config: ModelConfig,
    /// 模型元数据
    pub metadata: ModelMetadata,
    /// 资源使用情况
    pub resource_usage: Option<ResourceUsage>,
    /// 性能统计
    pub performance_stats: PerformanceStats,
    /// 健康状态
    pub health_status: HealthStatus,
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均延迟（毫秒）
    pub avg_latency_ms: f64,
    /// P95延迟（毫秒）
    pub p95_latency_ms: f64,
    /// P99延迟（毫秒）
    pub p99_latency_ms: f64,
    /// 平均吞吐量（请求/秒）
    pub avg_throughput_rps: f64,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 模型实体
#[derive(Debug, Clone)]
pub struct Model {
    /// 模型信息
    pub info: ModelInfo,
    /// 模型实例句柄
    pub instance: Option<ModelInstance>,
    /// 是否为热模型
    pub is_warm: bool,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
    /// 加载时间
    pub loaded_at: Option<DateTime<Utc>>,
}

/// 模型实例句柄
#[derive(Debug, Clone)]
pub struct ModelInstance {
    /// 实例ID
    pub id: String,
    /// 插件ID
    pub plugin_id: PluginId,
    /// 内部句柄
    pub handle: u64,
    /// 是否支持批处理
    pub supports_batching: bool,
    /// 最大批处理大小
    pub max_batch_size: u32,
}

impl Model {
    /// 创建新模型
    pub fn new(id: ModelId, name: String, model_type: ModelType, config: ModelConfig) -> Self {
        let now = Utc::now();
        let metadata = ModelMetadata {
            author: None,
            description: None,
            license: None,
            tags: vec![],
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            custom_metadata: HashMap::new(),
        };

        let performance_stats = PerformanceStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            avg_throughput_rps: 0.0,
            last_updated: now,
        };

        let info = ModelInfo {
            id,
            name,
            model_type,
            status: ModelStatus::Initializing,
            config,
            metadata,
            resource_usage: None,
            performance_stats,
            health_status: HealthStatus::Unknown,
        };

        Self {
            info,
            instance: None,
            is_warm: false,
            last_accessed: now,
            loaded_at: None,
        }
    }

    /// 更新模型状态
    pub fn update_status(&mut self, status: ModelStatus) {
        self.info.status = status;
        self.info.metadata.updated_at = Utc::now();

        if matches!(self.info.status, ModelStatus::Ready | ModelStatus::Running) {
            self.loaded_at = Some(Utc::now());
        }
    }

    /// 更新最后访问时间
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// 检查模型是否已加载
    pub fn is_loaded(&self) -> bool {
        matches!(self.info.status, ModelStatus::Ready | ModelStatus::Running)
    }

    /// 检查模型是否健康
    pub fn is_healthy(&self) -> bool {
        self.info.health_status == HealthStatus::Healthy
    }

    /// 更新性能统计
    pub fn update_performance_stats(&mut self, latency_ms: u64, success: bool) {
        let stats = &mut self.info.performance_stats;
        stats.total_requests += 1;

        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }

        // 更新平均延迟（简化的滑动平均）
        let alpha = 0.1; // 平滑因子
        stats.avg_latency_ms =
            stats.avg_latency_ms * (1.0 - alpha) + latency_ms as f64 * alpha;

        stats.last_updated = Utc::now();
    }
}
