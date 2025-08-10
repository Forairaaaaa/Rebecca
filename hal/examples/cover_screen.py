import requests
import zmq
import json
from PIL import Image, ImageDraw
import numpy as np


def get_screen_info():
    """获取screen0的信息"""
    try:
        response = requests.get("http://localhost:12580/screen0/info")
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"获取screen信息失败: {e}")
        return None


def create_hello_world_image(width, height):
    """使用PIL创建hello world图像"""
    # 创建图像
    image = Image.new("RGB", (width, height), color="black")
    draw = ImageDraw.Draw(image)

    # 绘制文本
    text = "Hello World!"
    draw.text((123, 123), text)

    return image


def convert_to_16bit(image):
    """将PIL图像转换为16位RGB565格式"""
    # 转换为RGB模式
    if image.mode != "RGB":
        image = image.convert("RGB")

    # 转换为numpy数组
    img_array = np.array(image)

    # 转换为16位RGB565格式
    r = (img_array[:, :, 0] >> 3).astype(np.uint16) << 11
    g = (img_array[:, :, 1] >> 2).astype(np.uint16) << 5
    b = (img_array[:, :, 2] >> 3).astype(np.uint16)

    rgb565 = r | g | b
    return rgb565.tobytes()


def send_to_frame_buffer(frame_data, port):
    """通过ZMQ发送帧数据到frame buffer"""
    try:
        context = zmq.Context()
        socket = context.socket(zmq.REQ)
        socket.connect(f"tcp://localhost:{port}")

        # 发送帧数据
        socket.send(frame_data)

        # 等待响应
        response = socket.recv()
        print(f"帧数据发送成功，响应: {response.decode()}")

        socket.close()
        context.term()
        return True
    except Exception as e:
        print(f"发送帧数据失败: {e}")
        return False


def main():
    print("Hello World 屏幕渲染程序启动...")

    # 获取screen信息
    screen_info = get_screen_info()
    if not screen_info:
        print("无法获取screen信息，程序退出")
        return

    print(f"获取到screen信息: {json.dumps(screen_info, indent=2, ensure_ascii=False)}")

    # 提取屏幕参数
    width, height = screen_info["screen_size"]
    bits_per_pixel = screen_info["bits_per_pixel"]
    frame_buffer_port = screen_info["frame_buffer_port"]

    print(f"屏幕尺寸: {width}x{height}")
    print(f"像素位数: {bits_per_pixel}")
    print(f"帧缓冲端口: {frame_buffer_port}")

    # 创建hello world图像
    print("创建Hello World图像...")
    image = create_hello_world_image(width, height)

    # 转换为16位格式
    print("转换图像格式...")
    frame_data = convert_to_16bit(image)

    # 发送到frame buffer
    print(f"发送图像到frame buffer (端口: {frame_buffer_port})...")
    if send_to_frame_buffer(frame_data, frame_buffer_port):
        print("✅ Hello World 图像渲染成功！")
    else:
        print("❌ 图像渲染失败")


if __name__ == "__main__":
    main()
