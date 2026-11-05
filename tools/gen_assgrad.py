"""Generate Assgrad's paved concrete bridge and sightseeing binocular viewer.

Requires Python 3 with Pillow: python tools/gen_assgrad.py [original-photo.jpg]
An optional source photo also regenerates the reduced-palette pixel backdrop.
"""
from pathlib import Path
import sys
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / "assets/backgrounds/assgrad"
SCALE = 1

# Match bridge materials to the landscape instead of a separate art palette.
if len(sys.argv) > 1:
    photo = Image.open(sys.argv[1]).convert("RGB")
    size = (640, round(640 * photo.height / photo.width))
    backdrop = photo.resize(size, Image.Resampling.BOX).quantize(colors=96)
    OUT.mkdir(parents=True, exist_ok=True)
    backdrop.convert("RGB").save(OUT / "near.png")
else:
    backdrop = Image.open(OUT / "near.png").convert("RGB").quantize(colors=96)


def canvas(width, height):
    image = Image.new("RGBA", (width * SCALE, height * SCALE))
    return image, ImageDraw.Draw(image)


def box(draw, bounds, fill, outline=None, width=1):
    draw.rectangle(tuple(round(v * SCALE) for v in bounds), fill=fill,
                   outline=outline, width=width * SCALE)


def line(draw, points, fill, width=1):
    draw.line([(round(x * SCALE), round(y * SCALE)) for x, y in points],
              fill=fill, width=width * SCALE)


def ellipse(draw, bounds, fill, outline=None):
    draw.ellipse(tuple(round(v * SCALE) for v in bounds), fill=fill,
                 outline=outline, width=SCALE)


def save(image, name, size):
    OUT.mkdir(parents=True, exist_ok=True)
    pixels = image.resize(size, Image.Resampling.NEAREST)
    # Reflected daylight softens the cold outlines; no dithering keeps flat pixel clusters.
    surface = Image.blend(pixels.convert("RGB"), Image.new("RGB", size, (131, 143, 131)), 0.22)
    surface = surface.quantize(palette=backdrop, dither=Image.Dither.NONE).convert("RGBA")
    surface.putalpha(pixels.getchannel("A"))
    surface.save(OUT / name)


# Grey interlocking pavers, drawn at half resolution for a chunky pixel grid.
floor, d = canvas(16, 20)
box(d, (0, 0, 15, 19), "#777c79")
for gy in range(-2, 12):
    for gx in range(-2, 10):
        orientation = (gx - gy) % 4
        x, y = gx * 2, gy * 2
        shade = ("#acafa5", "#999f98", "#b8b9ad")[(gx + gy) % 3]
        if orientation == 0:
            box(d, (x, y, x + 2, y), shade)
        elif orientation == 3:
            box(d, (x, y, x, y + 2), shade)
line(d, [(0, 0), (15, 0)], "#c0c1b3")
save(floor, "floor.png", (32, 40))

# Full-width world-space scenery, with the bottom precisely at the walking plane.
scene, d = canvas(800, 68)
# Low concrete parapet with a narrow metal handrail, as on the real bridge.
box(d, (0, 48, 799, 68), "#999e96")
box(d, (0, 46, 799, 48), "#b7b9ac")
line(d, [(0, 49), (799, 49)], "#777e78")
for x in range(0, 801, 80):
    box(d, (x, 49, x + 1, 68), "#7e8780")
    box(d, (x + 4, 49, x + 5, 58), "#8d958d")
    box(d, (x + 52, 57, x + 53, 66), "#8d958d")
    box(d, (x - 1, 39, x + 1, 46), "#526561")
    line(d, [(x, 40), (x, 45)], "#9aa79c")
box(d, (0, 38, 799, 40), "#5d716d")
line(d, [(0, 38), (799, 38)], "#a9b4a6")

# Coin-operated bridge binocular viewer, slightly right of the arena's centre.
x = 510
ellipse(d, (x - 12, 63, x + 12, 68), "#1d3039", "#9aa6a2")
box(d, (x - 5, 35, x + 5, 65), "#465f68", "#243740")
box(d, (x - 3, 36, x - 1, 63), "#a4b2ad")
box(d, (x - 7, 34, x + 7, 39), "#a8b3ad", "#2b414b")
box(d, (x - 9, 26, x + 9, 35), "#607c83", "#243a44")
# Rounded silver binocular housing and two dark, blue-glass eyepieces.
d.rounded_rectangle(((x - 18) * SCALE, 12 * SCALE,
                     (x + 18) * SCALE, 29 * SCALE),
                    radius=6 * SCALE, fill="#b4c1ba", outline="#314a56", width=SCALE)
line(d, [(x - 12, 13), (x + 11, 13)], "#edf0d9", 2)
for eye in (x - 10, x + 10):
    ellipse(d, (eye - 6, 16, eye + 6, 28), "#536e79", "#d9dfcb")
    ellipse(d, (eye - 4, 18, eye + 4, 26), "#101f30", "#2b434f")
    ellipse(d, (eye - 2, 19, eye + 1, 21), "#72bbcd")
line(d, [(x - 21, 25), (x - 24, 29), (x - 22, 33)], "#b5c3bb", 2)
line(d, [(x + 21, 25), (x + 24, 29), (x + 22, 33)], "#b5c3bb", 2)
box(d, (x - 3, 29, x + 3, 32), "#d3c799")
line(d, [(x - 1, 30), (x + 1, 30)], "#26343b")
for bolt in (x - 8, x + 8):
    ellipse(d, (bolt - 1, 65, bolt + 1, 67), "#c1c6b8")
save(scene, "bridge.png", (800, 68))

print(f"Wrote concrete bridge, pavers and sightseeing viewer to {OUT}")
