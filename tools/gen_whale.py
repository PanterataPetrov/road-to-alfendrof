"""Draws the Scribe's sperm whale (`whale_0..5`) and the pool it comes up out of (`pool_0`, `pool_1`).

    python tools/gen_whale.py [--sheet preview.png]

The whale faces right (the game turns it) and every drawing stands on the same line, the water, in an 84 x 64 picture, so
rising out of the pool and sinking back is drawn, not moved. Its head is blunt and square, with a narrow lower jaw full of
teeth: it rises (0, 1), opens wide (2, 3), snaps shut (4) and sinks (5).
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

SPRITES = Path(__file__).resolve().parent.parent / "assets" / "sprites" / "scribe"
W, H = 84, 64
BASE = 60  # the water line

OUTLINE = (24, 30, 44, 255)
SKIN = (84, 102, 128, 255)
SKIN_LIGHT = (118, 138, 166, 255)
SKIN_DARK = (58, 72, 96, 255)
JAW = (196, 202, 214, 255)
JAW_SHADE = (150, 158, 176, 255)
TEETH = (250, 248, 238, 255)
MOUTH = (128, 52, 72, 255)
EYE = (12, 12, 18, 255)
EYE_RING = (226, 232, 240, 255)
WATER = (64, 146, 214, 255)
WATER_LIGHT = (140, 206, 248, 255)
WATER_DARK = (36, 96, 168, 255)
FOAM = (236, 248, 255, 255)


def outline(img):
    px = img.load()
    out = img.copy()
    o = out.load()
    for y in range(H):
        for x in range(W):
            if px[x, y][3] == 0:
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, ny = x + dx, y + dy
                    if 0 <= nx < W and 0 <= ny < H and px[nx, ny][3] > 0 and px[nx, ny] not in (WATER, WATER_LIGHT, WATER_DARK, FOAM):
                        o[x, y] = OUTLINE
                        break
    return out


def whale(rise, gap, lunge=0):
    """`rise`: how tall the head stands out of the water; `gap`: how far the mouth is open; `lunge`: leaning forward."""
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    jaw_h = 8
    top = BASE - rise
    # the lower jaw: long and narrow, along the water, hinged at the back
    jx0, jx1 = 12, 70 + lunge
    d.rectangle([jx0, BASE - jaw_h, jx1, BASE], fill=JAW)
    d.rectangle([jx0, BASE - 2, jx1, BASE], fill=JAW_SHADE)
    # teeth along the jaw, pointing up
    for tx in range(jx0 + 6, jx1 - 2, 4):
        d.rectangle([tx, BASE - jaw_h - 3, tx + 1, BASE - jaw_h - 1], fill=TEETH)
    # the great square head above it, the front blunt and tall
    hx0, hx1 = 4, 72 + lunge
    head_bottom = BASE - jaw_h - gap
    if head_bottom - top >= 4:
        d.rectangle([hx0, top + 3, hx1, head_bottom], fill=SKIN)
        d.rectangle([hx0 + 3, top, hx1 - 3, top + 3], fill=SKIN)
        # the blunt front, a lighter forehead, wrinkles along the underside and a dark back
        d.rectangle([hx1 - 8, top + 3, hx1, head_bottom], fill=SKIN_LIGHT)
        d.rectangle([hx0, top + 3, hx0 + 5, head_bottom], fill=SKIN_DARK)
        for wy in range(top + 8, head_bottom - 1, 5):
            d.line([hx0 + 8, wy, hx1 - 12, wy], fill=SKIN_DARK)
        # the eye, set back and low
        ey = min(head_bottom - 6, top + 14)
        d.rectangle([hx0 + 12, ey, hx0 + 16, ey + 4], fill=EYE_RING)
        d.rectangle([hx0 + 13, ey + 1, hx0 + 15, ey + 3], fill=EYE)
        # the inside of the mouth when it is open, and its teeth hanging from the top jaw
        if gap > 2:
            d.rectangle([hx0 + 6, head_bottom + 1, hx1 - 2, head_bottom + gap], fill=MOUTH)
            for tx in range(hx0 + 10, hx1 - 4, 5):
                d.rectangle([tx, head_bottom, tx + 1, head_bottom + 2], fill=TEETH)
    # the water around its foot
    d.rectangle([0, BASE + 1, W - 1, BASE + 3], fill=WATER)
    d.rectangle([2, BASE + 1, W - 3, BASE + 1], fill=WATER_LIGHT)
    d.rectangle([4, BASE + 3, W - 5, BASE + 3], fill=WATER_DARK)
    img = outline(img)
    return img


def spray(img, rise, n):
    px = img.load()
    for i in range(n):
        x = 3 + (i * 11) % (W - 6)
        y = BASE - 2 - (i * 7) % max(6, rise // 2 + 4)
        if 0 <= y < H:
            px[x, y] = FOAM
    return img


def pool(frame):
    img = Image.new("RGBA", (W, 12), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([1, 3, W - 2, 10], fill=WATER_DARK)
    d.ellipse([3, 2, W - 4, 8], fill=WATER)
    # two rings that ripple outward, in step with the frame
    r = 4 + frame * 3
    d.ellipse([W // 2 - 10 - r, 3, W // 2 + 10 + r, 8], outline=WATER_LIGHT)
    d.line([8, 4, 14, 4], fill=FOAM)
    d.line([W - 16, 6, W - 10, 6], fill=FOAM)
    return img


def main():
    frames = [
        spray(whale(12, 0), 12, 5),
        spray(whale(28, 0), 28, 6),
        spray(whale(42, 8), 42, 6),
        spray(whale(52, 18), 52, 4),
        spray(whale(50, 3, lunge=4), 50, 4),
        spray(whale(24, 0), 24, 6),
    ]
    SPRITES.mkdir(parents=True, exist_ok=True)
    for i, f in enumerate(frames):
        f.save(SPRITES / f"whale_{i}.png")
    pools = [pool(0), pool(1)]
    for i, p in enumerate(pools):
        p.save(SPRITES / f"pool_{i}.png")
    if "--sheet" in sys.argv:
        out = Path(sys.argv[sys.argv.index("--sheet") + 1])
        scale = 4
        sheet = Image.new("RGBA", ((W * len(frames) + 8 * 2) * scale, (H + 14) * scale), (230, 238, 250, 255))
        for i, f in enumerate(frames):
            sheet.alpha_composite(f.resize((W * scale, H * scale), 0), (i * W * scale, 0))
        for i, p in enumerate(pools):
            sheet.alpha_composite(p.resize((W * scale, 12 * scale), 0), (i * W * scale, H * scale))
        sheet.save(out)
    print("wrote", len(frames), "whales and", len(pools), "pools")


main()
