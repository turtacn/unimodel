#!/bin/bash

# UniModel构建脚本
set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_info "Checking dependencies..."

    if ! command -v rustc &> /dev/null; then
        log_error "Rust not found. Please install Rust first."
        exit 1
    fi

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Cargo first."
        exit 1
    fi

    if ! command -v docker &> /dev/null; then
        log_warn "Docker not found. Docker builds will be skipped."
    fi

    log_info "Dependencies check passed."
}

# 代码格式化
format_code() {
    log_info "Formatting code..."
    cargo fmt --all
    log_info "Code formatting completed."
}

# 代码检查
check_code() {
    log_info "Running code checks..."
    cargo clippy --all-targets --all-features -- -D warnings
    log_info "Code checks passed."
}

# 运行测试
run_tests() {
    log_info "Running tests..."
    cargo test --all-features
    log_info "Tests passed."
}

# 构建发布版本
build_release() {
    log_info "Building release version..."
    cargo build --release --all-features
    log_info "Release build completed."
}

# 构建Docker镜像
build_docker() {
    if command -v docker &> /dev/null; then
        log_info "Building Docker image..."
        docker build -t unimodel:latest .
        log_info "Docker image built successfully."
    else
        log_warn "Docker not available, skipping Docker build."
    fi
}

# 清理构建产物
clean() {
    log_info "Cleaning build artifacts..."
    cargo clean
    log_info "Clean completed."
}

# 创建发布包
create_release_package() {
    log_info "Creating release package..."

    # 创建发布目录
    mkdir -p release

    # 复制二进制文件
    cp target/release/unimodel release/
    cp target/release/unimodel-cli release/

    # 复制配置文件
    cp -r config release/

    # 复制文档
    cp README.md release/
    cp LICENSE release/

    # 创建启动脚本
    cat > release/start.sh << 'EOF'
#!/bin/bash
./unimodel config/default.yaml
EOF
    chmod +x release/start.sh

    # 创建压缩包
    tar -czf unimodel-$(date +%Y%m%d).tar.gz -C release .

    log_info "Release package created."
}

# 主函数
main() {
    case "${1:-all}" in
        "deps")
            check_dependencies
            ;;
        "format")
            format_code
            ;;
        "check")
            check_code
            ;;
        "test")
            run_tests
            ;;
        "build")
            build_release
            ;;
        "docker")
            build_docker
            ;;
        "clean")
            clean
            ;;
        "package")
            create_release_package
            ;;
        "all")
            check_dependencies
            format_code
            check_code
            run_tests
            build_release
            build_docker
            create_release_package
            ;;
        *)
            echo "Usage: $0 {deps|format|check|test|build|docker|clean|package|all}"
            exit 1
            ;;
    esac
}

main "$@"