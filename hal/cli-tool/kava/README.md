# Kava

副屏的媒体播放工具喵～

## 功能

- 图片、GIF 显示
- 视频播放
- URL 资源支持
- 多种适应模式
- 彩色测试条显示

## 依赖

需要安装 ffmpeg 喵：

```bash
sudo apt update && sudo apt install ffmpeg
```

## 安装

```bash
cargo install --path .
```

## 使用

### 基本语法

```bash
kava <screen_name> [选项] [资源路径]
```

#### 参数

- `screen_name`: 目标屏幕设备名称，例如 `screen0`
- `resource`: 资源路径（可选），未指定时显示测试彩色条

#### 选项

- `-u, --url`: 指定资源为 URL 链接
- `-r, --repeat`: 循环播放模式
- `--video`: 指定资源为视频文件
- `--resize-mode`: 画面适应模式，默认填充
- `-h, --help`: 显示帮助信息

### 使用示例

显示可用屏幕：

```bash
kava
```

显示测试彩色条：

```bash
kava screen0
```

播放本地图片：

```bash
kava screen0 ~/image.png
```

循环播放 GIF：

```bash
kava screen0 -r ~/animation.gif
```

播放网络图片：

```bash
kava screen0 -u https://example.com/image.png
```

播放视频文件：

```bash
kava screen0 --video ~/video.mp4
```

---

_让副屏充满个性喵_ (´｡• ᵕ •｡`)
