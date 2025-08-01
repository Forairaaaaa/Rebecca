import utils.screen_socket as screen_socket
from multiprocessing import Process
from utils.logger import logger
from .lcd import PyGamePanel
import argparse
import asyncio


async def worker(name):
    logger.info("start desktop cover screen service")

    logger.info("create panel")
    panel = PyGamePanel(width=280, height=240, scale=1)

    logger.info("create frame buffer")
    screen_socket.create_socket(
        name=name,
        screen_size=panel.device.size,
        screen_depth=4,
        on_frame_buffer=panel.pushRaw,
    )

    while True:
        await asyncio.sleep(1)


def process_worker(name):
    asyncio.run(worker(name))


def main(panel_num):
    try:
        logger.info("start desktop cover screen service")

        # pygame can only create a window, wrap worker with process
        logger.info("create processes")
        processes = [
            Process(
                target=process_worker,
                args=(f"screen{i}",),
            )
            for i in range(panel_num)
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
    screen_socket.cleanup()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--panel-num", type=int, default=2)
    args = parser.parse_args()

    main(args.panel_num)
