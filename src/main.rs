//! UniModel服务器主程序

use std::env;
use tracing::{info, error};
use unimodel::{UniModelServer, Config, VERSION};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    init_tracing()?;

    info!("UniModel Server v{} starting...", VERSION);

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    let config_path = args.get(1)
        .map(String::as_str)
        .unwrap_or("config/default.yaml");

    // 加载配置
    let config = Config::from_file(config_path)
        .map_err(|e| {
            error!("Failed to load config from {}: {}", config_path, e);
            e
        })?;

    info!("Configuration loaded from: {}", config_path);

    // 创建并启动服务器
    let server = UniModelServer::new(config).await?;

    // 注册信号处理器
    setup_signal_handlers().await;

    // 启动服务器
    if let Err(e) = server.start().await {
        error!("Server failed to start: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

/// 初始化分布式追踪
fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "unimodel=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// 设置信号处理器用于优雅关闭
async fn setup_signal_handlers() {
    use tokio::signal;

    tokio::spawn(async {
        let mut term = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut int = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = term.recv() => {
                info!("Received SIGTERM, shutting down gracefully...");
            }
            _ = int.recv() => {
                info!("Received SIGINT, shutting down gracefully...");
            }
        }

        // 触发优雅关闭
        std::process::exit(0);
    });
}