//! UniModel - 统一模型服务引擎
//!
//! UniModel是一个高性能、分布式、多模型服务平台，为企业级AI应用提供统一的推理服务。
//!
//! # 特性
//!
//! - **统一API**: 支持LLM、CV、音频等多种模型类型的统一接口
//! - **高性能**: Rust核心引擎，零拷贝优化，智能批处理
//! - **分布式**: 基于etcd和NATS的云原生分布式架构
//! - **可扩展**: 插件化架构，支持多种推理后端
//!
//! # 快速开始
//!
//! ```rust
//! use unimodel::UniModelServer;
//! use unimodel::config::Config;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::from_file("config/default.yaml")?;
//!     let server = UniModelServer::new(config).await?;
//!     server.start().await?;
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod plugins;
pub mod common;

// 重新导出核心类型
pub use crate::common::error::{UniModelError, Result};
pub use crate::domain::model::{Model, ModelInfo, ModelStatus};
pub use crate::domain::service::{ModelManager, BatchProcessor, Scheduler};
pub use crate::application::services::{ModelService, PredictionService};
pub use crate::infrastructure::configuration::{Config, ServerConfig, EngineConfig};

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// UniModel服务器主入口
pub struct UniModelServer {
    config: Config,
    model_manager: ModelManager,
    batch_processor: BatchProcessor,
    scheduler: Scheduler,
}

impl UniModelServer {
    /// 创建新的UniModel服务器实例
    pub async fn new(config: Config) -> Result<Self> {
        let model_manager = ModelManager::new(&config).await?;
        let batch_processor = BatchProcessor::new(&config).await?;
        let scheduler = Scheduler::new(&config).await?;

        Ok(Self {
            config,
            model_manager,
            batch_processor,
            scheduler,
        })
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting UniModel Server v{}", VERSION);

        // 启动各个组件
        self.scheduler.start().await?;
        self.batch_processor.start().await?;

        // 启动API服务器
        let api_server = api::rest::server::ApiServer::new(&self.config).await?;
        let grpc_server = api::grpc::server::GrpcServer::new(&self.config).await?;

        // 并行启动HTTP和gRPC服务器
        tokio::try_join!(
            api_server.serve(),
            grpc_server.serve()
        )?;

        Ok(())
    }
}