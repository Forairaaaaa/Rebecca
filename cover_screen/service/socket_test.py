import numpy as np
import logging
import time
import json
import zmq
import os
import sys


TEMP_DIR = "/tmp/cover_screen"


def create_logger():
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    ch = logging.StreamHandler(sys.stdout)
    ch.setFormatter(logging.Formatter("[%(asctime)s] [%(levelname)s] %(message)s"))
    logger.addHandler(ch)

    return logger


logger = create_logger()


def fb_test(fb_name="fb0"):
    info_path = f"{TEMP_DIR}/{fb_name}.json"
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

            time.sleep(1)


def main():
    fb_test()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)
