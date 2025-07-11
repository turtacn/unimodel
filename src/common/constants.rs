// src/common/constants.rs
// 此文件定义项目全局常量，提供固定值以确保配置一致性。
// This file defines global constants for the project, providing fixed values for configuration consistency.

// 导入enums.rs以使用枚举作为常量基础（例如，将枚举变体字符串化为常量）。
// Import enums.rs to use enums as base for constants (e.g., stringifying enum variants into constants).
use crate::common::enums::{ModelType, Status}; // 假设common模块结构，相对路径导入。
// use super::enums::{ModelType, Status}; // 替代路径，如果在同一模块下。

// 首先定义一个常量分组接口：使用struct来抽象和分组常量，提供可扩展性。
// First define a constants grouping interface: Use a struct to abstract and group constants for extensibility.
// 这个struct不实例化，仅用于命名空间和文档目的；未来可通过宏扩展添加新分组。
// This struct is not instantiated, used only for namespace and documentation; extensible via macros in future.
pub struct ConfigConstants;

// 实现ConfigConstants：注入常量值，并添加文档注释。
// Implement ConfigConstants: Inject constant values with documentation comments.
// 逻辑分解：1. 定义API相关分组常量；2. 定义服务器配置常量；3. 定义基于枚举的常量（避免硬编码）。
// Logic breakdown: 1. Define API-related group constants; 2. Define server config constants; 3. Define enum-based constants (avoid hardcoding).
// 适应变更：使用macro_rules!允许动态添加常量，例如macro_rules! define_const { ($name:ident = $val:expr) => { pub const $name: &str = $val; } }
// Adaptation for changes: Use macro_rules! to allow dynamic addition of constants.
impl ConfigConstants {
    // API分组：路径前缀和版本。
    // API group: Path prefixes and versions.
    /// API版本常量，用于构建URL路径（如"/v1/models"）。
    /// API version constant, used for building URL paths (e.g., "/v1/models").
    pub const API_VERSION: &str = "v1";

    /// 预测端点路径模板。
    /// Predict endpoint path template.
    pub const PREDICT_PATH: &str = "/models/{model_name}:predict";

    // 服务器配置分组：默认端口等，可扩展。
    // Server config group: Default port, etc., extensible.
    /// 默认服务器端口。
    /// Default server port.
    pub const DEFAULT_PORT: u16 = 8080;

    /// 默认超时秒数。
    /// Default timeout in seconds.
    pub const DEFAULT_TIMEOUT: u64 = 30;

    // 基于enums.rs的常量：将枚举变体字符串化为常量，避免硬编码。
    // Enum-based constants from enums.rs: Stringify enum variants to avoid hardcoding.
    /// 默认模型类型字符串（基于ModelType::Llm(Gguf)）。
    /// Default model type string (based on ModelType::Llm(Gguf)).
    pub const DEFAULT_MODEL_TYPE: &str = "LLM(Gguf)"; // 来自ModelType的Display实现。

    /// 加载状态字符串（基于Status::Loaded）。
    /// Loaded status string (based on Status::Loaded).
    pub const LOADED_STATUS: &str = "Loaded";

    // 宏定义示例：允许扩展新常量。
    // Macro definition example: Allows extension of new constants.
    macro_rules! extend_const {
        ($name:ident = $val:expr) => {
            pub const $name: &str = $val;
        };
    }
    // 使用宏添加示例扩展常量（未来可用于适应变更，如添加新端口）。
    // Use macro to add example extension constant (for future changes, e.g., adding new ports).
    extend_const!(EXTENDED_PORT = "8081");

    // 方法：验证常量有效性（运行时检查，例如检查端口范围）。
    // Method: Validate constant validity (runtime check, e.g., port range).
    // 步骤：1. 检查端口是否在1-65535；2. 如果无效，规划log misuse（依赖logger.rs）；3. 返回Result。
    // Steps: 1. Check if port is in 1-65535; 2. If invalid, plan to log misuse (dependency on logger.rs); 3. Return Result.
    // 错误处理：编译时无法检查，但运行时抛出错误；替代方案：默认值回退。
    // Error handling: Compile-time checks not possible, but runtime error; alternative: fallback to default.
    pub fn validate_port(port: u16) -> Result<(), String> {
        if port == 0 || port > 65535 {
            // 规划：在实际使用中，这里调用logger记录misuse，例如logger::warn!("Invalid port: {}", port);
            // Planning: In actual usage, call logger to log misuse, e.g., logger::warn!("Invalid port: {}", port);
            return Err(format!("Invalid port: {} (must be 1-65535)", port));
        }
        Ok(())
    }

    // 类似验证方法：检查基于枚举的常量。
    // Similar validation method: Check enum-based constants.
    pub fn validate_model_type(type_str: &str) -> Result<ModelType, String> {
        // 使用FromStr from enums.rs进行转换。
        // Use FromStr from enums.rs for conversion.
        ModelType::from_str(type_str).map_err(|e| {
            // 规划log：logger::error!("Invalid model type: {}", type_str);
            // Plan log: logger::error!("Invalid model type: {}", type_str);
            e
        })
    }
}

// 规划：单元测试将在src/tests/common_tests.rs中实现，涵盖常量访问（如assert_eq!(ConfigConstants::API_VERSION, "v1")）、验证方法成功/失败案例，以及基于枚举的常量匹配。
// Planning: Unit tests will be implemented in src/tests/common_tests.rs, covering constant access (e.g., assert_eq!(ConfigConstants::API_VERSION, "v1")), validation method success/failure cases, and enum-based constant matching.

// 确保编译无错：本文件仅使用标准库和内部依赖，无外部crate。
// Ensure compilation without errors: This file uses only standard library and internal dependencies, no external crates.

// Personal.AI order the ending
