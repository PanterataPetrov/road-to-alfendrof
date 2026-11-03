"""Draws Anaran's goblins: 26 x 24, facing right, feet on the bottom row, one picture each.

    python tools/gen_goblins.py [--sheet sheet.png]

0, 1: waiting beside him (a bob, the ears twitching, a blink); 2 to 5: a hunched run in four steps with the arms
swinging and the ears streaming back; 6, 7: the bite (the jaws wide with the fangs out, then snapping shut as it
lunges). The game plays them from `animate_minions` in src/render.rs.
"""

import math
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets/sprites/anaran"

W, H = 32, 24
# everything is drawn as if the canvas were 26 wide and then moved right by this, to leave room for a lunge
DX = 3
SKIN = (112, 176, 88)
SKIN_LIGHT = (156, 212, 112)
SKIN_SHADE = (72, 126, 62)
OUTLINE = (24, 46, 30)
TUNIC = (132, 90, 54)
TUNIC_SHADE = (92, 60, 38)
BELT = (58, 38, 28)
BUCKLE = (228, 196, 84)
EYE = (255, 216, 72)
PUPIL = (24, 18, 18)
TOOTH = (244, 240, 222)
MOUTH = (96, 26, 34)
CLAW = (232, 228, 204)
NOSE = (92, 152, 76)


def blank():
    return Image.new("RGBA", (W, H), (0, 0, 0, 0))


def put(img, x, y, c):
    x += DX
    if 0 <= x < W and 0 <= y < H:
        img.putpixel((x, y), tuple(c) + (255,))


def line(img, a, b, c, width=1):
    (x0, y0), (x1, y1) = a, b
    steps = max(abs(x1 - x0), abs(y1 - y0), 1)
    for k in range(steps + 1):
        t = k / steps
        x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
        for ox in range(width):
            for oy in range(width):
                put(img, x + ox, y + oy, c)


def ellipse(img, cx, cy, rx, ry, c):
    for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
        for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
            if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1:
                put(img, x, y, c)


def outline(img):
    """A dark line round everything drawn."""
    src = img.copy()
    for y in range(H):
        for x in range(W):
            if src.getpixel((x, y))[3]:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < W and 0 <= ny < H and src.getpixel((nx, ny))[3]:
                    img.putpixel((x, y), OUTLINE + (255,))
                    break


def goblin(bob=0, lean=0, legs=(0, 0), arms=(0, 0), ears=0, mouth=0, blink=False, lunge=0):
    """One goblin. `bob` moves the upper body down, `lean` pitches the head and chest forward, `legs` is how far each
    foot is forward (back, front), `arms` how high each hand is (back, front), `ears` tilts both ears (negative: up,
    positive: streaming back), `mouth` is 0 shut, 1 open, 2 wide with the fangs out, `lunge` pushes the head forward."""
    img = blank()
    hx = 14 + lean + lunge
    hy = 7 + bob
    # --- legs (drawn first, behind the tunic): thin, bent, bare feet with toes
    for k, (hip_x, off) in enumerate(((10, legs[0]), (15, legs[1]))):
        knee = (hip_x + off // 2 + 1, 19 + bob // 2)
        foot = (hip_x + off, 23)
        line(img, (hip_x, 18 + bob), knee, SKIN_SHADE if k == 0 else SKIN, 2)
        line(img, knee, foot, SKIN_SHADE if k == 0 else SKIN, 2)
        for t in range(3):
            put(img, foot[0] + 1 + t, 23, SKIN_LIGHT if k else SKIN)
        put(img, foot[0] + 3, 22, CLAW)
    # --- the back arm, behind the body
    shoulder = (10 + lean, 13 + bob)
    hand = (6 + lean - arms[0] // 2, 16 + bob - arms[0])
    line(img, shoulder, hand, SKIN_SHADE, 2)
    put(img, hand[0] - 1, hand[1], CLAW)
    # --- the tunic: a ragged brown sack, belted, torn hem
    for y in range(12 + bob, 20 + bob):
        left = 8 + lean // 2 - (1 if y > 16 + bob else 0)
        right = 17 + lean // 2 + (1 if y > 16 + bob else 0)
        for x in range(left, right + 1):
            put(img, x, y, TUNIC_SHADE if x < left + 2 else TUNIC)
    for x in range(8 + lean // 2, 18 + lean // 2):
        put(img, x, 16 + bob, BELT)
    put(img, 13 + lean // 2, 16 + bob, BUCKLE)
    put(img, 14 + lean // 2, 16 + bob, BUCKLE)
    for x in range(7 + lean // 2, 19 + lean // 2, 3):
        put(img, x, 20 + bob, TUNIC)
        put(img, x + 1, 20 + bob, TUNIC_SHADE)
    # a bare chest showing at the neck
    for x in range(12 + lean // 2, 16 + lean // 2):
        put(img, x, 12 + bob, SKIN)
    # --- the head: a big round skull, brow, hooked nose, wide mouth
    ellipse(img, hx, hy, 6, 5, SKIN)
    for x in range(int(hx - 4), int(hx + 3)):
        put(img, x, int(hy - 5), SKIN_LIGHT)
    for y in range(int(hy + 2), int(hy + 5)):
        put(img, int(hx - 5), y, SKIN_SHADE)
    # ears: long, pointed, tilted by `ears`
    for side, (ex, sign) in enumerate(((hx - 5, -1), (hx + 5, 1))):
        tip = (ex + sign * (5 - max(ears, 0) // 2) - (ears if sign < 0 else 0) // 2, hy - 5 + ears)
        line(img, (ex, hy - 1), tip, SKIN_SHADE if sign < 0 else SKIN, 2)
        put(img, tip[0], tip[1], SKIN_LIGHT)
    # eyes: yellow, slit pupils, a heavy brow over them
    for ex in (hx - 3, hx + 1):
        if blink:
            put(img, int(ex), int(hy - 1), OUTLINE)
            put(img, int(ex) + 1, int(hy - 1), OUTLINE)
        else:
            for t in (0, 1):
                put(img, int(ex) + t, int(hy - 1), EYE)
                put(img, int(ex) + t, int(hy), EYE)
            put(img, int(ex) + 1, int(hy - 1), PUPIL)
            put(img, int(ex) + 1, int(hy), PUPIL)
        put(img, int(ex), int(hy - 2), SKIN_SHADE)
        put(img, int(ex) + 1, int(hy - 2), SKIN_SHADE)
    # nose, hooked down
    put(img, int(hx + 4), int(hy), NOSE)
    put(img, int(hx + 5), int(hy + 1), NOSE)
    put(img, int(hx + 5), int(hy + 2), SKIN_SHADE)
    # mouth and fangs
    mx, my = int(hx - 2), int(hy + 2)
    if mouth == 0:
        for x in range(mx, mx + 6):
            put(img, x, my + 1, MOUTH)
        put(img, mx + 1, my + 2, TOOTH)
        put(img, mx + 4, my + 2, TOOTH)
    else:
        # open: a dark cavity with the upper fangs down from the top and the lower teeth up from the chin
        rows = 2 if mouth == 1 else 3
        for r in range(rows):
            for x in range(mx - 1, mx + 7):
                put(img, x, my + r, MOUTH)
        put(img, mx - 1, my, SKIN)
        put(img, mx + 6, my, SKIN)
        for x in (mx, mx + 5):
            put(img, x, my, TOOTH)
            put(img, x, my + 1, TOOTH)
        if mouth == 2:
            for x in (mx + 1, mx + 3, mx + 4):
                put(img, x, my + 2, TOOTH)
    # --- the front arm
    shoulder = (17 + lean, 13 + bob)
    hand = (21 + lean + arms[1] // 2, 16 + bob - arms[1])
    line(img, shoulder, hand, SKIN, 2)
    for t in (0, 1):
        put(img, hand[0] + 1, hand[1] - t, CLAW)
    outline(img)
    return img


def frames():
    out = {}
    # waiting: a bob, the ears twitching, a blink
    out["goblin_0"] = goblin(bob=0, legs=(-1, 1), arms=(0, 0), ears=0)
    out["goblin_1"] = goblin(bob=1, legs=(-1, 1), arms=(1, -1), ears=-1, blink=True)
    # running: hunched forward, the legs and arms swinging, ears streaming back
    out["goblin_2"] = goblin(bob=1, lean=2, legs=(-4, 4), arms=(3, -2), ears=2)
    out["goblin_3"] = goblin(bob=0, lean=2, legs=(-1, 1), arms=(0, 0), ears=3)
    out["goblin_4"] = goblin(bob=1, lean=2, legs=(4, -4), arms=(-2, 3), ears=2)
    out["goblin_5"] = goblin(bob=0, lean=2, legs=(1, -1), arms=(0, 0), ears=3)
    # the bite: jaws wide with the fangs out, then snapping shut with a lunge
    out["goblin_6"] = goblin(bob=1, lean=3, legs=(-3, 3), arms=(3, 3), ears=-1, mouth=2, lunge=1)
    out["goblin_7"] = goblin(bob=1, lean=4, legs=(-3, 4), arms=(-1, 4), ears=3, mouth=0, lunge=3)
    return out


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    made = frames()
    for name, img in made.items():
        img.save(OUT / f"{name}.png")
    print(f"{len(made)} goblin drawings written")
    if "--sheet" in sys.argv:
        sheet = Path(sys.argv[sys.argv.index("--sheet") + 1])
        s = 8
        img = Image.new("RGBA", (len(made) * (W * s + 8), H * s + 8), (90, 120, 150, 255))
        for k, g in enumerate(made.values()):
            img.alpha_composite(g.resize((W * s, H * s), Image.NEAREST), (4 + k * (W * s + 8), 4))
        img.save(sheet)


if __name__ == "__main__":
    main()
