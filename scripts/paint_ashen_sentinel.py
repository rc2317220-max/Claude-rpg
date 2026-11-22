#!/usr/bin/env python3
"""Deterministic 64x64 ash/obsidian atlas for the Ashen Sentinel, no dependencies."""
from pathlib import Path
import struct,zlib

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"project/models/textures/ashen_sentinel.png"
W=H=64

def noise(x,y):
    h=(x*374761393+y*668265263+0xACE5A11)&0xFFFFFFFF
    h=(h^(h>>13))*1274126177&0xFFFFFFFF
    return ((h^(h>>16))&255)

def clamp(value):
    return max(0,min(255,int(value)))

pixels=[]
for y in range(H):
    row=[]
    for x in range(W):
        n=noise(x,y)
        shade= (n%19)-9
        plate=((x//8+y//8)&1)*5
        # Violet-tinted charcoal armor and hammered metallic grain.
        r,g,b=clamp(34+shade+plate),clamp(32+shade+plate),clamp(48+shade+plate*2)
        if x%8==0 or y%8==0:
            r,g,b=clamp(r-11),clamp(g-12),clamp(b-14)
        elif x%8==1 or y%8==1:
            r,g,b=clamp(r+10),clamp(g+8),clamp(b+15)
        # Fine ember fissures that meander through the forged plates.
        fx= (8 + (y//3)%7 - 3) %64
        gx= (36 + (y//5)%9 - 4) %64
        if min(abs(x-fx), abs(x-gx))==0:
            r,g,b=(230,108,63) if n%3 else (245,174,93)
        elif min(abs(x-fx), abs(x-gx))==1:
            r,g,b=(107,55,78)
        # Charged runes and purple fractures.
        if 12<=x<=51 and (x+y)%29==0 and 8<y<58:
            r,g,b=(159,94,232)
        if 20<=x<=44 and (x-y)%35==0:
            r,g,b=(115,78,173)
        if (x%16==4 and y%16==4) or (x%16==12 and y%16==12):
            r,g,b=(137,130,154)
        # High-value illuminated eye-band motifs in the upper half of atlas.
        if 24<=y<=27 and (16<=x<=23 or 39<=x<=46):
            r,g,b=(242,165,91) if y in (25,26) else (125,62,49)
        row.extend((r,g,b,255))
    pixels.append(bytes(row))

def chunk(name,payload):
    return struct.pack(">I",len(payload))+name+payload+struct.pack(">I",zlib.crc32(name+payload)&0xffffffff)

raw=b"".join(bytes([0])+row for row in pixels)
png=(bytes([137,80,78,71,13,10,26,10])+
     chunk(b"IHDR",struct.pack(">IIBBBBB",W,H,8,6,0,0,0))+
     chunk(b"IDAT",zlib.compress(raw,9))+chunk(b"IEND",b""))
OUT.parent.mkdir(parents=True,exist_ok=True)
OUT.write_bytes(png)
print(f"Created {OUT} ({len(png)} bytes) {W}x{H}")
