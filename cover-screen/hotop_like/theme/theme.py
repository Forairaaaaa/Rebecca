# https://github.com/warpdotdev/themes/tree/main/standard
from utils.logger import logger
from PIL import ImageFont
import yaml


DEFAULT_THEME = "theme/themes/default.yaml"
DEFAULT_FONT = "CascadiaMono-Bold.ttf"


_theme = None
_font = DEFAULT_FONT


def load_theme(theme_path: str):
    with open(theme_path, "r") as f:
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


def get_font(size=12):
    font = f"{_font}"
    try:
        return ImageFont.truetype(font, size)
    except Exception as e:
        logger.error(f"load font {_font} failed: {e}")
        return ImageFont.load_default()


def set_font(font: str):
    global _font
    _font = font
