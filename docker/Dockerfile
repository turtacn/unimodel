# 多阶段构建Docker镜像
FROM rust:1.75-slim as builder

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制Cargo文件
COPY Cargo.toml Cargo.lock ./

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false unimodel

# 创建应用目录
RUN mkdir -p /app/config /app/models /app/cache /app/logs /app/plugins \
    && chown -R unimodel:unimodel /app

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/unimodel ./
COPY --from=builder /app/target/release/unimodel-cli ./

# 复制配置文件
COPY config/ ./config/

# 设置权限
RUN chmod +x ./unimodel ./unimodel-cli

# 切换到应用用户
USER unimodel

# 暴露端口
EXPOSE 8000 9000 9090

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# 设置环境变量
ENV RUST_LOG=info
ENV UNIMODEL_CONFIG_PATH=/app/config/default.yaml

# 启动命令
CMD ["./unimodel", "/app/config/default.yaml"]