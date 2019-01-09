import argparse
from io import StringIO
import json
from pathlib import Path
import sys

import PIL.Image


CHAR_SIZE = 8


class Char:
    """An 8x8 tile found in a charmap."""
    def __init__(self, image, x0, y0):
        self.image = image

        if x0 % CHAR_SIZE != 0:
            raise ValueError(f"x0 must be a multiple of {CHAR_SIZE}; got {x0}")
        if y0 % CHAR_SIZE != 0:
            raise ValueError(f"y0 must be a multiple of {CHAR_SIZE}; got {y0}")

        self.x0 = x0
        self.y0 = y0

    def append_to(self, out):
        pixels = self.image.load()
        for y in range(CHAR_SIZE):
            for x in range(CHAR_SIZE):
                px = pixels[self.x0 + x, self.y0 + y]
                out.append(px)


def convert_spritesheet(image_path, sprite_width, sprite_height):
    """Convert a spritesheet to an arbitrary number of 8×8 tiles, grouped into
    blocks of size width by height.
    """

    name = image_path.stem

    im = PIL.Image.open(image_path)
    width, height = im.size

    out = bytearray()
    for sy in range(0, height, sprite_height):
        for sx in range(0, width, sprite_width):
            for ty in range(sy, sy + sprite_height, CHAR_SIZE):
                for tx in range(sx, sx + sprite_width, CHAR_SIZE):
                    Char(im, tx, ty).append_to(out)

    return out


def main(*argv):
    parser = argparse.ArgumentParser(description='Convert a PNG spritesheet into a set of 8x8 tiles.')
    parser.add_argument('-o', '--outfile', type=Path, default=None)
    parser.add_argument('infile')
    # TODO enforce that these are multiples of 8
    parser.add_argument('width', type=int, default=8)
    parser.add_argument('height', type=int, default=8)
    args = parser.parse_args(argv)

    out = convert_spritesheet(Path(args.infile), args.width, args.height)

    with args.outfile.open('wb') as f:
        f.write(out)


if __name__ == '__main__':
    main(*sys.argv[1:])
