from utils.logger import logger
from datetime import datetime
from typing import Any
import numpy as np
import json
import os
import zmq
import zmq.asyncio


TEMP_DIR = "/tmp/cover_screen"


class FrameBuffer:
    def __init__(self, name, panel, port=None):
        self._name = name
        self._panel = panel
        self._port = port
        self._zmq_socket = self._create_zmq_socket()
        self._create_info_file()

    def _create_zmq_socket(self):
        logger.info(f"[{self._name}] create zmq socket")

        context = zmq.asyncio.Context()
        socket = context.socket(zmq.REP)

        if self._port is None:
            self._port = socket.bind_to_random_port("tcp://*")
        else:
            socket.bind(f"tcp://*:{self._port}")
        logger.info(f"[{self._name}] bind to port: {self._port}")

        return socket

    def _create_info_file(self):
        logger.info(f"[{self._name}] create info file")

        os.makedirs(TEMP_DIR, exist_ok=True)

        buffer_shape = self._panel.frame_buffer.buffer.shape
        buffer_itemsize = self._panel.frame_buffer.buffer.itemsize

        info_path = f"{TEMP_DIR}/{self._name}.json"
        with open(info_path, "w") as f:
            json.dump(
                {
                    "name": self._name,
                    "port": self._port,
                    "width": buffer_shape[1],
                    "height": buffer_shape[0],
                    "depth": buffer_itemsize,
                    "created_at": datetime.now().isoformat(),
                },
                f,
            )

        logger.info(f"[{self._name}] write info file: {info_path}")

    async def listen(self):
        raw_data: Any = await self._zmq_socket.recv()

        try:
            src = np.frombuffer(raw_data, dtype=np.uint32)
            np.copyto(
                self._panel.frame_buffer.buffer,
                src.reshape(self._panel.frame_buffer.buffer.shape),
            )
            self._panel.frame_buffer.push()
        except Exception as e:
            logger.error(f"[{self._name}] error: {e}")
            await self._zmq_socket.send_json({"status": -1, "msg": str(e)})
            return

        await self._zmq_socket.send_json({"status": 0, "msg": "ok👌"})


def cleanup():
    for filename in os.listdir(TEMP_DIR):
        filepath = os.path.join(TEMP_DIR, filename)
        if os.path.isfile(filepath):
            os.remove(filepath)
