"""Draws the pencil the Scribe throws when he dashes away (`pencil_0`, `pencil_1`: flying right, turning a little).

    python tools/gen_pencil.py
"""

from pathlib import Path

from PIL import Image

SPRITES = Path(__file__).resolve().parent.parent / "assets" / "sprites" / "scribe"
OUTLINE = (40, 30, 20, 255)
ERASER = (236, 130, 140, 255)
FERRULE = (190, 190, 200, 255)
BODY = (250, 200, 40, 255)
BODY_DARK = (214, 160, 24, 255)
WOOD = (230, 190, 140, 255)
LEAD = (60, 60, 70, 255)

ROWS = [
    "OOOOOOOOOOOOOOO.",
    "OEEFFBBBBBBBWWLO",
    "OEEFFDDDDDDDWWLO",
    "OOOOOOOOOOOOOOO.",
]
COLORS = {"O": OUTLINE, "E": ERASER, "F": FERRULE, "B": BODY, "D": BODY_DARK, "W": WOOD, "L": LEAD}


def draw(tilt):
    img = Image.new("RGBA", (16, 6), (0, 0, 0, 0))
    px = img.load()
    for y, row in enumerate(ROWS):
        for x, ch in enumerate(row):
            if ch in COLORS:
                # turning: the tip end sits a row lower in the second drawing
                dy = 1 if (tilt and x >= 11) else 0
                px[x, y + 1 + dy - (1 if tilt else 0)] = COLORS[ch]
    return img


SPRITES.mkdir(parents=True, exist_ok=True)
draw(0).save(SPRITES / "pencil_0.png")
draw(1).save(SPRITES / "pencil_1.png")
print("wrote pencil_0 and pencil_1")
