from datetime import datetime
from typing import Any
import utils.lcd as lcd
import logging
import json
import sys
import zmq
import os


TEMP_DIR = "/tmp/cover_screen"


class CoverScreen:
    def __init__(self, name, panel, logger):
        self._name = name
        self._panel = panel
        self._logger = logger
        self._port = -1
        self._zmq_socket = self._create_zmq_socket()
        self._action_handlers = self._create_action_handlers()
        self._create_info_file()

    def _create_zmq_socket(self):
        self._logger.info(f"[{self._name}] create zmq socket")

        context = zmq.Context()
        socket = context.socket(zmq.REP)

        self._port = socket.bind_to_random_port("tcp://*")
        self._logger.info(f"[{self._name}] bind to port: {self._port}")

        return socket

    def _create_info_file(self):
        self._logger.info(f"[{self._name}] create info file")

        os.makedirs(TEMP_DIR, exist_ok=True)

        info_path = f"{TEMP_DIR}/{self._name}.json"
        with open(info_path, "w") as f:
            json.dump(
                {
                    "name": self._name,
                    "port": self._port,
                    "width": self._panel.device.width,
                    "height": self._panel.device.height,
                    "depth": self._panel.frame_buffer.depth,
                    "created_at": datetime.now().isoformat(),
                },
                f,
            )

        self._logger.info(f"[{self._name}] write info file: {info_path}")

    def _create_action_handlers(self):
        handlers = {}

        def get_fb(self):
            self._zmq_socket.send(self._panel.frame_buffer.tobytes())

        def set_fb(self):
            data = self._zmq_socket.recv()
            self._panel.frame_buffer.buffer.frombytes(data)
            self._zmq_socket.send("ok👌")

        def set_brightness(self):
            data = self._zmq_socket.recv()
            if isinstance(data, int):
                self._panel.device.backlight(data)
            else:
                self._logger.warning(f"[{self._name}] invalid brightness: {data}")
                self._zmq_socket.send("invalid brightness😰")
            self._zmq_socket.send("ok👌")

        handlers["get_fb"] = get_fb
        handlers["set_fb"] = set_fb
        handlers["set_brightness"] = set_brightness

        return handlers

    def update(self):
        message: Any = self._zmq_socket.recv_json()
        action = message["action"]
        if action in self._action_handlers:
            self._action_handlers[action](self)
        else:
            self._logger.warning(f"[{self._name}] unknown action: {action}")
            self._zmq_socket.send("unknown action😰")


def create_logger():
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    ch = logging.StreamHandler(sys.stdout)
    ch.setFormatter(logging.Formatter("[%(asctime)s] [%(levelname)s] %(message)s"))
    logger.addHandler(ch)

    return logger


logger = create_logger()


def main():
    logger.info("create panels")
    panels = []
    panels.append(lcd.Zjy169(gpio_DC=27, gpio_RST=17, gpio_LIGHT=25, rotate=3))

    logger.info("create screens")
    screens = [CoverScreen(f"screen{i}", panels[i], logger) for i in range(len(panels))]

    while True:
        for screen in screens:
            screen.update()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)

    logger.info("cleanup")
    for filename in os.listdir(TEMP_DIR):
        filepath = os.path.join(TEMP_DIR, filename)
        if os.path.isfile(filepath):
            os.remove(filepath)
