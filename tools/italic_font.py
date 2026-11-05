"""Makes assets/fonts/silkscreen-italic.ttf from silkscreen.ttf: every outline sheared about 12 degrees to the right
(Silkscreen has no italic of its own). Needs fontTools:  pip install fonttools
    python tools/italic_font.py
The family is renamed so the game can tell the two apart (the italic is picked by its font handle, not by style)."""
import pathlib

from fontTools.ttLib import TTFont

SLANT = 0.21  # tan(12 degrees)
fonts = pathlib.Path(__file__).resolve().parent.parent / "assets" / "fonts"
font = TTFont(fonts / "silkscreen.ttf")
glyf = font["glyf"]
for name in glyf.keys():
    glyph = glyf[name]
    if glyph.isComposite():
        # accents and the like: move each piece along the slant (the pieces themselves are sheared below)
        for part in glyph.components:
            part.x = round(part.x + SLANT * part.y)
    elif glyph.numberOfContours > 0:
        glyph.coordinates.transform(((1, 0), (SLANT, 1)))
for name in glyf.keys():
    glyf[name].recalcBounds(glyf)
font["head"].xMax = max(glyf[n].xMax for n in glyf.keys() if hasattr(glyf[n], "xMax"))
font["post"].italicAngle = -12.0
for record in font["name"].names:
    text = record.toUnicode()
    if "Silkscreen" in text:
        record.string = text.replace("Silkscreen", "SilkscreenItalic")
font.save(fonts / "silkscreen-italic.ttf")
print("wrote", fonts / "silkscreen-italic.ttf")
