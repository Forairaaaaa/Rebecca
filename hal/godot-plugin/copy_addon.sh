#!/bin/bash

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# 固定源子模块路径为脚本所在目录下的 addon/xxx
SOURCE_MODULE="$SCRIPT_DIR/addons/rebecca_hal"

# 检查参数
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <target_project_path>"
    exit 1
fi

# 获取目标项目绝对路径
TARGET_PROJECT="$(realpath "$1")"

# 检查源子模块是否存在
if [ ! -d "$SOURCE_MODULE" ]; then
    echo "源子模块不存在: $SOURCE_MODULE"
    exit 1
fi

# 检查目标项目路径是否存在
if [ ! -d "$TARGET_PROJECT" ]; then
    echo "目标项目路径不存在: $TARGET_PROJECT"
    exit 1
fi

# 判断是否为 Godot 项目（必须包含 project.godot）
if [ ! -f "$TARGET_PROJECT/project.godot" ]; then
    echo "目标路径不是 Godot 项目（缺少 project.godot）: $TARGET_PROJECT"
    exit 1
fi

# addons 文件夹路径
TARGET_ADDONS="$TARGET_PROJECT/addons"

# 如果 addons 文件夹不存在，则创建
if [ ! -d "$TARGET_ADDONS" ]; then
    mkdir -p "$TARGET_ADDONS"
    echo "创建 addons 文件夹: $TARGET_ADDONS"
fi

# 目标子模块路径
TARGET_MODULE="$TARGET_ADDONS/$(basename "$SOURCE_MODULE")"

# 检查目标子模块是否已存在
if [ -d "$TARGET_MODULE" ]; then
    echo "目标项目已存在同名子模块: $TARGET_MODULE"
    exit 1
fi

# 创建目标子模块文件夹
mkdir -p "$TARGET_MODULE"

# 复制子模块
cp -r "$SOURCE_MODULE/"* "$TARGET_MODULE/"

# 删除 .uid 文件
find "$TARGET_MODULE" -name "*.uid" -type f -delete

echo "子模块已成功复制到 $TARGET_MODULE 并删除所有 .uid 文件。"
