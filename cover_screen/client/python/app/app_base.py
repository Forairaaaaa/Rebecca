import utils.cover_screen as cover_screen
from utils.logger import logger
from PIL import Image
import asyncio


class AppBase:
    def __init__(self, screen_name: str):
        self.screen_name = screen_name
        self.task = asyncio.create_task(self.main())

    async def main(self):
        pass

    async def stop(self):
        self.task.cancel()
        try:
            await self.task
        except asyncio.CancelledError:
            pass

    async def render(self, image: Image.Image):
        try:
            await cover_screen.push(self.screen_name, image)
        except Exception as e:
            logger.error(f"render {self.screen_name} failed: {e}")
