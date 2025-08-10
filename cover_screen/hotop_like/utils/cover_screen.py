from .logger import logger
from PIL import Image
import numpy as np
import zmq
import zmq.asyncio
import aiohttp


_screens = []


async def _load_screen_infos(host="127.0.0.1", port=12580):
    global _screens

    # 首先获取设备列表
    devices_url = f"http://{host}:{port}/devices"
    logger.info(f"load devices from {devices_url}")

    try:
        async with aiohttp.ClientSession() as session:
            # 获取设备列表
            async with session.get(devices_url) as response:
                if response.status == 200:
                    # 先获取响应文本，然后手动解析JSON
                    response_text = await response.text()
                    logger.debug(f"Raw response from devices endpoint: {response_text}")

                    try:
                        import json

                        device_names = json.loads(response_text)
                    except json.JSONDecodeError as e:
                        logger.error(f"Failed to parse JSON from devices endpoint: {e}")
                        raise e

                    logger.info(f"Found devices: {device_names}")

                    # 为每个screen设备获取详细信息
                    for device_name in device_names:
                        if device_name.startswith("screen"):  # 只处理screen设备
                            info_url = f"http://{host}:{port}/{device_name}/info"
                            logger.info(
                                f"Getting info for {device_name} from {info_url}"
                            )

                            async with session.get(info_url) as info_response:
                                if info_response.status == 200:
                                    # 同样处理info响应的JSON解析
                                    info_text = await info_response.text()
                                    logger.debug(
                                        f"Raw response from {device_name}/info: {info_text}"
                                    )

                                    try:
                                        device_info = json.loads(info_text)
                                    except json.JSONDecodeError as e:
                                        logger.error(
                                            f"Failed to parse JSON from {device_name}/info: {e}"
                                        )
                                        continue

                                    screen_info = {
                                        "name": device_name,
                                        "frame_buffer_port": device_info[
                                            "frame_buffer_port"
                                        ],
                                        "bits_per_pixel": device_info["bits_per_pixel"],
                                        "screen_size": device_info["screen_size"],
                                        "description": device_info["description"],
                                        "device_type": device_info["device_type"],
                                    }
                                    _screens.append(screen_info)
                                    logger.info(
                                        f"Loaded screen info for {device_name}: {screen_info}"
                                    )
                                else:
                                    logger.error(
                                        f"Failed to get info for {device_name}, status code: {info_response.status}"
                                    )
                else:
                    logger.error(
                        f"Failed to get devices, status code: {response.status}"
                    )
                    raise Exception(
                        f"HTTP request failed with status {response.status}"
                    )
    except Exception as e:
        logger.error(f"Failed to load screen infos from HTTP: {e}")
        raise e

    logger.info(f"Loaded {len(_screens)} screens: {[s['name'] for s in _screens]}")


def _create_sockets():
    global _screens

    context = zmq.asyncio.Context()

    for screen in _screens:
        zmq_port = f"tcp://127.0.0.1:{screen['frame_buffer_port']}"
        logger.info(f"connect to {zmq_port}")

        socket = context.socket(zmq.REQ)
        socket.connect(zmq_port)
        screen["socket"] = socket

        async def push(data, sock=socket):
            logger.debug(f"push data: {len(data)} bytes")
            await sock.send(data)
            response = await sock.recv()
            logger.debug(f"response: {response.decode()}")

        screen["push"] = push


async def connect(host="127.0.0.1", port=12580):
    global _screens
    logger.info("connect cover screens")
    if _screens:
        stop()
    await _load_screen_infos(host, port)
    _create_sockets()


def get_screens():
    return _screens


def get_screen_num():
    return len(_screens)


def exists(screen_name):
    for screen in _screens:
        if screen["name"] == screen_name:
            return True
    return False


async def push_rgba(screen_name, img: Image.Image):
    for screen in _screens:
        if screen["name"] == screen_name:
            await screen["push"](img.tobytes())
            return
    raise ValueError(f"Screen {screen_name} not found")


async def push_rgb565(screen_name, img: Image.Image):
    def image_to_rgb565_bytes(img: Image.Image) -> bytes:
        img = img.convert("RGB")
        arr = np.array(img)

        r = (arr[:, :, 0] >> 3).astype(np.uint16)
        g = (arr[:, :, 1] >> 2).astype(np.uint16)
        b = (arr[:, :, 2] >> 3).astype(np.uint16)

        rgb565 = (r << 11) | (g << 5) | b

        return rgb565.flatten().astype("<u2").tobytes()

    for screen in _screens:
        if screen["name"] == screen_name:
            await screen["push"](image_to_rgb565_bytes(img))
            return
    raise ValueError(f"Screen {screen_name} not found")


async def push(screen_name, img: Image.Image):
    bits_per_pixel = get_screen_bits_per_pixel(screen_name)
    if bits_per_pixel == 16:
        await push_rgb565(screen_name, img)
    elif bits_per_pixel == 32:
        await push_rgba(screen_name, img)
    else:
        raise ValueError(
            f"Screen {screen_name} has unsupported bits per pixel: {bits_per_pixel}"
        )


def get_screen_size(screen_name) -> tuple[int, int]:
    for screen in _screens:
        if screen["name"] == screen_name:
            return screen["screen_size"]
    raise ValueError(f"Screen {screen_name} not found")


def get_screen_bits_per_pixel(screen_name) -> int:
    for screen in _screens:
        if screen["name"] == screen_name:
            return screen["bits_per_pixel"]
    raise ValueError(f"Screen {screen_name} not found")


def stop():
    global _screens
    logger.info("stop cover screen")
    for screen in _screens:
        if "socket" in screen:
            screen["socket"].close()
    _screens = []
