from utils.logger import logger
import utils.cover_screen as cover_screen
from PIL import Image, ImageDraw
import asyncio


async def main():
    for i in range(10):
        for screen in cover_screen.get_screens():
            img = Image.new("RGBA", (280, 240), (0, 0, 0, 255))
            draw = ImageDraw.Draw(img)
            draw.rectangle((100, 100, 300, 300), fill=(255, 0, 0, 255))
            draw.text((120, 120), f"Hello {i}", fill=(255, 255, 255, 255))

            await cover_screen.push(screen["name"], img)


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
