import utils.cover_screen as cover_screen
from theme.theme import set_theme, set_font
from app.htop_like import HtopLike
from app.terminal import Terminal
from utils.logger import logger
import argparse
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
        await asyncio.sleep(2333)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--theme", type=str, default=None)
    parser.add_argument("--font", type=str, default=None)
    args = parser.parse_args()

    if args.theme:
        set_theme(args.theme)
    if args.font:
        set_font(args.font)

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
