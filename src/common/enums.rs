// src/common/enums.rs
// 此文件定义项目全局枚举类型，提供命名常量以确保类型安全。
// This file defines global enum types for the project, providing named constants for type safety.

// 导入标准库以支持字符串转换和错误处理。
// Import standard library for string conversion and error handling.
use std::fmt::{self, Display};
use std::str::FromStr;

// 规划：未来可定义一个通用枚举trait来抽象行为，例如支持通用的转换方法。
// Planning: A general trait for enums to abstract common behaviors, such as conversion methods.
pub trait EnumTrait: Display + FromStr {
    // 将枚举转换为字符串的方法（中英文注释）。
    // Method to convert enum to string (bilingual comment).
    fn to_string(&self) -> String {
        self.to_string()
    }
}

// 枚举：模型类型（ModelType），用于区分不同模型格式。
// Enum: ModelType, used to distinguish different model formats.
// 设计为可扩展，例如未来添加新变体如Speech。
// Designed for extensibility, e.g., add new variants like Speech in the future.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelType {
    // LLM 类型，支持子变体如Gguf。
    // LLM type, supporting sub-variants like Gguf.
    Llm(LlmFormat),
    // 通用AI类型，支持子变体如Onnx。
    // General AI type, supporting sub-variants like Onnx.
    General(GeneralFormat),
    // CV 类型（计算机视觉）。
    // CV type (Computer Vision).
    Cv,
    // 其他类型，作为占位符以支持扩展。
    // Other type, as a placeholder for extension.
    Other(String),
}

// 子枚举：LLM格式。
// Sub-enum: LlmFormat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LlmFormat {
    Gguf,
    TensorRtLlm,
}

// 子枚举：通用格式。
// Sub-enum: GeneralFormat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeneralFormat {
    Onnx,
    TensorFlow,
    PyTorch,
}

// 为ModelType实现Display trait，用于字符串表示。
// Implement Display trait for ModelType for string representation.
impl Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelType::Llm(format) => write!(f, "LLM({})", format),
            ModelType::General(format) => write!(f, "General({})", format),
            ModelType::Cv => write!(f, "CV"),
            ModelType::Other(s) => write!(f, "Other({})", s),
        }
    }
}

// 为ModelType实现FromStr trait，支持从字符串解析。
// Implement FromStr trait for ModelType to support parsing from string.
// 步骤化解决方案：1. 小写化输入；2. 匹配已知变体；3. 如果无效，返回自定义错误（规划依赖errors.rs）。
// Step-by-step solution: 1. Lowercase input; 2. Match known variants; 3. If invalid, return custom error (planned dependency on errors.rs).
impl FromStr for ModelType {
    type Err = String; // 临时使用String作为错误类型，未来替换为UniModelError from errors.rs。
                       // Temporarily use String as error type, replace with UniModelError from errors.rs in future.

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_str() {
            "llm(gguf)" => Ok(ModelType::Llm(LlmFormat::Gguf)),
            "llm(tensorrtllm)" => Ok(ModelType::Llm(LlmFormat::TensorRtLlm)),
            "general(onnx)" => Ok(ModelType::General(GeneralFormat::Onnx)),
            "general(tensorflow)" => Ok(ModelType::General(GeneralFormat::TensorFlow)),
            "general(pytorch)" => Ok(ModelType::General(GeneralFormat::PyTorch)),
            "cv" => Ok(ModelType::Cv),
            other => {
                // 规划：在实际实现中，这里可调用logger记录无效转换（依赖logger.rs）。
                // Planning: In actual implementation, log invalid conversion here (dependency on logger.rs).
                // 对于无效变体，返回错误；如果初步变体不足，可灵活添加新匹配。
                // For invalid variants, return error; flexibly add new matches if initial variants are insufficient.
                Ok(ModelType::Other(other.to_string()))
                // 替代方案：如果严格模式，返回Err("Invalid model type".to_string())。
                // Alternative: In strict mode, return Err("Invalid model type".to_string()).
            }
        }
    }
}

// 实现EnumTrait for ModelType。
// Implement EnumTrait for ModelType.
impl EnumTrait for ModelType {}

// 枚举：模型状态（Status），用于跟踪模型加载状态。
// Enum: Status, used to track model loading states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    // 已加载。
    // Loaded.
    Loaded,
    // 未加载。
    // Unloaded.
    Unloaded,
    // 加载中。
    // Loading.
    Loading,
    // 错误状态。
    // Error.
    Error,
}

// 为Status实现Display。
// Implement Display for Status.
impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Loaded => write!(f, "Loaded"),
            Status::Unloaded => write!(f, "Unloaded"),
            Status::Loading => write!(f, "Loading"),
            Status::Error => write!(f, "Error"),
        }
    }
}

// 为Status实现FromStr。
// Implement FromStr for Status.
impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "loaded" => Ok(Status::Loaded),
            "unloaded" => Ok(Status::Unloaded),
            "loading" => Ok(Status::Loading),
            "error" => Ok(Status::Error),
            _ => {
                // 规划：记录日志（依赖logger.rs）。
                // Planning: Log here (dependency on logger.rs).
                Err("Invalid status".to_string())
            }
        }
    }
}

// 实现Enum
