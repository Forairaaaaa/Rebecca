from utils.logger import logger
from datetime import datetime
import json
import os
import zmq
import zmq.asyncio


TEMP_DIR = "/tmp/cover_screen"


class Ctrl:
    def __init__(self, name="ctrl", panels=[]):
        self._name = name
        self._port = -1
        self._panels = panels
        self._zmq_socket = self._create_zmq_socket()
        self._create_info_file()
        self._handlers = {
            "backlight": self._handle_backlight,
        }

    def _create_zmq_socket(self):
        logger.info(f"[{self._name}] create zmq socket")

        context = zmq.asyncio.Context()
        socket = context.socket(zmq.REP)

        self._port = socket.bind_to_random_port("tcp://*")
        logger.info(f"[{self._name}] bind to port: {self._port}")

        return socket

    def _create_info_file(self):
        logger.info(f"[{self._name}] create info file")

        os.makedirs(TEMP_DIR, exist_ok=True)

        info_path = f"{TEMP_DIR}/{self._name}.json"
        with open(info_path, "w") as f:
            json.dump(
                {
                    "name": self._name,
                    "port": self._port,
                    "created_at": datetime.now().isoformat(),
                },
                f,
            )

        logger.info(f"[{self._name}] write info file: {info_path}")

    async def listen(self):
        raw_data = await self._zmq_socket.recv_json()

        try:
            msg = json.loads(raw_data.decode("utf-8"))
            cmd = msg.get("cmd")
            handler = self._handlers.get(cmd)

            if handler:
                result = handler(msg.get("value"))
            else:
                result = {"status": -1, "msg": f"unknown command: {cmd}"}

        except Exception as e:
            logger.error(f"[{self._name}] error: {e}")
            result = {"status": -1, "msg": str(e)}

        await self._zmq_socket.send_json(result)

    def _handle_backlight(self, value):
        if isinstance(value, int):
            value = max(0, min(value, 100))
            for panel in self._panels:
                panel.device.backlight(value)
        else:
            raise ValueError(f"invalid value: {value}")

        return {"status": 0, "msg": "ok👌"}
