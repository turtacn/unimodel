# UniModel: Universal Model Serving Engine

[![GitHub license](https://img.shields.io/github/license/turtacn/unimodel)](https://github.com/turtacn/unimodel/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/turtacn/unimodel)](https://github.com/turtacn/unimodel/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/turtacn/unimodel)](https://github.com/turtacn/unimodel/issues)

## Project Overview
UniModel is a lightweight, efficient, and unified serving engine designed to simplify the deployment and serving of various AI models, including Large Language Models (LLMs) and Computer Vision (CV) models. It acts as the technical foundation for "simplifying complexity," enabling seamless integration and management of heterogeneous AI assets in enterprise environments.

For the Chinese version, please refer to [README-zh.md](README-zh.md).

### Main Pain Points and Core Value
In today's AI landscape, enterprises face challenges such as:
- **Heterogeneous Models**: Diverse formats (e.g., GGUF for LLMs, ONNX for general AI) leading to fragmented serving solutions.
- **Resource Inefficiency**: Static model loading wastes GPU/NPU resources.
- **Complexity in Deployment**: Multiple APIs and configurations complicate operations for ops, developers, and DevOps teams.

UniModel addresses these by providing:
- **Unified API**: A single RESTful/gRPC interface for all models, shielding backend differences.
- **Simplicity and Efficiency**: Plugin-based architecture for easy extension, dynamic resource management, and intelligent batching to boost throughput.
- **Integration with Astraea**: Deep embedding as a core service in Astraea AI systems, ensuring high availability and observability.

Compared to vLLM (LLM-focused) and Triton (feature-rich but complex), UniModel emphasizes breadth, unity, and extreme ease-of-use.

### Key Features
- **Unified Interfaces**: RESTful APIs (OpenAPI-compliant) and gRPC for prediction, e.g., `POST /v1/models/{model_name}:predict`.
- **Multi-Backend Support**: Plugins for GGUF, TensorRT-LLM, ONNX, TensorFlow, PyTorch.
- **Dynamic Resource Management**: Auto-load/unload models on GPU/NPU based on demand, supporting multi-model on single cards.
- **Intelligent Batching**: Dynamic merging of requests for higher throughput.
- **Observability**: Prometheus metrics for latency, throughput, GPU usage, error rates.
- **Lightweight and Embeddable**: Minimal footprint, stateless design for scalability.
- **Middleware Integration**: etcd for service discovery, NATS for internal messaging.

### Architecture Overview
UniModel adopts a layered, modular design with Rust for core performance and Python for plugins. For details, see [docs/architecture.md](docs/architecture.md).

### Building and Running
#### Prerequisites
- Rust (1.70+)
- Python (3.10+)
- Cargo, pip
- Optional: CUDA for GPU support

#### Build
```bash
git clone https://github.com/turtacn/unimodel.git
cd unimodel
cargo build --release
pip install -r requirements.txt
````

#### Run

```bash
cargo run -- --config config.toml
```

This starts the API server on port 8080.

### Demonstration

Here's a simple code snippet to demonstrate model prediction via the unified API (using Python client):

```python
import requests

url = "http://localhost:8080/v1/models/my-llm-model:predict"
payload = {"input": "Hello, world!"}
response = requests.post(url, json=payload)
print(response.json())  # Output: {"output": "Generated response"}
```

This showcases the unified interface for LLM inference. For CV models, the payload would include image data.

### Contribution Guide

We welcome contributions! Please follow these steps:

1. Fork the repo.
2. Create a feature branch (`git checkout -b feature/xxx`).
3. Commit changes (`git commit -m 'Add feature'`).
4. Push to the branch (`git push origin feature/xxx`).
5. Open a Pull Request.

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## License

MIT License. See [LICENSE](LICENSE) for details.