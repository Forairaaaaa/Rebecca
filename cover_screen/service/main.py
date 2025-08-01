from cover_screen.service.utils.screen_socket import create_socket, cleanup
from utils.logger import logger
import utils.lcd as lcd
import asyncio


fbs = []


async def main():
    logger.info("start cover screen service")

    logger.info("create panels")
    panels = []
    panels.append(lcd.Zjy169(gpio_DC=27, gpio_RST=17, gpio_LIGHT=25, rotate=3))

    logger.info("create frame buffers")

    global fbs

    for i, panel in enumerate(panels):
        create_socket(
            name=f"screen{i}",
            screen_size=panel.device.size,
            screen_depth=4,
            on_fb_data=panel.pushRaw,
        )

    while True:
        await asyncio.sleep(1)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except Exception as e:
        logger.error(e)

    logger.info("cleanup")
    cleanup()
