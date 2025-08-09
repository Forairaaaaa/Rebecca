from .logger import logger
from PIL import Image
import numpy as np
import zmq
import zmq.asyncio
import aiohttp


_screens = []


async def _load_screen_infos(host="127.0.0.1", port=12580):
    global _screens
    logger.info(f"load screen from http://{host}:{port}/get-device/all")
    url = f"http://{host}:{port}/get-device/all"

    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as response:
                if response.status == 200:
                    devices = await response.json()
                    for device in devices:
                        screen_info = {
                            "name": device["id"],  # 使用id作为name
                            "frame_buffer_port": device["info"]["frame_buffer_port"],
                            "bits_per_pixel": device["info"]["bits_per_pixel"],
                            "screen_size": device["info"]["screen_size"],
                            "description": device["info"]["description"],
                            "device_type": device["info"]["device_type"],
                        }
                        _screens.append(screen_info)
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
