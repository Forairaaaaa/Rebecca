import utils.screen_socket as screen_socket
from utils.logger import logger
import utils.lcd as lcd
import asyncio


fbs = []


async def main():
    logger.info("start cover screen service")

    logger.info("create panels")
    panels = []
    panels.append(
        lcd.ZhongJingYuan169(
            spi_port=0, spi_device=0, gpio_DC=17, gpio_RST=22, gpio_LIGHT=23, rotate=3
        )
    )
    panels.append(
        lcd.ZhongJingYuan169(
            spi_port=0,
            spi_device=1,
            gpio_DC=27,
            gpio_RST=None,
            gpio_LIGHT=None,
            rotate=1,
        )
    )

    logger.info("create frame buffers")

    global fbs

    for i, panel in enumerate(panels):
        screen_socket.create_socket(
            name=f"screen{i}",
            screen_size=panel.device.size,
            screen_depth=4,
            on_frame_buffer=panel.pushRaw,
        )

    while True:
        await asyncio.sleep(1)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)

    logger.info("cleanup")
    screen_socket.cleanup()
