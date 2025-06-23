from multiprocessing import Process, Event
from utils.logger import logger
from .lcd import PyGamePanel
import time
import api


def worker(stop_event, name):
    logger.info("start desktop cover screen service")

    logger.info("create panel")
    panel = PyGamePanel(width=280, height=240, scale=1)

    logger.info("create frame buffer")
    fb = api.FrameBuffer(name=name, panel=panel)

    while not stop_event.is_set():
        fb.update()


def main():
    try:
        logger.info("start desktop cover screen service")

        logger.info("create processes")
        stop_event = Event()
        processes = [
            Process(target=worker, args=(stop_event, f"fb{i}")) for i in range(2)
        ]
        for p in processes:
            p.start()

        while True:
            time.sleep(1)

    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)
    finally:
        stop_event.set()
        for p in processes:
            p.join()

    logger.info("cleanup")
    api.frame_buffer.cleanup()


if __name__ == "__main__":
    main()
