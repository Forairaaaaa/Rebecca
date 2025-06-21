from luma.core.interface.serial import spi
from luma.core.render import canvas
from luma.lcd.device import st7789
from RPI5.GPIO import GPIO
import time
import random


class st7789_with_offset(st7789):
    def __init__(self, serial_interface=None, width=240, height=240, rotate=0, offset=(0, 0), **kwargs):
        self._offset = offset
        super().__init__(serial_interface=serial_interface,
                         width=width, height=height, rotate=rotate, **kwargs)

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
        draw.text((23, 167), "fuck that shit", fill="pink", font_size=36)
    time.sleep(1)

    def random_color(): return random.choice(
        ["red", "green", "blue", "yellow", "purple", "orange", "pink", "brown", "gray", "black", "white"])

    def random_point(): return (random.randint(-123, 280), random.randint(-123, 240))

    while True:
        for i in range(2):
            for bl in range(100, 0, -2):
                device.backlight(bl)
                print(f"bl: {bl}")
                time.sleep(0.01)
            for bl in range(0, 100, 2):
                device.backlight(bl)
                print(f"bl: {bl}")
                time.sleep(0.01)

        print("keep render shit")
        device.backlight(100)
        for i in range(123):
            with canvas(device) as draw:
                for i in range(3):

                    draw.circle(random_point(), random.randint(
                        1, 66), fill=random_color())

                    x0, y0 = random_point()
                    x1, y1 = random.randint(x0, 321), random.randint(y0, 321)
                    draw.rectangle((x0, y0, x1, y1),
                                   outline=random_color(), fill=random_color())

                    # line
                    draw.line((random_point(), random_point()),
                              fill=random_color(), width=random.randint(1, 12))

                    for i in range(6):
                        draw.text(random_point(), "fuck",
                                  fill=random_color(), font_size=18)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
