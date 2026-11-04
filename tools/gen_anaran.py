"""Redraws every Anaran pose from the single idle drawing in tools/anaran_new.png.

    python tools/gen_anaran.py assets/sprites/anaran [--sheet contact_sheet.png]

The drawing is cut into a body (head, hair, torso, cape, skirt, boots) and two arms. Each pose moves the
body's bands, draws the arms again in the drawing's own colours between a shoulder and a hand, and adds the
staff or scroll. `<frame>_mark.png` is the ruby at the throat, put wherever the neck ended up.
"""

import math
import sys
from pathlib import Path

from PIL import Image

W, H = 84, 63
# the drawing is 38 x 56 with its feet on row 54; the game wants the body centre at x = 33 and the feet on the last row
OFF_X, OFF_Y = 14, 8

SKIN = (251, 205, 174)
SKIN_SHADE = (229, 174, 137)
SKIN_LIGHT = (242, 184, 146)
SLEEVE = (179, 61, 61)
SLEEVE_SHADE = (145, 49, 50)
SLEEVE_DARK = (112, 28, 36)
BRACER = (235, 168, 83)
BRACER_SHADE = (186, 110, 62)
BRACER_LIGHT = (255, 195, 119)
OUTLINE = (34, 34, 38)
EYE_DARK = (48, 8, 22)
GEM = (214, 84, 76)
STAFF = (138, 78, 44)
STAFF_LIGHT = (186, 110, 62)
ORB = (255, 214, 120)
PAPER = (240, 224, 196)
PAPER_SHADE = (205, 180, 150)

ARM_COLOURS = {SKIN, SKIN_SHADE, SKIN_LIGHT, (221, 163, 126), (209, 150, 112), (232, 171, 131),
               BRACER, BRACER_SHADE, BRACER_LIGHT, (234, 166, 104), (187, 133, 64), STAFF, GEM}
HAIR_COLOURS = {(160, 44, 70), (112, 22, 46), (48, 8, 22), (70, 12, 32)}

src = Image.open(Path(__file__).with_name("anaran_new.png")).convert("RGBA")


def px(img, x, y):
    return img.getpixel((x, y))


def split_arms(img):
    """The drawing without its arms, and the pixels of each arm (x, y) -> colour."""
    body = img.copy()
    back, front = {}, {}
    for y in range(img.height):
        for x in range(img.width):
            c = px(img, x, y)
            if c[3] == 0:
                continue
            rgb = c[:3]
            take = None
            # the back arm: the fist and cuff left of the hair
            if 3 <= x <= 11 and 21 <= y <= 31 and rgb not in HAIR_COLOURS:
                if rgb in ARM_COLOURS or rgb in (SLEEVE_DARK, SLEEVE_SHADE, SLEEVE) and y <= 24:
                    take = back
            # the front arm: sleeve, forearm, bracer and hand right of the cape
            if x >= 26 and 21 <= y <= 37:
                if rgb in ARM_COLOURS or rgb in (SLEEVE_DARK, SLEEVE_SHADE, SLEEVE):
                    take = front
            if take is not None:
                take[(x, y)] = c
                body.putpixel((x, y), (0, 0, 0, 0))
    # where the back arm hung inside the silhouette (body on both sides of it) fill the gap with the nearest colour to the
    # right, so a pose that moves the arm away doesn't leave a hole
    def body_left(y):
        """The leftmost body pixel in the rows round `y` that aren't part of the arm."""
        xs = [xx for yy in range(max(0, y - 4), min(img.height, y + 5)) for xx in range(img.width) if body.getpixel((xx, yy))[3] == 255 and (xx, yy) not in back]
        return min(xs) if xs else None

    for (x, y) in sorted(back):
        right = [body.getpixel((xx, y)) for xx in range(x + 1, img.width) if body.getpixel((xx, y))[3] == 255]
        edge = body_left(y)
        if right and edge is not None and x >= edge:
            body.putpixel((x, y), right[0])
    return body, back, front


BODY, BACK_ARM, FRONT_ARM = split_arms(src)


def blank():
    return Image.new("RGBA", (W, H), (0, 0, 0, 0))


def paste_rows(canvas, body, rows, dx=0, dy=0, flip_boots=False):
    """Copy the body's rows `rows` (a range of drawing rows) to the canvas, shifted."""
    for y in rows:
        for x in range(body.width):
            c = px(body, x, y)
            if c[3]:
                cx, cy = x + OFF_X + dx, y + OFF_Y + dy
                if 0 <= cx < W and 0 <= cy < H:
                    canvas.putpixel((cx, cy), c)


def put(canvas, x, y, c):
    if 0 <= x < W and 0 <= y < H:
        canvas.putpixel((x, y), c + (255,) if len(c) == 3 else c)


def line(canvas, a, b, colour, width=1):
    """A thick line of single-colour pixels (Bresenham, then widened)."""
    (x0, y0), (x1, y1) = a, b
    steps = max(abs(x1 - x0), abs(y1 - y0), 1)
    for k in range(steps + 1):
        t = k / steps
        x, y = round(x0 + (x1 - x0) * t), round(y0 + (y1 - y0) * t)
        for ox in range(-(width // 2), width - width // 2):
            for oy in range(-(width // 2), width - width // 2):
                put(canvas, x + ox, y + oy, colour)


def two_bone(shoulder, hand, upper, fore, bend):
    """Elbow position for an arm from `shoulder` to `hand`; `bend` = +1 or -1 picks which way the elbow points."""
    (sx, sy), (hx, hy) = shoulder, hand
    dx, dy = hx - sx, hy - sy
    dist = max(math.hypot(dx, dy), 0.001)
    dist_c = min(dist, upper + fore - 0.01)
    a = (upper * upper - fore * fore + dist_c * dist_c) / (2 * dist_c)
    h = math.sqrt(max(upper * upper - a * a, 0))
    ux, uy = dx / dist, dy / dist
    ex = sx + ux * a - uy * h * bend
    ey = sy + uy * a + ux * h * bend
    if dist > upper + fore - 0.01:
        # out of reach: the arm is straight and the hand stays as far as it can go
        hx, hy = sx + ux * (upper + fore - 0.01), sy + uy * (upper + fore - 0.01)
        ex, ey = sx + ux * upper, sy + uy * upper
    return (round(ex), round(ey)), (round(hx), round(hy))


def fist(canvas, c, size=5, up=False):
    """A clenched hand centred on `c`: skin with a shaded underside and a light knuckle."""
    cx, cy = c
    r = size // 2
    for y in range(cy - r, cy + r + 1):
        for x in range(cx - r, cx + r + 1):
            corner = abs(x - cx) == r and abs(y - cy) == r
            if corner:
                continue
            col = SKIN
            if y >= cy + r - (0 if size < 5 else 1):
                col = SKIN_SHADE
            if x == cx + r and y == cy - r + 1:
                col = SKIN_LIGHT
            put(canvas, x, y, col)


def arm(canvas, shoulder, hand, bend=1, upper=8, fore=9, sleeve=True, hand_size=5):
    """Draw one arm: the sleeve down to the elbow, the forearm with its gold bracer, then the fist."""
    elbow, hand = two_bone(shoulder, hand, upper, fore, bend)
    # forearm first so the sleeve covers the elbow
    line(canvas, elbow, hand, SKIN, 3)
    # shade the side of the forearm that faces down
    line(canvas, (elbow[0], elbow[1] + 1), (hand[0], hand[1] + 1), SKIN_SHADE, 1)
    # the bracer sits about two thirds down the forearm
    bx, by = round(elbow[0] + (hand[0] - elbow[0]) * 0.62), round(elbow[1] + (hand[1] - elbow[1]) * 0.62)
    bx2, by2 = round(elbow[0] + (hand[0] - elbow[0]) * 0.80), round(elbow[1] + (hand[1] - elbow[1]) * 0.80)
    line(canvas, (bx, by), (bx2, by2), BRACER, 3)
    line(canvas, (bx, by + 1), (bx2, by2 + 1), BRACER_SHADE, 1)
    fist(canvas, hand, hand_size)
    if sleeve:
        line(canvas, shoulder, elbow, SLEEVE, 4)
        line(canvas, (shoulder[0], shoulder[1] + 1), (elbow[0], elbow[1] + 1), SLEEVE_SHADE, 1)
        put(canvas, shoulder[0] - 1, shoulder[1] - 1, SLEEVE_DARK)
    return elbow, hand


def stamp_arm(canvas, pixels, dx=0, dy=0):
    """Put one of the original arms back (used for the resting pose)."""
    for (x, y), c in pixels.items():
        put(canvas, x + OFF_X + dx, y + OFF_Y + dy, c)


# shoulders in drawing coordinates (the front arm is the one towards the enemy, on the right)
BACK_SHOULDER = (7, 23)
FRONT_SHOULDER = (27, 22)


def canvas_pt(p, dx=0, dy=0):
    return (p[0] + OFF_X + dx, p[1] + OFF_Y + dy)


class Pose:
    """One drawing: where the bands of the body are and what the arms and props do."""

    def __init__(self):
        self.canvas = blank()

    def body(self, up=(0, 0), skirt=(0, 0), boots=((0, 0), (0, 0)), drop=()):
        """Lay out the body's three bands. `drop` lists drawing rows to cut out (a crouch): the head and chest
        sink by the number of rows cut below them while the feet stay on the floor."""
        c = self.canvas
        drop = set(drop)

        def below(r):
            return sum(1 for d in drop if d > r)

        # the arms hang from the chest, so they sink with it
        self.up = (up[0], up[1] + below(22))
        # boots first, then the skirt, then the chest over both
        for y in [*range(48, 55), *range(32, 48), *range(0, 32)]:
            if y in drop:
                continue
            band_dx, band_dy = (up if y < 32 else skirt if y < 48 else (0, 0))
            for x in range(BODY.width):
                p = px(BODY, x, y)
                if not p[3]:
                    continue
                dx, dy = band_dx, band_dy
                if y >= 48:
                    dx, dy = boots[0] if x < 19 else boots[1]
                put(c, x + OFF_X + dx, y + OFF_Y + dy + below(y), p)
        return self

    def face(self, kind):
        c = self.canvas
        ox, oy = OFF_X + self.up[0], OFF_Y + self.up[1]
        if kind in ("closed", "hurt"):
            # eyes shut: a short dark line where the lids meet, as wide as the open eye (two pixels)
            for x in (16, 17, 20, 21):
                put(c, x + ox, 9 + oy, SKIN)
                put(c, x + ox, 10 + oy, SKIN)
            for x in (16, 17, 20, 21):
                put(c, x + ox, 10 + oy, EYE_DARK)
        if kind in ("open", "hurt"):
            for x in (17, 18, 19):
                for y in (12, 13):
                    put(c, x + ox, y + oy, EYE_DARK)
            put(c, 18 + ox, 13 + oy, (186, 80, 80))
        return self

    def shoulder(self, which):
        sx, sy = BACK_SHOULDER if which == "back" else FRONT_SHOULDER
        return canvas_pt((sx, sy), self.up[0], self.up[1])

    def mark(self):
        """The ruby at the throat, on a transparent drawing of its own."""
        if getattr(self, "mark_img", None) is not None:
            return self.mark_img
        m = blank()
        x, y = 19 + OFF_X + self.up[0], 18 + OFF_Y + self.up[1]
        for dx, dy in ((0, -1), (-1, 0), (0, 0), (1, 0), (0, 1)):
            put(m, x + dx, y + dy, (255, 255, 255))
        return m


def staff_line(canvas, a, b):
    """A staff from `a` to the orb at `b`."""
    line(canvas, a, b, STAFF, 2)
    line(canvas, (a[0] + 1, a[1]), (b[0] + 1, b[1]), STAFF_LIGHT, 1)
    ox, oy = b
    for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
        put(canvas, ox + dx, oy + dy, ORB)
    put(canvas, ox, oy - 2, ORB)
    put(canvas, ox + 2, oy, (255, 240, 200))


def scroll(canvas, x0, y0, x1, y1, glow=False):
    """A scroll as a rectangle of paper with rolled ends; with `glow` its writing burns gold and glints rise off it."""
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(canvas, x, y, PAPER if y < y1 else PAPER_SHADE)
    for y in range(y0 - 1, y1 + 2):
        put(canvas, x0 - 1, y, BRACER_SHADE)
        put(canvas, x1 + 1, y, BRACER_SHADE)
    ink = ORB if glow else (150, 110, 90)
    for row in range(y0 + 1, y1, 2):
        for x in range(x0 + 2, x1 - 1, 3):
            put(canvas, x, row, ink)
            if glow and (x + row) % 2:
                put(canvas, x + 1, row, (255, 240, 200))
    if glow:
        for k, x in enumerate(range(x0 + 1, x1, 4)):
            put(canvas, x, y0 - 3 - (k % 3), (255, 240, 200))
            put(canvas, x + 1, y0 - 5 - (k % 2) * 2, ORB)


def resting(p, back=True, front=True):
    """The arms exactly as drawn (fist at the chest, the other hand low)."""
    if back:
        stamp_arm(p.canvas, BACK_ARM, *p.up)
    if front:
        stamp_arm(p.canvas, FRONT_ARM, *p.up)


def save(p, out, name):
    p.canvas.save(out / f"{name}.png")
    p.mark().save(out / f"{name}_mark.png")


def at(p, x, y):
    """Canvas position of drawing coordinates (x, y), moved with the chest."""
    return (x + OFF_X + p.up[0], y + OFF_Y + p.up[1])


def arms(p, back=None, front=None, back_bend=-1, front_bend=1, **kw):
    """Draw the arms to the given hands (drawing coordinates); the back arm goes first so the front one overlaps it."""
    # "rest" puts the arm back as drawn, which reads better than a redrawn arm at the same spot
    if back == "rest":
        stamp_arm(p.canvas, BACK_ARM, *p.up)
    elif back:
        arm(p.canvas, p.shoulder("back"), at(p, *back), bend=back_bend, **kw)
    if front == "rest":
        stamp_arm(p.canvas, FRONT_ARM, *p.up)
    elif front:
        arm(p.canvas, p.shoulder("front"), at(p, *front), bend=front_bend, **kw)


def rotated(p, angle, scale=1.0, lift=0):
    """Turn a finished pose (and its ruby) about the middle of the body and stand it on the floor again."""
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
    result = []
    for img in out:
        canvas = blank()
        canvas.paste(img, (left, top))
        result.append(canvas)
    p.canvas, p.mark_img = result
    return p


# rows cut out of the drawing for a crouch: four of the chest and ten of the skirt
CROUCH = tuple(range(25, 29)) + tuple(range(36, 46))


def tilt(p, degrees):
    """Pitch a finished pose (and its ruby) about the feet: positive leans him back, negative forward."""
    pivot = (33, H - 1)
    mark = p.mark()
    p.canvas = p.canvas.rotate(degrees, resample=Image.NEAREST, center=pivot)
    p.mark_img = mark.rotate(degrees, resample=Image.NEAREST, center=pivot)
    return p


def glints(canvas, at_, n=6, r=4, seed=0):
    """Gold glints scattered round a point (the orb charging)."""
    x, y = at_
    for k in range(n):
        a = (k * 2.4 + seed) % (2 * math.pi)
        d = r * (0.6 + 0.4 * ((k * 37 + seed * 11) % 10) / 10)
        gx, gy = round(x + math.cos(a) * d), round(y + math.sin(a) * d)
        put(canvas, gx, gy, (255, 240, 200))
        if k % 2 == 0:
            put(canvas, gx + 1, gy, ORB)


def floor_circle(p, cx, strength):
    """A gold summoning circle flat on the floor round his feet (an ellipse with runes on its rim): its far half
    behind him, its near half in front. `strength` 0..1 is how far it has kindled; at full strength sparks rise."""
    cy = H - 3
    rx, ry = 20, 3.2
    steps = int(96 * strength)
    behind = blank()
    canvas = p.canvas
    for k in range(steps):
        a = 2 * math.pi * k / 96
        x, y = round(cx + math.cos(a) * rx), round(cy + math.sin(a) * ry)
        layer = behind if math.sin(a) < 0 else canvas
        put(layer, x, y, ORB if k % 6 else (255, 240, 200))
        if k % 12 == 0:
            put(layer, x, y - 1, BRACER_LIGHT)
    behind.alpha_composite(canvas)
    p.canvas = canvas = behind
    if strength >= 1:
        for k, x in enumerate(range(cx - 16, cx + 17, 5)):
            for j in range(1 + k % 3):
                put(canvas, x, cy - 4 - j * 3 - (k % 2) * 2, (255, 240, 200) if j == 0 else ORB)


def swoosh(canvas, centre, radius, a0, a1):
    """The gold trail of a swinging hand: an arc round `centre` from angle `a0` to `a1` (degrees, 0 = forward,
    negative = up), faint where it starts and bright and thick where the hand is now."""
    cx, cy = centre
    steps = int(abs(a1 - a0) * radius / 25) + 8
    for lane in (-1, 0, 1):
        for k in range(steps + 1):
            f = k / steps
            if lane and f < 0.4:
                continue
            a = math.radians(a0 + (a1 - a0) * f)
            put(canvas, round(cx + math.cos(a) * (radius + lane)), round(cy + math.sin(a) * (radius + lane)),
                BRACER_SHADE if f < 0.5 else ORB if f < 0.85 else (255, 240, 200))


def plunge_lines(canvas, x0, x1, top, bottom):
    """Speed streaks above a body diving down."""
    for k, x in enumerate(range(x0, x1, 4)):
        for y in range(top + (k % 3) * 2, bottom - (k % 2) * 3):
            if (y + k) % 5:
                put(canvas, x, y, (255, 240, 200) if y > bottom - 4 else BRACER_LIGHT)


def impact(canvas, at_):
    """The orb striking the floor: a gold flash with rays, chips flying, and cracks running out along the ground."""
    x, y = at_
    for dx, dy, length in ((-1, -1, 5), (1, -1, 5), (0, -1, 6), (-1, 0, 6), (1, 0, 6), (-2, -1, 4), (2, -1, 4)):
        for k in range(2, length):
            put(canvas, x + dx * k, y + dy * k, (255, 240, 200) if k < 4 else ORB)
    for dx in range(-9, 10):
        if abs(dx) > 2 and dx % 3 != 0:
            put(canvas, x + dx, y + 1 + (abs(dx) % 2), (60, 40, 30))
    for cx, cy in ((-6, -5), (5, -6), (-3, -8), (8, -3)):
        put(canvas, x + cx, y + cy, BRACER_SHADE)


def build():
    frames = {}

    def pose(name, **kw):
        p = Pose().body(**kw)
        frames[name] = p
        return p

    # standing, breathing
    p = pose("idle_0"); resting(p)
    p = pose("idle_1", up=(0, 1)); resting(p)

    # walking: a long stride with the passing foot lifted, the body dips as the feet pass, the front hand swings
    for k in range(6):
        a = 2 * math.pi * k / 6
        s_, c_ = math.sin(a), math.cos(a)
        boots = ((round(4 * s_), -2 * int(c_ > 0.4)), (-round(4 * s_), -2 * int(c_ < -0.4)))
        p = pose(f"walk_{k}", up=(0, int(abs(c_) < 0.5)), boots=boots)
        arms(p, back="rest", front=(31 + round(5 * s_), 33 - int(abs(c_) > 0.5)))

    # crouching
    p = pose("crouch", drop=CROUCH); resting(p)
    p = pose("crouch_soft_0", drop=CROUCH); arms(p, back="rest", front=(25, 28))
    p = pose("crouch_soft_1", drop=CROUCH); arms(p, back="rest", front=(42, 27), upper=10, fore=11)
    p = pose("crouch_hard_0", drop=CROUCH); arms(p, back=(10, 31), front=(30, 31))
    p = pose("crouch_hard_1", drop=CROUCH); arms(p, back="rest", front=(39, 12), upper=10, fore=11)

    # in the air
    p = pose("jump_up"); arms(p, back=(2, 11), front=(33, 11))
    p = pose("jump_top"); arms(p, back=(-9, 21), front=(44, 21), upper=10, fore=11)
    p = pose("jump_fall"); arms(p, back=(4, 7), front=(32, 7))

    # hurt, blocking, down
    p = pose("hurt_0", up=(-2, 0)); arms(p, back=(-5, 29), front=(38, 22), upper=10, fore=10); p.face("hurt")
    p = pose("hurt_1", up=(-3, 1)); arms(p, back=(3, 32), front=(28, 31)); p.face("closed")
    p = pose("block", up=(0, 1)); arms(p, back=(23, 13), front=(25, 10), upper=10, fore=11)

    # light attacks (a quick jab with the front hand)
    p = pose("soft_0"); arms(p, back="rest", front=(24, 27))
    p = pose("soft_1", up=(1, 0)); arms(p, back="rest", front=(43, 24), upper=10, fore=11)
    # the air whip: the hand raised high, then lashed down and out with a gold trail
    p = pose("air_soft_0", up=(-1, 0)); arms(p, back=(2, 24), front=(31, -3), upper=10, fore=11)
    p = pose("air_soft_1", up=(1, 0)); arms(p, back=(2, 20), front=(42, 31), upper=10, fore=11)
    swoosh(p.canvas, at(p, 30, 22), 13, -85, 25)
    for k, (back, front) in enumerate([("rest", (30, 32)), ("rest", (41, 25)), ((22, 23), (30, 33)), ("rest", (44, 28))]):
        p = pose(f"punch_{k}"); arms(p, back=back, front=front, upper=10, fore=11)

    # speaking
    p = pose("speak_0"); arms(p, back="rest", front=(30, 24)); p.face("open")
    p = pose("speak_1"); arms(p, back="rest", front=(34, 16)); p.face("open")

    # the scroll: rolled, unrolled and held out in front, read, rolled up again
    # (he snaps it open, holds it out at arm's length with the writing burning gold as he reads, rolls it shut)
    p = pose("read_0", up=(-1, 0)); arms(p, back=(16, 22), front=(30, 18)); scroll(p.canvas, *at(p, 18, 17), *at(p, 28, 20))
    p = pose("read_1", up=(1, 0)); arms(p, back=(20, 27), front=(44, 25), upper=10, fore=11); scroll(p.canvas, *at(p, 21, 20), *at(p, 43, 26), glow=True)
    p = pose("read_2", up=(1, 1)); arms(p, back=(20, 27), front=(44, 25), upper=10, fore=11); scroll(p.canvas, *at(p, 21, 20), *at(p, 43, 26), glow=True); p.face("open")
    glints(p.canvas, at(p, 32, 14), 6, 6, 2)
    p = pose("read_3"); arms(p, back=(13, 29), front=(30, 28)); scroll(p.canvas, *at(p, 14, 26), *at(p, 24, 29))

    # the staff: crouched with it held across him in both hands; heaved up over his head (the orb glinting); the
    # slam, low, both hands driving the orb into the floor in a burst of gold; rising with it planted
    p = pose("staff_0", up=(-1, 0), drop=CROUCH[:4]); arms(p, back=(14, 32), front=(30, 24))
    staff_line(p.canvas, at(p, 6, 40), at(p, 38, 16)); fist(p.canvas, at(p, 14, 32)); fist(p.canvas, at(p, 30, 24))
    p = pose("staff_1", up=(-1, 0)); arms(p, back=(14, 2), front=(26, 0), upper=10, fore=11)
    staff_line(p.canvas, at(p, 4, 8), at(p, 38, -6)); fist(p.canvas, at(p, 14, 2)); fist(p.canvas, at(p, 26, 0))
    glints(p.canvas, at(p, 38, -6), 7, 5, 1); p.face("open")
    p = pose("staff_2", up=(2, 0), drop=CROUCH); arms(p, back=(30, 34), front=(38, 32), back_bend=1, upper=10, fore=11)
    # (the orb is placed on the floor itself, not relative to the sunken chest)
    floor = (OFF_X + 50, H - 3)
    staff_line(p.canvas, at(p, 26, 22), floor); fist(p.canvas, at(p, 30, 34)); fist(p.canvas, at(p, 38, 32))
    impact(p.canvas, floor); p.face("open")
    p = pose("staff_3"); arms(p, back="rest", front=(34, 28)); staff_line(p.canvas, at(p, 34, 54), at(p, 35, 14)); fist(p.canvas, at(p, 34, 28))

    # airborne heavy: the staff over the head, then driven down
    # the plunge: the staff raised overhead in both hands, then driven down at the enemy with the orb blazing and
    # streaks above him as he drops
    p = pose("air_hard_0"); arms(p, back=(14, 0), front=(26, -1), upper=10, fore=11)
    staff_line(p.canvas, at(p, 2, 6), at(p, 40, -6)); fist(p.canvas, at(p, 14, 0)); fist(p.canvas, at(p, 26, -1))
    glints(p.canvas, at(p, 40, -6), 6, 4, 7); p.face("open")
    p = pose("air_hard_1", up=(1, 1)); arms(p, back=(30, 28), front=(37, 30), back_bend=1, upper=10, fore=10)
    staff_line(p.canvas, at(p, 30, 18), at(p, 48, 46)); fist(p.canvas, at(p, 30, 28)); fist(p.canvas, at(p, 37, 30))
    glints(p.canvas, at(p, 48, 46), 8, 5, 8)
    plunge_lines(p.canvas, OFF_X + 2, OFF_X + 36, 0, 8); p.face("open")
    heavy = [((31, 8), ((31, 36), (32, -3))), ((38, 15), ((33, 34), (47, 2))), ((44, 24), None), ((34, 28), ((34, 54), (35, 14)))]
    for k, (front, tip) in enumerate(heavy):
        p = pose(f"hard_{k}"); arms(p, back="rest", front=front, upper=10, fore=11)
        if tip:
            staff_line(p.canvas, at(p, *tip[0]), at(p, *tip[1])); fist(p.canvas, at(p, *front))

    # summoning: the arms come up, cross over the head, then throw forward
    # (a gold circle kindles on the floor round him as the arms rise, blazes while they are crossed overhead,
    # and he flings the goblins out with a burst of glints at his hand)
    centre = OFF_X + 19
    p = pose("summon_0"); floor_circle(p, centre, 0.25); arms(p, back=(4, 31), front=(33, 31))
    p = pose("summon_1"); floor_circle(p, centre, 0.6); arms(p, back=(-4, 19), front=(41, 19), upper=10, fore=11)
    p = pose("summon_2"); floor_circle(p, centre, 1.0); arms(p, back=(3, 6), front=(32, 6))
    glints(p.canvas, at(p, 3, 4), 5, 4, 3); glints(p.canvas, at(p, 32, 4), 5, 4, 4)
    p = pose("summon_3"); floor_circle(p, centre, 1.0); arms(p, back=(16, -2), front=(22, -2), upper=12, fore=14)
    glints(p.canvas, at(p, 19, -4), 8, 6, 5); p.face("open")
    p = pose("summon_4", up=(2, 0)); floor_circle(p, centre, 1.0); arms(p, back=(8, 6), front=(45, 21), upper=10, fore=11)
    glints(p.canvas, at(p, 48, 21), 9, 6, 6); p.face("open")
    p = pose("summon_5"); floor_circle(p, centre, 0.5); arms(p, back=(4, 27), front=(45, 27), upper=10, fore=11)

    # knocked out: lying on his back, head to the left
    p = pose("ko"); arms(p, back=(-3, 33), front=(33, 36), upper=10, fore=10); p.face("closed")
    rotated(p, 90)

    # rolling: tuck, turn over as a ball, come up
    p = pose("roll_0", drop=CROUCH); resting(p)
    for k in range(1, 9):
        base = Pose().body(drop=CROUCH, up=(0, -2)); resting(base)
        frames[f"roll_{k}"] = rotated(base, -45 * k, scale=0.8, lift=2 if k in (3, 4, 5, 6) else 0)
    p = pose("roll_9", drop=CROUCH[:7]); resting(p)
    return frames


def main():
    out = Path(sys.argv[1])
    sheet = None
    if "--sheet" in sys.argv:
        sheet = Path(sys.argv[sys.argv.index("--sheet") + 1])
    frames = build()
    for name, p in frames.items():
        save(p, out, name)
    print(f"{len(frames)} poses written to {out}")
    if sheet:
        from PIL import ImageDraw

        names = sorted(frames)
        cols, s = 10, 3
        rows = (len(names) + cols - 1) // cols
        img = Image.new("RGBA", (cols * W * s, rows * (H * s + 12)), (90, 120, 150, 255))
        draw = ImageDraw.Draw(img)
        for k, name in enumerate(names):
            x, y = (k % cols) * W * s, (k // cols) * (H * s + 12)
            img.alpha_composite(frames[name].canvas.resize((W * s, H * s), Image.NEAREST), (x, y + 12))
            draw.text((x + 2, y), name, fill=(255, 255, 255, 255))
        img.save(sheet)


if __name__ == "__main__":
    main()
