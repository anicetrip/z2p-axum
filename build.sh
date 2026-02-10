#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="zero2prod:latest"
CONTAINER_NAME="zero2prod_test"

# =====================
# 1️⃣ 代码格式化
# =====================
echo "🔹 Running cargo fmt..."
cargo fmt --all

# =====================
# 2️⃣ 运行测试
# =====================
echo "🔹 Running cargo test..."
cargo test --all

# =====================
# 3️⃣ 代码覆盖率
# =====================
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "⚠️ cargo-tarpaulin not found, installing..."
    cargo install cargo-tarpaulin
fi

echo "🔹 Running code coverage..."
cargo tarpaulin --ignore-tests --out Html
echo "Coverage report: ./tarpaulin-report.html"

# =====================
# 4️⃣ 构建 Docker 镜像
# =====================
echo "🔹 Building Docker image: $IMAGE_NAME..."
docker build -t $IMAGE_NAME .

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
# 6️⃣ 健康检查
# =====================
echo "🔹 Checking /health_check..."
sleep 2  # 给容器一点时间启动

HTTP_STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8000/health_check)

if [ "$HTTP_STATUS" -eq 200 ]; then
    echo "✅ Health check passed!"
else
    echo "❌ Health check failed (HTTP $HTTP_STATUS)"
    docker logs $CONTAINER_NAME
    exit 1
fi

# =====================
# 7️⃣ 清理
# =====================
echo "🔹 Stopping and removing container..."
docker stop $CONTAINER_NAME
docker rm $CONTAINER_NAME

echo "✅ All done!"
