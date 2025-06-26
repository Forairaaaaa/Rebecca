from PIL import ImageFont
from utils.logger import logger


_font = "IBMPlexMono"


def get_font(size=12, weight="Bold"):
    font = f"{_font}-{weight}.ttf"
    try:
        return ImageFont.truetype(font, size)
    except Exception as e:
        logger.error(f"load font {_font} failed: {e}")
        return ImageFont.load_default()


def set_font(font: str):
    global _font
    _font = font
