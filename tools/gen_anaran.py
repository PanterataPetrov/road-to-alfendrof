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
            # eyes shut: a dark line where the lids meet
            for x in (15, 16, 19, 20, 21):
                put(c, x + ox, 9 + oy, SKIN)
            for x in (16, 17, 20, 21):
                put(c, x + ox, 10 + oy, SKIN)
            for x in (15, 16, 17, 19, 20, 21):
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


def scroll(canvas, x0, y0, x1, y1):
    """A scroll as a rectangle of paper with rolled ends."""
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            put(canvas, x, y, PAPER if y < y1 else PAPER_SHADE)
    for y in range(y0 - 1, y1 + 2):
        put(canvas, x0 - 1, y, BRACER_SHADE)
        put(canvas, x1 + 1, y, BRACER_SHADE)
    for x in range(x0 + 2, x1 - 1, 3):
        put(canvas, x, y0 + 1, (150, 110, 90))


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


def build():
    frames = {}

    def pose(name, **kw):
        p = Pose().body(**kw)
        frames[name] = p
        return p

    # standing, breathing
    p = pose("idle_0"); resting(p)
    p = pose("idle_1", up=(0, 1)); resting(p)

    # walking: the boots swap, the body dips as the feet pass, the front hand swings
    for k in range(6):
        a = 2 * math.pi * k / 6
        s_, c_ = math.sin(a), math.cos(a)
        boots = ((round(3 * s_), -int(c_ > 0.4)), (-round(3 * s_), -int(c_ < -0.4)))
        p = pose(f"walk_{k}", up=(0, int(abs(c_) < 0.5)), boots=boots)
        arms(p, back="rest", front=(31 + round(4 * s_), 33 - int(abs(c_) > 0.5)))

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
    p = pose("air_soft_0"); arms(p, back="rest", front=(26, 26))
    p = pose("air_soft_1", up=(1, 0)); arms(p, back=(2, 20), front=(42, 31), upper=10, fore=11)
    for k, (back, front) in enumerate([("rest", (30, 32)), ("rest", (41, 25)), ((22, 23), (30, 33)), ("rest", (44, 28))]):
        p = pose(f"punch_{k}"); arms(p, back=back, front=front, upper=10, fore=11)

    # speaking
    p = pose("speak_0"); arms(p, back="rest", front=(30, 24)); p.face("open")
    p = pose("speak_1"); arms(p, back="rest", front=(34, 16)); p.face("open")

    # the scroll: rolled, unrolled and held out in front, read, rolled up again
    p = pose("read_0"); arms(p, back=(13, 29), front=(25, 29)); scroll(p.canvas, *at(p, 14, 26), *at(p, 22, 29))
    p = pose("read_1"); arms(p, back=(14, 28), front=(37, 28)); scroll(p.canvas, *at(p, 15, 22), *at(p, 36, 27))
    p = pose("read_2", up=(0, 1)); arms(p, back=(14, 28), front=(37, 28)); scroll(p.canvas, *at(p, 15, 22), *at(p, 36, 27)); p.face("open")
    p = pose("read_3"); arms(p, back=(13, 29), front=(30, 28)); scroll(p.canvas, *at(p, 14, 26), *at(p, 24, 29))

    # the staff: raised, the top of the swing, the slam, planted
    p = pose("staff_0"); arms(p, back="rest", front=(31, 9)); staff_line(p.canvas, at(p, 31, 34), at(p, 32, -4)); fist(p.canvas, at(p, 31, 9))
    p = pose("staff_1", up=(-1, 0)); arms(p, back="rest", front=(23, 5)); staff_line(p.canvas, at(p, 30, 14), at(p, 12, -4)); fist(p.canvas, at(p, 23, 5))
    p = pose("staff_2", up=(1, 1)); arms(p, back="rest", front=(40, 30), upper=10, fore=11); staff_line(p.canvas, at(p, 40, 54), at(p, 40, 18)); fist(p.canvas, at(p, 40, 30)); p.face("open")
    p = pose("staff_3"); arms(p, back="rest", front=(34, 28)); staff_line(p.canvas, at(p, 34, 54), at(p, 35, 14)); fist(p.canvas, at(p, 34, 28))

    # airborne heavy: the staff over the head, then driven down
    p = pose("air_hard_0"); arms(p, back=(4, 9), front=(31, 9)); staff_line(p.canvas, at(p, 31, 30), at(p, 32, -4)); fist(p.canvas, at(p, 31, 9))
    p = pose("air_hard_1", up=(1, 1)); arms(p, back=(3, 21), front=(37, 30), upper=10, fore=10); staff_line(p.canvas, at(p, 33, 20), at(p, 47, 44)); fist(p.canvas, at(p, 37, 30))
    heavy = [((31, 8), ((31, 36), (32, -3))), ((38, 15), ((33, 34), (47, 2))), ((44, 24), None), ((34, 28), ((34, 54), (35, 14)))]
    for k, (front, tip) in enumerate(heavy):
        p = pose(f"hard_{k}"); arms(p, back="rest", front=front, upper=10, fore=11)
        if tip:
            staff_line(p.canvas, at(p, *tip[0]), at(p, *tip[1])); fist(p.canvas, at(p, *front))

    # summoning: the arms come up, cross over the head, then throw forward
    p = pose("summon_0"); arms(p, back=(4, 31), front=(33, 31))
    p = pose("summon_1"); arms(p, back=(-4, 19), front=(41, 19), upper=10, fore=11)
    p = pose("summon_2"); arms(p, back=(3, 6), front=(32, 6))
    p = pose("summon_3"); arms(p, back=(16, -2), front=(22, -2), upper=12, fore=14)
    p = pose("summon_4"); arms(p, back=(8, 6), front=(45, 21), upper=10, fore=11)
    p = pose("summon_5"); arms(p, back=(4, 27), front=(45, 27), upper=10, fore=11)

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
