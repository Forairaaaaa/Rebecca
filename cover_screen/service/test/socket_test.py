from utils.logger import logger
import numpy as np
import asyncio
import random
import json
import zmq
import os
import api


async def fb_test_task(fb_name):
    info_path = f"{api.frame_buffer.TEMP_DIR}/{fb_name}.json"
    if not os.path.exists(info_path):
        logger.error(f"[{fb_name}] info file not found: {info_path}")
        return

    port = -1
    width = -1
    height = -1
    depth = -1

    with open(info_path, "r") as f:
        info = json.load(f)
        port = info.get("port")
        width = info.get("width")
        height = info.get("height")
        depth = info.get("depth")

    logger.info(
        f"[{fb_name}] port: {port}, width: {width}, height: {height}, depth: {depth}"
    )

    context = zmq.Context()
    socket = context.socket(zmq.REQ)
    socket.connect(f"tcp://localhost:{port}")

    fb = np.zeros((height, width), dtype=np.uint32)

    colors = [0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFFFF, 0x00000000]

    while True:
        for color in colors:
            fb.fill(color)
            logger.info(f"[{fb_name}] set fb: #{hex(color)}")

            socket.send(fb.tobytes())
            response = socket.recv_json()
            logger.info(f"[{fb_name}] response: {response}")

            await asyncio.sleep(random.uniform(0.1, 1))


async def main():
    logger.info("start socket test")

    fb_names = []
    for filename in os.listdir(api.frame_buffer.TEMP_DIR):
        if filename.endswith(".json"):
            fb_name = filename.replace(".json", "")
            fb_names.append(fb_name)
    logger.info(f"available frame buffers: {fb_names}")

    if len(fb_names) == 0:
        logger.error("no frame buffers found")
        return

    logger.info("create tasks")
    tasks = [fb_test_task(fb_name) for fb_name in fb_names]

    logger.info("start tasks")
    await asyncio.gather(*tasks)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)
