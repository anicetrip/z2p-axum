#!/usr/bin/env bash
# 这个脚本用于方便地运行build脚本

# 确保配置脚本存在
if [ ! -f "./docker-config.sh" ]; then
    echo "❌ docker-config.sh not found!"
    echo "Please create docker-config.sh with your registry credentials"
    exit 1
fi

# 运行build脚本
./build.sh