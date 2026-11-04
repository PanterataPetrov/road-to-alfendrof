"""Draws one of the bats in Gray Persona's spray (`batspray_0`, `batspray_1`: wings up, wings down), 15 x 9, in the colours
of the big flock (`bats_*`).

    python tools/gen_batspray.py
"""

from pathlib import Path

from PIL import Image

SPRITES = Path(__file__).resolve().parent.parent / "assets" / "sprites" / "gray_persona"
BODY = (60, 8, 30, 255)
WING = (26, 18, 34, 255)
EDGE = (140, 50, 150, 255)
EYE = (255, 60, 60, 255)

UP = [
    "E.............E",
    "EW...BBBB...WE.",
    "EWW.BBBBBB.WWE.",
    ".EWWBBRBRBWWE..",
    "..EWBBBBBBWE...",
    "...EWBBBBWE....",
    ".....BBBB......",
    "......BB.......",
    "...............",
]
DOWN = [
    "...............",
    "......BBBB.....",
    "..E..BBBBBB..E.",
    ".EWWBBRBRBWWWE.",
    "EWWWBBBBBBWWWWE",
    "EWW..BBBBB..WWE",
    "E....BBBB....E.",
    "......BB.......",
    "...............",
]
COLORS = {"B": BODY, "W": WING, "E": EDGE, "R": EYE}


def draw(rows):
    img = Image.new("RGBA", (15, 9), (0, 0, 0, 0))
    px = img.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row[:15]):
            if ch in COLORS:
                px[x, y] = COLORS[ch]
    return img


SPRITES.mkdir(parents=True, exist_ok=True)
draw(UP).save(SPRITES / "batspray_0.png")
draw(DOWN).save(SPRITES / "batspray_1.png")
print("wrote batspray_0 and batspray_1")
