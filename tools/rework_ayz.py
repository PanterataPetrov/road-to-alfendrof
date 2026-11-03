"""The Professor's reworked default pose, made from the original drawing.

    cargo run --bin gen_ayz -- --ayz-original ayz_original.png
    python tools/rework_ayz.py ayz_original.png tools/ayz_reworked.png

The edits are made on the drawing as it sits in an idle frame (placed at OX=12, OY=8 on the 84x63 canvas, the
coordinates below), and the result is cut back out at OY=6: his legs are two pixels longer, so the rest of him sits
two pixels higher. The crescent scar stays where it was.
"""
import sys
from PIL import Image

original=Image.open(sys.argv[1]).convert("RGBA")
GW,GH=original.size
src=Image.new("RGBA",(84,63),(0,0,0,0))
src.paste(original,(12,8))
W,H=src.size
px=src.load()
# the scar is skin while the edits are made, and goes back where it was afterwards
SCAR=(196,132,108,255)
scar=[(x,y) for y in range(H) for x in range(W) if px[x,y]==SCAR]
for x,y in scar: px[x,y]=(238,196,158,255)
# the little hole in his raised arm (our left): two transparent pixels inside the skin
for hy in (29,30):
    if px[15,hy][3]<10: px[15,hy]=px[14,hy]
# a thinner raised arm (our left): two columns out of the middle of the forearm, the outer edge comes in by two,
# and the bottom row of the fist goes
for y in range(29,39):
    row=[px[x,y] for x in range(8,20)]
    kept=row[:15-8]+row[17-8:]
    row2=[(0,0,0,0)]*2+kept
    for i,x in enumerate(range(8,20)): px[x,y]=row2[i]
for x in range(8,19):
    if px[x,38][3]>200 and px[x,39][3]<10: px[x,38]=(0,0,0,0)
out=src.copy(); o=out.load()
def skin(p): return p[3]>200 and p[0]>200 and p[1]>150 and p[2]>110 and p[0]>=p[1]>=p[2]
# forehead: only the skin moves down a row; hair, sideburns, glasses and moustache stay exactly where they are
for y in range(11,7,-1):
    for x in range(16,40):
        if skin(px[x,y]) and skin(px[x,y+1] if y+1<H else (0,0,0,0)) or (skin(px[x,y]) and y+1<=12):
            o[x,y+1]=px[x,y]
for x in range(16,40):
    if skin(px[x,8]) : o[x,8]=(0,0,0,0)
# and one row more: the skin moves down again in the same way
snap={(x,y):o[x,y] for x in range(16,40) for y in range(8,15)}
for y in range(12,8,-1):
    for x in range(16,40):
        if skin(snap[x,y]) and (skin(snap[x,y+1]) or y+1<=13):
            o[x,y+1]=snap[x,y]
for x in range(16,40):
    if skin(snap[x,9]): o[x,9]=(0,0,0,0)
# a round top to his head: the top row of skin loses two pixels at each end, the next row one
tops=[y for y in range(8,16) if any(skin(o[x,y]) for x in range(16,40))]
for y,n in zip(tops[:2],(2,1)):
    xs=[x for x in range(16,40) if skin(o[x,y])]
    left,right=xs[0],xs[-1]
    for x in xs[:n]+xs[-n:]: o[x,y]=(0,0,0,0)
    # the hair on each side of that row moves in by the same amount, so it still meets the head
    row=[o[x,y] for x in range(W)]
    for x in range(W): o[x,y]=(0,0,0,0) if (x<left+n or x>right-n) else row[x]
    for x in range(left-1,-1,-1):
        if x+n<left+n: o[x+n,y]=row[x]
    for x in range(right+1,W):
        if x-n>right-n: o[x-n,y]=row[x]
# no loose hair pixels floating above the head: a pixel with nothing next to it (not even diagonally) goes
for y in range(4,16):
    for x in range(10,50):
        if o[x,y][3]>200 and not any(o[x+dx,y+dy][3]>200 for dx in (-1,0,1) for dy in (-1,0,1) if (dx,dy)!=(0,0)):
            o[x,y]=(0,0,0,0)
# the body: columns taken out of the middle (the arms and hair keep their shape); the belly rows lose more than the chest
for y in range(24,50):
    cuts=[24,29] if y<37 else [21,24,29,32]
    row=[px[x,y] for x in range(W)]
    kept=[row[x] for x in range(W) if x not in cuts]
    pad=len(cuts)//2
    row2=[(0,0,0,0)]*pad+kept+[(0,0,0,0)]*(len(cuts)-pad)
    for x in range(W): o[x,y]=row2[x]
# cover the belly: the pants come up over it and the shirt ends above the waistband, so the whole pants show
SHIRT={(240,241,250,255),(186,192,216,255),(150,156,188,255)}
SKIN={(238,196,158,255),(204,152,112,255),(222,174,134,255),(166,118,90,255)}
ORANGE=(246,142,32,255); SHADOW=(196,98,26,255)
for y in range(44,50):
    for x in range(W):
        p=o[x,y]
        if y<=45 and p in SHIRT:
            o[x,y]=SHADOW if y==44 else ORANGE
        elif y>=46 and p in SKIN:
            o[x,y]=ORANGE
for x in range(W):
    if o[x,50] in SKIN: o[x,50]=(140,64,22,255)
# a dark brown leather belt: it dips a pixel under the belly in the middle (the shirt hangs over it there),
# with a shine along the top edge, two belt loops, a shadow on the pants under it and a brass buckle
PANTS={(246,142,32,255),(255,190,104,255),(196,98,26,255)}
xs=[x for x in range(W) for y in (44,45) if o[x,y] in PANTS]
xl,xr=min(xs),max(xs)
mid=(xl+xr)//2
TOP=(58,38,26,255); BOT=(30,20,15,255); SHINE=(96,66,44,255)
LOOP=(214,112,28,255); UNDER=(196,98,26,255); SHIRT_SHADE=(186,192,216,255)
for x in range(xl,xr+1):
    dy=1 if abs(x-mid)<=6 else 0
    top=44+dy
    if dy and o[x,44] in PANTS: o[x,44]=SHIRT_SHADE
    o[x,top]=SHINE if (x-xl)%5==2 else TOP
    o[x,top+1]=BOT
    if o[x,top+2] in PANTS: o[x,top+2]=UNDER
for x in (xl+4,xr-4):
    dy=1 if abs(x-mid)<=6 else 0
    o[x,44+dy]=LOOP; o[x,45+dy]=LOOP
# the buckle: a brass frame round a bit of belt, with the prong and a glint
BRASS=(178,138,62,255); BRASS_D=(116,86,38,255); GLINT=(236,206,128,255)
for y in range(44,48):
    for x in range(mid-2,mid+2):
        frame=y in (44,47) or x in (mid-2,mid+1)
        o[x,y]=BRASS if frame else BOT
o[mid-1,45]=BRASS_D; o[mid-1,46]=BRASS_D
o[mid-2,44]=GLINT
o[mid+1,47]=BRASS_D

# the right side of the belly comes in: the outermost shirt pixels on the right go (one on the upper belly, two lower
# down), and the new edge gets the shirt's darker edge colour; the arm stays where it is
SHIRT_ALL={(240,241,250,255),(186,192,216,255),(150,156,188,255)}
for y in range(32,44):
    n=1 if y<36 else 2
    xs=[x for x in range(30,W) if o[x,y] in SHIRT_ALL]
    if not xs: continue
    edge=max(xs)
    for x in range(edge-n+1,edge+1):
        if o[x,y] in SHIRT_ALL: o[x,y]=(0,0,0,0)
    if o[edge-n,y] in SHIRT_ALL: o[edge-n,y]=(150,156,188,255)

# no belly button showing through the shirt: plain white cloth there
for y in range(36,44):
    for x in range(25,47):
        p=o[x,y]
        if p[3]>200 and p[0]>140 and p[0]>p[2]+30: o[x,y]=(240,241,250,255)

# a lone bit of shirt left beside his right hand by the trimming
for y in range(30,46):
    for x in range(40,W-1):
        if o[x,y] in SHIRT_ALL and o[x-1,y][3]<10:
            o[x,y]=(0,0,0,0)

# the scar back on the forehead
for x,y in scar:
    assert o[x,y][:3]==(238,196,158), ("scar spot is no longer skin", x, y)
    o[x,y]=SCAR

# legs two pixels longer: two rows of the lower legs (where they run straight) are doubled, and everything above
# moves up two so his shoes stay on the floor
rows=[[o[x,y] for x in range(W)] for y in range(H)]
assert all(p[3]<10 for r in rows[:2] for p in r), "room above his head"
longer=rows[:53]+[rows[52]]+rows[53:55]+[rows[54]]+rows[55:]
for y in range(H):
    for x in range(W): o[x,y]=longer[y+2][x]

# cut him back out: the same columns, from row 6 (two rows higher), two rows taller
box=(12,6,12+GW,6+GH+2)
for y in range(H):
    for x in range(W):
        inside=box[0]<=x<box[2] and box[1]<=y<box[3]
        assert inside or o[x,y][3]<10, ("drawing outside the box", x, y)
out.crop(box).save(sys.argv[2])
print("wrote", sys.argv[2], out.crop(box).size)
