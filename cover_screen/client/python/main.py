import utils.cover_screen as cover_screen
from app.htop_like import HtopLike
from app.terminal import Terminal
from utils.logger import logger
import asyncio


async def main():
    app_map = {
        "fb0": HtopLike,
        "fb1": Terminal,
    }

    app_list = []
    for screen_name, app in app_map.items():
        if not cover_screen.exists(screen_name):
            continue
        app_list.append(app(screen_name))

    while True:
        await asyncio.sleep(1)


if __name__ == "__main__":
    try:
        logger.info("start cover screen client")
        cover_screen.connect()
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)
    finally:
        cover_screen.stop()

    logger.info("stop cover screen client")
