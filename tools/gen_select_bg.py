"""Pixel-art backdrop for the character select: `python tools/gen_select_bg.py` writes assets/backgrounds/select.png
(384x216): a dithered dusk sky with stars and a crescent, two layers of Istanbul rooftops, domes and minarets, and a
stone floor. Palette colours only, no blending, so it stays crisp when the window scales it."""
import random
from PIL import Image

W, H = 384, 216
rnd = random.Random(7)
img = Image.new("RGB", (W, H))
px = img.load()

# sky: palette steps from night to the glow at the horizon, blended with a 4x4 ordered dither
SKY = [(18, 14, 44), (32, 20, 66), (54, 28, 84), (88, 40, 100), (132, 56, 108), (182, 82, 106), (226, 120, 100), (250, 170, 110)]
BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
HORIZON = 168
for y in range(H):
    t = min(1.0, y / HORIZON) ** 1.35 * (len(SKY) - 1)
    lo = int(t)
    frac = t - lo
    for x in range(W):
        use_hi = frac * 16 > BAYER[y % 4][x % 4]
        px[x, y] = SKY[min(lo + use_hi, len(SKY) - 1)]

# stars, thinning out towards the horizon
for _ in range(120):
    x, y = rnd.randrange(W), rnd.randrange(0, 105)
    if rnd.random() < y / 140:
        continue
    px[x, y] = (255, 244, 214) if rnd.random() < 0.6 else (190, 180, 230)
    if rnd.random() < 0.08:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            px[x + dx, y + dy] = (190, 180, 230)

# crescent moon
MX, MY, R = 318, 40, 14
for y in range(-R, R + 1):
    for x in range(-R, R + 1):
        if x * x + y * y <= R * R and (x + 6) ** 2 + (y - 2) ** 2 > (R - 1) ** 2:
            px[MX + x, MY + y] = (255, 240, 196)
            if x * x + y * y > (R - 2) ** 2:
                px[MX + x, MY + y] = (255, 214, 150)


def fill(x0, y0, x1, y1, c):
    for y in range(max(0, y0), min(H, y1)):
        for x in range(max(0, x0), min(W, x1)):
            px[x, y] = c


def dome(cx, base, r, c):
    for y in range(-r, 1):
        w = int((r * r - y * y) ** 0.5)
        fill(cx - w, base + y, cx + w + 1, base + y + 1, c)


def minaret(cx, base, h, c):
    fill(cx - 1, base - h, cx + 2, base, c)
    fill(cx - 2, base - h + 10, cx + 3, base - h + 12, c)       # balcony
    for i in range(7):                                           # pointed cap
        fill(cx - (3 - i // 2) // 1 if False else cx - max(0, 1 - i // 4), base - h - 1 - i, cx + 2 + max(0, 0), base - h - i, c)
    fill(cx, base - h - 12, cx + 1, base - h - 7, c)


def skyline(base, colour, seed, density, tall):
    r = random.Random(seed)
    x = 0
    while x < W:
        w = r.randrange(12, 30)
        h = r.randrange(8, tall)
        fill(x, base - h, x + w, base + 40, colour)
        kind = r.random()
        if kind < density:
            dome(x + w // 2, base - h, min(w // 2, r.randrange(7, 13)), colour)
        elif kind < density + 0.25:
            fill(x + 2, base - h - 3, x + w - 2, base - h, colour)
        if r.random() < 0.18:
            minaret(x + w - 2, base - h, r.randrange(18, 34), colour)
        x += w - r.randrange(0, 3)


skyline(HORIZON - 6, (74, 36, 96), 3, 0.35, 36)

# Galata Tower on the left: tall round body with its conical cap and a ring of windows
GT = (46, 24, 66)
fill(70, 70, 92, HORIZON + 4, GT)
fill(66, 62, 96, 70, GT)
for i in range(16):
    fill(66 + i // 2 + 3, 62 - i, 96 - i // 2 - 3, 63 - i, GT)
fill(80, 40, 82, 47, GT)
for wy in (84, 100, 118, 136):
    fill(79, wy, 83, wy + 5, (255, 190, 100))

skyline(HORIZON + 8, (30, 16, 48), 11, 0.30, 26)

# lit windows scattered over the near rooftops
for _ in range(70):
    x, y = rnd.randrange(W), rnd.randrange(HORIZON - 4, HORIZON + 30)
    if px[x, y] == (30, 16, 48) and px[x, y - 1] == (30, 16, 48):
        px[x, y] = (255, 196, 110) if rnd.random() < 0.7 else (255, 140, 90)

# floor: dark stone slabs in rows that widen towards the viewer, with a lit rim
FLOOR_TOP = 184
for y in range(FLOOR_TOP, H):
    row = y - FLOOR_TOP
    for x in range(W):
        px[x, y] = (40, 30, 54) if (x // 16 + row // 8) % 2 == 0 else (46, 35, 62)
    if row % 8 == 0:
        for x in range(W):
            px[x, y] = (24, 16, 38)
for y, c in ((FLOOR_TOP - 2, (120, 70, 100)), (FLOOR_TOP - 1, (200, 120, 110)), (FLOOR_TOP, (74, 48, 84))):
    for x in range(W):
        px[x, y] = c

# darker corners so the menu in the middle reads
for y in range(H):
    for x in range(W):
        edge = max(abs(x - W / 2) / (W / 2), abs(y - H / 2) / (H / 2))
        if edge > 0.8 and BAYER[y % 4][x % 4] < (edge - 0.8) * 60:
            r, g, b = px[x, y]
            px[x, y] = (r * 2 // 3, g * 2 // 3, b * 2 // 3)

img.save("assets/backgrounds/select.png")
print("wrote assets/backgrounds/select.png")
