//! 配置管理模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

use crate::common::types::*;
use crate::common::error::*;

/// 主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub engine: EngineConfig,
    pub plugins: PluginConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub grpc_port: u16,
    pub max_connections: u32,
    pub request_timeout_secs: u64,
    pub enable_tls: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub worker_threads: Option<usize>,
}

/// 引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub max_models: u32,
    pub default_batch_size: u32,
    pub max_batch_wait_ms: u64,
    pub batch_config: BatchConfig,
    pub gpu: GpuConfig,
    pub memory: MemoryConfig,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_dir: String,
    pub enabled_plugins: Vec<String>,
    pub plugin_configs: HashMap<String, serde_json::Value>,
    pub plugin_timeout_secs: u64,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub jaeger_enabled: bool,
    pub jaeger_endpoint: Option<String>,
    pub health_check_interval_secs: u64,
    pub metrics_collection_interval_secs: u64,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub auth_enabled: bool,
    pub jwt_secret: Option<String>,
    pub api_keys: Vec<String>,
    pub cors_enabled: bool,
    pub cors_allowed_origins: Vec<String>,
    pub rate_limiting: RateLimitConfig,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub model_storage_path: String,
    pub cache_storage_path: String,
    pub log_storage_path: String,
    pub max_storage_gb: u64,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub console_output: bool,
    pub file_output: bool,
    pub file_path: Option<String>,
    pub rotation_size_mb: u64,
    pub retention_count: u32,
}

/// GPU配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub device_ids: Vec<u32>,
    pub memory_fraction: f32,
    pub enable_pooling: bool,
    pub enable_p2p: bool,
}

/// 内存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_memory_gb: f32,
    pub enable_mmap: bool,
    pub cache_size_mb: u64,
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Config {
    /// 从文件加载配置
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).await
            .map_err(|e| UniModelError::config(format!("Failed to read config file: {}", e)))?;

        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| UniModelError::config(format!("Failed to parse config: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("UNIMODEL_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("UNIMODEL_PORT") {
            config.server.port = port.parse()
                .map_err(|_| UniModelError::config("Invalid UNIMODEL_PORT"))?;
        }
        if let Ok(grpc_port) = std::env::var("UNIMODEL_GRPC_PORT") {
            config.server.grpc_port = grpc_port.parse()
                .map_err(|_| UniModelError::config("Invalid UNIMODEL_GRPC_PORT"))?;
        }
        if let Ok(max_models) = std::env::var("UNIMODEL_MAX_MODELS") {
            config.engine.max_models = max_models.parse()
                .map_err(|_| UniModelError::config("Invalid UNIMODEL_MAX_MODELS"))?;
        }
        if let Ok(device_ids) = std::env::var("UNIMODEL_GPU_DEVICES") {
            config.engine.gpu.device_ids = device_ids
                .split(',')
                .map(|s| s.trim().parse())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| UniModelError::config("Invalid UNIMODEL_GPU_DEVICES"))?;
        }
        if let Ok(plugin_dir) = std::env::var("UNIMODEL_PLUGIN_DIR") {
            config.plugins.plugin_dir = plugin_dir;
        }

        config.validate()?;
        Ok(config)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.server.port == 0 || self.server.port > 65535 {
            return Err(UniModelError::config("Invalid server port"));
        }
        if self.server.grpc_port == 0 || self.server.grpc_port > 65535 {
            return Err(UniModelError::config("Invalid gRPC port"));
        }
        if self.server.port == self.server.grpc_port {
            return Err(UniModelError::config("HTTP and gRPC ports cannot be the same"));
        }
        if self.engine.batch_config.max_batch_size == 0 {
            return Err(UniModelError::config("Max batch size must be greater than 0"));
        }
        if self.engine.batch_config.max_wait_time_ms == 0 {
            return Err(UniModelError::config("Max wait time must be greater than 0"));
        }
        if self.engine.gpu.device_ids.is_empty() {
            return Err(UniModelError::config("At least one GPU device must be specified"));
        }
        if self.engine.gpu.memory_fraction <= 0.0 || self.engine.gpu.memory_fraction > 1.0 {
            return Err(UniModelError::config("GPU memory fraction must be between 0 and 1"));
        }
        if self.storage.model_storage_path.is_empty() {
            return Err(UniModelError::config("Model storage path cannot be empty"));
        }
        if self.server.enable_tls {
            if self.server.tls_cert_path.is_none() || self.server.tls_key_path.is_none() {
                return Err(UniModelError::config("TLS cert and key paths must be provided when TLS is enabled"));
            }
        }
        Ok(())
    }

    /// 合并配置
    pub fn merge(mut self, other: Config) -> Self {
        self.server = other.server;
        self.engine = other.engine;
        self.plugins = other.plugins;
        self.monitoring = other.monitoring;
        self.security = other.security;
        self.storage = other.storage;
        self.logging = other.logging;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8000,
                grpc_port: 9000,
                max_connections: 1000,
                request_timeout_secs: 300,
                enable_tls: false,
                tls_cert_path: None,
                tls_key_path: None,
                worker_threads: None,
            },
            engine: EngineConfig {
                max_models: 10,
                default_batch_size: 8,
                max_batch_wait_ms: 50,
                batch_config: BatchConfig::default(),
                gpu: GpuConfig {
                    device_ids: vec![0],
                    memory_fraction: 0.8,
                    enable_pooling: true,
                    enable_p2p: false,
                },
                memory: MemoryConfig {
                    max_memory_gb: 16.0,
                    enable_mmap: true,
                    cache_size_mb: 1024,
                },
            },
            plugins: PluginConfig {
                plugin_dir: "./plugins".to_string(),
                enabled_plugins: vec![
                    "pytorch".to_string(),
                    "onnx".to_string(),
                    "tensorrt".to_string(),
                ],
                plugin_configs: HashMap::new(),
                plugin_timeout_secs: 300,
            },
            monitoring: MonitoringConfig {
                prometheus_enabled: true,
                prometheus_port: 9090,
                jaeger_enabled: false,
                jaeger_endpoint: None,
                health_check_interval_secs: 30,
                metrics_collection_interval_secs: 60,
            },
            security: SecurityConfig {
                auth_enabled: false,
                jwt_secret: None,
                api_keys: vec![],
                cors_enabled: true,
                cors_allowed_origins: vec!["*".to_string()],
                rate_limiting: RateLimitConfig {
                    enabled: true,
                    requests_per_minute: 1000,
                    burst_size: 100,
                },
            },
            storage: StorageConfig {
                model_storage_path: "./models".to_string(),
                cache_storage_path: "./cache".to_string(),
                log_storage_path: "./logs".to_string(),
                max_storage_gb: 1000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                console_output: true,
                file_output: true,
                file_path: Some("./logs/unimodel.log".to_string()),
                rotation_size_mb: 100,
                retention_count: 10,
            },
        }
    }
}
