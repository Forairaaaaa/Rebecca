import utils.screen_socket as screen_socket
from utils.logger import logger
import numpy as np
import asyncio
import random
import zmq
import json


def convert_rgb_to_16bit(rgb: int) -> int:
    r = (rgb >> 16) & 0xFF
    g = (rgb >> 8) & 0xFF
    b = rgb & 0xFF

    return ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)


async def fb_test_task(info):
    name = info.get("name")
    frame_buffer_port = info.get("frame_buffer_port")
    screen_size = info.get("screen_size")
    bits_per_pixel = info.get("bits_per_pixel")

    logger.info(
        f"[{name}] frame_buffer_port: {frame_buffer_port}, screen_size: {screen_size}, bits_per_pixel: {bits_per_pixel}"
    )

    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    socket.connect(f"tcp://localhost:{frame_buffer_port}")

    if bits_per_pixel == 16:
        colors = [
            convert_rgb_to_16bit(0xFF0000),
            convert_rgb_to_16bit(0x00FF00),
            convert_rgb_to_16bit(0x0000FF),
            convert_rgb_to_16bit(0xFFFFFF),
            convert_rgb_to_16bit(0x000000),
        ]
        fb = np.zeros(screen_size, dtype=np.uint16)
    elif bits_per_pixel == 32:
        colors = [0xFF0000, 0x00FF00, 0x0000FF, 0xFFFFFF, 0x000000]
        fb = np.zeros(screen_size, dtype=np.uint32)
    else:
        logger.error(f"[{name}] unsupported bits_per_pixel: {bits_per_pixel}")
        return

    while True:
        for color in colors:
            fb.fill(color)
            logger.info(f"[{name}] set fb: #{hex(color)}")

            socket.send(fb.tobytes())
            response = socket.recv_json()
            logger.info(f"[{name}] response: {response}")

            await asyncio.sleep(random.uniform(0.1, 1))


async def main():
    logger.info("start socket test")

    fb_infos = screen_socket.get_available_socket_infos()

    logger.info(f"get fb infos: {json.dumps(fb_infos, indent=4)}")

    if len(fb_infos) == 0:
        logger.error("no frame buffers found")
        return

    logger.info("create tasks")

    tasks = [fb_test_task(info) for info in fb_infos]

    logger.info("start tasks")
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)
