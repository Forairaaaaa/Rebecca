from .lcd import PyGamePanel
import utils.logger

logger = utils.logger.create(tag="desktop")


def main():
    logger.info("start cover screen service")


if __name__ == "__main__":
    main()
