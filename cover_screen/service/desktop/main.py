from multiprocessing import Process
from utils.logger import logger
from .lcd import PyGamePanel
import argparse
import asyncio
import api


async def worker(name):
    logger.info("start desktop cover screen service")

    logger.info("create panel")
    panel = PyGamePanel(width=280, height=240, scale=1)

    logger.info("create frame buffer")
    fb = api.FrameBuffer(name=name, panel=panel)

    while True:
        await fb.listen()


def process_worker(name):
    asyncio.run(worker(name))


def main(panel_num):
    try:
        logger.info("start desktop cover screen service")

        # pygame can only create a window, wrap worker with process
        logger.info("create processes")
        processes = [
            Process(target=process_worker, args=(f"fb{i}",)) for i in range(panel_num)
        ]
        for p in processes:
            p.start()

        for p in processes:
            p.join()

    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)

    logger.info("cleanup")
    api.frame_buffer.cleanup()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--panel-num", type=int, default=2)
    args = parser.parse_args()

    main(args.panel_num)
