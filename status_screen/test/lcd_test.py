from luma.core.interface.serial import spi
from luma.core.render import canvas
from luma.lcd.device import st7789
from RPI5.GPIO import GPIO
import time
import random


class st7789_with_offset(st7789):
    def __init__(self, serial_interface=None, width=240, height=240, rotate=0, offset=(0, 0), **kwargs):
        self._offset = offset
        super().__init__(serial_interface=serial_interface, width=width,
                         height=height, rotate=rotate, **kwargs)

    def display(self, image):
        self.set_window(self._offset[0], self._offset[1],
                        self._offset[0] + self._w, self._offset[1] + self._h)

        image = self.preprocess(image)
        self.data(list(image.convert("RGB").tobytes()))


def main():
    serial = spi(gpio=GPIO, port=0, device=0, gpio_DC=27,
                 gpio_RST=17, bus_speed_hz=52 * 1000000)
    device = st7789_with_offset(serial, width=280, height=240,
                                gpio_LIGHT=25, pwm_frequency=1000, gpio=GPIO, offset=(20, 0))

    with canvas(device) as draw:
        draw.rectangle(device.bounding_box, outline="white", fill="black")
        draw.text((0, 40), "Hello World", fill="red", font_size=48)

    # while True:
    #     time.sleep(1)

    # bl_list = [0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
    # while True:
    #     for bl in bl_list:
    #         device.backlight(bl)
    #         print(f"Backlight: {bl}")
    #         time.sleep(0.5)

    device.backlight(100)
    while True:
        with canvas(device) as draw:
            for i in range(6):
                x0 = random.randint(0, 280)
                y0 = random.randint(0, 240)
                x1 = random.randint(x0, 280)
                y1 = random.randint(y0, 240)
                draw.rectangle((x0, y0, x1, y1), outline="white", fill=random.choice(
                    ["red", "green", "blue", "yellow", "purple", "orange", "pink", "brown", "gray", "black", "white"]))
        # time.sleep(0.2)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
