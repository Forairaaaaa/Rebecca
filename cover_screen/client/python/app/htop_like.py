from PIL import Image, ImageDraw
from app.app_base import AppBase
from theme.font import get_font
from theme.theme import get_theme
import asyncio
import psutil
import os


class HtopLike(AppBase):
    def __init__(self, screen_name: str, width=280, height=240):
        super().__init__(screen_name)

        # Load resources
        self.font = get_font()
        self.theme = get_theme()
        self.color_green = self.theme["terminal_colors"]["normal"]["green"]
        self.color_blue = self.theme["terminal_colors"]["normal"]["blue"]
        self.color_white = self.theme["terminal_colors"]["normal"]["white"]
        self.color_black = self.theme["terminal_colors"]["normal"]["black"]

        # Init image
        self.width = width
        self.height = height
        self.image = Image.new("RGBA", (self.width, self.height), (0, 0, 0, 255))
        self.draw = ImageDraw.Draw(self.image)

    async def main(self):
        while True:
            self.draw.rectangle(
                [0, 0, self.width, self.height],
                fill=self.theme["background"],
            )

            # 获取系统信息
            cpu_percents = psutil.cpu_percent(percpu=True)[:4]  # 前4个CPU核
            mem = psutil.virtual_memory()
            load_avg = os.getloadavg()  # (1, 5, 15分钟)

            # 获取进程列表，按CPU使用率排序
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

            processes = sorted(processes, key=lambda p: p["cpu_percent"], reverse=True)[
                :5
            ]

            # 顶部系统信息
            mem_str = f"MEM: {mem.used // (1024 * 1024)}M/{mem.total // (1024 * 1024)}M"
            self.draw.text(
                (5, 5),
                f"CPU: {int(sum(cpu_percents) / len(cpu_percents))}%   {mem_str}",
                font=self.font,
                fill=self.color_green,
            )
            self.draw.text(
                (5, 20),
                f"Load avg: {load_avg[0]:.2f}  {load_avg[1]:.2f}  {load_avg[2]:.2f}",
                font=self.font,
                fill=self.color_blue,
            )

            # CPU 条
            def draw_bar(x, y, usage, label):
                bar_w = 180
                bar_h = 10
                filled = int(bar_w * usage / 100)
                self.draw.text(
                    (x - 30, y - 2), label, font=self.font, fill=self.color_white
                )
                self.draw.rectangle(
                    [x, y, x + bar_w, y + bar_h], outline=self.color_white, fill=None
                )
                self.draw.rectangle(
                    [x, y, x + filled, y + bar_h], fill=self.color_green
                )

            for i, usage in enumerate(cpu_percents):
                draw_bar(50, 40 + i * 15, usage, f"CPU{i}")

            self.draw_process_infos(processes)

            await self.render(self.image)

            await asyncio.sleep(1)

    def draw_process_infos(self, processes):
        y_base = 140
        # Title panel
        self.draw.rectangle(
            [7, y_base, self.image.width - 7, y_base + 15],
            fill=self.theme["terminal_colors"]["normal"]["green"],
        )
        # Title
        self.draw.text(
            (7, y_base),
            "  PID USER     CPU%  MEM%  Command",
            font=self.font,
            fill=self.color_black,
        )
        # Process list
        for i, proc in enumerate(processes):
            y = y_base + 15 * (i + 1)
            cmd = proc["name"][:9] + ".."
            text = f"{proc['pid']:>5} {proc['username'][:8]:<8} {proc['cpu_percent']:>4.1f}  {proc['memory_percent']:>4.1f}  {cmd}"
            self.draw.text((7, y), text, font=self.font, fill=self.color_white)
