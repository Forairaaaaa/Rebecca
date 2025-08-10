# 🔮 Rebecca HAL

一个静静运行的硬件抽象层服务喵～默默地在后台管理着设备们 ✨

## 🎯 功能

- 🌐 HTTP 接口管理服务（默认端口 12580）
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

- `GET /apis` - 获取所有可用接口
- `GET /devices` - 获取所有可用设备

#### 🎯 使用示例

获取可用设备：
```bash
curl http://localhost:12580/devices
```

返回：
```json
[
  "screen0",
  "screen1",
  "imu0"
]
```

获取可用接口：
```bash
curl http://localhost:12580/apis
```

返回：
```json
[
  {
    "path": "/apis",
    "method": "GET",
    "description": "🔮 List all available APIs"
  },
  {
    "path": "/devices",
    "method": "GET",
    "description": "🪄 List all available devices"
  },
  {
    "path": "/screen0/info",
    "method": "GET",
    "description": "📜 Get device info"
  },
  {
    "path": "/screen1/info",
    "method": "GET",
    "description": "📜 Get device info"
  },
  {
    "path": "/imu0/info",
    "method": "GET",
    "description": "📜 Get device info"
  },
  {
    "path": "/imu0/start",
    "method": "GET",
    "description": "✨ Start publishing data"
  },
  {
    "path": "/imu0/stop",
    "method": "GET",
    "description": "💤 Stop publishing data"
  }
]
```

获取设备信息：
```bash
curl http://localhost:12580/screen0/info
```

```json
{
  "screen_size": [
    320,
    240
  ],
  "bits_per_pixel": 16,
  "frame_buffer_port": 37173,
  "device_type": "../../../spi0.0",
  "description": "🕊️ Render a frame by sending a raw buffer to <frame_buffer_port> via ZMQ REP socket"
}
```

```bash
curl http://localhost:12580/imu0/info
```

```json
{
  "device_type": "mpu6500",
  "status": "idle",
  "sample_rate": 50,
  "imu_data_port": 34571,
  "description": "📫 Subscribe IMU data from <imu_data_port> via ZMQ SUB socket. When running, data will be published in protobuf format, schema available at /imu0/schema"
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
