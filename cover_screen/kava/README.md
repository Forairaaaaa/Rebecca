# 🐱 Kava

一个可爱的小工具喵～用于在副屏上播放图片、GIF、视频等媒体内容 ✨

## 📦 安装依赖

首先需要安装ffmpeg哦

```bash
sudo apt update
```

```bash
sudo apt install ffmpeg
```

## 🎮 使用方法

### 基本语法
```bash
kava <screen_name> [选项] [资源路径]
```

### 📝 参数说明
- `screen_name`: 屏幕名称喵～例如 `screen0`
- `resource`: 资源路径（可选），如果不提供的话会显示漂亮的彩色条哦

### ⚙️ 选项
- `-u, --url`: 指定资源为URL链接
- `-r, --repeat`: 循环播放（可以一直看下去呢～）
- `    --video`: 指定资源为视频文件
- `    --resize-mode`: 适应模式（默认填充哦）

### 🎯 使用示例

1. **显示能用的屏幕** 🖥️
```bash
kava
```

2. **显示测试彩色条** 🌈
```bash
kava screen0
```

3. **播放本地图片** 🖼️
```bash
kava screen0 ~/image.png
```

4. **播放GIF并循环** 🎬
```bash
kava screen0 -r ~/animation.gif
```

5. **从URL下载并播放图片** 🌐
```bash
kava screen0 -u https://example.com/image.png
```

6. **播放视频** 🎥
```bash
kava screen0 --video ~/video.mp4
```

### ❓ 查看帮助
```bash
kava -h
```

## 📥 安装

```bash
cargo install --path .
```

## 🗑️ 卸载

```bash
cargo uninstall kava
```

---

*希望这个小工具能帮到你。如果遇到问题，记得查看帮助信息。* 🐾
