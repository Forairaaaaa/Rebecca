import json
import signal
import sys
import time
from typing import Optional, Sequence

import requests
import zmq
from google.protobuf.message import DecodeError

import imu_data_pb2


def format_float(value: float, width: int = 9, precision: int = 2) -> str:
    """格式化浮点数，固定宽度并始终带符号。"""
    return f"{value:+{width}.{precision}f}"


def format_triplet(values: Sequence[float], width: int = 6, precision: int = 2) -> str:
    """格式化三元向量，固定每项宽度并始终带符号。"""
    safe = list(values)[:3] + [0.0] * max(0, 3 - len(values))
    ax, ay, az = safe[:3]
    return f"{format_float(ax, width, precision)} {format_float(ay, width, precision)} {format_float(az, width, precision)}"


def format_quadruplet(
    values: Sequence[float], width: int = 6, precision: int = 2
) -> str:
    """格式化四元组，固定每项宽度并始终带符号。"""
    safe = list(values)[:4] + [0.0] * max(0, 4 - len(values))
    a, b, c, d = safe[:4]
    return (
        f"{format_float(a, width, precision)} {format_float(b, width, precision)} "
        f"{format_float(c, width, precision)} {format_float(d, width, precision)}"
    )


def get_imu_info() -> Optional[dict]:
    """获取 IMU 信息。"""
    try:
        response = requests.get("http://localhost:12580/imu0/info")
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as error:
        print(f"获取IMU信息失败: {error}")
        return None


def start_publishing() -> bool:
    """启动 IMU 数据发布。"""
    try:
        response = requests.get("http://localhost:12580/imu0/start")
        response.raise_for_status()
        print("已发送启动发布指令 (/imu0/start)")
        return True
    except requests.exceptions.RequestException as error:
        print(f"启动发布失败: {error}")
        return False


def stop_publishing() -> bool:
    """停止 IMU 数据发布。"""
    try:
        response = requests.get("http://localhost:12580/imu0/stop")
        response.raise_for_status()
        print("已发送停止发布指令 (/imu0/stop)")
        return True
    except requests.exceptions.RequestException as error:
        print(f"停止发布失败: {error}")
        return False


def subscribe_and_print_imu_data(
    imu_data_port: int, max_messages: int = 2333, recv_timeout_ms: int = 5000
) -> None:
    """订阅 IMU 数据并打印若干条样本。

    参数:
      - imu_data_port: ZMQ SUB 端口
      - max_messages: 接收并打印的消息条数
      - recv_timeout_ms: 接收超时时间，毫秒
    """
    context = zmq.Context()
    socket = context.socket(zmq.SUB)
    socket.setsockopt(zmq.SUBSCRIBE, b"")
    socket.setsockopt(zmq.RCVTIMEO, recv_timeout_ms)
    socket.connect(f"tcp://localhost:{imu_data_port}")

    print(f"已连接到 IMU 数据端口: {imu_data_port} (ZMQ SUB)")

    received_count = 0

    try:
        while received_count < max_messages:
            try:
                raw_bytes = socket.recv()
            except zmq.Again:
                print("等待数据超时，继续等待...")
                continue

            message = imu_data_pb2.ImuDataProto()
            try:
                message.ParseFromString(raw_bytes)
            except DecodeError as error:
                print(f"Protobuf 解码失败: {error}")
                continue

            received_count += 1

            accel = list(message.accel)
            gyro = list(message.gyro)
            mag = list(message.mag)
            temp = message.temp
            timestamp = message.timestamp
            quaternion = list(message.quaternion)
            euler_angles = list(message.euler_angles)

            line = (
                f"[{received_count:04d}] "
                f"[{timestamp:>16d}] | "
                f"accel: {format_triplet(accel)} | "
                f"gyro: {format_triplet(gyro)} | "
                f"mag: {format_triplet(mag)} | "
                f"temp: {format_float(temp)} | "
                f"quat: {format_quadruplet(quaternion)} | "
                f"euler(y,p,r): {format_triplet(euler_angles)}"
            )
            print(line)

    finally:
        socket.close(0)
        context.term()
        print("已关闭 ZMQ 连接")


def main() -> None:
    print("IMU 简单演示程序启动...")

    imu_info = get_imu_info()
    if not imu_info:
        print("无法获取IMU信息，程序退出")
        return

    print(f"获取到IMU信息: {json.dumps(imu_info, indent=2, ensure_ascii=False)}")

    device_type = imu_info.get("device_type")
    status = imu_info.get("status")
    sample_rate = imu_info.get("sample_rate")
    imu_data_port = imu_info.get("imu_data_port")

    print(f"设备类型: {device_type}")
    print(f"当前状态: {status}")
    print(f"采样率: {sample_rate} Hz")
    print(f"数据端口: {imu_data_port}")

    if imu_data_port is None:
        print("IMU 数据端口为空，程序退出")
        return

    should_stop = True

    def handle_sigint(_signum, _frame):
        print("\n收到中断信号，准备停止发布...")
        stop_publishing()
        sys.exit(0)

    signal.signal(signal.SIGINT, handle_sigint)

    if not start_publishing():
        print("启动发布失败，程序退出")
        return

    start_time = time.time()
    try:
        subscribe_and_print_imu_data(imu_data_port)
    finally:
        if should_stop:
            stop_publishing()
    elapsed = time.time() - start_time
    print(f"演示完成，用时 {elapsed:.2f}s")


if __name__ == "__main__":
    main()
