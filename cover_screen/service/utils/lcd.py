from luma.lcd.device import backlit_device
from luma.core.interface.serial import spi
import luma.lcd.const
from .rpi5_gpio import GPIO
from PIL import Image
import numpy as np


class st7789v2(backlit_device):
    def __init__(
        self,
        serial_interface=None,
        width=240,
        height=240,
        rotate=0,
        offset=(0, 0),
        **kwargs,
    ):
        self._offset = offset

        super(st7789v2, self).__init__(
            luma.lcd.const.st7789, serial_interface, **kwargs
        )
        self.capabilities(width, height, rotate, mode="RGB")

        # COLMOD (3Ah): Interface Pixel Format: 18bit/pixel
        self.command(0x3A, 0x06)
        # PORCTRL (B2h): Porch Setting: Disable separate porch control, 0xC in normal mode, 0x3 in idle and partial modes
        self.command(0xB2, 0x0C, 0x0C, 0x00, 0x33, 0x33)
        # GCTRL (B7h): Gate Control: VGH = 13.26V, VGL = -10.43V
        self.command(0xB7, 0x35)
        # VCOMS (BBh): VCOM Setting: 0.725V
        self.command(0xBB, 0x19)
        # LCMCTRL (C0h): LCM Control: XBGR, XMX, XMH
        self.command(0xC0, 0x2C)
        # VDVVRHEN (C2h): VDV and VRH Command Enable: VDV and VRH register value comes from command write
        self.command(0xC2, 0x01)
        # VRHS (C3h): VRH Set: 4.45V + (vcom + vcom offset + vdv)
        self.command(0xC3, 0x12)
        # VDVS (C4h): VDV Set: 0V
        self.command(0xC4, 0x20)
        # FRCTRL2 (C6h): Frame Rate Control in Normal Mode: 60Hz
        self.command(0xC6, 0x0F)
        # PWCTRL1 (D0h): Power Control 1: AVDD = 6.8V, AVCL = -4.8V, VDDS = 2.3V
        self.command(0xD0, 0xA4, 0xA1)
        # PVGAMCTRL (E0h): Positive Voltage Gamma Control
        self.command(
            0xE0,
            0xD0,
            0x04,
            0x0D,
            0x11,
            0x13,
            0x2B,
            0x3F,
            0x54,
            0x4C,
            0x18,
            0x0D,
            0x0B,
            0x1F,
            0x23,
        )
        # NVGAMCTRL (E1h): Negative Voltage Gamma Control
        self.command(
            0xE1,
            0xD0,
            0x04,
            0x0C,
            0x11,
            0x13,
            0x2C,
            0x3F,
            0x44,
            0x51,
            0x2F,
            0x1F,
            0x1F,
            0x20,
            0x23,
        )
        # INVON (21h): Display Inversion On
        self.command(0x21)
        # SLPOUT (11h): Sleep Out
        self.command(0x11)
        # DISPON (29h): Display On
        self.command(0x29)

        self.clear()
        self.show()

    def command(self, cmd, *args):
        """Send a command to the display, with optional arguments.
        The arguments are sent as data bytes, in accordance with the ST7789 datasheet."""
        super(st7789v2, self).command(cmd)
        if args:
            self.data(args)

    def set_window(self, x1, y1, x2, y2):
        # CASET (2Ah): Column Address Set
        self.command(
            0x2A,
            x1 >> 8,
            x1 & 0xFF,
            (x2 - 1) >> 8,
            (x2 - 1) & 0xFF,
        )
        # RASET (2Bh): Row Address Set
        self.command(
            0x2B,
            y1 >> 8,
            y1 & 0xFF,
            (y2 - 1) >> 8,
            (y2 - 1) & 0xFF,
        )
        # RAMWR (2Ch): Memory Write
        self.command(0x2C)

    def display(self, image):
        self.set_window(
            self._offset[0],
            self._offset[1],
            self._offset[0] + self._w,
            self._offset[1] + self._h,
        )

        image = self.preprocess(image)
        self.data(list(image.convert("RGB").tobytes()))

    def contrast(self, level):
        """
        NOT SUPPORTED

        :param level: Desired contrast level in the range of 0-255.
        :type level: int
        """
        assert 0 <= level <= 255


class Zjy169:
    """
    中景园 1.69" ST7789V2 240x280 SPI LCD.
    """

    def __init__(
        self,
        spi_port=0,
        spi_device=0,
        gpio_DC=27,
        gpio_RST=17,
        gpio_LIGHT=25,
        bus_speed_hz=52 * 1000000,
        pwm_frequency=1000,
        rotate=0,
    ):
        self.serial = spi(
            gpio=GPIO,
            port=spi_port,
            device=spi_device,
            gpio_DC=gpio_DC,
            gpio_RST=gpio_RST,
            bus_speed_hz=bus_speed_hz,
        )

        self.device = st7789v2(
            self.serial,
            width=240,
            height=280,
            gpio_LIGHT=gpio_LIGHT,
            pwm_frequency=pwm_frequency,
            gpio=GPIO,
            offset=(0, 20),
            rotate=rotate,
        )

    def push(self, img: Image.Image):
        self.device.display(img)

    def pushRaw(self, raw: bytes):
        self.push(Image.frombytes("RGBA", self.device.size, raw))
