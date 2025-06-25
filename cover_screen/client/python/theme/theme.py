# https://github.com/warpdotdev/themes
import yaml


THEMES_DIR = "theme/themes"
DEFAULT_THEME = "ayu_dark"


_theme = None


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
