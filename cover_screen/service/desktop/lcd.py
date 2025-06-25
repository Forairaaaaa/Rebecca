from luma.emulator.device import pygame
from PIL import Image


class PyGamePanel:
    def __init__(self, width=240, height=280, scale=1, rotate=0):
        self.device = pygame(width=width, height=height, scale=scale, rotate=rotate)

        # Trigger window instance
        self.pushRaw(b"\x00" * width * height * 4)

    def push(self, img: Image.Image):
        self.device.display(img)

    def pushRaw(self, raw: bytes):
        self.push(Image.frombytes("RGBA", self.device.size, raw))
