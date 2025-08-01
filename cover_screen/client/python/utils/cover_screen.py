from .logger import logger
from pathlib import Path
from PIL import Image
import json
import zmq
import zmq.asyncio


_screens = []


def _load_screen_infos(directory):
    global _screens
    logger.info(f"load screen from {directory}")
    dir_path = Path(directory)
    for file_path in dir_path.iterdir():
        if file_path.suffix == ".json":
            try:
                with open(file_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    _screens.append(data)
            except Exception as e:
                print(f"Failed to load {file_path.name}:", e)

    print(_screens)


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


def connect(fb_temp_dir="/tmp/cover_screen"):
    global _screens
    logger.info("connect cover screens")
    if _screens:
        stop()
    _load_screen_infos(fb_temp_dir)
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


async def push(screen_name, img: Image.Image):
    for screen in _screens:
        if screen["name"] == screen_name:
            await screen["push"](img.tobytes())
            return
    raise ValueError(f"Screen {screen_name} not found")


def stop():
    global _screens
    logger.info("stop cover screen")
    for screen in _screens:
        if "socket" in screen:
            screen["socket"].close()
    _screens = []
