from datetime import datetime
from typing import Any
import utils.lcd as lcd
import utils.logger
import numpy as np
import json
import zmq
import os


TEMP_DIR = "/tmp/cover_screen"


logger = utils.logger.create()


class CoverScreenFrameBuffer:
    def __init__(self, name, panel):
        self._name = name
        self._panel = panel
        self._port = -1
        self._logger = utils.logger.create(tag=self._name)
        self._zmq_socket = self._create_zmq_socket()
        self._create_info_file()

    def _create_zmq_socket(self):
        self._logger.info("create zmq socket")

        context = zmq.Context()
        socket = context.socket(zmq.REP)

        self._port = socket.bind_to_random_port("tcp://*")
        self._logger.info(f"bind to port: {self._port}")

        return socket

    def _create_info_file(self):
        self._logger.info("create info file")

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

        self._logger.info(f"write info file: {info_path}")

    def update(self):
        raw_data: Any = self._zmq_socket.recv()

        try:
            src = np.frombuffer(raw_data, dtype=np.uint32)
            np.copyto(
                self._panel.frame_buffer.buffer,
                src.reshape(self._panel.frame_buffer.buffer.shape),
            )
            self._panel.frame_buffer.push()
        except Exception as e:
            self._logger.error(f"error: {e}")
            self._zmq_socket.send_json({"status": -1, "msg": str(e)})
            return

        self._zmq_socket.send_json({"status": 0, "msg": "ok👌"})


def main():
    logger.info("start cover screen service")

    logger.info("create panels")
    panels = []
    panels.append(lcd.Zjy169(gpio_DC=27, gpio_RST=17, gpio_LIGHT=25, rotate=3))

    logger.info("create frame buffers")
    fbs = [CoverScreenFrameBuffer(f"fb{i}", panels[i]) for i in range(len(panels))]

    while True:
        for fb in fbs:
            fb.update()


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
