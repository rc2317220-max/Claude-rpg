#!/usr/bin/env python3
"""One-pass visual lift for RustPG's two hand-authored Blockbench creature textures.

Reads the embedded PNG (the source of truth), applies a deterministic tonal curve,
writes the matching runtime texture and rewrites only the embedded PNG URI.
Standard library only. Intended for the visual-remaster migration, not repeated
on an already remastered source.
"""
from __future__ import annotations
import argparse, base64, json, struct, zlib
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
PNG_SIG=b"\x89PNG\r\n\x1a\n"

def chunks(data:bytes):
    if not data.startswith(PNG_SIG): raise ValueError("not a PNG")
    at=8
    while at<len(data):
        n=struct.unpack(">I",data[at:at+4])[0]
        kind=data[at+4:at+8]
        body=data[at+8:at+8+n]
        yield kind,body
        at+=12+n
        if kind==b"IEND": break

def paeth(a,b,c):
    p=a+b-c; pa=abs(p-a); pb=abs(p-b); pc=abs(p-c)
    return a if pa<=pb and pa<=pc else b if pb<=pc else c

def decode(data:bytes):
    ihdr=None; raw=b""
    for kind,body in chunks(data):
        if kind==b"IHDR": ihdr=body
        elif kind==b"IDAT": raw+=body
    if ihdr is None: raise ValueError("missing IHDR")
    w,h,depth,color,comp,filt,interlace=struct.unpack(">IIBBBBB",ihdr)
    if depth!=8 or color not in (2,6) or interlace!=0:
        raise ValueError(f"unsupported PNG format depth={depth} color={color} interlace={interlace}")
    bpp=4 if color==6 else 3
    stream=zlib.decompress(raw); stride=w*bpp
    rows=[]; pos=0; prev=bytearray(stride)
    for _ in range(h):
        ft=stream[pos]; pos+=1
        src=stream[pos:pos+stride]; pos+=stride
        row=bytearray(stride)
        for x,v in enumerate(src):
            a=row[x-bpp] if x>=bpp else 0
            b=prev[x]; c=prev[x-bpp] if x>=bpp else 0
            if ft==0: val=v
            elif ft==1: val=(v+a)&255
            elif ft==2: val=(v+b)&255
            elif ft==3: val=(v+((a+b)//2))&255
            elif ft==4: val=(v+paeth(a,b,c))&255
            else: raise ValueError(f"bad filter {ft}")
            row[x]=val
        rows.append(row); prev=row
    return w,h,color,rows

def png_chunk(kind:bytes,body:bytes):
    return struct.pack(">I",len(body))+kind+body+struct.pack(">I",zlib.crc32(kind+body)&0xffffffff)

def encode(w,h,color,rows):
    bpp=4 if color==6 else 3
    raw=b"".join(b"\x00"+bytes(r) for r in rows)
    ihdr=struct.pack(">IIBBBBB",w,h,8,color,0,0,0)
    return PNG_SIG+png_chunk(b"IHDR",ihdr)+png_chunk(b"IDAT",zlib.compress(raw,9))+png_chunk(b"IEND",b"")

def curve(c:int)->int:
    # Gamma lift that keeps true black dark, but opens the near-black material range.
    x=max(0.0,min(1.0,c/255.0))
    return round(min(255.0,255.0*(x**0.56)*0.97+11.0))

def remaster(png:bytes,kind:str)->bytes:
    w,h,color,rows=decode(png); bpp=4 if color==6 else 3
    for row in rows:
        for x in range(0,len(row),bpp):
            if bpp==4 and row[x+3]==0: continue
            r,g,b=row[x:x+3]
            r,g,b=curve(r),curve(g),curve(b)
            if kind=="ashen_sentinel":
                r=min(255,round(r*1.02+2)); g=min(255,round(g*1.05+3)); b=min(255,round(b*1.13+7))
            else:
                r=min(255,round(r*0.98+2)); g=min(255,round(g*1.10+5)); b=min(255,round(b*1.12+6))
            row[x:x+3]=bytes((r,g,b))
    return encode(w,h,color,rows)

def one(stem:str):
    model=ROOT/f"project/models/creatures/{stem}.bbmodel"
    runtime=ROOT/f"project/assets/rustpg/textures/entity/{stem}.png"
    text=model.read_text(encoding="utf-8")
    source=json.loads(text)
    textures=source.get("textures",[])
    if len(textures)!=1: raise ValueError(f"{stem}: expected one texture")
    old=textures[0].get("source","")
    prefix="data:image/png;base64,"
    if not old.startswith(prefix): raise ValueError(f"{stem}: embedded PNG missing")
    original=base64.b64decode(old[len(prefix):],validate=True)
    lifted=remaster(original,stem)
    new=prefix+base64.b64encode(lifted).decode("ascii")
    if text.count(old)!=1: raise ValueError(f"{stem}: embedded URI is not unique")
    model.write_text(text.replace(old,new),encoding="utf-8")
    runtime.write_bytes(lifted)
    print(stem,len(original),"->",len(lifted))

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument("creatures",nargs="*",default=["ashen_sentinel","rift_hunter"])
    args=ap.parse_args()
    for stem in args.creatures: one(stem)

if __name__=="__main__": main()
