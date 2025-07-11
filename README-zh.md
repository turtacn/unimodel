# UniModel: 统一模型服务引擎

[![GitHub license](https://img.shields.io/github/license/turtacn/unimodel)](https://github.com/turtacn/unimodel/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/turtacn/unimodel)](https://github.com/turtacn/unimodel/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/turtacn/unimodel)](https://github.com/turtacn/unimodel/issues)

## 项目简介
UniModel 是一个轻量、高效、统一的模型服务引擎，旨在简化各种 AI 模型（包括大语言模型 LLM 和计算机视觉 CV 模型）的部署和服务。它作为“化繁为简”的技术基石，实现无缝整合企业异构 AI 资产。

英文版请参考 [README.md](README.md)。

### 主要痛点与核心价值
当今 AI 领域，企业面临挑战如：
- **模型异构**：多样格式（如 LLM 的 GGUF、一般 AI 的 ONNX）导致服务碎片化。
- **资源低效**：静态加载浪费 GPU/NPU 资源。
- **部署复杂**：多 API 和配置困扰运维、开发者、DevOps 团队。

UniModel 通过以下方式解决：
- **统一 API**：单一 RESTful/gRPC 接口屏蔽后端差异。
- **简洁高效**：插件架构易扩展、动态资源管理、智能批处理提升吞吐。
- **与 Astraea 集成**：作为核心服务嵌入，确保高可用和可观测性。

相较 vLLM（专注 LLM）和 Triton（功能强大但复杂），UniModel 强调广度、统一性和极致易用性。

### 主要功能特性
- **统一接口**：符合 OpenAPI 的 RESTful API 和 gRPC，用于预测，如 `POST /v1/models/{model_name}:predict`。
- **多后端支持**：插件支持 GGUF、TensorRT-LLM、ONNX、TensorFlow、PyTorch。
- **动态资源管理**：基于需求自动加载/卸载模型，支持单卡多模型。
- **智能批处理**：动态合并请求，提高吞吐量。
- **可观测性**：Prometheus 指标，包括延迟、吞吐、GPU 利用率、错误率。
- **轻量可嵌入**：最小资源占用，无状态设计便于扩展。
- **中间件集成**：etcd 用于服务发现，NATS 用于内部消息。

### 架构概览
UniModel 采用分层模块化设计，Rust 处理核心性能，Python 处理插件。详情见 [docs/architecture.md](docs/architecture.md)。

### 构建与运行指南
#### 前置条件
- Rust (1.70+)
- Python (3.10+)
- Cargo, pip
- 可选：CUDA 支持 GPU

#### 构建
```bash
git clone https://github.com/turtacn/unimodel.git
cd unimodel
cargo build --release
pip install -r requirements.txt
````

#### 运行

```bash
cargo run -- --config config.toml
```

启动 API 服务器于 8080 端口。

### 能力展示

以下代码片段演示通过统一 API 进行模型预测（Python 客户端）：

```python
import requests

url = "http://localhost:8080/v1/models/my-llm-model:predict"
payload = {"input": "Hello, world!"}
response = requests.post(url, json=payload)
print(response.json())  # 输出: {"output": "Generated response"}
```

这展示了 LLM 推理的统一接口。对于 CV 模型，负载可包含图像数据。

### 贡献指南

欢迎贡献！请遵循以下步骤：

1. Fork 仓库。
2. 创建功能分支 (`git checkout -b feature/xxx`)。
3. 提交变更 (`git commit -m 'Add feature'`)。
4. 推送分支 (`git push origin feature/xxx`)。
5. 开启 Pull Request。

详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可

MIT License。详见 [LICENSE](LICENSE)。