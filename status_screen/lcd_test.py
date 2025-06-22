from luma.core.render import canvas
import utils.lcd as lcd
import random
import time


def main():
    panel = lcd.Zjy169(gpio_DC=27, gpio_RST=17, gpio_LIGHT=25, rotate=3)

    # colors = [0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFFFF, 0x00000000]
    # while True:
    #     for color in colors:
    #         panel.frame_buffer.buffer.fill(color)
    #         panel.frame_buffer.push()
    #         time.sleep(1)

    with canvas(panel.device) as draw:
        draw.rectangle(panel.device.bounding_box, outline="white", fill="black")
        draw.text((23, 167), "fuck that shit", fill="pink", font_size=36)
    time.sleep(1)

    def random_color():
        return random.choice(
            [
                "red",
                "green",
                "blue",
                "yellow",
                "purple",
                "orange",
                "pink",
                "brown",
                "gray",
                "black",
                "white",
            ]
        )

    # while True:
    #     with canvas(device) as draw:
    #         draw.rectangle(device.bounding_box, random_color())
    #     time.sleep(1)

    def random_point():
        return (random.randint(-123, 280), random.randint(-123, 240))

    while True:
        for i in range(2):
            for bl in range(100, 0, -2):
                panel.device.backlight(bl)
                print(f"bl: {bl}")
                time.sleep(0.01)
            for bl in range(0, 100, 2):
                panel.device.backlight(bl)
                print(f"bl: {bl}")
                time.sleep(0.01)

        print("keep render shit")
        panel.device.backlight(100)
        for i in range(123):
            with canvas(panel.device) as draw:
                for i in range(3):
                    draw.circle(
                        random_point(), random.randint(1, 66), fill=random_color()
                    )

                    x0, y0 = random_point()
                    x1, y1 = random.randint(x0, 321), random.randint(y0, 321)
                    draw.rectangle(
                        (x0, y0, x1, y1), outline=random_color(), fill=random_color()
                    )

                    draw.line(
                        (random_point(), random_point()),
                        fill=random_color(),
                        width=random.randint(1, 12),
                    )

                    for i in range(2):
                        draw.text(
                            random_point(),
                            "fuck",
                            fill=random_color(),
                            font_size=random.randint(14, 144),
                        )


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
