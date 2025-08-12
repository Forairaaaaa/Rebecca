from theme.theme import get_font, get_theme
from utils.ansi_render import AnsiRender
from rich.console import Console
from app.app_base import AppBase


class Terminal(AppBase):
    def __init__(self, screen_name: str, width=280, height=240):
        super().__init__(screen_name)

        # Load resources
        self.font = get_font()
        self.theme = get_theme()

        self.ansi_render = AnsiRender(width, height, self.font, self.theme)
        self.cols = self.ansi_render.cols
        self.rows = self.ansi_render.rows

    async def main(self):
        print("???")

        console = Console(width=self.cols, height=self.rows, record=True)

        console.print("[red]Looks like a link")
        console.print(
            "??????????????????????????????????????????????????????????????????????????????"
        )
        console.print("[underline]Looks like a link")
        console.print("[reverse]Looks like a link")
        console.print("[yellow]Looks like a link")

        print("--------------------------------")
        ahah = console.export_text(styles=True)
        # print(ahah)

        # print(self.get_theme_color(SetColor(color=ochre.Ansi256(code=1))))

        self.ansi_render.parse_and_render(ahah)
        await self.render(self.ansi_render.get_image())
