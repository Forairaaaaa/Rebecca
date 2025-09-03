# 🧭 Rebecca IMU

用于快速获取 IMU 数据的工具喵～

## 🎮 使用方法

### 查看可用设备
```bash
rebecca-imu
```

返回：

```
Available IMUs: imu0
```

### 获取设备信息
```bash
rebecca-imu imu0 info
```

会显示设备的详细信息喵：

```shell
DeviceInfo {
    device_type: "mpu6500",
    status: "idle",
    sample_rate: 50,
    imu_data_port: 33217,
    description: "📫 Subscribe to IMU data from <imu_data_port> using a ZMQ SUB socket. The data is published in Protobuf format, and its schema is available at /imu0/schema.",
}
```

等效 `curl http://localhost:12580/imu0/info` 

### 开始数据发布

```bash
rebecca-imu imu0 start
```

### 停止数据发布

```bash
rebecca-imu imu0 stop
```

等效 `curl http://localhost:12580/imu0/` 的 `start` 和 `stop` 

### 订阅并读取数据

```bash
rebecca-imu imu0 read
```

会以 `JSON` 打印收到的 IMU 数据喵

**配合 `jq` 使用，只打印欧拉角：**

`rebecca-imu imu0 read | jq '{euler_angles}'`

```shell
...
{
  "euler_angles": [
    0.9298685,
    0.14228408,
    1.4700449
  ]
}
{
  "euler_angles": [
    0.9320044,
    0.13620058,
    1.4701023
  ]
}
...
```

## 📦 安装

要先获取 proto 格式文件喵，需要 HAL 已经在运行了哦
```bash
curl http://localhost:12580/imu0/schema -o src/imu_data.proto
```

```bash
cargo install --path .
```

---

*有了这个工具🪄，只要能执行系统命令，就可以轻松读取 IMU 数据了喵* 🐾

