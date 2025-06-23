from luma.emulator.device import pygame
from PIL import Image
import numpy as np


class _FrameBuffer:
    def __init__(self, lcd_device):
        self._device = lcd_device
        self.depth = 4
        self.buffer = np.zeros((self._device._h, self._device._w), dtype=np.uint32)

    def push(self):
        image = Image.fromarray(self.buffer, mode="RGBA")
        self._device.display(image)


class PyGamePanel:
    def __init__(self, width=240, height=280, scale=1, rotate=0):
        self.device = pygame(width=width, height=height, scale=scale, rotate=rotate)

    @property
    def frame_buffer(self):
        if not hasattr(self, "_frame_buffer"):
            self._frame_buffer = _FrameBuffer(self.device)
        return self._frame_buffer
