import logging
import sys


def create(tag=""):
    logger = logging.getLogger()
    logger.setLevel(logging.INFO)

    ch = logging.StreamHandler(sys.stdout)
    if tag:
        ch.setFormatter(
            logging.Formatter(f"[%(asctime)s] [%(levelname)s] [{tag}] %(message)s")
        )
    else:
        ch.setFormatter(logging.Formatter("[%(asctime)s] [%(levelname)s] %(message)s"))
    logger.addHandler(ch)

    return logger
