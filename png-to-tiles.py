import argparse
from io import StringIO
import json
from pathlib import Path
import sys

import PIL.Image


CHAR_SIZE = 8
TILE_SIZE = 16


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


def convert_spritesheet(image_path):
    """Convert a spritesheet to a full set of palettes (64 bytes) followed by
    an arbitrary number of tiles (paired such that each 8×16 object is two
    adjacent tiles).
    """

    name = image_path.stem

    im = PIL.Image.open(image_path)
    width, height = im.size

    # -------------------------------------------------------------------------

    # Get the palette.  There's a getpalette() method, but the palette it
    # returns has been padded to 256 colors for some reason, which makes it
    # useless for our purposes.  Instead, check out the poorly-documented
    # palette attribute, which has the palette as a flat list of channels.
    # God, I hate PIL.
    if im.palette is None:
        print("This tool only works on paletted PNGs!", file=sys.stderr)
        sys.exit(1)

    pal = im.palette.palette
    # TODO?  or divisible by four?  if len(pal) != 32:
    gbc_palettes = []
    asm_colors = []
    out = bytearray()
    for i in range(0, len(pal), 12):
        gbc_palette = []
        for j in range(i, i + 12, 3):
            r, g, b = color = pal[j:j + 3]
            gbc_palette.append(color)

            # Convert to RGB555
            asm_color = (r >> 3) | ((g >> 3) << 5) | ((b >> 3) << 10)
            asm_colors.append(asm_color)
            out.extend(asm_color.to_bytes(2, 'little'))

    # Pad out to a full set of 8 palettes == 32 colors
    for _ in range(len(asm_colors), 64):
        out.extend(b'\x00\x00')

    # -------------------------------------------------------------------------

    # TODO if two colors in different palettes are identical, Do The Right
    # Thing (vastly more complicated but worth it i think)
    small_tile_index = 0
    # Flat list of the four little tiles that make up each big tile
    big_tile_subtiles = {}
    for ty in range(0, height, CHAR_SIZE):
        for tx in range(0, width, CHAR_SIZE):
            #seen_pixels = {}
            big_tile_index = (ty // TILE_SIZE) * (width // TILE_SIZE) + (tx // TILE_SIZE)
            big_tile_subtiles.setdefault(big_tile_index, []).append(small_tile_index)
            Char(im, tx, ty).append_to(out)
            small_tile_index += 1

    return out


def main(*argv):
    actions = dict(
        spritesheet=convert_spritesheet,
    )
    parser = argparse.ArgumentParser(description='Convert a PNG into a set of Game Boy tiles.')
    parser.add_argument('-o', '--outfile', type=Path, default=None)
    parser.add_argument('mode', choices=actions)
    parser.add_argument('infile')
    args = parser.parse_args(argv)

    # Handle the output file here, so the functions above can just print()
    out = actions[args.mode](Path(args.infile))

    with args.outfile.open('wb') as f:
        f.write(out)


if __name__ == '__main__':
    main(*sys.argv[1:])
