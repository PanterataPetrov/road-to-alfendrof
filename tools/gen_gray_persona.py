"""Draws Gray Persona: every pose from tools/gray_persona.png (and the eyeless variant for the fang moves), her bats,
portals and grabbing hand, and her skill icons. (Her portrait, assets/portraits/gray_persona.png, is kept as is.)

    python tools/gen_gray_persona.py [--sheet contact_sheet.png]

The drawing is cut into a body and two arms. Each pose moves the body's bands, draws the arms again in the
drawing's colours (black upper sleeve, teal cuff, pale clawed hand) between a shoulder and a hand, and adds slashes,
fangs or bats. `<frame>_mark.png` is the ruby brooch at her throat, wherever the neck ended up.
"""

import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SPRITES = ROOT / "assets/sprites/gray_persona"
ICONS = ROOT / "assets/icons/gray_persona"

W, H = 84, 63
# the drawing is 33 x 55 with its feet on row 54; the game wants the body centre at x = 33 and the feet on the last row
OFF_X, OFF_Y = 17, 8

SKIN = (238, 210, 172)
SKIN_SHADE = (212, 186, 152)
SKIN_LIGHT = (255, 235, 208)
FACE = (193, 170, 141)
OUTLINE = (12, 12, 17)
DRESS = (37, 37, 52)
DRESS_DARK = (21, 21, 29)
DRESS_LIGHT = (58, 58, 78)
TEAL = (39, 56, 61)
TEAL_LIGHT = (81, 94, 98)
TEAL_DARK = (15, 27, 30)
CLAW = (120, 20, 30)
MOUTH = (98, 16, 16)
FANG = (250, 250, 250)
# the closed-eye line, half way from the dark brown (60, 38, 33) to white
LID = (158, 147, 144)
SCRATCH_TAIL = (140, 24, 44)
SCRATCH = (226, 110, 130)
SCRATCH_TIP = (255, 232, 236)
BAT = (28, 22, 36)
BAT_LIGHT = (62, 50, 78)
BAT_EYE = (255, 60, 50)
BAT_NEAR, BAT_NEAR_RIM = (26, 18, 34), (140, 50, 150)
BAT_FAR, BAT_FAR_RIM = (40, 30, 52), (80, 40, 96)

TEAL_SET = {TEAL, TEAL_LIGHT, TEAL_DARK, (91, 115, 122)}
SKIN_SET = {SKIN, SKIN_SHADE, SKIN_LIGHT, (229, 182, 153), (224, 196, 158), (23, 12, 10)}

here = Path(__file__).resolve().parent
SRC = Image.open(here / "gray_persona.png").convert("RGBA")
SRC_EYELESS = Image.open(here / "gray_persona_eyeless.png").convert("RGBA")


def blank(w=W, h=H):
    return Image.new("RGBA", (w, h), (0, 0, 0, 0))


def put(img, x, y, c):
    if 0 <= x < img.width and 0 <= y < img.height:
        img.putpixel((x, y), tuple(c) + (255,) if len(c) == 3 else tuple(c))


def is_sleeve(rgb):
    """The black of the puffed sleeves (and their outline)."""
    return sum(rgb) < 130 and rgb not in TEAL_SET and rgb not in SKIN_SET


def split_arms(img):
    """The drawing without its arms (puffed sleeves, cuffs and hands), and the pixels of each, (x, y) -> colour.
    Where the back hand covered the hair and dress, the hole is filled with the dress."""
    body = img.copy()
    back, front = {}, {}
    for y in range(img.height):
        for x in range(img.width):
            c = img.getpixel((x, y))
            if c[3] == 0:
                continue
            rgb = c[:3]
            arm_colour = rgb in TEAL_SET or rgb in SKIN_SET
            if x <= 12 and 20 <= y <= 31 and arm_colour:
                back[(x, y)] = c
            elif x >= 25 and 22 <= y <= 36 and arm_colour:
                front[(x, y)] = c
            # the puffed upper sleeves, out to the sides of the bodice
            elif x <= 7 and 20 <= y <= 29 and is_sleeve(rgb):
                back[(x, y)] = c
            elif x >= 26 and 19 <= y <= 28 and is_sleeve(rgb):
                front[(x, y)] = c
            else:
                continue
            body.putpixel((x, y), (0, 0, 0, 0))
    for (x, y) in back:
        # inside the silhouette (body on both sides): fill with the nearest body colour to the right
        left = any(body.getpixel((xx, y))[3] == 255 for xx in range(0, x))
        right = [body.getpixel((xx, y)) for xx in range(x + 1, img.width) if body.getpixel((xx, y))[3] == 255]
        if left and right:
            body.putpixel((x, y), right[0])
    return body, back, front


BODY, BACK_ARM, FRONT_ARM = split_arms(SRC)
BODY_EYELESS, _, _ = split_arms(SRC_EYELESS)

BACK_SHOULDER = (10, 19)
FRONT_SHOULDER = (25, 19)


def line(img, a, b, colour, width=1):
    (x0, y0), (x1, y1) = a, b
    steps = max(abs(x1 - x0), abs(y1 - y0), 1)
    for k in range(steps + 1):
        t = k / steps
        x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
        for ox in range(-(width // 2), width - width // 2):
            for oy in range(-(width // 2), width - width // 2):
                put(img, x + ox, y + oy, colour)


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


def hand(img, c, toward, claws=True):
    """A pale hand at `c` with dark red claws pointing along `toward` (a direction)."""
    cx, cy = c
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            put(img, cx + dx, cy + dy, SKIN_SHADE if dy == 1 else SKIN)
    put(img, cx, cy - 1, SKIN_LIGHT)
    if claws:
        tx, ty = toward
        n = max(math.hypot(tx, ty), 0.001)
        tx, ty = tx / n, ty / n
        # three fingers, each ending in a claw
        for side in (-1, 0, 1):
            fx, fy = cx + tx * 2 - ty * side, cy + ty * 2 + tx * side
            put(img, round(fx), round(fy), SKIN)
            put(img, round(fx + tx), round(fy + ty), CLAW)


def puff(img, c, r):
    """A round puff of black sleeve: dark outline, dress black, a lighter top and a shadowed underside."""
    cx, cy = c
    for y in range(int(cy - r - 2), int(cy + r + 3)):
        for x in range(int(cx - r - 2), int(cx + r + 3)):
            d = math.hypot(x - cx, y - cy)
            if d <= r - 0.6:
                col = DRESS_LIGHT if y < cy - r * 0.6 else DRESS_DARK if y > cy + r * 0.3 else DRESS
                put(img, x, y, col)
            elif d <= r + 0.4:
                put(img, x, y, OUTLINE)


def arm(img, shoulder, target, bend=1, upper=9, fore=9, claws=True):
    """One arm: the black upper sleeve, the teal cuff flaring at the wrist, the clawed hand."""
    elbow, wrist = two_bone(shoulder, target, upper, fore, bend)
    line(img, elbow, wrist, TEAL, 3)
    line(img, (elbow[0], elbow[1] - 1), (wrist[0], wrist[1] - 1), TEAL_LIGHT, 1)
    line(img, (elbow[0], elbow[1] + 1), (wrist[0], wrist[1] + 1), TEAL_DARK, 1)
    # the cuff flares over the last part of the forearm
    fx, fy = round(elbow[0] + (wrist[0] - elbow[0]) * 0.7), round(elbow[1] + (wrist[1] - elbow[1]) * 0.7)
    line(img, (fx, fy), wrist, TEAL, 4)
    line(img, shoulder, elbow, DRESS, 4)
    line(img, (shoulder[0], shoulder[1] + 2), (elbow[0], elbow[1] + 2), DRESS_DARK, 1)
    # the sleeve is puffed: a round bulge at the shoulder and a smaller one just above the elbow, kept round
    # whichever way the arm points
    ux, uy = elbow[0] - shoulder[0], elbow[1] - shoulder[1]
    n = max(math.hypot(ux, uy), 0.001)
    ux, uy = ux / n, uy / n
    puff(img, (shoulder[0] + ux * 2, shoulder[1] + uy * 2), 2.5)
    puff(img, (elbow[0] - ux * 1.2, elbow[1] - uy * 1.2), 1.8)
    hand(img, (wrist[0] + (wrist[0] - elbow[0] > 0) - (wrist[0] - elbow[0] < 0), wrist[1]), (wrist[0] - elbow[0], wrist[1] - elbow[1]), claws)
    return elbow, wrist


def slash(img, at, direction=(1, 1), length=8):
    """The swipe of three fingernails in the air: thin curved streaks along `direction`, fading from a dark red
    tail to a bright, sharp tip."""
    dx, dy = direction
    n = math.hypot(dx, dy)
    dx, dy = dx / n, dy / n
    x, y = at
    steps = length * 2
    for k in (-2, 0, 2):
        for t in range(steps + 1):
            f = t / steps
            # bowed sideways in the middle, like the arc of a swiping hand
            bow = math.sin(f * math.pi) * 1.5
            px_ = x - dy * (k + bow) + dx * length * f
            py_ = y + dx * (k + bow) + dy * length * f
            col = SCRATCH_TAIL if f < 0.4 else SCRATCH if f < 0.85 else SCRATCH_TIP
            put(img, round(px_), round(py_), col)


def bat(img, x, y, up=True, eye=True):
    """A little bat, 9 x 4, centred near (x, y); wings up or down."""
    rows = ["X.......X", "XX.X.X.XX", ".XXXXXXX.", "..XX.XX.."] if up else ["...X.X...", ".XXXXXXX.", "XXXXXXXXX", "X.X...X.X"]
    for r, row in enumerate(rows):
        for c, ch in enumerate(row):
            if ch == "X":
                put(img, x - 4 + c, y - 2 + r, BAT_LIGHT if r == 0 else BAT)
    if eye:
        put(img, x - 1, y, BAT_EYE)
        put(img, x + 1, y, BAT_EYE)


BIG_BAT_UP = ["X...........X", "XX.........XX", "XXX..X.X..XXX", ".XXXXXXXXXXX.", "..XXX.X.XXX..", "...X.....X..."]
BIG_BAT_DOWN = [".....X.X.....", "....XXXXX....", ".XXXXXXXXXXX.", "XXXXX.X.XXXXX", "XX.X.....X.XX", "X...........X"]
BAT_RIM = (110, 40, 120)


def big_bat(img, x, y, up=True):
    """A bigger bat, 13 x 6: dark body, a purple rim along the top of the wings, two red eyes."""
    rows = BIG_BAT_UP if up else BIG_BAT_DOWN
    pts = {(x - 6 + c, y - 3 + r) for r, row in enumerate(rows) for c, ch in enumerate(row) if ch == "X"}
    for (px_, py_) in pts:
        col = BAT_RIM if (px_, py_ - 1) not in pts else BAT_LIGHT if (px_, py_ - 2) not in pts else BAT
        put(img, px_, py_, col)
    head_y = y - 1 if up else y
    put(img, x - 1, head_y, BAT_EYE)
    put(img, x + 1, head_y, BAT_EYE)


def flying_bat(img, cx, cy, span, lift, far=False):
    """A bat of any wingspan (9 to 18 pixels look right): wings from level to raised (`lift` -1..1) with a scalloped
    edge and a purple rim, a round body, pointed ears and red eyes (glowing on the near ones; a far one is darker)."""
    half = span / 2
    draw = ImageDraw.Draw(img)
    BAT, BAT_RIM = (BAT_FAR, BAT_FAR_RIM) if far else (BAT_NEAR, BAT_NEAR_RIM)
    body_r = max(1.5, span / 9)
    for side in (-1, 1):
        tip = (cx + side * half, cy - lift * span * 0.35)
        elbow = (cx + side * half * 0.55, cy - 1 - lift * span * 0.25)
        edge = [tip, (cx + side * half * 0.8, cy + span * 0.1 - lift * span * 0.15),
                (cx + side * half * 0.62, cy + span * 0.02 - lift * span * 0.1),
                (cx + side * half * 0.4, cy + span * 0.14), (cx + side * body_r, cy + body_r)]
        draw.polygon([(round(x), round(y)) for x, y in [(cx + side * body_r * 0.5, cy - body_r), elbow] + edge], fill=BAT + (255,))
        line(img, (round(cx + side * body_r * 0.5), round(cy - body_r)), (round(elbow[0]), round(elbow[1])), BAT_RIM)
        line(img, (round(elbow[0]), round(elbow[1])), (round(tip[0]), round(tip[1])), BAT_RIM)
    r = body_r
    for y in range(int(cy - r - 2), int(cy + r + 2)):
        for x in range(int(cx - r - 1), int(cx + r + 2)):
            if ((x - cx) / (r + 0.3)) ** 2 + ((y - cy) / (r + 0.8)) ** 2 <= 1:
                put(img, x, y, BAT)
    top = round(cy - r - 1)
    put(img, round(cx - r * 0.6), top - 1, BAT_RIM)
    put(img, round(cx + r * 0.6), top - 1, BAT_RIM)
    ex, ey = round(cx) - (1 if span >= 12 else 0), round(cy - r * 0.3)
    put(img, ex, ey, BAT_EYE)
    put(img, round(cx) + 1, ey, BAT_EYE)
    if not far:
        # the glow round the eyes
        for gx, gy in ((ex - 1, ey), (round(cx) + 2, ey), (ex, ey - 1), (round(cx) + 1, ey - 1)):
            if 0 <= gx < img.width and 0 <= gy < img.height and img.getpixel((gx, gy))[:3] in (BAT, BAT_RIM):
                put(img, gx, gy, (150, 20, 30))


def tilt(p, degrees):
    """Pitch a finished pose (and its brooch) about her feet: positive leans her back, negative forward."""
    pivot = (33, H - 1)
    p.canvas = p.canvas.rotate(degrees, resample=Image.NEAREST, center=pivot)
    p.mark_img = p.mark().rotate(degrees, resample=Image.NEAREST, center=pivot)
    return p


def swoosh(img, centre, radius, a0, a1, claws=False):
    """The trail of a swinging hand: an arc round `centre` from angle `a0` to `a1` (degrees, 0 = forward,
    negative = up), thin and dark where it starts, bright and wide where the hand is now. With `claws`, three
    parallel streaks: the fingernails raking through the air."""
    cx, cy = centre
    lanes = (-3, 0, 3) if claws else (-1, 0, 1)
    steps = int(abs(a1 - a0) * radius / 25) + 8
    for lane in lanes:
        r = radius + lane
        for k in range(steps + 1):
            f = k / steps
            a = math.radians(a0 + (a1 - a0) * f)
            x, y = round(cx + math.cos(a) * r), round(cy + math.sin(a) * r)
            if claws:
                col = SCRATCH_TAIL if f < 0.35 else SCRATCH if f < 0.85 else SCRATCH_TIP
            else:
                if f < 0.3 and lane != 0:
                    continue
                col = (120, 110, 150) if f < 0.5 else (190, 180, 220) if f < 0.85 else (245, 240, 255)
            put(img, x, y, col)


def sigil(img, centre, ry, bright=False):
    """A crimson ring of runes seen edge-on (a tall ellipse) in front of her hands, with ticks round it like
    letters and a glowing core."""
    cx, cy = centre
    rx = max(2.5, ry * 0.38)
    ring = (255, 90, 100) if bright else (200, 30, 50)
    inner = (120, 10, 30)
    for k in range(72):
        a = 2 * math.pi * k / 72
        x, y = round(cx + math.cos(a) * rx), round(cy + math.sin(a) * ry)
        put(img, x, y, ring)
        # runes: little ticks outside the ring, every so often
        if k % 9 == 0:
            put(img, round(cx + math.cos(a) * (rx + 2)), round(cy + math.sin(a) * (ry + 2)), (255, 170, 160) if bright else ring)
    for k in range(48):
        a = 2 * math.pi * k / 48
        put(img, round(cx + math.cos(a) * rx * 0.55), round(cy + math.sin(a) * ry * 0.6), inner)
    if bright:
        for dy in range(-2, 3):
            put(img, round(cx), round(cy) + dy, (255, 220, 220))


def mist(img, cx, cy, rx, ry, colour, alpha, seed=0):
    """A soft puff of see-through colour, speckled at its edge."""
    for y in range(int(cy - ry), int(cy + ry) + 1):
        for x in range(int(cx - rx), int(cx + rx) + 1):
            d = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2
            if d > 1 or (d > 0.6 and (x * 7 + y * 13 + seed) % 3 == 0):
                continue
            if 0 <= x < img.width and 0 <= y < img.height and img.getpixel((x, y))[3] == 0:
                img.putpixel((x, y), colour + (int(alpha * (1 - d * 0.6)),))


def blood(img, at, n=3):
    """A few red drops falling from `at`."""
    x, y = at
    for k in range(n):
        put(img, x + (k % 2), y + k * 2, (170, 16, 24))
        put(img, x + (k % 2), y + k * 2 + 1, (120, 8, 16))


def sparks(img, around, colour, n=6, r=5, seed=0):
    """Little glints scattered around a point."""
    x, y = around
    for k in range(n):
        a = (k * 2.4 + seed) % (2 * math.pi)
        d = r * (0.6 + 0.4 * ((k * 37 + seed * 11) % 10) / 10)
        put(img, round(x + math.cos(a) * d), round(y + math.sin(a) * d), colour)


class Pose:
    def __init__(self):
        self.canvas = blank()
        self.up = (0, 0)
        self.mark_img = None

    def body(self, up=(0, 0), skirt=(0, 0), boots=((0, 0), (0, 0)), drop=(), eyeless=False):
        src = BODY_EYELESS if eyeless else BODY
        drop = set(drop)

        def below(r):
            return sum(1 for d in drop if d > r)

        self.up = (up[0], up[1] + below(20))
        for y in [*range(47, 55), *range(30, 47), *range(0, 30)]:
            if y in drop:
                continue
            bdx, bdy = up if y < 30 else skirt if y < 47 else (0, 0)
            for x in range(src.width):
                p = src.getpixel((x, y))
                if not p[3]:
                    continue
                dx, dy = bdx, bdy
                if y >= 47:
                    dx, dy = boots[0] if x < 16 else boots[1]
                cx, cy = x + OFF_X + dx, y + OFF_Y + dy + below(y)
                if 0 <= cx < W and 0 <= cy < H:
                    if p[3] < 255 and self.canvas.getpixel((cx, cy))[3]:
                        continue
                    self.canvas.putpixel((cx, cy), p)
        return self

    def at(self, x, y):
        return (x + OFF_X + self.up[0], y + OFF_Y + self.up[1])

    def shoulder(self, which):
        return self.at(*(BACK_SHOULDER if which == "back" else FRONT_SHOULDER))

    def face(self, kind):
        c = self.canvas
        ox, oy = OFF_X + self.up[0], OFF_Y + self.up[1]
        if kind in ("closed", "hurt"):
            for x in (16, 17, 20, 21):
                put(c, x + ox, 9 + oy, FACE)
                put(c, x + ox, 10 + oy, LID)
        if kind in ("open", "hurt"):
            # a small open mouth, where the lips are
            for x in (18, 19):
                put(c, x + ox, 13 + oy, MOUTH)
        if kind == "fangs":
            # open with two vampire fangs at its corners
            for x in range(17, 21):
                put(c, x + ox, 13 + oy, MOUTH)
            for x in (18, 19):
                put(c, x + ox, 14 + oy, MOUTH)
            for x in (17, 20):
                put(c, x + ox, 13 + oy, FANG)
        return self

    def arms(self, back=None, front=None, back_bend=-1, front_bend=1, back_shift=(0, 0), front_shift=(0, 0), **kw):
        if back == "rest":
            for (x, y), col in BACK_ARM.items():
                put(self.canvas, x + OFF_X + self.up[0] + back_shift[0], y + OFF_Y + self.up[1] + back_shift[1], col)
        elif back:
            arm(self.canvas, self.shoulder("back"), self.at(*back), bend=back_bend, **kw)
        if front == "rest":
            for (x, y), col in FRONT_ARM.items():
                put(self.canvas, x + OFF_X + self.up[0] + front_shift[0], y + OFF_Y + self.up[1] + front_shift[1], col)
        elif front:
            arm(self.canvas, self.shoulder("front"), self.at(*front), bend=front_bend, **kw)
        return self

    def mark(self):
        """The ruby brooch at her throat."""
        if self.mark_img is not None:
            return self.mark_img
        m = blank()
        for x, y in ((17, 17), (18, 17), (17, 18), (18, 18), (19, 17)):
            put(m, x + OFF_X + self.up[0], y + OFF_Y + self.up[1], (255, 255, 255))
        return m


def rotated(p, angle, scale=1.0, lift=0):
    """Turn a finished pose and its brooch about the middle and stand it on the floor again."""
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


# rows cut out for a crouch: four of the chest, ten of the skirt
CROUCH = tuple(range(22, 26)) + tuple(range(34, 44))


def build():
    frames = {}

    def pose(name, **kw):
        p = Pose().body(**kw)
        frames[name] = p
        return p

    # standing, breathing
    pose("idle_0").arms("rest", "rest")
    pose("idle_1", up=(0, 1)).arms("rest", "rest")

    # walking: the boots swap, she dips as they pass, the skirt sways and both arms swing against the legs
    for k in range(6):
        a = 2 * math.pi * k / 6
        s_, c_ = math.sin(a), math.cos(a)
        boots = ((round(3 * s_), -int(c_ > 0.4)), (-round(3 * s_), -int(c_ < -0.4)))
        p = pose(f"walk_{k}", up=(0, int(abs(c_) < 0.5)), boots=boots)
        # the arm opposite the forward boot comes forward, rising a pixel at the front of its swing
        p.arms("rest", "rest", back_shift=(-round(2 * s_), -int(-s_ > 0.6)), front_shift=(round(2 * s_), -int(s_ > 0.6)))

    # nail scratch: front hand cocks and rakes, then the back hand
    # (she leans back as the hand cocks, the swing carries her forward, and on the strike she is stepping in,
    # front boot out, body tipped over it; coming back she settles)
    # right hand: cocked high behind her head as she rears back; sweeping over the top; the rake down and out
    # with her whole body pitched over the front boot and a claw arc across the swing; pulling back
    p = pose("punch_0", up=(-2, 0), boots=((-2, 0), (0, 0))); p.arms((6, 27), (14, 2), front_bend=-1, upper=10, fore=10)
    tilt(p, 7)
    p = pose("punch_mid_0", up=(1, 0), boots=((0, 0), (2, -1))); p.arms((4, 28), (36, 4), upper=10, fore=11)
    swoosh(p.canvas, p.at(22, 10), 15, -150, -70)
    tilt(p, -4)
    p = pose("punch_1", up=(3, 1), boots=((-1, 0), (5, -1))); p.arms((1, 25), (44, 30), upper=11, fore=12)
    swoosh(p.canvas, p.at(17, 25), 27, -112, 22, claws=True)
    tilt(p, -12)
    p = pose("punch_mid_1", up=(2, 0), boots=((0, 0), (3, 0))); p.arms((4, 27), (34, 33))
    tilt(p, -6)
    # left hand: the same, from the other side
    p = pose("punch_2", up=(-2, 0), boots=((-2, 0), (0, 0))); p.arms((2, 0), (28, 31), back_bend=1, upper=10, fore=10)
    tilt(p, 7)
    p = pose("punch_mid_2", up=(1, 0), boots=((0, 0), (2, -1))); p.arms((30, 3), (27, 32), back_bend=1, upper=11, fore=12)
    swoosh(p.canvas, p.at(16, 10), 15, -150, -70)
    tilt(p, -4)
    p = pose("punch_3", up=(3, 1), boots=((-1, 0), (5, -1))); p.arms((42, 30), (25, 32), back_bend=1, upper=13, fore=14)
    swoosh(p.canvas, p.at(15, 25), 27, -112, 22, claws=True)
    tilt(p, -12)
    p = pose("punch_mid_3", up=(2, 0), boots=((0, 0), (3, 0))); p.arms((30, 32), (27, 32), back_bend=1, upper=11, fore=11)
    tilt(p, -6)

    # the fang frenzy (the eyeless face): claws up, lunge, both hands rake, recover
    p = pose("hard_0", eyeless=True); p.arms((4, 6), (30, 6)); p.face("fangs")
    p = pose("hard_1", up=(1, 0), eyeless=True); p.arms((22, 8), (36, 10), back_bend=1, upper=12, fore=12); p.face("fangs")
    p = pose("hard_2", up=(2, 1), eyeless=True); p.arms((34, 27), (41, 24), back_bend=1, upper=12, fore=13); p.face("fangs")
    slash(p.canvas, p.at(32, 15), (3, 1), 13); slash(p.canvas, p.at(36, 22), (3, 1), 11)
    p = pose("hard_3", eyeless=True); p.arms("rest", (30, 30)); p.face("open")

    # generic light and air lights
    p = pose("soft_0"); p.arms("rest", (23, 22))
    p = pose("soft_1", up=(1, 0)); p.arms("rest", (40, 25), upper=10, fore=10); slash(p.canvas, p.at(34, 18), (3, 1), 12)
    p = pose("air_soft_0"); p.arms("rest", (26, 16))
    p = pose("air_soft_1"); p.arms("rest", (38, 31), upper=10, fore=10); slash(p.canvas, p.at(31, 25), (3, 1), 12)
    p = pose("air_hard_0", eyeless=True); p.arms((4, 6), (30, 6)); p.face("fangs")
    p = pose("air_hard_1", eyeless=True); p.arms((30, 32), (38, 30), back_bend=1, upper=12, fore=12); p.face("fangs"); slash(p.canvas, p.at(36, 24), (1, 2), 10)

    # crouching
    pose("crouch", drop=CROUCH).arms("rest", "rest")
    p = pose("crouch_soft_0", drop=CROUCH); p.arms("rest", (24, 26))
    p = pose("crouch_soft_1", drop=CROUCH); p.arms("rest", (40, 32), upper=10, fore=10); slash(p.canvas, p.at(33, 26), (3, 1), 12)
    p = pose("crouch_hard_0", drop=CROUCH); p.arms((8, 28), (30, 28))
    p = pose("crouch_hard_1", drop=CROUCH); p.arms("rest", (38, 10), upper=10, fore=10); slash(p.canvas, p.at(36, 2), (1, -1))

    # in the air
    pose("jump_up").arms((2, 9), (31, 9))
    pose("jump_top").arms((-8, 20), (42, 20), upper=10, fore=10)
    pose("jump_fall").arms((4, 5), (30, 5))

    # the bat wave: she pushes off and hops back (the hop itself is the move's), arms swept back, then up in the air
    # gathering bats, then both hands thrust at the enemy and the swarm bursts out of them
    p = pose("summon_0", up=(-1, 2), drop=CROUCH[:4]); p.arms((-2, 28), (22, 30))
    p = pose("summon_1", up=(-2, 0), boots=((-1, -2), (1, -1))); p.arms((-6, 16), (24, 14), upper=10, fore=10)
    p = pose("summon_2", eyeless=True, boots=((-1, -2), (1, -1))); p.arms((3, 3), (30, 3)); p.face("open")
    sparks(p.canvas, p.at(3, 1), (220, 40, 60), 7, 5, 1); sparks(p.canvas, p.at(30, 1), (220, 40, 60), 7, 5, 2)
    flying_bat(p.canvas, *p.at(5, -1), 9, 0.8); flying_bat(p.canvas, *p.at(29, -2), 10, -0.4)
    p = pose("summon_3", up=(1, 0), eyeless=True); p.arms((30, 20), (38, 18), back_bend=1, upper=11, fore=12); p.face("open")
    sigil(p.canvas, p.at(44, 19), 9)
    sparks(p.canvas, p.at(44, 19), (220, 40, 60), 9, 11, 3)
    flying_bat(p.canvas, *p.at(44, 12), 10, 0.6); flying_bat(p.canvas, *p.at(46, 24), 9, -0.6)
    p = pose("summon_4", up=(2, 0), eyeless=True); p.arms((33, 20), (42, 18), back_bend=1, upper=12, fore=13); p.face("open")
    sigil(p.canvas, p.at(47, 19), 12, bright=True)
    mist(p.canvas, *p.at(53, 18), 8, 8, (90, 20, 60), 110, 1)
    flying_bat(p.canvas, *p.at(49, 11), 14, 0.7); flying_bat(p.canvas, *p.at(55, 22), 12, -0.5); flying_bat(p.canvas, *p.at(47, 26), 9, 0.2)
    pose("summon_5").arms((14, 28), (36, 26), back_bend=1, upper=10, fore=10)

    # speaking
    for k, face in enumerate(("fangs", "open")):
        p = pose(f"speak_{k}", up=(0, k)); p.arms("rest", (21, 14), upper=10, fore=9); p.face(face)
        flying_bat(p.canvas, *p.at(36 + k, 2 - 2 * k), 11, 0.8 if k else -0.6)

    # hurt, blocking, down
    pose("hurt_0", up=(-2, 0)).arms((-4, 28), (36, 22), upper=10, fore=10).face("hurt")
    pose("hurt_1", up=(-3, 1)).arms((4, 31), (27, 31)).face("closed")
    pose("block", up=(0, 1)).arms((22, 12), (25, 9), back_bend=1, upper=10, fore=11)
    p = pose("ko"); p.arms((-3, 32), (32, 35), upper=10, fore=10); p.face("closed"); rotated(p, 90)

    # the portal: crouch, arms up, leap back, dive in; out behind the enemy: lunge with the claws, rear back with
    # the fangs bared, bite (blood), wipe her mouth
    p = pose("portal_0", up=(-2, 2), drop=CROUCH[:4]); p.arms((0, 30), (24, 32))
    p = pose("portal_1", up=(-1, -1), eyeless=True); p.arms((-1, 0), (27, -1), upper=10, fore=10); p.face("open")
    sparks(p.canvas, p.at(-1, -2), (190, 110, 250), 6, 4, 5); sparks(p.canvas, p.at(27, -3), (190, 110, 250), 6, 4, 6)
    p = Pose().body(up=(-2, 0)); p.arms((-2, 12), (34, 14), upper=10, fore=10)
    frames["portal_2"] = rotated(p, 22, lift=6)
    p = Pose().body(up=(-2, 0), drop=CROUCH[:6]); p.arms((-6, 4), (30, 2), upper=10, fore=10)
    frames["portal_3"] = rotated(p, 55, scale=0.85, lift=12)
    p = pose("portal_4", up=(4, 0), eyeless=True); p.arms((36, 22), (44, 19), back_bend=1, upper=12, fore=13); p.face("fangs")
    p = pose("portal_5", up=(-1, 0), eyeless=True); p.arms((34, 24), (40, 22), back_bend=1, upper=12, fore=13); p.face("fangs")
    p = pose("portal_6", up=(5, 1), eyeless=True); p.arms((35, 26), (41, 25), back_bend=1, upper=12, fore=13); p.face("fangs")
    blood(p.canvas, p.at(22, 15)); blood(p.canvas, p.at(25, 12), 2)
    p = pose("portal_7"); p.arms("rest", (22, 15)); p.face("open"); blood(p.canvas, p.at(19, 15), 2)

    # turned into a bat
    for k in range(4):
        frames[f"batform_{k}"] = bat_form(k)

    # her roll is a bat: she ducks into a puff of smoke (the smoke is the game's), flits along as a bat a little
    # lower than the bat form, and comes out of another puff
    pose("roll_0", drop=CROUCH).arms("rest", "rest")
    for k in range(1, 9):
        p = bat_form(k % 4)
        for img_name in ("canvas", "mark_img"):
            img = getattr(p, img_name)
            moved = blank()
            moved.paste(img, (0, 8))
            setattr(p, img_name, moved)
        frames[f"roll_{k}"] = p
    pose("roll_9", drop=CROUCH[:7]).arms("rest", "rest")
    return frames


def bat_form(k):
    """Her as a bat, seen from the side and facing right (the game mirrors it), one of four wingbeat drawings (up,
    level, down, level) at chest height: the head in front with a pointed ear, a red eye, a fang and her ruby at
    the throat, a wisp of her dark hair, the body and the tail membrane behind. The near wing is big, the far one
    darker and a beat ahead. On the downstroke pale gusts curl away under the wings."""
    img = blank()
    mark = blank()
    draw = ImageDraw.Draw(img)
    bob = (0, 1, 2, 1)[k]
    cx, cy = 31, 31 + bob
    shoulder = (cx + 2, cy - 2)

    def wing(phase, colour, rim, bones, shift, size):
        # where the wrist and the tip are for this part of the beat, relative to the shoulder
        wrist = {0: (-3, -10), 1: (-7, -5), 2: (-2, 7), 3: (-7, -2)}[phase]
        tip = {0: (-9, -19), 1: (-17, -8), 2: (-7, 17), 3: (-16, -1)}[phase]
        sx, sy = shoulder[0] + shift[0], shoulder[1] + shift[1]
        w = (sx + wrist[0] * size, sy + wrist[1] * size)
        t = (sx + tip[0] * size, sy + tip[1] * size)
        back = (cx - 5 + shift[0], cy + 1 + shift[1])
        # the trailing edge bellies out between three finger bones, from the tip back to the body, with notches
        # between them; "out" is away from the shoulder
        dx, dy = back[0] - t[0], back[1] - t[1]
        n = max(math.hypot(dx, dy), 0.001)
        nx, ny = dy / n, -dx / n
        if (sx - t[0]) * nx + (sy - t[1]) * ny > 0:
            nx, ny = -nx, -ny
        bulge = 4.0 * size

        def along(f, out):
            return (t[0] + dx * f + nx * out, t[1] + dy * f + ny * out)

        f1, f2 = along(0.33, bulge), along(0.66, bulge)
        n1, n2, n3 = along(0.16, bulge * 0.3), along(0.5, bulge * 0.4), along(0.84, bulge * 0.3)
        outline = [(sx, sy), w, t, n1, f1, n2, f2, n3, back]
        draw.polygon([(round(x), round(y)) for x, y in outline], fill=colour + (255,))
        line(img, (sx, sy), (round(w[0]), round(w[1])), rim)
        line(img, (round(w[0]), round(w[1])), (round(t[0]), round(t[1])), rim)
        for f in (f1, f2):
            line(img, (round(w[0]), round(w[1])), (round(f[0]), round(f[1])), bones)
        put(img, round(w[0]), round(w[1]) - 1, (200, 200, 210))  # the little thumb claw
        return t

    # the far wing first, a beat ahead, smaller and darker
    wing((k + 1) % 4, (44, 34, 58), (80, 44, 96), (60, 46, 76), (2, -1), 0.8)
    # the tail membrane, then the body
    draw.polygon([(cx - 6, cy - 1), (cx - 11, cy + 2), (cx - 8, cy + 4), (cx - 4, cy + 2)], fill=BAT + (255,))
    for y in range(cy - 4, cy + 5):
        for x in range(cx - 6, cx + 6):
            if ((x - cx) / 5.2) ** 2 + ((y - cy) / 3.8) ** 2 <= 1:
                put(img, x, y, BAT_LIGHT if y < cy - 2 else BAT)
    # feet tucked under
    put(img, cx - 3, cy + 4, OUTLINE)
    put(img, cx - 1, cy + 4, OUTLINE)
    # the head: round, a pointed ear (and the far one behind it), a snout
    hx, hy = cx + 6, cy - 2
    for y in range(hy - 3, hy + 4):
        for x in range(hx - 3, hx + 4):
            if (x - hx) ** 2 + (y - hy) ** 2 <= 10:
                put(img, x, y, BAT)
    for x, y in ((hx - 1, hy - 4), (hx - 1, hy - 5), (hx, hy - 4), (hx - 1, hy - 6)):
        put(img, x, y, BAT)
    put(img, hx - 2, hy - 6, BAT_RIM)
    put(img, hx - 3, hy - 4, (60, 46, 76))
    put(img, hx + 4, hy, BAT)
    put(img, hx + 4, hy + 1, BAT)
    put(img, hx + 2, hy - 1, BAT_EYE)
    put(img, hx + 3, hy - 1, (150, 20, 30))
    put(img, hx + 3, hy + 2, FANG)
    # a wisp of her long dark hair streaming back from the head
    for x, y in ((hx - 3, hy - 2), (hx - 4, hy - 2), (hx - 5, hy - 1), (hx - 6, hy - 1), (hx - 7, hy)):
        put(img, x, y, (60, 38, 33))
    # her ruby at the throat
    put(img, hx + 1, hy + 3, (200, 20, 40))
    put(mark, hx + 1, hy + 3, (255, 255, 255))
    put(mark, hx + 2, hy + 3, (255, 255, 255))
    # the near wing over the body
    tip = wing(k, BAT, BAT_RIM, BAT_LIGHT, (0, 0), 1.0)
    # wind: gusts curling away under the wings on the downstroke, fainter as the wings come back up
    if k in (2, 3):
        pale = (225, 232, 245) if k == 2 else (170, 180, 205)
        for n, (gx, gy, length) in enumerate(((round(tip[0]) + 2, round(tip[1]) + 3, 7), (cx - 6, cy + 9 + (k - 2) * 2, 9), (cx + 2, cy + 7 + (k - 2) * 2, 5))):
            for j in range(length):
                # each gust bows down then flicks back up at its end
                y = gy + round(1.5 * math.sin(j / max(length - 1, 1) * math.pi))
                if (j + n) % 4 != 3:
                    put(img, gx - j, y, pale)
    p = Pose()
    p.canvas, p.mark_img = img, mark
    return p


def effects():
    """Her bats (the heavy's wave, two drawings), the portal (two drawings) and the hand that grabs."""
    out = {}
    # (x, y, wingspan, phase): the small ones at the back, the big ones leading
    # (x, y, wingspan, phase), drawn back to front: far ones first
    swarm = [(10, 12, 9, 0.0), (14, 34, 9, 2.1), (20, 22, 10, 4.0), (24, 6, 10, 1.2), (28, 40, 11, 3.3),
             (31, 15, 12, 5.1), (35, 29, 12, 2.6), (41, 8, 13, 0.7), (44, 38, 14, 4.4), (47, 20, 15, 1.9),
             (53, 30, 16, 3.8), (55, 10, 16, 5.6), (61, 22, 18, 0.4), (64, 36, 15, 2.9)]
    for k in range(6):
        img = blank(74, 48)
        # a dark crimson cloud the swarm flies in, thinning out behind
        for n, (x, y, rx, ry) in enumerate(((10, 24, 10, 14), (22, 12, 9, 8), (22, 36, 9, 8), (36, 24, 12, 14), (4, 18, 5, 6))):
            mist(img, x - k % 2, y + (1 if (n + k) % 3 == 0 else 0), rx, ry, (60, 8, 30), 110, k + n)
        t = 2 * math.pi * k / 6
        for x, y, span, ph in swarm:
            bob = round(2.5 * math.sin(t + ph))
            flap = math.sin(2 * t + ph * 1.7)
            far = span < 12
            # speed streaks behind the near ones
            if not far:
                for dy in (-1, 1):
                    for dx in range(3, 3 + span // 2):
                        if (dx + k + int(ph * 3)) % 3:
                            put(img, x - span // 2 - dx, y + bob + dy, (90, 30, 70) if dx < 3 + span // 4 else (60, 16, 46))
            flying_bat(img, x, y + bob, span, flap, far=far)
        out[f"bats_{k}"] = img
    for k in range(2):
        img = blank(24, 44)
        cx, cy = 11.5, 21.5
        for y in range(44):
            for x in range(24):
                d = ((x - cx) / 11.5) ** 2 + ((y - cy) / 21.5) ** 2
                if d > 1:
                    continue
                ang = math.atan2(y - cy, x - cx)
                swirl = (math.sin(ang * 3 + d * 9 + k * math.pi / 3) + 1) / 2
                if d > 0.8:
                    col = (40, 10, 62) if d > 0.92 else (130, 46, 186)
                elif d > 0.55:
                    col = (90, 30, 140) if swirl < 0.6 else (190, 110, 250)
                else:
                    col = (10, 4, 18) if swirl < 0.75 else (60, 20, 96)
                put(img, x, y, col)
        out[f"gate_{k}"] = img
    img = blank(18, 10)
    for y in range(2, 8):
        for x in range(0, 5):
            put(img, x, y, TEAL_LIGHT if y == 2 else TEAL_DARK if y == 7 else TEAL)
    for y in range(3, 7):
        for x in range(5, 10):
            put(img, x, y, SKIN_SHADE if y == 6 else SKIN)
    # spread fingers with claws, reaching right
    for fy, length in ((1, 5), (3, 7), (5, 7), (8, 5)):
        line(img, (9, min(max(fy, 3), 6)), (9 + length - 2, fy), SKIN)
        put(img, 9 + length - 1, fy, CLAW)
    out["hand_0"] = img
    return out


def icon(top, bottom, border, draw):
    """A 24 x 24 skill icon in the game's style: a dark outline, a light border, a shaded background."""
    img = blank(24, 24)
    for y in range(24):
        t = y / 23
        col = tuple(round(top[i] + (bottom[i] - top[i]) * t) for i in range(3))
        for x in range(24):
            put(img, x, y, col)
    # a darker diagonal band, like the others
    for y in range(24):
        for x in range(24):
            if 10 <= x + y <= 20:
                r, g, b, _ = img.getpixel((x, y))
                put(img, x, y, (r * 85 // 100, g * 85 // 100, b * 85 // 100))
    for k in range(24):
        for x, y in ((k, 0), (k, 23), (0, k), (23, k)):
            put(img, x, y, OUTLINE)
        for x, y in ((k, 1), (1, k)):
            if 1 <= k <= 22:
                put(img, x, y, border)
    draw(img)
    return img


def outlined(img, pixels, colour):
    """Draw `pixels` in `colour` with a black outline around them."""
    pixels = set(pixels)
    for x, y in pixels:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if (x + dx, y + dy) not in pixels:
                put(img, x + dx, y + dy, OUTLINE)
    for x, y in pixels:
        put(img, x, y, colour)


def icons():
    def light(img):
        # three claw marks raking down across a pale clawed hand
        hand_px = [(x, y) for x in range(4, 9) for y in range(4, 9)]
        outlined(img, hand_px, SKIN)
        for x, y in ((9, 4), (9, 6), (9, 8)):
            put(img, x, y, CLAW)
        for k in range(3):
            pts = [(8 + k * 4 + t // 2 + round(math.sin(t / 11 * math.pi) * 1.5), 8 + t) for t in range(12)]
            outlined(img, pts, SCRATCH)
            for x, y in pts[:4]:
                put(img, x, y, SCRATCH_TAIL)
            for x, y in pts[-2:]:
                put(img, x, y, SCRATCH_TIP)

    def heavy(img):
        for (x, y), up in (((7, 7), True), ((16, 9), False), ((9, 16), False), ((17, 17), True)):
            for dx, dy in ((-1, -1), (1, -1), (-1, 1), (1, 1), (0, -1), (0, 1), (-1, 0), (1, 0)):
                put(img, x + dx, y + dy + 0, OUTLINE)
            bat(img, x, y, up=up)

    def special(img):
        cx, cy = 11.5, 11.5
        for y in range(2, 22):
            for x in range(2, 22):
                d = ((x - cx) / 9) ** 2 + ((y - cy) / 9.5) ** 2
                if 0.62 < d <= 1:
                    put(img, x, y, (180, 100, 240) if d < 0.82 else (90, 30, 140))
                elif d <= 0.62:
                    put(img, x, y, (12, 4, 20))
        # fangs in the dark
        for x, y in ((9, 10), (9, 11), (9, 12), (14, 10), (14, 11), (14, 12)):
            put(img, x, y, FANG)
        for x in range(8, 16):
            put(img, x, 9, MOUTH)
        put(img, 9, 13, (200, 200, 200))
        put(img, 14, 13, (200, 200, 200))

    def turn_bat(img):
        # one big bat, wings up
        rows = ["X.........X", "XX...X.X..XX", "XXX.XXXXX.XXX", ".XXXXXXXXXXX.", "..XXXXXXXXX..", "...XX.X.XX...", "....X...X...."]
        pts = {(5 + c, 8 + r) for r, row in enumerate(rows) for c, ch in enumerate(row) if ch == "X"}
        outlined(img, pts, BAT)
        for x, y in pts:
            if (x, y - 1) not in pts:
                put(img, x, y, BAT_RIM)
        put(img, 10, 10, BAT_EYE)
        put(img, 12, 10, BAT_EYE)

    def poof(img):
        # a grey cloud with stars over it (the stun)
        for cx, cy, r in ((8, 14, 4.5), (15, 14, 4.5), (11.5, 11, 4.5), (11.5, 16, 4)):
            for y in range(24):
                for x in range(24):
                    d = ((x - cx) ** 2 + (y - cy) ** 2) ** 0.5
                    if d <= r:
                        put(img, x, y, (205, 205, 210) if y < cy - 1 else (150, 150, 158))
        for x, y in ((6, 5), (12, 3), (18, 5)):
            for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
                put(img, x + dx, y + dy, (255, 230, 90))

    return {
        "bat": icon((96, 44, 140), (40, 14, 70), (220, 90, 120), turn_bat),
        "poof": icon((110, 110, 120), (50, 50, 58), (200, 200, 210), poof),
        "light": icon((150, 30, 44), (70, 10, 22), (220, 90, 100), light),
        "heavy": icon((96, 44, 140), (40, 14, 70), (170, 120, 220), heavy),
        "special": icon((28, 70, 80), (8, 26, 32), (110, 170, 180), special),
    }


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
    print(f"{len(frames)} poses, effects and icons written")
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
        # the effects and icons along the bottom
        y = (rows - 1) * (H * s + 12) + 12
        x = 0
        extras = list(effects().items()) + [(f"icon {n}", i) for n, i in icons().items()]
        for name, e in extras:
            img.alpha_composite(e.resize((e.width * s, e.height * s), Image.NEAREST), (x, y))
            draw.text((x, y - 12), name, fill=(255, 255, 255, 255))
            x += e.width * s + 12
        img.save(sheet)


if __name__ == "__main__":
    main()
