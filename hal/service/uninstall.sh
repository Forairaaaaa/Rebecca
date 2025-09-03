#!/bin/bash

# Rebecca HAL 卸载脚本
# 此脚本将停止并移除 rebecca-hal-service 服务，并卸载二进制文件

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SERVICE_NAME="rebecca-hal-service"
BINARY_NAME="rebecca-hal-service"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

echo -e "${GREEN}开始卸载 Rebecca HAL...${NC}"

# 获取cargo安装路径
CARGO_BIN_PATH="$HOME/.cargo/bin/$BINARY_NAME"

echo -e "${YELLOW}1. 停止并禁用服务...${NC}"
if sudo systemctl is-active --quiet $SERVICE_NAME; then
    echo "正在停止服务..."
    sudo systemctl stop $SERVICE_NAME
fi

if sudo systemctl is-enabled --quiet $SERVICE_NAME 2>/dev/null; then
    echo "正在禁用服务..."
    sudo systemctl disable $SERVICE_NAME
fi

echo -e "${YELLOW}2. 移除systemd服务文件...${NC}"
if [ -f "$SERVICE_FILE" ]; then
    sudo rm -f "$SERVICE_FILE"
    echo "已删除服务文件: $SERVICE_FILE"
else
    echo "服务文件不存在: $SERVICE_FILE"
fi

echo -e "${YELLOW}3. 重新加载systemd配置...${NC}"
sudo systemctl daemon-reload
sudo systemctl reset-failed 2>/dev/null || true

echo -e "${YELLOW}4. 卸载二进制文件...${NC}"
if [ -f "$CARGO_BIN_PATH" ]; then
    cargo uninstall $BINARY_NAME
    echo "已卸载二进制文件: $CARGO_BIN_PATH"
else
    echo "二进制文件不存在: $CARGO_BIN_PATH"
fi

echo -e "${YELLOW}5. 清理检查...${NC}"
# 检查是否还有残留的进程
if pgrep -f $BINARY_NAME > /dev/null; then
    echo -e "${YELLOW}发现残留进程，正在终止...${NC}"
    pkill -f $BINARY_NAME || true
    sleep 2
    if pgrep -f $BINARY_NAME > /dev/null; then
        echo -e "${YELLOW}强制终止残留进程...${NC}"
        pkill -9 -f $BINARY_NAME || true
    fi
fi

echo -e "${GREEN}✓ 卸载完成！${NC}"
echo ""
echo "已完成以下操作:"
echo "- 停止并禁用了 $SERVICE_NAME 服务"
echo "- 删除了服务配置文件"
echo "- 卸载了二进制文件"
echo "- 清理了相关进程"
echo ""
echo -e "${GREEN}Rebecca HAL 已完全卸载${NC}"
