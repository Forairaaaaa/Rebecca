# 🐱 Rebecca HAL

一个静静运行的硬件抽象层服务喵～默默地在后台管理着设备们 ✨

## 🎯 功能

- 🌐 HTTP 接口管理服务（默认端口 12580）
- 🖥️ 副屏 Frame Buffer 推送接口
- 🔧 设备状态查询和控制
- 🚀 systemd 服务支持，开机自启

## 📦 安装

```bash
./install.sh
```

*会自动安装二进制文件并设置为系统服务哦*

## 🗑️ 卸载

```bash
./uninstall.sh
```

*会干净地清理所有痕迹呢*

## 🎮 使用

### 启动参数
```bash
rebecca-hal [选项]
```

#### ⚙️ 选项
- `-p, --port`: 指定 HTTP 服务端口（默认 12580）
- `-v, --verbose`: 详细日志模式
- `-h, --help`: 显示帮助信息

### 🌐 API 接口

- `GET /get-device/all` - 获取所有设备信息
- `GET /get-device/{device_id}` - 获取指定设备信息

#### 🎯 使用示例

获取屏幕设备信息：
```bash
curl http://127.0.0.1:12580/get-device/screen0
```

返回：
```json
{
  "bits_per_pixel": 16,
  "description": "Render a frame by sending a raw buffer to <frame_buffer_port> via ZMQ REP socket.",
  "device_type": "../../../spi0.0",
  "frame_buffer_port": 37029,
  "screen_size": [
    320,
    240
  ]
}
```

### 📋 服务管理

```bash
# 查看服务状态
sudo systemctl status rebecca-hal

# 查看实时日志
sudo journalctl -u rebecca-hal -f

# 重启服务
sudo systemctl restart rebecca-hal
```

## 🛠️ 开发

### 编译
```bash
cargo build --release
```

### 运行
```bash
cargo run -- --verbose
```

---

*安静地在后台工作，需要的时候就在这里。* 🐾
