from utils.logger import logger
from PIL import ImageFont
import yaml


THEMES_DIR = "theme/themes"
DEFAULT_THEME = "ayu_dark"
DEFAULT_FONT = "IBMPlexMono"


_theme = None
_font = DEFAULT_FONT


def load_theme(theme_name: str):
    with open(f"{THEMES_DIR}/{theme_name}.yaml", "r") as f:
        theme = yaml.safe_load(f)
    return theme


def get_theme():
    global _theme
    if _theme is None:
        _theme = load_theme(DEFAULT_THEME)
    return _theme


def set_theme(theme_name: str):
    global _theme
    _theme = load_theme(theme_name)


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
