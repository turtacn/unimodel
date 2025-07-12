#!/bin/bash

# UniModel部署脚本
set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 配置
DEFAULT_ENV="production"
DEFAULT_COMPOSE_FILE="docker-compose.yml"
DEFAULT_CONFIG_FILE="config/production.yaml"

# 参数解析
ENVIRONMENT="${1:-$DEFAULT_ENV}"
COMPOSE_FILE="${2:-$DEFAULT_COMPOSE_FILE}"
CONFIG_FILE="${3:-$DEFAULT_CONFIG_FILE}"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查环境
check_environment() {
    log_info "Checking deployment environment..."

    # 检查Docker和Docker Compose
    if ! command -v docker &> /dev/null; then
        log_error "Docker not found. Please install Docker first."
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose not found. Please install Docker Compose first."
        exit 1
    fi

    # 检查配置文件
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Configuration file $CONFIG_FILE not found."
        exit 1
    fi

    # 检查Docker Compose文件
    if [ ! -f "$COMPOSE_FILE" ]; then
        log_error "Docker Compose file $COMPOSE_FILE not found."
        exit 1
    fi

    log_info "Environment check passed."
}

# 准备部署
prepare_deployment() {
    log_info "Preparing deployment..."

    # 创建必要的目录
    mkdir -p models cache logs plugins

    # 设置权限
    chmod 755 models cache logs plugins

    # 复制配置文件
    cp "$CONFIG_FILE" config/current.yaml

    log_info "Deployment preparation completed."
}

# 构建镜像
build_images() {
    log_info "Building Docker images..."
    docker-compose -f "$COMPOSE_FILE" build --no-cache
    log_info "Images built successfully."
}

# 启动服务
start_services() {
    log_info "Starting services..."

    # 启动依赖服务
    docker-compose -f "$COMPOSE_FILE" up -d etcd nats redis postgres minio

    # 等待依赖服务就绪
    log_info "Waiting for dependencies to be ready..."
    sleep 30

    # 启动主服务
    docker-compose -f "$COMPOSE_FILE" up -d unimodel

    # 启动监控服务
    docker-compose -f "$COMPOSE_FILE" up -d prometheus grafana jaeger

    log_info "Services started successfully."
}

# 健康检查
health_check() {
    log_info "Performing health check..."

    # 等待服务启动
    sleep 60

    # 检查主服务
    if ! curl -f http://localhost:8000/health > /dev/null 2>&1; then
        log_error "UniModel service health check failed."
        docker-compose -f "$COMPOSE_FILE" logs unimodel
        exit 1
    fi

    log_info "Health check passed."
}

# 停止服务
stop_services() {
    log_info "Stopping services..."
    docker-compose -f "$COMPOSE_FILE" down
    log_info "Services stopped."
}

# 重启服务
restart_services() {
    log_info "Restarting services..."
    stop_services
    start_services
    health_check
    log_info "Services restarted successfully."
}

# 查看日志
view_logs() {
    docker-compose -f "$COMPOSE_FILE" logs -f unimodel
}

# 查看状态
show_status() {
    log_info "Service Status:"
    docker-compose -f "$COMPOSE_FILE" ps

    log_info "Service URLs:"
    echo "  - API: http://localhost:8000"
    echo "  - gRPC: localhost:9000"
    echo "  - Metrics: http://localhost:9090"
    echo "  - Grafana: http://localhost:3000 (admin/admin)"
    echo "  - Jaeger: http://localhost:16686"
    echo "  - MinIO: http://localhost:9002 (minioadmin/minioadmin123)"
}

# 清理部署
cleanup() {
    log_info "Cleaning up deployment..."
    docker-compose -f "$COMPOSE_FILE" down -v
    docker system prune -f
    log_info "Cleanup completed."
}

# 主函数
main() {
    case "${4:-deploy}" in
        "deploy")
            check_environment
            prepare_deployment
            build_images
            start_services
            health_check
            show_status
            ;;
        "start")
            start_services
            health_check
            ;;
        "stop")
            stop_services
            ;;
        "restart")
            restart_services
            ;;
        "status")
            show_status
            ;;
        "logs")
            view_logs
            ;;
        "cleanup")
            cleanup
            ;;
        *)
            echo "Usage: $0 [environment] [compose_file] [config_file] {deploy|start|stop|restart|status|logs|cleanup}"
            echo "  environment: production (default), development, staging"
            echo "  compose_file: docker-compose.yml (default)"
            echo "  config_file: config/production.yaml (default)"
            exit 1
            ;;
    esac
}

main "$@"