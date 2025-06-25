from PIL import Image, ImageDraw, ImageFont
from app.app_base import AppBase
from utils.logger import logger
import asyncio
import psutil
import os


class HtopLike(AppBase):
    def __init__(self, screen_name: str):
        super().__init__(screen_name)

    async def main(self):
        while True:
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
                    processes.append(proc.info)
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
            processes = sorted(processes, key=lambda p: p["cpu_percent"], reverse=True)[
                :5
            ]

            # 创建图像
            width, height = 280, 240
            image = Image.new("RGBA", (width, height), (0, 0, 0, 255))
            draw = ImageDraw.Draw(image)

            try:
                font = ImageFont.truetype("DejaVuSansMono.ttf", 12)
            except Exception as e:
                logger.error(f"load font failed: {e}")
                font = ImageFont.load_default()

            GREEN = (0, 255, 0, 255)
            BLUE = (100, 200, 255, 255)
            WHITE = (255, 255, 255, 255)

            # 顶部系统信息
            mem_str = f"MEM: {mem.used // (1024*1024)}M/{mem.total // (1024*1024)}M"
            draw.text(
                (5, 5),
                f"CPU: {int(sum(cpu_percents)/len(cpu_percents))}%   {mem_str}",
                font=font,
                fill=GREEN,
            )
            draw.text(
                (5, 20),
                f"Load avg: {load_avg[0]:.2f}  {load_avg[1]:.2f}  {load_avg[2]:.2f}",
                font=font,
                fill=BLUE,
            )

            # CPU 条
            def draw_bar(x, y, usage, label):
                bar_w = 180
                bar_h = 10
                filled = int(bar_w * usage / 100)
                draw.text((x - 30, y - 2), label, font=font, fill=WHITE)
                draw.rectangle([x, y, x + bar_w, y + bar_h], outline=WHITE, fill=None)
                draw.rectangle([x, y, x + filled, y + bar_h], fill=GREEN)

            for i, usage in enumerate(cpu_percents):
                draw_bar(50, 40 + i * 15, usage, f"CPU{i}")

            # 进程列表
            y_base = 110
            draw.text(
                (5, y_base), "PID   USER     %CPU  %MEM  COMMAND", font=font, fill=BLUE
            )
            for i, proc in enumerate(processes):
                y = y_base + 15 * (i + 1)
                cmd = proc["name"][:15]  # 防止太长
                text = f"{proc['pid']:<5} {proc['username'][:8]:<8} {proc['cpu_percent']:>4.1f}  {proc['memory_percent']:>4.1f}  {cmd}"
                draw.text((5, y), text, font=font, fill=WHITE)

            # 显示
            await self.render(image)

            await asyncio.sleep(1)
