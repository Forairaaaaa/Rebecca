import utils.cover_screen as cover_screen
from theme.theme import set_theme, set_font
from app.htop_like import HtopLike
from app.terminal import Terminal
from utils.logger import logger
import argparse
import asyncio


async def main():
    app_map = {
        "screen0": HtopLike,
        # "screen1": Terminal,
    }

    app_list = []
    for screen_name, app in app_map.items():
        if not cover_screen.exists(screen_name):
            continue
        screen_size = cover_screen.get_screen_size(screen_name)
        app_list.append(app(screen_name, width=screen_size[0], height=screen_size[1]))

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
