"""Draws the Scribe's plot armour bubble (`shield_0`, `shield_1`): a see-through blue ball with a bright rim and a glint
that moves a little between the two drawings. 64 x 76, centred on the body.

    python tools/gen_shield.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

SPRITES = Path(__file__).resolve().parent.parent / "assets" / "sprites" / "scribe"
W, H = 64, 76


def bubble(glint):
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # the body of the bubble, a light see-through blue, darker towards the edge
    d.ellipse([0, 0, W - 1, H - 1], fill=(70, 130, 255, 60))
    d.ellipse([3, 3, W - 4, H - 4], fill=(110, 170, 255, 46))
    d.ellipse([0, 0, W - 1, H - 1], outline=(150, 200, 255, 210))
    d.ellipse([1, 1, W - 2, H - 2], outline=(90, 150, 255, 120))
    # the glint on the upper left, sliding along the rim between the two drawings
    shift = 3 * glint
    for i in range(7):
        d.point((9 + i + shift, 22 - i - shift // 2), fill=(240, 250, 255, 235))
        d.point((9 + i + shift, 23 - i - shift // 2), fill=(200, 230, 255, 180))
    d.point((14 + shift, 10 + shift), fill=(255, 255, 255, 255))
    return img


SPRITES.mkdir(parents=True, exist_ok=True)
bubble(0).save(SPRITES / "shield_0.png")
bubble(1).save(SPRITES / "shield_1.png")
print("wrote shield_0 and shield_1")
