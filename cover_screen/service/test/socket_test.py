import utils.screen_socket as screen_socket
from utils.logger import logger
import numpy as np
import asyncio
import random
import zmq
import json


async def fb_test_task(info):
    name = info.get("name")
    frame_buffer_port = info.get("frame_buffer_port")
    screen_size = info.get("screen_size")
    screen_depth = info.get("screen_depth")

    logger.info(
        f"[{name}] frame_buffer_port: {frame_buffer_port}, screen_size: {screen_size}, screen_depth: {screen_depth}"
    )

    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    socket.connect(f"tcp://localhost:{frame_buffer_port}")

    fb = np.zeros(screen_size, dtype=np.uint32)

    colors = [0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFFFF, 0x00000000]

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
