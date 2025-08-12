from stransi import Ansi, SetColor, SetAttribute
from PIL import Image, ImageDraw, ImageFont
from stransi.attribute import Attribute
import ochre


class AnsiRender:
    """
    Parse and render ansi text into PIL image.
    """

    def __init__(
        self, width: int, height: int, font: ImageFont.FreeTypeFont, theme: dict
    ):
        self.width = width
        self.height = height
        self.font = font
        self.theme = theme

        self.char_width = self.font.getbbox(" ")[2]
        self.char_height = self.font.getbbox(" ")[3]
        self.rows = self.height // self.char_height
        self.cols = self.width // self.char_width

        # Attributes
        self.x, self.y = 0, 0
        self.fg_color = self.theme["foreground"]
        self.bg_color = self.theme["background"]
        self.bold = False
        self.underline = False
        self.reverse = False

        # Image
        self.image = Image.new("RGBA", (self.width, self.height), (0, 0, 0, 255))
        self.draw = ImageDraw.Draw(self.image)

    def reset_attributes(self):
        self.fg_color = self.theme["foreground"]
        self.bg_color = self.theme["background"]
        self.bold = False
        self.underline = False
        self.reverse = False

    def get_color_from_theme(self, instr: SetColor | None, bold: bool = False) -> str:
        if not isinstance(instr, SetColor):
            return self.theme["foreground"]

        if instr.color is None:
            # Reset to default foreground/background
            return (
                self.theme["foreground"]
                if instr.role.name == "FOREGROUND"
                else self.theme["background"]
            )

        # Handle Ansi256 (maps 0–15 to Ansi16)
        elif isinstance(instr.color, ochre.Ansi256):
            if instr.color.code < 16:
                index = instr.color.code
                brightness = "bright" if (index >= 8 or bold) else "normal"
                ansi_index = index % 8
                name = [
                    "black",
                    "red",
                    "green",
                    "yellow",
                    "blue",
                    "magenta",
                    "cyan",
                    "white",
                ][ansi_index]
                return self.theme["terminal_colors"][brightness][name]
            else:
                # Optional: map 256-color to RGB here
                # Here we just return a gray fallback
                return "#888888"

    def parse_and_render(self, ansi_text):
        ansi = Ansi(ansi_text)
        # print(list(ansi.instructions()))

        x, y = 0, 0
        fg_color = self.theme["foreground"]
        bg_color = self.theme["background"]
        bold = False
        underline = False
        reverse = False

        for instr in ansi.instructions():
            if isinstance(instr, str):
                for ch in instr:
                    if ch == "\n":
                        x = 0
                        y += 1
                    else:
                        actual_fg = fg_color
                        actual_bg = bg_color
                        if reverse:
                            actual_fg, actual_bg = actual_bg, actual_fg

                        # Draw background
                        self.draw.rectangle(
                            [
                                x * self.char_width,
                                y * self.char_height + 3,
                                (x + 1) * self.char_width,
                                (y + 1) * self.char_height,
                            ],
                            fill=actual_bg,
                        )

                        # Draw text
                        self.draw.text(
                            (x * self.char_width, y * self.char_height),
                            ch,
                            font=self.font,
                            fill=actual_fg,
                        )

                        # Draw underline
                        if underline:
                            self.draw.line(
                                [
                                    x * self.char_width,
                                    (y + 1) * self.char_height + 1,
                                    (x + 1) * self.char_width,
                                    (y + 1) * self.char_height + 1,
                                ],
                                fill=actual_fg,
                            )

                        x += 1

            elif isinstance(instr, SetColor):
                if instr.role.name == "FOREGROUND":
                    fg_color = self.get_color_from_theme(instr, bold=bold)
                elif instr.role.name == "BACKGROUND":
                    bg_color = self.get_color_from_theme(instr, bold=bold)

            elif isinstance(instr, SetAttribute):
                attr = instr.attribute
                if attr == Attribute.BOLD:
                    bold = True
                elif attr == Attribute.UNDERLINE:
                    underline = True  # 暂未用，可加绘线逻辑
                elif attr == Attribute.REVERSE:
                    reverse = True
                elif attr == Attribute.NORMAL:
                    # 重置所有样式
                    bold = False
                    underline = False
                    reverse = False
                    fg_color = self.theme["foreground"]
                    bg_color = self.theme["background"]

    def get_image(self):
        return self.image
