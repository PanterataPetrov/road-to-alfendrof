"""Draws Katmata, The Crepe: every pose from tools/katmata.png, plus his cigarette and code shots, the two robot
agents, his skill icons and his portrait.

    python tools/gen_katmata.py [--sheet contact_sheet.png]

The drawing (33 x 55, feet on row 54) is cut into a body and two skin arms. Each pose moves the body's bands (head and
chest, hips, boots), draws the arms again between a shoulder and a hand (a short white sleeve, then skin), and adds
the props: the lit cigarette in his mouth, the pack on his belt, a laptop, a lighter. `<frame>_mark.png` holds the
ember of his cigarette, wherever it ended up; the game tints it when his special is ready.
"""

import math
import random
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SPRITES = ROOT / "assets/sprites/katmata"
ICONS = ROOT / "assets/icons/katmata"
PORTRAIT = ROOT / "assets/portraits/katmata.png"

W, H = 84, 63
# the drawing is 33 x 55 with its feet on row 54; the game wants the body centre at x = 33 and the feet on the last row
OFF_X, OFF_Y = 17, 8

SKIN = (199, 171, 146)
SKIN_SHADE = (166, 140, 116)
SKIN_LIGHT = (223, 192, 164)
SKIN_SET = {(199, 171, 146), (185, 157, 131), (166, 140, 116), (223, 192, 164), (232, 198, 169)}
SLEEVE = (229, 240, 240)
SLEEVE_SHADE = (199, 207, 207)
OUTLINE = (40, 34, 38)
MOUTH = (110, 60, 50)
FILTER = (232, 196, 140)
PAPER = (246, 246, 240)
EMBER = (255, 130, 40)
EMBER_HOT = (255, 222, 110)
SMOKE = (206, 210, 218)
SMOKE_FAINT = (150, 156, 170)
FLAME = (255, 196, 70)
PACK = (196, 52, 52)
PACK_LIGHT = (240, 236, 228)
SILVER = (186, 192, 204)
SILVER_DARK = (110, 118, 136)
SILVER_LIGHT = (226, 230, 240)
GLOW = (110, 255, 150)
GLOW_DARK = (30, 80, 50)
TOBACCO = (126, 84, 44)

here = Path(__file__).resolve().parent
SRC = Image.open(here / "katmata.png").convert("RGBA")


def blank(w=W, h=H):
    return Image.new("RGBA", (w, h), (0, 0, 0, 0))


def put(img, x, y, c):
    if 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((x, y), tuple(c) + (255,) if len(c) == 3 else tuple(c))


def line(img, a, b, colour, width=1):
    (x0, y0), (x1, y1) = a, b
    steps = max(abs(x1 - x0), abs(y1 - y0), 1)
    for k in range(steps + 1):
        t = k / steps
        x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
        for ox in range(-(width // 2), width - width // 2):
            for oy in range(-(width // 2), width - width // 2):
                put(img, x + ox, y + oy, colour)


def split_arms(img):
    """The drawing without the skin of its arms, and the pixels of each arm, (x, y) -> colour. The short white
    sleeves stay on the body."""
    body = img.copy()
    back, front = {}, {}
    for y in range(img.height):
        for x in range(img.width):
            c = img.getpixel((x, y))
            if c[3] == 0 or c[:3] not in SKIN_SET:
                continue
            if x <= 10 and 21 <= y <= 31:
                back[(x, y)] = c
            elif x >= 23 and y >= 22:
                front[(x, y)] = c
            else:
                continue
            body.putpixel((x, y), (0, 0, 0, 0))
    # the pack of cigarettes on his belt, at the right hip
    for (x, y, c) in ((21, 36, PACK_LIGHT), (22, 36, PACK_LIGHT), (23, 36, PACK_LIGHT), (21, 37, PACK), (22, 37, PACK), (23, 37, PACK),
                      (21, 38, PACK), (22, 38, PACK), (23, 38, PACK), (21, 39, (150, 36, 36)), (22, 39, (150, 36, 36)), (23, 39, (150, 36, 36))):
        body.putpixel((x, y), c + (255,))
    return body, back, front


BODY, BACK_ARM, FRONT_ARM = split_arms(SRC)

BACK_SHOULDER = (8, 20)
FRONT_SHOULDER = (24, 20)


def two_bone(shoulder, hand, upper, fore, bend):
    (sx, sy), (hx, hy) = shoulder, hand
    dx, dy = hx - sx, hy - sy
    dist = max(math.hypot(dx, dy), 0.001)
    ux, uy = dx / dist, dy / dist
    if dist >= upper + fore - 0.01:
        reach = upper + fore - 0.01
        return (round(sx + ux * upper), round(sy + uy * upper)), (round(sx + ux * reach), round(sy + uy * reach))
    a = (upper * upper - fore * fore + dist * dist) / (2 * dist)
    h = math.sqrt(max(upper * upper - a * a, 0))
    return (round(sx + ux * a - uy * h * bend), round(sy + uy * a + ux * h * bend)), (round(hx), round(hy))


def fist(img, c, size=5):
    cx, cy = c
    for dy in range(-2, 3):
        for dx in range(-2, 3):
            if abs(dx) == 2 and abs(dy) == 2:
                continue
            put(img, cx + dx, cy + dy, SKIN_SHADE if dy == 2 else SKIN)
    put(img, cx + 1, cy - 2, SKIN_LIGHT)
    put(img, cx + 2, cy - 1, SKIN_LIGHT)


def arm(img, shoulder, hand, bend=1, upper=8, fore=8):
    """One arm: a short white sleeve at the shoulder, bare skin from there to the fist."""
    elbow, wrist = two_bone(shoulder, hand, upper, fore, bend)
    line(img, elbow, wrist, SKIN, 5)
    line(img, (elbow[0], elbow[1] + 2), (wrist[0], wrist[1] + 2), SKIN_SHADE, 1)
    # the upper arm: skin, with the sleeve over its first part
    line(img, shoulder, elbow, SKIN, 5)
    sx, sy = shoulder
    mx, my = round(sx + (elbow[0] - sx) * 0.5), round(sy + (elbow[1] - sy) * 0.5)
    line(img, shoulder, (mx, my), SLEEVE, 6)
    line(img, (shoulder[0], shoulder[1] + 3), (mx, my + 3), SLEEVE_SHADE, 1)
    fist(img, wrist)
    return elbow, wrist


def stamp_arm(img, pixels, dx=0, dy=0):
    for (x, y), c in pixels.items():
        put(img, x + OFF_X + dx, y + OFF_Y + dy, c)


CROUCH = tuple(range(25, 29)) + tuple(range(40, 47))


class Pose:
    def __init__(self):
        self.canvas = blank()
        self.up = (0, 0)
        self.ember = None
        self.mark_img = None

    def body(self, up=(0, 0), skirt=(0, 0), boots=((0, 0), (0, 0)), drop=()):
        c = self.canvas
        drop = set(drop)

        def below(r):
            return sum(1 for d in drop if d > r)

        self.up = (up[0], up[1] + below(22))
        for y in [*range(49, 55), *range(34, 49), *range(0, 34)]:
            if y in drop:
                continue
            bdx, bdy = up if y < 34 else skirt if y < 49 else (0, 0)
            for x in range(BODY.width):
                p = BODY.getpixel((x, y))
                if not p[3]:
                    continue
                dx, dy = bdx, bdy
                if y >= 49:
                    dx, dy = boots[0] if x < 16 else boots[1]
                put(c, x + OFF_X + dx, y + OFF_Y + dy + below(y), p)
        return self

    def at(self, x, y):
        return (x + OFF_X + self.up[0], y + OFF_Y + self.up[1])

    def shoulder(self, which):
        return self.at(*(BACK_SHOULDER if which == "back" else FRONT_SHOULDER))

    def face(self, kind):
        ox, oy = OFF_X + self.up[0], OFF_Y + self.up[1]
        c = self.canvas
        if kind in ("closed", "hurt"):
            for x in (14, 15, 18, 19):
                put(c, x + ox, 9 + oy, SKIN)
                put(c, x + ox, 10 + oy, OUTLINE)
        if kind in ("open", "hurt"):
            for x in (15, 16, 17):
                put(c, x + ox, 13 + oy, OUTLINE)
                put(c, x + ox, 14 + oy, MOUTH)
        return self

    def cig(self, lit=True, smoke=-1, drooping=False, mouth=True):
        """The cigarette in the corner of his mouth, its ember, and (smoke 0..2) a wisp rising from it. With
        `mouth` off there is none in his mouth at all (it has been thrown)."""
        ox, oy = OFF_X + self.up[0], OFF_Y + self.up[1]
        c = self.canvas
        y = 13 + (1 if drooping else 0)
        if not mouth:
            self.ember = None
            return self
        for k, col in enumerate((FILTER, FILTER, PAPER, PAPER, PAPER)):
            put(c, 19 + k + ox, y + oy, col)
        if lit:
            put(c, 24 + ox, y + oy, EMBER)
            put(c, 25 + ox, y + oy, EMBER_HOT)
            self.ember = (24 + ox, y + oy)
            if smoke >= 0:
                wisp = [[(25, y - 2), (26, y - 4), (25, y - 6), (26, y - 8)], [(26, y - 2), (25, y - 4), (26, y - 6), (25, y - 8)], [(25, y - 3), (26, y - 5), (27, y - 7), (26, y - 9)]][smoke % 3]
                for n, (sx, sy) in enumerate(wisp):
                    put(c, sx + ox, sy + oy, SMOKE if n < 2 else SMOKE_FAINT)
        else:
            self.ember = None
        return self

    def arms(self, back=None, front=None, back_bend=-1, front_bend=1, **kw):
        if back == "rest":
            stamp_arm(self.canvas, BACK_ARM, *self.up)
        elif back:
            arm(self.canvas, self.shoulder("back"), self.at(*back), bend=back_bend, **kw)
        if front == "rest":
            stamp_arm(self.canvas, FRONT_ARM, *self.up)
        elif front:
            arm(self.canvas, self.shoulder("front"), self.at(*front), bend=front_bend, **kw)
        return self

    def mark(self):
        if self.mark_img is not None:
            return self.mark_img
        m = blank()
        if self.ember:
            x, y = self.ember
            for dx, dy in ((0, 0), (1, 0), (0, -1)):
                put(m, x + dx, y + dy, (255, 255, 255))
        return m


def rotated(p, angle, scale=1.0, lift=0):
    """Turn a finished pose and its ember about the middle and stand it on the floor again."""
    mark = p.mark()
    box = None
    for img in (p.canvas, mark):
        b = img.getbbox()
        if b:
            box = b if box is None else (min(box[0], b[0]), min(box[1], b[1]), max(box[2], b[2]), max(box[3], b[3]))
    out = []
    for img in (p.canvas, mark):
        crop = img.crop(box)
        if scale != 1.0:
            crop = crop.resize((max(1, round(crop.width * scale)), max(1, round(crop.height * scale))), Image.NEAREST)
        out.append(crop.rotate(angle, resample=Image.NEAREST, expand=True))
    b = out[0].getbbox()
    left = round(33 - (b[0] + b[2]) / 2)
    top = (H - 1 - lift) - (b[3] - 1)
    res = []
    for img in out:
        canvas = blank()
        canvas.paste(img, (left, top))
        res.append(canvas)
    p.canvas, p.mark_img = res
    return p


def ball(p, angle):
    """A roll: the tucked body, smaller, turned `angle` degrees about a fixed centre just above the floor, so it
    spins in place instead of jumping about from frame to frame."""
    out = []
    for img in (p.canvas, p.mark()):
        box = p.canvas.getbbox()
        crop = img.crop(box)
        crop = crop.resize((max(1, round(crop.width * 0.8)), max(1, round(crop.height * 0.8))), Image.NEAREST)
        canvas = blank()
        # the same spot every frame: its middle at x 33, 14 pixels above the floor
        canvas.paste(crop, (33 - crop.width // 2, H - 1 - 14 - crop.height // 2))
        out.append(canvas.rotate(angle, resample=Image.NEAREST, center=(33, H - 1 - 14)))
    p.canvas, p.mark_img = out
    return p


def glints(img, around, colour, n=6, r=5, seed=0):
    x, y = around
    for k in range(n):
        a = (k * 2.4 + seed) % (2 * math.pi)
        d = r * (0.6 + 0.4 * ((k * 37 + seed * 11) % 10) / 10)
        put(img, round(x + math.cos(a) * d), round(y + math.sin(a) * d), colour)


def lighter(img, at_, flame=True):
    """A silver lighter in the fingers with a little flame."""
    x, y = at_
    for dx in range(0, 3):
        for dy in range(0, 4):
            put(img, x + dx, y + dy, SILVER if dy else SILVER_LIGHT)
    if flame:
        for dx, dy, c in ((1, -1, FLAME), (1, -2, FLAME), (1, -3, EMBER_HOT), (0, -2, EMBER), (2, -2, EMBER)):
            put(img, x + dx, y + dy, c)


def loose_cigarette(img, x, y, angle=0.0, length=6, lit=True):
    """A cigarette held between fingers, pointing along `angle` (radians, 0 = forward)."""
    for k in range(length):
        put(img, round(x + math.cos(angle) * k), round(y + math.sin(angle) * k), PAPER if k > 1 else FILTER)
    if lit:
        ex, ey = round(x + math.cos(angle) * length), round(y + math.sin(angle) * length)
        put(img, ex, ey, EMBER)
        put(img, round(x + math.cos(angle) * (length + 1)), round(y + math.sin(angle) * (length + 1)), EMBER_HOT)


CUP = (214, 230, 240)
CUP_SHADE = (160, 184, 200)
LATTE_MILK = (236, 220, 190)
LATTE_COFFEE = (122, 78, 48)
ICE = (240, 250, 255)
STRAW = (46, 130, 86)


def cup(img, cx, bottom, tilt=0):
    """An iced latte in a clear plastic cup, 6 wide and 10 tall, standing on row `bottom` centred on column `cx`: milky
    coffee in two layers, ice cubes, a domed lid and a green straw leaning out."""
    for dy in range(0, 8):
        y = bottom - dy
        for dx in range(-3, 3):
            wall = dx in (-3, 2)
            if wall:
                put(img, cx + dx, y, CUP_SHADE if dx == 2 else CUP)
            elif dy < 4:
                put(img, cx + dx, y, LATTE_MILK if dy < 2 else (200, 158, 112))
            else:
                put(img, cx + dx, y, LATTE_COFFEE if dy < 6 else (150, 100, 68))
    for dx, dy in ((-1, 5), (1, 3), (0, 1)):
        put(img, cx + dx, bottom - dy, ICE)
        put(img, cx + dx + 1, bottom - dy, ICE)
    for dx in range(-3, 3):
        put(img, cx + dx, bottom - 8, (240, 240, 246))
    for dx in range(-2, 2):
        put(img, cx + dx, bottom - 9, (240, 240, 246))
    for k in range(5):
        put(img, cx + 1 + k // 2 + tilt, bottom - 8 - k, STRAW)


def laptop(img, base_left, open_screen=True, keys=0, closed=False):
    """A laptop seen from the side, its screen facing the player on the left: the base, the hinge at the far end,
    and the lid tipped back with a green glow on the inside."""
    bx, by = base_left
    if closed:
        for x in range(bx, bx + 13):
            put(img, x, by, SILVER_LIGHT)
            put(img, x, by + 1, SILVER)
            put(img, x, by + 2, SILVER_DARK)
        return
    for x in range(bx, bx + 13):
        put(img, x, by, SILVER_LIGHT)
        put(img, x, by + 1, SILVER)
        put(img, x, by + 2, SILVER_DARK)
    # the lid, leaning back from the hinge at the right end
    hx, hy = bx + 12, by
    for k in range(13):
        lx, ly = hx - k // 3, hy - k
        put(img, lx, ly, SILVER_LIGHT)
        put(img, lx + 1, ly, SILVER)
        put(img, lx + 2, ly, SILVER_DARK)
        # the screen's glow on the player's side
        put(img, lx - 1, ly, GLOW if (k + keys) % 3 else GLOW_DARK)
    # the logo dot on the lid's back
    put(img, hx - 2, hy - 7, (255, 255, 255))
    for k in range(3):
        put(img, bx + 2 + k * 3 + (keys % 2), by - 1, (80, 90, 110))


# ------------------------------------------------------------------------------------------------ the poses
def build():
    frames = {}

    def pose(name, **kw):
        p = Pose().body(**kw)
        frames[name] = p
        return p

    # standing, breathing, smoke curling up from the lit cigarette
    p = pose("idle_0"); p.arms("rest", "rest"); p.cig(smoke=0)
    p = pose("idle_1", up=(0, 1)); p.arms("rest", "rest"); p.cig(smoke=1)

    # rolling a cigarette: the paper held in both hands, tobacco sprinkled in, rolled between the thumbs, the edge licked
    p = pose("craft_0", up=(0, 1)); p.arms((15, 29), (21, 28), front_bend=1); p.cig(smoke=2)
    for dx in range(0, 5):
        put(p.canvas, p.at(16, 27)[0] + dx, p.at(16, 27)[1] + 1, PAPER)
    p = pose("craft_1", up=(0, 1)); p.arms((15, 29), (22, 28), front_bend=1); p.cig(smoke=0)
    for dx in range(0, 5):
        put(p.canvas, p.at(16, 27)[0] + dx, p.at(16, 27)[1] + 1, PAPER)
    for dx, dy in ((1, -1), (2, -2), (3, -1), (2, 0)):
        put(p.canvas, p.at(17, 27)[0] + dx, p.at(17, 27)[1] + dy, TOBACCO)
    p = pose("craft_2"); p.arms((15, 28), (21, 28), front_bend=1); p.cig(smoke=1)
    for dx in range(0, 6):
        put(p.canvas, p.at(15, 27)[0] + dx, p.at(15, 27)[1], PAPER)
        put(p.canvas, p.at(15, 27)[0] + dx, p.at(15, 27)[1] + 1, SLEEVE_SHADE)
    p = pose("craft_3"); p.arms((19, 15), (22, 28), front_bend=1, back_bend=1); p.cig(smoke=2)
    for dx in range(0, 5):
        put(p.canvas, p.at(18, 14)[0] + dx, p.at(18, 14)[1], PAPER)

    # walking: the boots swap, the body dips as the feet pass, the hands swing a little
    for k in range(6):
        a = 2 * math.pi * k / 6
        s_, c_ = math.sin(a), math.cos(a)
        boots = ((round(4 * s_), -2 * int(c_ > 0.4)), (-round(4 * s_), -2 * int(c_ < -0.4)))
        p = pose(f"walk_{k}", up=(0, int(abs(c_) < 0.5)), boots=boots)
        p.arms("rest", (30 + round(4 * s_), 34 - int(abs(c_) > 0.5)), front_bend=1)
        p.cig(smoke=k % 3 if k % 2 == 0 else -1)

    # crouching
    p = pose("crouch", drop=CROUCH); p.arms("rest", "rest"); p.cig()
    p = pose("crouch_soft_0", drop=CROUCH); p.arms("rest", (24, 26)); p.cig()
    p = pose("crouch_soft_1", drop=CROUCH); p.arms("rest", (40, 30), upper=9, fore=9); p.cig()
    lighter(p.canvas, (p.at(41, 28)[0], p.at(41, 28)[1] - 1))
    p = pose("crouch_hard_0", drop=CROUCH); p.arms((8, 28), (26, 28)); p.cig()
    p = pose("crouch_hard_1", drop=CROUCH); p.arms("rest", (38, 12), upper=9, fore=9); p.cig()

    # in the air
    p = pose("jump_up"); p.arms((3, 12), (28, 12)); p.cig()
    p = pose("jump_top"); p.arms((-4, 22), (38, 22), upper=9, fore=9); p.cig()
    p = pose("jump_fall"); p.arms((4, 8), (28, 8)); p.cig()

    # hurt, blocking, down
    p = pose("hurt_0", up=(-2, 0)); p.arms((-3, 28), (32, 22), upper=9, fore=9); p.face("hurt"); p.cig(drooping=True)
    p = pose("hurt_1", up=(-3, 1)); p.arms((4, 31), (24, 31)); p.face("closed"); p.cig(drooping=True)
    p = pose("block", up=(0, 1)); p.arms((20, 14), (22, 11), back_bend=1, upper=9, fore=9); p.cig()
    p = pose("ko"); p.arms((-2, 32), (28, 34), upper=9, fore=9); p.face("closed"); p.cig(lit=False, drooping=True); rotated(p, 90)

    # the lighter flick (out of cigarettes), and the plain air and low lights
    p = pose("soft_0"); p.arms("rest", (22, 24)); p.cig()
    p = pose("soft_1", up=(1, 0)); p.arms("rest", (38, 25), upper=9, fore=9); p.cig()
    lighter(p.canvas, (p.at(39, 24)[0], p.at(39, 24)[1] - 1))
    p = pose("air_soft_0"); p.arms("rest", (24, 18)); p.cig()
    p = pose("air_soft_1"); p.arms("rest", (38, 30), upper=9, fore=9); p.cig()
    lighter(p.canvas, (p.at(39, 29)[0], p.at(39, 29)[1] - 1))
    p = pose("air_hard_0"); p.arms((4, 8), (28, 8)); p.cig()
    p = pose("air_hard_1"); p.arms((8, 30), (36, 32), upper=9, fore=9); p.cig()

    # unused heavy drawings the game still expects
    p = pose("hard_0"); p.arms("rest", (24, 26)); p.cig()
    p = pose("hard_1", up=(1, 0)); p.arms("rest", (32, 22)); p.cig()
    p = pose("hard_2", up=(1, 0)); p.arms("rest", (40, 24), upper=9, fore=9); p.cig()
    p = pose("hard_3"); p.arms("rest", (30, 28)); p.cig()

    # the cigarette toss: the arm cocked back with the cigarette between the fingers; the throw; a new one from the
    # pack at his belt; lighting it
    # (the cigarette leaves his mouth with the throw, so there is none there until the new one is put in)
    p = pose("throw_0", up=(-1, 0)); p.arms("rest", (30, 12), upper=9, fore=9); p.cig(smoke=0)
    loose_cigarette(p.canvas, p.at(33, 11)[0], p.at(33, 11)[1], -0.4)
    p = pose("throw_1", up=(2, 1)); p.arms("rest", (44, 24), upper=10, fore=10); p.cig(mouth=False)
    for dx in range(3, 9):
        put(p.canvas, p.at(44, 24)[0] + dx, p.at(44, 24)[1] + 1, EMBER if dx % 2 else EMBER_HOT)
    # the reload, in six drawings: the hand reaches the pack; a cigarette slides out; it is lifted; the filter goes into
    # his mouth; the lighter flames at its tip; the first drag, the ember glowing and a puff of smoke
    p = pose("throw_2", up=(0, 1)); p.arms("rest", (23, 37), front_bend=1); p.cig(mouth=False)
    p = pose("throw_3", up=(0, 1)); p.arms("rest", (23, 36), front_bend=1); p.cig(mouth=False)
    for k in range(5):
        put(p.canvas, p.at(22, 34)[0], p.at(22, 34)[1] - k, PAPER if k < 4 else FILTER)
    p = pose("throw_4"); p.arms("rest", (22, 24), front_bend=1); p.cig(mouth=False)
    for k in range(6):
        put(p.canvas, p.at(23, 22)[0] + k // 2, p.at(23, 22)[1] - k // 2 - 1, PAPER if k > 1 else FILTER)
    p = pose("throw_5", up=(0, 1)); p.arms("rest", (22, 16), front_bend=-1); p.cig(lit=False)
    p = pose("throw_6", up=(0, 1)); p.arms("rest", (24, 14), front_bend=-1); p.cig(lit=False)
    lighter(p.canvas, (p.at(24, 14)[0] + 1, p.at(24, 14)[1]))
    for dx, dy, c in ((0, -3, FLAME), (1, -4, EMBER_HOT), (-1, -2, EMBER), (1, -2, FLAME), (1, -5, EMBER), (0, -4, FLAME)):
        put(p.canvas, p.at(24, 14)[0] + 1 + dx, p.at(24, 14)[1] + dy, c)
    p = pose("throw_7", up=(0, 1)); p.arms("rest", (22, 24), front_bend=1); p.cig(lit=True, smoke=0)
    for dx, dy in ((26, 11), (27, 9), (28, 7), (27, 5), (26, 3)):
        put(p.canvas, dx + OFF_X + p.up[0], dy + OFF_Y + p.up[1], SMOKE if dy > 6 else SMOKE_FAINT)
    for dx, dy in ((24, 12), (25, 12), (25, 14), (24, 14)):
        put(p.canvas, dx + OFF_X + p.up[0], dy + OFF_Y + p.up[1], EMBER_HOT)

    # the iced latte: held cocked back; thrown, a splash at the end of the arm; the follow-through
    p = pose("latte_0", up=(-1, 0)); p.arms("rest", (31, 14), upper=9, fore=9); p.cig(smoke=1)
    cup(p.canvas, p.at(33, 14)[0], p.at(33, 14)[1] - 2)
    p = pose("latte_1", up=(2, 1)); p.arms("rest", (43, 25), upper=10, fore=10); p.cig(smoke=2)
    glints(p.canvas, p.at(47, 24), ICE, 7, 5, 4)
    p = pose("latte_2", up=(1, 0)); p.arms("rest", (36, 28), upper=9, fore=9); p.cig(smoke=0)

    # speaking
    p = pose("speak_0"); p.arms("rest", (28, 25)); p.face("open"); p.cig()
    p = pose("speak_1"); p.arms("rest", (32, 17)); p.face("open"); p.cig(smoke=1)

    # the laptop: shut and held out; opened, typing (the right hand on the keys, then the left); shut again
    p = pose("read_0"); p.arms((20, 31), (30, 30), front_bend=-1); p.cig(); laptop(p.canvas, p.at(20, 29), closed=True)
    p = pose("read_1", up=(1, 0)); p.arms((22, 32), (33, 26), front_bend=-1, back_bend=1); p.cig(); laptop(p.canvas, p.at(22, 30), keys=0)
    p = pose("read_2", up=(1, 1)); p.arms((26, 31), (30, 27), front_bend=-1, back_bend=1); p.cig(smoke=0); laptop(p.canvas, p.at(22, 30), keys=1)
    p = pose("read_3"); p.arms((20, 31), (30, 30), front_bend=-1); p.cig(); laptop(p.canvas, p.at(20, 29), closed=True)

    # the sub-agents: a snap of the fingers, the hands thrown up, the call
    code = (120, 255, 170)
    p = pose("summon_0"); p.arms((6, 31), (28, 31)); p.cig()
    p = pose("summon_1"); p.arms((-3, 20), (34, 20), upper=9, fore=9); p.cig(smoke=0)
    p = pose("summon_2"); p.arms((3, 8), (29, 8)); p.cig(smoke=1)
    glints(p.canvas, p.at(3, 6), code, 5, 4, 1); glints(p.canvas, p.at(29, 6), code, 5, 4, 2)
    p = pose("summon_3"); p.arms((12, 0), (22, 0), upper=10, fore=11); p.face("open"); p.cig()
    glints(p.canvas, p.at(17, -2), code, 9, 7, 3)
    p = pose("summon_4", up=(2, 0)); p.arms((10, 8), (40, 18), upper=9, fore=9); p.cig()
    glints(p.canvas, p.at(41, 18), code, 8, 6, 4)
    p = pose("summon_5"); p.arms((6, 28), (38, 28), upper=9, fore=9); p.cig()

    # rolling: tuck, turn over as a ball, come up
    p = pose("roll_0", drop=CROUCH); p.arms("rest", "rest"); p.cig()
    for k in range(1, 9):
        base = Pose().body(drop=CROUCH, up=(0, -2)); base.arms("rest", "rest"); base.cig()
        frames[f"roll_{k}"] = ball(base, -45 * k)
    p = pose("roll_9", drop=CROUCH[:7]); p.arms("rest", "rest"); p.cig()
    return frames


# ------------------------------------------------------------------------------------------------ effects
def cigarette_shot(k):
    """The flicked cigarette: 8 x 4, spinning (two drawings), a glowing ember at the front and a few sparks."""
    img = blank(10, 6)
    if k == 0:
        for x in range(1, 8):
            put(img, x, 3, PAPER if x > 2 else FILTER)
        put(img, 8, 3, EMBER)
        put(img, 9, 3, EMBER_HOT)
        put(img, 0, 1, EMBER_HOT)
        put(img, 2, 5, EMBER)
    else:
        for x in range(1, 8):
            put(img, x, 2 + (x - 1) // 3, PAPER if x > 2 else FILTER)
        put(img, 8, 4, EMBER)
        put(img, 9, 4, EMBER_HOT)
        put(img, 0, 4, EMBER_HOT)
        put(img, 3, 0, EMBER)
    return img


def latte_shot(k):
    """The thrown iced latte, 10 x 14: upright, wobbling a little (two drawings) with ice rattling in it."""
    img = blank(10, 14)
    cup(img, 5, 12, tilt=k)
    for dx, dy in ((0, 0), (9, 2)) if k else ((9, 0), (0, 3)):
        put(img, dx, dy, ICE)
    return img


def spill(k):
    """A puddle of iced latte on the floor, 44 x 10: milky brown with a lighter edge and highlights, two ice cubes
    and a drip or two; the second drawing has the ripple moved."""
    img = blank(44, 10)
    cx, cy, rx, ry = 22, 5, 21, 4
    for y in range(10):
        for x in range(44):
            d = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2
            if d > 1:
                continue
            col = (176, 128, 86) if d < 0.55 else (206, 168, 124) if d < 0.85 else (232, 208, 170)
            put(img, x, y, col)
    for x in range(10 + 4 * k, 20 + 4 * k):
        put(img, x, 3, (236, 214, 176))
    for x, y in ((12 + k, 4), (29 - k, 5)):
        for dx in range(3):
            for dy in range(2):
                put(img, x + dx, y + dy, ICE)
    for x, y in ((3, 2), (40, 6), (36, 1)):
        put(img, x, y, (150, 106, 70))
    return img


def glitch_field(k):
    """A glitched patch of the map, 160 x 80: a faint violet haze with scanlines, blocks of cyan, magenta and green
    that slide about, torn strips, bright noise and bracket corners; it fades out towards its sides. Four drawings."""
    rnd = random.Random(1000 + k)
    img = blank(160, 80)
    px_ = img.load()
    palette = [(0, 255, 240), (255, 0, 190), (130, 255, 70), (255, 255, 255)]

    def blend(x, y, col, a):
        if 0 <= x < 160 and 0 <= y < 80:
            r0, g0, b0, a0 = px_[x, y]
            na = max(a0, a)
            px_[x, y] = (col[0], col[1], col[2], na)

    for y in range(80):
        for x in range(160):
            edge = min(x, 159 - x) / 24.0
            fade = max(0.0, min(1.0, edge))
            a = 46 + (22 if y % 4 == 0 else 0) + (24 if y > 70 else 0)
            px_[x, y] = (96, 34, 150, int(a * fade))
    # sliding blocks
    for n in range(18):
        w, h = rnd.randint(10, 64), rnd.randint(2, 9)
        x0, y0 = rnd.randint(-6, 150), rnd.randint(0, 76)
        col = palette[rnd.randrange(4)]
        alpha = rnd.randint(70, 150)
        for y in range(y0, y0 + h):
            for x in range(x0, x0 + w):
                if 0 <= x < 160:
                    fade = max(0.0, min(1.0, min(x, 159 - x) / 24.0))
                    blend(x, y, col, int(alpha * fade))
    # torn strips: a row of the haze pushed sideways, with a bright edge
    for n in range(4):
        y0 = rnd.randint(4, 72)
        shift = rnd.randint(-10, 10)
        for y in range(y0, y0 + 2):
            for x in range(20, 140):
                blend(x + shift, y, (220, 255, 255), 120)
    # noise
    for n in range(150):
        x, y = rnd.randint(8, 151), rnd.randint(0, 79)
        blend(x, y, palette[rnd.randrange(4)], rnd.randint(120, 220))
    # brackets in the corners and a bright strip on the floor
    for (cx, cy, sx, sy) in ((6, 2, 1, 1), (153, 2, -1, 1), (6, 77, 1, -1), (153, 77, -1, -1)):
        for t in range(9):
            blend(cx + sx * t, cy, (0, 255, 240), 210)
            blend(cx, cy + sy * t, (0, 255, 240), 210)
    for x in range(12, 148):
        blend(x, 78, (0, 255, 240), 150)
    return img


METAL = (178, 186, 200)
METAL_DARK = (104, 112, 132)
METAL_LIGHT = (226, 232, 244)
JOINT = (40, 44, 58)
VISOR = (90, 230, 255)
VISOR_DARK = (30, 140, 170)
LIGHT_ORANGE = (255, 150, 50)
CLOTH = (56, 60, 80)
CLOTH_LIGHT = (84, 90, 118)
CLOTH_DARK = (36, 40, 56)
BURST = (222, 122, 88)
BURST_LIGHT = (246, 170, 132)

# the orange starburst on the chest: a dozen uneven rays round a centre
STARBURST = [
    "....x....",
    ".x..x..x.",
    "..x.x.x..",
    "x..xxx..x",
    ".xxxxxxx.",
    "x..xxx..x",
    "..x.x.x..",
    ".x..x..x.",
    "....x....",
]


def starburst(img, cx, cy):
    for r, row in enumerate(STARBURST):
        for c, ch in enumerate(row):
            if ch == "x":
                put(img, cx - 4 + c, cy - 4 + r, BURST_LIGHT if (r + c) % 4 == 0 else BURST)


def robotise(img, drawing=True):
    """Katmata's drawing as a robot: the skin becomes metal, the white shirt dark cloth, the jeans dark trousers, the
    boots dark; the cap becomes a smooth metal head; the eyes glow; the pack and the shirt's little logo go. `drawing`
    is the 33 x 55 drawing itself (so the face and chest can be found), otherwise a canvas of arms and props."""
    out = img.copy()
    for y in range(out.height):
        for x in range(out.width):
            r, g, b, a = out.getpixel((x, y))
            if not a:
                continue
            rgb = (r, g, b)
            new = None
            lum = (r + g + b) / 3
            if rgb in SKIN_SET or rgb == SKIN or rgb == SKIN_SHADE or rgb == SKIN_LIGHT:
                new = METAL_LIGHT if lum > 205 else METAL if lum > 170 else METAL_DARK
            elif drawing and (y <= 6 or (y <= 15 and r > g > b and r - b > 30 and lum < 170)) and not (r > 150 and g > 150 and b > 150):
                # the cap: green, orange and brown, all metal now
                new = METAL_LIGHT if lum > 150 else METAL if lum > 120 else METAL_DARK
            elif r > 195 and g > 195 and b > 190:
                new = CLOTH_LIGHT if lum > 225 else CLOTH if lum > 205 else CLOTH_DARK
            elif b > r + 25 and b > g + 10:
                new = (74, 78, 98) if lum > 130 else (56, 60, 78) if lum > 100 else (42, 46, 62)
            elif g > r + 10 and g > b:
                new = CLOTH
            elif rgb in ((196, 52, 52), (240, 236, 228), (150, 36, 36)):
                new = (56, 60, 78)
            elif drawing and y >= 49 and r > 90 and r > b + 20:
                new = (92, 96, 116) if lum > 100 else (60, 64, 82)
            elif r > 90 and r > b + 20 and not drawing:
                new = (92, 96, 116)
            if new is not None:
                out.putpixel((x, y), new + (a,))
    if drawing:
        # eyes: the whites and pupils on rows 9 and 10 glow
        for x in (14, 18):
            for y in (9, 10):
                out.putpixel((x, y), VISOR + (255,))
                out.putpixel((x + 1, y), VISOR_DARK + (255,))
        # the little logo on his chest goes
        for y in range(19, 24):
            for x in range(15, 22):
                r, g, b, a = out.getpixel((x, y))
                if a and g > r + 10 and g > b:
                    out.putpixel((x, y), CLOTH + (255,))
    return out


def agent_pose(k):
    """A sub-agent: Katmata's body and poses as a robot with the starburst on its chest, 84 x 63 like a fighter. 0:
    standing guard; 1: a step; 2: a punch wound up; 3: the punch; 4: a dash; 5..8: rolling (a ball turned a
    quarter at a time)."""
    global BODY, BACK_ARM, FRONT_ARM, SKIN, SKIN_SHADE, SKIN_LIGHT, SLEEVE, SLEEVE_SHADE
    saved = (BODY, BACK_ARM, FRONT_ARM, SKIN, SKIN_SHADE, SKIN_LIGHT, SLEEVE, SLEEVE_SHADE)
    # the arms in the robot's colours
    BODY = robotise(BODY)

    def recolour_arm(arm_pixels):
        return {pt: tuple(robotise(Image.new("RGBA", (1, 1), c), drawing=False).getpixel((0, 0))) for pt, c in arm_pixels.items()}

    BACK_ARM, FRONT_ARM = recolour_arm(BACK_ARM), recolour_arm(FRONT_ARM)
    SKIN, SKIN_SHADE, SKIN_LIGHT = METAL, METAL_DARK, METAL_LIGHT
    SLEEVE, SLEEVE_SHADE = CLOTH_LIGHT, CLOTH
    try:
        if k >= 5:
            base = Pose().body(drop=CROUCH, up=(0, -2))
            base.arms("rest", "rest")
            starburst(base.canvas, *base.at(15, 28))
            ball(base, -90 * (k - 5))
            return base.canvas
        poses = {
            0: dict(up=(0, 0), boots=((0, 0), (0, 0)), back="rest", front="rest"),
            1: dict(up=(0, 1), boots=((-3, 0), (3, -2)), back="rest", front=(30, 33)),
            2: dict(up=(-2, 0), boots=((-2, 0), (0, 0)), back=(15, 25), front=(20, 17)),
            3: dict(up=(3, 1), boots=((-1, 0), (4, -1)), back=(16, 27), front=(44, 23)),
            4: dict(up=(3, 0), skirt=(2, 0), boots=((-5, 0), (5, -2)), back=(2, 26), front=(34, 28)),
        }[k]
        arms = (poses.pop("back"), poses.pop("front"))
        p = Pose().body(**poses)
        p.arms(*arms, upper=9, fore=9)
        starburst(p.canvas, *p.at(15, 26))
        if k == 4:
            # speed lines behind the dash
            for dy, ln in ((6, 9), (14, 12), (22, 10), (30, 8)):
                for x in range(ln):
                    put(p.canvas, 4 + x, p.at(0, dy)[1], (200, 224, 240))
        return p.canvas
    finally:
        BODY, BACK_ARM, FRONT_ARM, SKIN, SKIN_SHADE, SKIN_LIGHT, SLEEVE, SLEEVE_SHADE = saved


def effects():
    out = {}
    for k in range(2):
        out[f"cigarette_{k}"] = cigarette_shot(k)
        out[f"icedcup_{k}"] = latte_shot(k)
        out[f"spill_{k}"] = spill(k)
    for k in range(4):
        out[f"glitch_{k}"] = glitch_field(k)
    for k in range(9):
        out[f"agent_{k}"] = agent_pose(k)
    return out


# ------------------------------------------------------------------------------------------------ icons, portrait
OUTLINE_ICON = (12, 12, 17)


def icon(top, bottom, border, draw):
    img = blank(24, 24)
    for y in range(24):
        t = y / 23
        col = tuple(round(top[i] + (bottom[i] - top[i]) * t) for i in range(3))
        for x in range(24):
            put(img, x, y, col)
    for y in range(24):
        for x in range(24):
            if 10 <= x + y <= 20:
                r, g, b, _ = img.getpixel((x, y))
                put(img, x, y, (r * 85 // 100, g * 85 // 100, b * 85 // 100))
    for k in range(24):
        for x, y in ((k, 0), (k, 23), (0, k), (23, k)):
            put(img, x, y, OUTLINE_ICON)
        for x, y in ((k, 1), (1, k)):
            if 1 <= k <= 22:
                put(img, x, y, border)
    draw(img)
    return img


def outlined(img, pixels, colour):
    pixels = set(pixels)
    for x, y in pixels:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if (x + dx, y + dy) not in pixels:
                put(img, x + dx, y + dy, OUTLINE_ICON)
    for x, y in pixels:
        put(img, x, y, colour)


def icons():
    def light(img):
        # a cigarette flying left to right with its ember and a trail of sparks
        body = [(x, 12 + (x - 4) // 6) for x in range(4, 17)]
        outlined(img, body, PAPER)
        for x, y in body[:3]:
            put(img, x, y, FILTER)
        for dx, dy, c in ((17, 14, EMBER), (18, 14, EMBER_HOT), (19, 15, EMBER_HOT), (17, 13, EMBER), (20, 16, EMBER)):
            put(img, dx, dy, c)
        for x, y in ((3, 8), (5, 6), (7, 9), (2, 17), (6, 18)):
            put(img, x, y, EMBER_HOT)

    def heavy(img):
        # an open laptop whose screen is glitching: green code torn up by cyan and magenta blocks
        screen = [(x, y) for x in range(5, 19) for y in range(4, 14)]
        outlined(img, screen, (14, 36, 26))
        for y in (6, 8, 10, 12):
            for x in range(7, 7 + 4 + (y % 3) * 3):
                put(img, x, y, GLOW)
        for x0, y0, w, col in ((8, 5, 7, (255, 0, 190)), (6, 9, 9, (0, 255, 240)), (12, 11, 5, (255, 0, 190)), (7, 7, 3, (255, 255, 255))):
            for x in range(x0, x0 + w):
                put(img, x, y0, col)
        base = [(x, y) for x in range(2, 22) for y in range(15, 18)]
        outlined(img, base, SILVER)
        for x in range(3, 21):
            put(img, x, 15, SILVER_LIGHT)

    def special(img):
        # the starburst on its own: a dozen thin rays of uneven length, thicker near the middle
        lengths = [9, 6, 8, 5, 9, 6, 8, 5, 9, 6, 8, 5]
        cx, cy = 11.5, 11.5
        for k, ln in enumerate(lengths):
            a = -math.pi / 2 + k * 2 * math.pi / len(lengths)
            nx, ny = -math.sin(a), math.cos(a)
            for t in range(4, ln * 2 + 1):
                r = t / 2
                for off in ((-0.5, 0.5) if r < 3.6 else (0.0,)):
                    x, y = round(cx + math.cos(a) * r + nx * off), round(cy + math.sin(a) * r + ny * off)
                    put(img, x, y, BURST_LIGHT if r > ln * 0.6 else BURST)
        for x in range(8, 16):
            for y in range(8, 16):
                d = math.hypot(x - cx, y - cy)
                if d <= 3.4:
                    put(img, x, y, BURST_LIGHT if d < 1.8 else BURST)

    return {
        "light": icon((150, 70, 40), (70, 24, 12), (236, 150, 90), light),
        "heavy": icon((34, 70, 56), (12, 30, 24), (110, 220, 160), heavy),
        "special": icon((74, 46, 36), (30, 18, 16), (236, 150, 110), special),
    }


def portrait():
    """His head and shoulders from the drawing, doubled to 40 x 40 on a dark ground, with the cigarette in his mouth."""
    src = SRC.crop((7, 0, 27, 20)).resize((40, 40), Image.NEAREST)
    out = Image.new("RGBA", (40, 40), (44, 52, 40, 255))
    for y in range(40):
        for x in range(40):
            v = round(44 + 14 * (1 - y / 39))
            out.putpixel((x, y), (v, v + 8, v - 4, 255))
    out.alpha_composite(src)
    # the lit cigarette (the mouth is at about x 16..19, y 13 of the drawing)
    for k, col in enumerate((FILTER, FILTER, PAPER, PAPER, PAPER)):
        for dy in (0, 1):
            put(out, (19 + k - 7) * 2, (13 - 0) * 2 + dy, col)
            put(out, (19 + k - 7) * 2 + 1, (13 - 0) * 2 + dy, col)
    for dy in (0, 1):
        put(out, (24 - 7) * 2, 26 + dy, EMBER)
        put(out, (24 - 7) * 2 + 1, 26 + dy, EMBER)
        put(out, (25 - 7) * 2, 26 + dy, EMBER_HOT)
        put(out, (25 - 7) * 2 + 1, 26 + dy, EMBER_HOT)
    return out


def main():
    SPRITES.mkdir(parents=True, exist_ok=True)
    ICONS.mkdir(parents=True, exist_ok=True)
    frames = build()
    for name, p in frames.items():
        p.canvas.save(SPRITES / f"{name}.png")
        p.mark().save(SPRITES / f"{name}_mark.png")
    for name, img in effects().items():
        img.save(SPRITES / f"{name}.png")
    for name, img in icons().items():
        img.save(ICONS / f"{name}.png")
    portrait().save(PORTRAIT)
    print(f"{len(frames)} poses, effects, icons and the portrait written")
    if "--sheet" in sys.argv:
        sheet = Path(sys.argv[sys.argv.index("--sheet") + 1])
        names = sorted(frames)
        cols, s = 10, 3
        rows = (len(names) + cols - 1) // cols + 1
        img = Image.new("RGBA", (cols * W * s, rows * (H * s + 12)), (90, 120, 150, 255))
        draw = ImageDraw.Draw(img)
        for k, name in enumerate(names):
            x, y = (k % cols) * W * s, (k // cols) * (H * s + 12)
            img.alpha_composite(frames[name].canvas.resize((W * s, H * s), Image.NEAREST), (x, y + 12))
            draw.text((x + 2, y), name, fill=(255, 255, 255, 255))
        y = (rows - 1) * (H * s + 12) + 12
        x = 0
        extras = list(effects().items()) + [(f"icon {n}", i) for n, i in icons().items()]
        for name, e in extras:
            img.alpha_composite(e.resize((e.width * s, e.height * s), Image.NEAREST), (x, y))
            x += e.width * s + 8
        img.save(sheet)


if __name__ == "__main__":
    main()
