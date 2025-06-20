import time
from luma.emulator.device import pygame
from luma.core.render import canvas
import psutil


def htop_like(device):
    while True:
        with canvas(device) as draw:
            cpu_percent = psutil.cpu_percent(percpu=True)[:4]
            memory_percent = psutil.virtual_memory().percent

            draw.text((0, 0), f'cpu shit: {cpu_percent}', fill="white")
            draw.text((0, 10), f'mem shit: {memory_percent}', fill="white")
            print(cpu_percent)
            print(memory_percent)
        time.sleep(1)


def main():
    device = pygame(width=128, height=128, scale=1)
    htop_like(device)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
