#!/usr/bin/env python3
"""Render genuine RustPG original 3D cuboid meshes into transparent GUI portraits.

Starling 0.9.0 GUI previews use only the vanilla base mesh even for a mod's
registered world renderer. These static snapshots use the SAME authored cube
data as the actual native renderer, rather than displaying a zombie.
No Pillow/Blender or proprietary runtime data is required.
"""
from __future__ import annotations
import importlib.util, json, math
from pathlib import Path

REPO=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("rustpg_original_creatures",
                                            REPO/"scripts/generate_creatures.py")
source=importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(source)
M=source.M
ROOT=source.ROOT
MODELS_LIST=source.MODELS_LIST
PALETTES=source.PALETTES
_save_png=source._save_png
PORTRAITS=ROOT/"assets/rustpg/textures/gui/portraits"
PORTRAITS.mkdir(parents=True,exist_ok=True)

def shade(base,factor):
    return tuple(max(0,min(255,int(c*factor))) for c in base)+(255,)

def rotate(point,origin,rotation):
    x,y,z=[float(point[i])-float(origin[i]) for i in range(3)]
    rx,ry,rz=[math.radians(float(a)) for a in rotation]
    if rx:y,z=y*math.cos(rx)-z*math.sin(rx),y*math.sin(rx)+z*math.cos(rx)
    if ry:x,z=x*math.cos(ry)+z*math.sin(ry),-x*math.sin(ry)+z*math.cos(ry)
    if rz:x,y=x*math.cos(rz)-y*math.sin(rz),x*math.sin(rz)+y*math.cos(rz)
    return (x+float(origin[0]),y+float(origin[1]),z+float(origin[2]))

def project(p):
    x,y,z=p
    return (x*.82+z*.57,-y*.95+x*.20-z*.16,x*.57-z*.82+y*.13)

def polygon(image,width,points,color):
    pts=[(float(p[0]),float(p[1])) for p in points]
    if len(pts)!=4:return
    lo_x=max(0,int(math.floor(min(p[0] for p in pts))))
    hi_x=min(width-1,int(math.ceil(max(p[0] for p in pts))))
    lo_y=max(0,int(math.floor(min(p[1] for p in pts))))
    hi_y=min(width-1,int(math.ceil(max(p[1] for p in pts))))
    for y in range(lo_y,hi_y+1):
        py=y+.5
        for x in range(lo_x,hi_x+1):
            px=x+.5
            signs=[]
            for i in range(4):
                a,b=pts[i],pts[(i+1)%4]
                signs.append((b[0]-a[0])*(py-a[1])-(b[1]-a[1])*(px-a[0]))
            if min(signs)>=-1e-5 or max(signs)<=1e-5:
                image[y*width+x]=color

def authored_cubes(model):
    if isinstance(model,M):
        return [(c["name"],c["from_"],c["to"],c["origin"],c["rotation"])
                for c in model.cubes]
    return [(str(c.get("name","part")),c["from"],c["to"],
             c.get("origin",[(float(a)+float(b))/2 for a,b in zip(c["from"],c["to"])]),
             c.get("rotation",[0,0,0]))
            for c in model.get("elements",[]) if c.get("type")=="cube"]

def render(name,model,palette):
    n=128
    faces=[]
    for part,lo,hi,origin,rotation in authored_cubes(model):
        verts=[rotate([x,y,z],origin,rotation)
               for x in (float(lo[0]),float(hi[0]))
               for y in (float(lo[1]),float(hi[1]))
               for z in (float(lo[2]),float(hi[2]))]
        def at(x,y,z):return verts[x*4+y*2+z]
        faces_for_cube=[
            ([at(0,0,0),at(1,0,0),at(1,1,0),at(0,1,0)],1.05),
            ([at(1,0,0),at(1,0,1),at(1,1,1),at(1,1,0)],.78),
            ([at(0,1,0),at(1,1,0),at(1,1,1),at(0,1,1)],1.26),
        ]
        part=part.lower()
        if any(t in part for t in ("core","crystal","heart","rune","eye","fang","fire","sigil")):
            base=palette[2]
        elif any(t in part for t in ("horn","blade","crest","visor","wing","talon")):
            base=palette[3]
        elif any(t in part for t in ("skull","helmet","head","shell","armor","plate")):
            base=palette[1]
        else:
            base=tuple(int((a+b)/2) for a,b in zip(palette[0],palette[1]))
        for corners,light in faces_for_cube:
            pp=[project(v) for v in corners]
            faces.append((sum(v[2] for v in pp)/4,pp,shade(base,light)))
    if not faces:
        raise ValueError("No original cube geometry for "+name)
    projected=[p for _,corners,_ in faces for p in corners]
    min_x,max_x=min(p[0] for p in projected),max(p[0] for p in projected)
    min_y,max_y=min(p[1] for p in projected),max(p[1] for p in projected)
    scale=min((n-18)/max(max_x-min_x,1),(n-18)/max(max_y-min_y,1))
    dx=(n-(max_x-min_x)*scale)/2-min_x*scale
    dy=(n-(max_y-min_y)*scale)/2-min_y*scale
    image=[(0,0,0,0)]*(n*n)
    for _,corners,color in sorted(faces,key=lambda f:f[0]):
        polygon(image,n,[(dx+p[0]*scale,dy+p[1]*scale) for p in corners],color)
    out=PORTRAITS/(name+".png")
    _save_png(out,n,n,image)
    return out

def main():
    generated=[]
    for m in MODELS_LIST:
        generated.append(render(m.id,m,PALETTES[m.id]))
    for name,palette in [
        ("ashen_sentinel",((22,21,35),(59,52,72),(243,151,74),(170,114,221))),
        ("rift_hunter",((12,34,40),(41,79,86),(77,225,216),(171,250,233)))]:
        model=json.loads((ROOT/"models/creatures"/(name+".bbmodel")).read_text(encoding="utf-8"))
        generated.append(render(name,model,palette))
    assert len(generated)==12
    assert all(p.read_bytes()[:8]==b"\x89PNG\r\n\x1a\n" for p in generated)
    print("Original 3D mesh portraits:",*[p.name for p in generated],sep=" ")

if __name__=="__main__":main()
