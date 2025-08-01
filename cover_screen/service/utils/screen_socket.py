from utils.logger import logger
from datetime import datetime
from typing import Any, Callable
import asyncio
import json
import os
import zmq
import zmq.asyncio


_SCREEN_SOCKET_TEMP_DIR = "/tmp/cover_screen"


class _ScreenSocket:
    """
    A bridge between screen controll and zmq socket.
    """

    def __init__(
        self,
        name,
        screen_size: tuple[int, int] = (280, 240),
        screen_depth: int = 4,
        frame_buffer_port=None,
        command_port=None,
        on_frame_buffer: Callable[[Any], None] = None,
        on_command: Callable[[Any], None] = None,
    ):
        self._name = name
        self._screen_size = screen_size
        self._screen_depth = screen_depth
        self._on_frame_buffer = on_frame_buffer
        self._on_command = on_command

        self._fb_socket, self._frame_buffer_port = self._create_zmq_socket(
            name, frame_buffer_port
        )
        self._ctrl_socket, self._command_port = self._create_zmq_socket(
            name, command_port
        )

        self._create_info_file()

        asyncio.create_task(self._fb_worker())

    def _create_zmq_socket(self, name, port):
        logger.info(f"[{name}] create zmq socket")

        context = zmq.asyncio.Context()
        socket = context.socket(zmq.REP)

        if port is None:
            port = socket.bind_to_random_port("tcp://*")
        else:
            socket.bind(f"tcp://*:{port}")
        logger.info(f"[{name}] bind to port: {port}")

        return socket, port

    def _create_info_file(self):
        logger.info(f"[{self._name}] create info file")

        os.makedirs(_SCREEN_SOCKET_TEMP_DIR, exist_ok=True)

        info_path = f"{_SCREEN_SOCKET_TEMP_DIR}/{self._name}.json"
        with open(info_path, "w") as f:
            json.dump(
                {
                    "name": self._name,
                    "screen_size": self._screen_size,
                    "screen_depth": self._screen_depth,
                    "frame_buffer_port": self._frame_buffer_port,
                    "command_port": self._command_port,
                    "created_at": datetime.now().isoformat(),
                },
                f,
            )

        logger.info(f"[{self._name}] write info file: {info_path}")

    async def _fb_worker(self):
        while True:
            raw_data: Any = await self._fb_socket.recv()

            try:
                if self._on_frame_buffer is None:
                    raise Exception("empty on_fb_data callback")
                self._on_frame_buffer(raw_data)
            except Exception as e:
                logger.error(f"[{self._name}] error: {e}")
                await self._fb_socket.send_json({"status": -1, "msg": str(e)})
                return

            await self._fb_socket.send_json({"status": 0, "msg": "ok👌"})


_screen_socket_instances = []


def create_socket(
    name,
    screen_size: tuple[int, int] = (280, 240),
    screen_depth: int = 4,
    frame_buffer_port=None,
    command_port=None,
    on_frame_buffer: Callable[[Any], None] = None,
    on_command: Callable[[Any], None] = None,
):
    global _screen_socket_instances

    screen_socket = _ScreenSocket(
        name=name,
        screen_size=screen_size,
        screen_depth=screen_depth,
        frame_buffer_port=frame_buffer_port,
        command_port=command_port,
        on_frame_buffer=on_frame_buffer,
        on_command=on_command,
    )
    _screen_socket_instances.append(screen_socket)

    return screen_socket


def cleanup():
    global _screen_socket_instances

    _screen_socket_instances.clear()

    for filename in os.listdir(_SCREEN_SOCKET_TEMP_DIR):
        filepath = os.path.join(_SCREEN_SOCKET_TEMP_DIR, filename)
        if os.path.isfile(filepath):
            os.remove(filepath)


def get_available_socket_infos():
    infos = []

    for filename in os.listdir(_SCREEN_SOCKET_TEMP_DIR):
        info_path = f"{_SCREEN_SOCKET_TEMP_DIR}/{filename}"
        if not os.path.exists(info_path):
            raise Exception(f"info file not found: {info_path}")

        with open(info_path, "r") as f:
            infos.append(json.load(f))

    return infos
