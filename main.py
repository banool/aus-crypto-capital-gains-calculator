#!/usr/bin/env python

import argparse
import logging


LOG = logging.getLogger(__name__)
formatter = logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
ch = logging.StreamHandler()
ch.setFormatter(formatter)
LOG.addHandler(ch)


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("-d", "--debug", action="store_true")
    args = parser.parse_args()
    return args


def main():
    args = parse_args()

    if args.debug:
        LOG.setLevel("DEBUG")
    else:
        LOG.setLevel("INFO")


if __name__ == "__main__":
    main()
            
