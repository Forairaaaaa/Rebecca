# Rebecca HAL CLI

简化读写操作，方便脚本和上层应用的集成喵~

## 功能

- 实时读取 IMU 数据流
- 背光获取、调节

## 安装

从完整的 Rebecca 仓库进入 `hal/cli-tool/rebecca-hal` 目录。构建时使用仓库中的 `hal/service/src/imu_data.proto`，不需要先运行 HAL 服务。

树莓派系统需要安装 Protobuf 编译器：

```bash
sudo apt install protobuf-compiler
```

```bash
cargo install --path .
```

## 使用

### 基本语法

```bash
rebecca-hal [选项] <子命令> [设备ID] [操作]
```

#### 全局选项

- `--host`: HAL 服务地址，默认 localhost
- `-p, --port`: HAL 服务端口，默认 12580
- `-v, --verbose`: 详细日志输出模式
- `-h, --help`: 显示帮助信息喵

### IMU

列出所有 IMU 设备：

```bash
rebecca-hal imu
```

返回：

```json
["imu0"]
```

获取 IMU 设备信息：

```bash
rebecca-hal imu imu0 info
```

启动 IMU 数据发布：

```bash
rebecca-hal imu imu0 start
```

实时读取 IMU 数据：

```bash
rebecca-hal imu imu0 read
```

_按 Ctrl+C 停止读取数据流喵_

停止 IMU 数据发布：

```bash
rebecca-hal imu imu0 stop
```

### Backlight

列出所有背光设备：

```bash
rebecca-hal backlight
```

返回：

```json
["backlight0"]
```

获取背光设备信息：

```bash
rebecca-hal backlight backlight0 info
```

获取当前亮度：

```bash
rebecca-hal backlight backlight0 get
```

返回：

```json
{
  "brightness": 0.8
}
```

设置亮度：

```bash
rebecca-hal backlight backlight0 set 0.5
```

### 远程连接

连接到远程服务：

```bash
rebecca-hal --host 192.168.1.233 --port 12580 imu imu0 info
```

## 开发

### 编译

```bash
cargo build --release
```

### 运行

```bash
cargo run -- --verbose -h
```


### 无硬件验证

在 Rebecca 仓库根目录运行，测试会启动项目自带的模拟 IMU 和亮度设备：

```bash
cargo build --manifest-path hal/service/Cargo.toml
```

```bash
cargo build --manifest-path hal/cli-tool/rebecca-hal/Cargo.toml
```

```bash
python3 hal/tests/test_cli.py
```

---

_不用重复写接口了喵_ (´｡• ᵕ •｡`)
