//! 统一错误处理模块

use std::fmt;
use thiserror::Error;

/// UniModel统一错误类型
#[derive(Error, Debug)]
pub enum UniModelError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Batch processing error: {0}")]
    BatchProcessing(String),

    #[error("Scheduling error: {0}")]
    Scheduling(String),

    #[error("Resource error: {0}")]
    Resource(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// UniModel结果类型别名
pub type Result<T> = std::result::Result<T, UniModelError>;

impl UniModelError {
    /// 创建配置错误
    pub fn config<T: Into<String>>(msg: T) -> Self {
        UniModelError::Config(msg.into())
    }

    /// 创建模型错误
    pub fn model<T: Into<String>>(msg: T) -> Self {
        UniModelError::Model(msg.into())
    }

    /// 创建插件错误
    pub fn plugin<T: Into<String>>(msg: T) -> Self {
        UniModelError::Plugin(msg.into())
    }

    /// 创建内部错误
    pub fn internal<T: Into<String>>(msg: T) -> Self {
        UniModelError::Internal(msg.into())
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            UniModelError::Config(_) => "CONFIG_ERROR",
            UniModelError::Model(_) => "MODEL_ERROR",
            UniModelError::Plugin(_) => "PLUGIN_ERROR",
            UniModelError::BatchProcessing(_) => "BATCH_ERROR",
            UniModelError::Scheduling(_) => "SCHEDULE_ERROR",
            UniModelError::Resource(_) => "RESOURCE_ERROR",
            UniModelError::Network(_) => "NETWORK_ERROR",
            UniModelError::Authentication(_) => "AUTH_ERROR",
            UniModelError::Authorization(_) => "AUTHZ_ERROR",
            UniModelError::Validation(_) => "VALIDATION_ERROR",
            UniModelError::Io(_) => "IO_ERROR",
            UniModelError::Serialization(_) => "SERIALIZATION_ERROR",
            UniModelError::Http(_) => "HTTP_ERROR",
            UniModelError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// 获取HTTP状态码
    pub fn status_code(&self) -> u16 {
        match self {
            UniModelError::Config(_) => 500,
            UniModelError::Model(_) => 404,
            UniModelError::Plugin(_) => 500,
            UniModelError::BatchProcessing(_) => 500,
            UniModelError::Scheduling(_) => 503,
            UniModelError::Resource(_) => 503,
            UniModelError::Network(_) => 502,
            UniModelError::Authentication(_) => 401,
            UniModelError::Authorization(_) => 403,
            UniModelError::Validation(_) => 400,
            UniModelError::Io(_) => 500,
            UniModelError::Serialization(_) => 400,
            UniModelError::Http(_) => 500,
            UniModelError::Internal(_) => 500,
        }
    }
}