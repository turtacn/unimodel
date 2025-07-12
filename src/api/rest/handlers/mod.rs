//! REST API处理器模块

pub mod model_handler;
pub mod predict_handler;
pub mod health_handler;
pub mod metrics_handler;

pub use model_handler::*;
pub use predict_handler::*;
pub use health_handler::*;
pub use metrics_handler::*;