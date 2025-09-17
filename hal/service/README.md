# Rebecca HAL

一个静静运行的硬件抽象层服务喵～

## 功能

- HTTP 接口管理服务（默认端口 12580）
- 设备状态查询和控制：
  - 副屏 Framer Buffer 推送
  - IMU 四元数、欧拉角解算，数据订阅
  - 屏幕背光获取、设置
- Systemd 服务支持，开机自启

## 安装

```bash
./install.sh
```

_会自动安装并配置系统服务哦_

## 卸载

```bash
./uninstall.sh
```

_会把所有痕迹清除干净_

## 使用

### 启动参数

```bash
rebecca-hal-service [选项]
```

#### 选项

- `--host`: 服务监听的 IP 地址，默认 localhost，设置为 0.0.0.0 可以在局域网访问喵~

- `-p, --port`: 指定 HTTP 服务端口，默认 12580
- `-h, --help`: 显示帮助信息喵

### API 接口

- `GET /apis` - 获取所有可用接口
- `GET /devices` - 获取所有可用设备

#### 使用示例

获取可用设备：

```bash
curl http://localhost:12580/devices
```

返回：

```json
["screen0", "screen1", "imu0", ...]
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
    "description": "List all available APIs"
  },
  {
    "path": "/devices",
    "method": "GET",
    "description": "List all available devices"
  },
  {
    "path": "/screen0/info",
    "method": "GET",
    "description": "Get device info"
  },
  {
    "path": "/screen1/info",
    "method": "GET",
    "description": "Get device info"
  },
  {
    "path": "/imu0/info",
    "method": "GET",
    "description": "Get device info."
  },
  ...
]
```

获取设备信息：

```bash
curl http://localhost:12580/screen0/info
```

```json
{
  "screen_size": [320, 240],
  "bits_per_pixel": 16,
  "frame_buffer_port": 46065,
  "device_type": "../../../spi0.0",
  "description": "Render a frame by sending a raw buffer to <frame_buffer_port> using a ZMQ REP socket."
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
  "imu_data_port": 34897,
  "description": "Subscribe to IMU data from <imu_data_port> using a ZMQ SUB socket. The data is published in Protobuf format, and its schema is available at /imu0/schema."
}
```

### 服务管理

```bash
# 查看服务状态
sudo systemctl status rebecca-hal-service

# 查看实时日志
sudo journalctl -u rebecca-hal-service -f

# 重启服务
sudo systemctl restart rebecca-hal-service
```

## 开发

### 编译

```bash
cargo build --release
```

### 运行

```bash
cargo run -- --verbose
```

---

_安静地在后台运行喵_ (´｡• ᵕ •｡`)
