#!/usr/bin/env python3
"""Deterministic 64x64 rift crystal/void-iron atlas for the Rift Hunter, no dependencies."""
from pathlib import Path
import struct,zlib

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/"project/models/textures/rift_hunter.png"
W=H=64

def noise(x,y):
    h=(x*374761393+y*668265263+0x6EEDB7A)&0xFFFFFFFF
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
        r,g,b=clamp(24+shade+plate),clamp(43+shade+plate),clamp(54+shade+plate*2)
        if x%8==0 or y%8==0:
            r,g,b=clamp(r-11),clamp(g-12),clamp(b-14)
        elif x%8==1 or y%8==1:
            r,g,b=clamp(r+10),clamp(g+8),clamp(b+15)
        # Glowing rift fissures that meander through the forged plates.
        fx= (8 + (y//3)%7 - 3) %64
        gx= (36 + (y//5)%9 - 4) %64
        if min(abs(x-fx), abs(x-gx))==0:
            r,g,b=(69,245,229) if n%3 else (149,252,246)
        elif min(abs(x-fx), abs(x-gx))==1:
            r,g,b=(30,100,122)
        # Bioluminescent runes and cyan fractures.
        if 12<=x<=51 and (x+y)%29==0 and 8<y<58:
            r,g,b=(54,210,204)
        if 20<=x<=44 and (x-y)%35==0:
            r,g,b=(39,147,178)
        if (x%16==4 and y%16==4) or (x%16==12 and y%16==12):
            r,g,b=(87,130,150)
        # High-value illuminated eye-band motifs in the upper half of atlas.
        if 24<=y<=27 and (16<=x<=23 or 39<=x<=46):
            r,g,b=(148,255,237) if y in (25,26) else (34,118,126)
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
