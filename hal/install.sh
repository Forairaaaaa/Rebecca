#!/bin/bash

# Rebecca HAL 安装脚本
# 此脚本将安装 rebecca-hal 并设置为系统服务

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SERVICE_NAME="rebecca-hal"
BINARY_NAME="rebecca-hal"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

echo -e "${GREEN}开始安装 Rebecca HAL...${NC}"

# 检查cargo是否已安装
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}错误: 未找到cargo，请先安装Rust工具链${NC}"
    exit 1
fi

echo -e "${YELLOW}1. 使用cargo安装二进制文件...${NC}"
cargo install --path .

# 获取cargo安装路径
CARGO_BIN_PATH="$HOME/.cargo/bin/$BINARY_NAME"

if [ ! -f "$CARGO_BIN_PATH" ]; then
    echo -e "${RED}错误: 安装失败，未找到二进制文件${NC}"
    exit 1
fi

echo -e "${YELLOW}2. 创建systemd服务文件...${NC}"
sudo tee "$SERVICE_FILE" > /dev/null << EOF
[Unit]
Description=Rebecca HAL Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
Group=$USER
ExecStartPre=$CARGO_BIN_PATH --version
ExecStart=$CARGO_BIN_PATH --port 12580
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=rebecca-hal

# 安全设置
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=no
ReadWritePaths=/tmp

[Install]
WantedBy=multi-user.target
EOF

echo -e "${YELLOW}3. 重新加载systemd配置...${NC}"
sudo systemctl daemon-reload

echo -e "${YELLOW}4. 启用并启动服务...${NC}"
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

# 检查服务状态
sleep 2
if sudo systemctl is-active --quiet $SERVICE_NAME; then
    echo -e "${GREEN}✓ 服务启动成功！${NC}"
    echo -e "${GREEN}✓ 服务已设置为开机自启${NC}"
    echo ""
    echo "服务状态:"
    sudo systemctl status $SERVICE_NAME --no-pager -l
    echo ""
    echo -e "${GREEN}安装完成！${NC}"
    echo "- 服务名称: $SERVICE_NAME"
    echo "- 二进制文件: $CARGO_BIN_PATH"
    echo "- 配置文件: $SERVICE_FILE"
    echo ""
    echo "常用命令:"
    echo "  查看状态: sudo systemctl status $SERVICE_NAME"
    echo "  查看日志: sudo journalctl -u $SERVICE_NAME -f"
    echo "  重启服务: sudo systemctl restart $SERVICE_NAME"
    echo "  停止服务: sudo systemctl stop $SERVICE_NAME"
else
    echo -e "${RED}✗ 服务启动失败${NC}"
    echo "查看错误日志:"
    sudo journalctl -u $SERVICE_NAME --no-pager -l
    exit 1
fi
