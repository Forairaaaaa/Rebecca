from utils.logger import logger
import utils.lcd as lcd
import api


def main():
    logger.info("start cover screen service")

    logger.info("create panels")
    panels = []
    panels.append(lcd.Zjy169(gpio_DC=27, gpio_RST=17, gpio_LIGHT=25, rotate=3))

    logger.info("create frame buffers")
    fbs = [api.FrameBuffer(f"fb{i}", panels[i]) for i in range(len(panels))]

    while True:
        for fb in fbs:
            fb.update()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)

    logger.info("cleanup")
    api.frame_buffer.cleanup()
