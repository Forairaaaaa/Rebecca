from PIL import Image, ImageDraw
from app.app_base import AppBase
from theme.font import get_font
from theme.theme import get_theme
import asyncio
import psutil
import time
import math
import os


class HtopLike(AppBase):
    def __init__(self, screen_name: str, width=280, height=240):
        super().__init__(screen_name)

        # Load resources
        self.font = get_font()
        self.theme = get_theme()
        self.color_background = self.theme["background"]
        self.color_red = self.theme["terminal_colors"]["normal"]["red"]
        self.color_blue = self.theme["terminal_colors"]["normal"]["blue"]
        self.color_green = self.theme["terminal_colors"]["normal"]["green"]
        self.color_white = self.theme["terminal_colors"]["normal"]["white"]
        self.color_black = self.theme["terminal_colors"]["normal"]["black"]
        self.bright_black = self.theme["terminal_colors"]["bright"]["black"]
        self.bright_yellow = self.theme["terminal_colors"]["bright"]["yellow"]

        self.font_width = self.font.getbbox(" ")[2]

        # Init image
        self.width = width
        self.height = height
        self.image = Image.new("RGBA", (self.width, self.height), (0, 0, 0, 255))
        self.draw = ImageDraw.Draw(self.image)

    async def main(self):
        while True:
            self.draw.rectangle(
                [0, 0, self.width, self.height],
                fill=self.color_background,
            )

            base_x = 8
            base_y = 8

            self.draw_cpu_bars(base_x, base_y)
            self.draw_mem_bar(base_x, base_y + 60)
            self.draw_infos(base_x, base_y + 75)
            self.draw_process_infos(base_x - 1, base_y + 110)

            await self.render(self.image)
            await asyncio.sleep(1)

    def draw_mem_bar(self, x_base, y_base):
        mem = psutil.virtual_memory()

        bar_width = 33

        # Draw panel
        self.draw.text((x_base, y_base), "Mem", font=self.font, fill=self.color_blue)
        self.draw.text(
            (x_base + self.font_width * 3, y_base),
            f"[{' ' * (bar_width - 1)}]",
            font=self.font,
            fill=self.color_white,
        )

        # Get bar width
        used_bar_width = int(math.floor(mem.used / mem.total * bar_width))

        # Fill bar content
        bar_content = [" "] * (bar_width - 1)

        usage_text = f"{mem.used / (1024 * 1024 * 1024):>4.1f}G/{mem.total / (1024 * 1024 * 1024):>4.1f}G"
        bar_content[-len(usage_text) :] = usage_text

        for i in range(used_bar_width):
            if i > len(bar_content) - 1:
                break
            if bar_content[i] == " ":
                bar_content[i] = "|"

        # print("".join(bar_content))

        # Divide bar content
        user_part_content = "".join(bar_content[:used_bar_width])
        usage_part_content = "".join(bar_content[used_bar_width:])

        # Get bar start x
        used_part_start_x = x_base + self.font_width * 4
        usage_part_start_x = used_part_start_x + self.font_width * used_bar_width

        self.draw.text(
            (used_part_start_x, y_base),
            user_part_content,
            font=self.font,
            fill=self.color_green,
        )
        self.draw.text(
            (usage_part_start_x, y_base),
            usage_part_content,
            font=self.font,
            fill=self.bright_black,
        )

    def draw_infos(self, x_base, y_base):
        # Avg load
        load_avg = os.getloadavg()
        self.draw.text(
            (x_base, y_base),
            f"Load avg: {load_avg[0]:.2f} {load_avg[1]:.2f} {load_avg[2]:.2f}",
            font=self.font,
            fill=self.color_blue,
        )

        # Uptime
        def get_uptime():
            boot_time = psutil.boot_time()
            now = time.time()
            uptime_seconds = int(now - boot_time)

            days = uptime_seconds // 86400
            hours = (uptime_seconds % 86400) // 3600
            minutes = (uptime_seconds % 3600) // 60
            seconds = uptime_seconds % 60

            if days > 0:
                return f"Uptime: {days} days, {hours:02}:{minutes:02}:{seconds:02}"
            else:
                return f"Uptime: {hours:02}:{minutes:02}:{seconds:02}"

        self.draw.text(
            (x_base, y_base + 15),
            get_uptime(),
            font=self.font,
            fill=self.color_blue,
        )

        # Nice
        text = "(..•˘_˘•..)"
        self.draw.text(
            (x_base + self.font_width * 37 - self.font_width * len(text), y_base + 8),
            text,
            font=self.font,
            fill=self.bright_yellow,
        )

    def draw_cpu_bars(self, x_base, y_base):
        cpu_percents = psutil.cpu_percent(percpu=True)[:4]
        cpu_times_percent = psutil.cpu_times_percent(percpu=True)[:4]

        bar_width = 35

        def draw_panel(x, y, cpu_num):
            self.draw.text(
                (x, y),
                str(cpu_num),
                font=self.font,
                fill=self.color_blue,
            )
            self.draw.text(
                (x + self.font_width, y),
                f"[{' ' * (bar_width - 1)}]",
                font=self.font,
                fill=self.color_white,
            )

        def draw_bar(x, y, times_percent, usage):
            # Get bar width
            user_bar_width = int(math.floor(times_percent.user * bar_width / 100))
            system_bar_width = int(math.floor(times_percent.system * bar_width / 100))

            # Fill bar content
            bar_content = [" "] * (bar_width - 1)

            usage_text = f"{usage:>5.1f}%"
            bar_content[-len(usage_text) :] = usage_text

            for i in range(user_bar_width):
                if i > len(bar_content) - 1:
                    break
                if bar_content[i] == " ":
                    bar_content[i] = "|"

            for i in range(system_bar_width):
                if i > len(bar_content) - 1:
                    break
                if bar_content[user_bar_width + i] == " ":
                    bar_content[user_bar_width + i] = "|"

            # Divide bar content
            user_part_content = "".join(bar_content[:user_bar_width])
            system_part_content = "".join(
                bar_content[user_bar_width : user_bar_width + system_bar_width]
            )
            usage_part_content = "".join(
                bar_content[user_bar_width + system_bar_width :]
            )

            # Get part start x
            user_part_start_x = x + self.font_width * 2
            system_part_start_x = user_part_start_x + self.font_width * user_bar_width
            usage_part_start_x = (
                user_part_start_x
                + self.font_width * user_bar_width
                + self.font_width * system_bar_width
            )

            self.draw.text(
                (user_part_start_x, y),
                user_part_content,
                font=self.font,
                fill=self.color_green,
            )
            self.draw.text(
                (system_part_start_x, y),
                system_part_content,
                font=self.font,
                fill=self.color_red,
            )
            self.draw.text(
                (usage_part_start_x, y),
                usage_part_content,
                font=self.font,
                fill=self.bright_black,
            )

        for i, times in enumerate(cpu_times_percent):
            draw_panel(x_base, y_base + i * 15, i)
            draw_bar(x_base, y_base + i * 15, times, cpu_percents[i])

    def draw_process_infos(self, x_base, y_base):
        processes = []
        for proc in psutil.process_iter(
            ["pid", "username", "cpu_percent", "memory_percent", "name"]
        ):
            try:
                info = proc.info
                info["cpu_percent"] = info.get("cpu_percent") or 0.0
                info["memory_percent"] = info.get("memory_percent") or 0.0
                info["username"] = info.get("username") or "unknown"
                info["name"] = info.get("name") or ""
                processes.append(info)
            except (psutil.NoSuchProcess, psutil.AccessDenied, KeyError):
                continue
        processes = sorted(processes, key=lambda p: p["cpu_percent"], reverse=True)[:6]

        # Title panel
        self.draw.rectangle(
            [x_base - 2, y_base, self.image.width - x_base - 2, y_base + 15],
            fill=self.theme["terminal_colors"]["normal"]["green"],
        )
        # Title
        self.draw.text(
            (x_base, y_base),
            "  PID USER     CPU% MEM%  Command",
            font=self.font,
            fill=self.color_black,
        )
        # Process list
        for i, proc in enumerate(processes):
            y = y_base + 2 + 15 * (i + 1)
            cmd = f"{proc['name'][:9]}.." if len(proc["name"]) > 9 else proc["name"]
            username = (
                f"{proc['username'][:6]}.."
                if len(proc["username"]) > 6
                else proc["username"]
            )
            text = f"{proc['pid']:>5} {username:<8} {proc['cpu_percent']:>4.1f} {proc['memory_percent']:>4.1f}  {cmd}"
            self.draw.text((x_base, y), text, font=self.font, fill=self.color_white)
