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
TEAL = (39, 56, 61)
TEAL_LIGHT = (81, 94, 98)
TEAL_DARK = (15, 27, 30)
CLAW = (120, 20, 30)
MOUTH = (98, 16, 16)
FANG = (250, 250, 250)
# the closed-eye line, half way from the dark brown (60, 38, 33) to white
LID = (158, 147, 144)
SLASH = (236, 226, 255)
SLASH_EDGE = (170, 150, 220)
BAT = (28, 22, 36)
BAT_LIGHT = (62, 50, 78)
BAT_EYE = (230, 40, 40)

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


def split_arms(img):
    """The drawing without its forearms and hands, and the pixels of each, (x, y) -> colour. Where the back hand
    covered the hair and dress, the hole is filled with the dress."""
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
    hand(img, (wrist[0] + (wrist[0] - elbow[0] > 0) - (wrist[0] - elbow[0] < 0), wrist[1]), (wrist[0] - elbow[0], wrist[1] - elbow[1]), claws)
    return elbow, wrist


def slash(img, at, direction=(1, 1), length=8):
    """Three claw marks in the air, parallel, running along `direction`."""
    dx, dy = direction
    n = math.hypot(dx, dy)
    dx, dy = dx / n, dy / n
    x, y = at
    for k in (-3, 0, 3):
        a = (round(x - dy * k), round(y + dx * k))
        b = (round(a[0] + dx * length), round(a[1] + dy * length))
        line(img, a, b, SLASH_EDGE, 2)
        line(img, a, b, SLASH, 1)


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

    def arms(self, back=None, front=None, back_bend=-1, front_bend=1, **kw):
        if back == "rest":
            for (x, y), col in BACK_ARM.items():
                put(self.canvas, x + OFF_X + self.up[0], y + OFF_Y + self.up[1], col)
        elif back:
            arm(self.canvas, self.shoulder("back"), self.at(*back), bend=back_bend, **kw)
        if front == "rest":
            for (x, y), col in FRONT_ARM.items():
                put(self.canvas, x + OFF_X + self.up[0], y + OFF_Y + self.up[1], col)
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

    # walking: the boots swap, she dips as they pass, the front hand swings
    for k in range(6):
        a = 2 * math.pi * k / 6
        s_, c_ = math.sin(a), math.cos(a)
        boots = ((round(3 * s_), -int(c_ > 0.4)), (-round(3 * s_), -int(c_ < -0.4)))
        pose(f"walk_{k}", up=(0, int(abs(c_) < 0.5)), boots=boots).arms("rest", (29 + round(4 * s_), 33 - int(abs(c_) > 0.5)))

    # nail scratch: front hand cocks and rakes, then the back hand
    p = pose("punch_0"); p.arms("rest", (22, 20))
    p = pose("punch_1", up=(1, 0)); p.arms("rest", (40, 24), upper=10, fore=10); slash(p.canvas, p.at(41, 15), (1, 2))
    p = pose("punch_2"); p.arms((15, 17), "rest", back_bend=1)
    p = pose("punch_3", up=(1, 0)); p.arms((36, 23), "rest", back_bend=1, upper=12, fore=13); slash(p.canvas, p.at(37, 14), (1, 2))

    # the fang frenzy (the eyeless face): claws up, lunge, both hands rake, recover
    p = pose("hard_0", eyeless=True); p.arms((4, 6), (30, 6)); p.face("fangs")
    p = pose("hard_1", up=(1, 0), eyeless=True); p.arms((22, 8), (36, 10), back_bend=1, upper=12, fore=12); p.face("fangs")
    p = pose("hard_2", up=(2, 1), eyeless=True); p.arms((34, 27), (41, 24), back_bend=1, upper=12, fore=13); p.face("fangs")
    slash(p.canvas, p.at(38, 13), (1, 2), 10); slash(p.canvas, p.at(43, 17), (1, 2), 8)
    p = pose("hard_3", eyeless=True); p.arms("rest", (30, 30)); p.face("open")

    # generic light and air lights
    p = pose("soft_0"); p.arms("rest", (23, 22))
    p = pose("soft_1", up=(1, 0)); p.arms("rest", (40, 25), upper=10, fore=10); slash(p.canvas, p.at(41, 16), (1, 2))
    p = pose("air_soft_0"); p.arms("rest", (26, 16))
    p = pose("air_soft_1"); p.arms("rest", (38, 31), upper=10, fore=10); slash(p.canvas, p.at(36, 26), (1, 2))
    p = pose("air_hard_0", eyeless=True); p.arms((4, 6), (30, 6)); p.face("fangs")
    p = pose("air_hard_1", eyeless=True); p.arms((30, 32), (38, 30), back_bend=1, upper=12, fore=12); p.face("fangs"); slash(p.canvas, p.at(36, 24), (1, 2), 10)

    # crouching
    pose("crouch", drop=CROUCH).arms("rest", "rest")
    p = pose("crouch_soft_0", drop=CROUCH); p.arms("rest", (24, 26))
    p = pose("crouch_soft_1", drop=CROUCH); p.arms("rest", (40, 32), upper=10, fore=10); slash(p.canvas, p.at(38, 26), (1, 1))
    p = pose("crouch_hard_0", drop=CROUCH); p.arms((8, 28), (30, 28))
    p = pose("crouch_hard_1", drop=CROUCH); p.arms("rest", (38, 10), upper=10, fore=10); slash(p.canvas, p.at(36, 2), (1, -1))

    # in the air
    pose("jump_up").arms((2, 9), (31, 9))
    pose("jump_top").arms((-8, 20), (42, 20), upper=10, fore=10)
    pose("jump_fall").arms((4, 5), (30, 5))

    # the bat wave: arms rise, spread high (bats gather at her hands), thrust forward
    pose("summon_0").arms((5, 30), (30, 30))
    pose("summon_1").arms((-2, 18), (37, 18), upper=10, fore=10)
    p = pose("summon_2", eyeless=True); p.arms((3, 3), (30, 3)); p.face("open")
    p = pose("summon_3", eyeless=True); p.arms((-3, 1), (36, 1), upper=10, fore=11); p.face("open")
    for bx, by, upw in ((-6, -4, True), (40, -5, False), (0, -9, False), (34, -10, True)):
        bat(p.canvas, *p.at(bx, by), up=upw)
    p = pose("summon_4", up=(1, 0)); p.arms((6, 6), (43, 17), upper=10, fore=11)
    bat(p.canvas, *p.at(47, 12)); bat(p.canvas, *p.at(52, 18), up=False)
    pose("summon_5").arms("rest", (40, 26), upper=10, fore=10)

    # speaking
    pose("speak_0").arms("rest", (27, 25)).face("open")
    pose("speak_1").arms("rest", (31, 17)).face("open")

    # hurt, blocking, down
    pose("hurt_0", up=(-2, 0)).arms((-4, 28), (36, 22), upper=10, fore=10).face("hurt")
    pose("hurt_1", up=(-3, 1)).arms((4, 31), (27, 31)).face("closed")
    pose("block", up=(0, 1)).arms((22, 12), (25, 9), back_bend=1, upper=10, fore=11)
    p = pose("ko"); p.arms((-3, 32), (32, 35), upper=10, fore=10); p.face("closed"); rotated(p, 90)

    # the portal: step back, leap back into it, out behind the enemy with fangs bared and claws gripping, recover
    pose("portal_0", up=(-2, 1)).arms("rest", (33, 22))
    p = Pose().body(up=(-2, 0)); p.arms((-2, 12), (34, 14), upper=10, fore=10)
    frames["portal_1"] = rotated(p, 22, lift=6)
    p = pose("portal_2", up=(3, 0), eyeless=True); p.arms((34, 24), (40, 21), back_bend=1, upper=12, fore=13); p.face("fangs")
    pose("portal_3").arms("rest", (31, 30)).face("open")

    # rolling: tuck, turn over as a ball, come up
    pose("roll_0", drop=CROUCH).arms("rest", "rest")
    for k in range(1, 9):
        base = Pose().body(drop=CROUCH, up=(0, -2)).arms("rest", "rest")
        frames[f"roll_{k}"] = rotated(base, -45 * k, scale=0.8, lift=2 if k in (3, 4, 5, 6) else 0)
    pose("roll_9", drop=CROUCH[:7]).arms("rest", "rest")
    return frames


def effects():
    """Her bats (the heavy's wave, two drawings), the portal (two drawings) and the hand that grabs."""
    out = {}
    for k in range(2):
        img = blank(36, 26)
        spots = [(6, 6), (16, 3), (27, 7), (10, 14), (22, 13), (31, 17), (5, 21), (17, 22), (27, 23)]
        for n, (x, y) in enumerate(spots):
            bat(img, x, y + (1 if (n + k) % 2 else 0), up=(n + k) % 2 == 0)
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
            pts = [(8 + k * 4 + t // 2, 8 + t) for t in range(12)]
            outlined(img, pts, SLASH)
            for x, y in pts[-3:]:
                put(img, x, y, (230, 60, 70))

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

    return {
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
