from .lcd import PyGamePanel
import utils.logger
import api


logger = utils.logger.create(tag="desktop")


def main():
    logger.info("start cover screen service")

    logger.info("create panel")
    panel = PyGamePanel(width=280, height=240, scale=1)

    logger.info("create frame buffer")
    frame_buffer = api.FrameBuffer(name="fb0", panel=panel)

    while True:
        frame_buffer.update()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
    except Exception as e:
        logger.error(e)

    logger.info("cleanup")
    api.frame_buffer.cleanup()
