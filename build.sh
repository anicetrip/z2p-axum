#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="zero2prod:latest"
CONTAINER_NAME="zero2prod_test"

export APP_ENVIRONMENT=LOCAL
export APP_DATABASE__HOST="127.0.0.1"

# =====================
# 加载外部配置（如果存在）
# =====================
CONFIG_SCRIPT="./docker-config.sh"
if [ -f "$CONFIG_SCRIPT" ]; then
    echo "🔹 Loading docker registry configuration from $CONFIG_SCRIPT..."
    source "$CONFIG_SCRIPT"
else
    echo "⚠️ No docker-config.sh found, skipping registry push"
    # 如果没有配置文件，设置默认空值
    PRIVATE_REGISTRY=${PRIVATE_REGISTRY:-}
    PRIVATE_REGISTRY_USER=${PRIVATE_REGISTRY_USER:-}
    PRIVATE_REGISTRY_PASSWORD=${PRIVATE_REGISTRY_PASSWORD:-}
fi

# # =====================
# # Docker 登录函数
# # =====================
docker_login() {
    if [ -n "$PRIVATE_REGISTRY" ] && [ -n "$PRIVATE_REGISTRY_USER" ] && [ -n "$PRIVATE_REGISTRY_PASSWORD" ]; then
        echo "🔹 Logging into $PRIVATE_REGISTRY..."
        echo "$PRIVATE_REGISTRY_PASSWORD" | docker login "$PRIVATE_REGISTRY" \
            --username "$PRIVATE_REGISTRY_USER" \
            --password-stdin
        return $?
    else
        echo "⚠️ Registry credentials not fully provided"
        return 1
    fi
}

# # =====================
# # 1️⃣ 代码格式化
# # =====================
# echo "🔹 Running cargo fmt..."
# cargo fmt --all

# # =====================
# # 2️⃣ 运行测试
# # =====================
# echo "🔹 Running cargo test..."
# cargo test --all

# # =====================
# # 3️⃣ 代码覆盖率
# # =====================
# if ! command -v cargo-tarpaulin &> /dev/null; then
#     echo "⚠️ cargo-tarpaulin not found, installing..."
#     cargo install cargo-tarpaulin
# fi

# echo "🔹 Running code coverage..."
# cargo tarpaulin --ignore-tests --out Html
# echo "Coverage report: ./tarpaulin-report.html"

# # =====================
# # 4️⃣ 构建 Docker 镜像
# # =====================
# echo "🔹 Building Docker image: $IMAGE_NAME..."
# docker build -t $IMAGE_NAME .

# =====================
# 5️⃣ 启动容器
# =====================
echo "🔹 Running container $CONTAINER_NAME..."
docker stop $CONTAINER_NAME || true
docker rm $CONTAINER_NAME || true

docker run -d \
    --name $CONTAINER_NAME \
    -p 8000:8000 \
    $IMAGE_NAME


# =====================
# 7️⃣ 推送镜像到私有仓库（如果有配置）
# =====================
if [ -n "$PRIVATE_REGISTRY" ]; then
    echo "🔹 Preparing to push to private registry: $PRIVATE_REGISTRY"
    
    # 登录到私有仓库
    if docker_login; then
        # 标记镜像
        PRIVATE_IMAGE_NAME="$PRIVATE_REGISTRY/zero2prod:latest"
        echo "🔹 Tagging image as: $PRIVATE_IMAGE_NAME"
        docker tag $IMAGE_NAME $PRIVATE_IMAGE_NAME
        
        # 推送镜像
        echo "🔹 Pushing image to private registry..."
        docker push $PRIVATE_IMAGE_NAME
        
        # 登出（可选）
        docker logout "$PRIVATE_REGISTRY"
        
        echo "✅ Image successfully pushed to $PRIVATE_IMAGE_NAME"
    else
        echo "❌ Failed to login to $PRIVATE_REGISTRY, skipping push"
    fi
else
    echo "🔹 No private registry configured, skipping push"
fi

# =====================
# 8️⃣ 清理
# =====================
echo "🔹 Stopping and removing container..."
docker stop $CONTAINER_NAME
docker rm $CONTAINER_NAME

echo "✅ All done!"