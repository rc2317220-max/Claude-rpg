"""Compile one genuine per-face texture atlas per creature from the actual Rust rig.
No UI portraits or tiled PNG is used for the world renderer. The palette and painted
details are authored procedurally, with a separate atlas cell for each cuboid face.
Run only against staged candidate; preserve source .bbmodel and original texture.
"""
from pathlib import Path
import json, math, hashlib, re
from PIL import Image, ImageDraw
from creature_inspector import parse_generated

DIRS=("down","up","north","south","west","east")
PALETTES={
"ashen_sentinel":((43,37,47),(199,113,53)),"rift_hunter":((28,43,42),(92,213,170)),
"mana_hound":((24,39,53),(58,188,235)),"grave_knight":((52,49,65),(163,112,221)),
"rift_mantis":((37,60,52),(230,96,170)),"crystal_wyrm":((40,71,91),(121,233,251)),
"eclipse_ogre":((75,44,37),(242,103,51)),"dusk_revenant":((43,30,61),(181,114,246)),
"storm_gryphon":((49,67,85),(107,193,235)),"rune_colossus":((56,67,63),(98,225,182)),
"ember_imp":((77,40,27),(248,137,65)),"abyss_weaver":((45,31,56),(179,87,215))}
def shade(color,v):return tuple(max(0,min(255,int(c*v))) for c in color)+(255,)
def material(name,base,glow):
    n=name.lower()
    if any(k in n for k in ("eye","visor","core","crystal","rune","sigil","heart","mana_spine")):return glow,"energy"
    if any(k in n for k in ("tooth","teeth","fang","claw","horn","beak","blade","talon")):return tuple(int(c*.35+170) for c in base),"bone"
    if any(k in n for k in ("wing","feather","cape","cloth")):return tuple(int(c*.72) for c in base),"cloth"
    if any(k in n for k in ("plate","shell","guard","helmet","shield","armor","pauldron")):return tuple(int(c*1.18) for c in base),"plate"
    return base,"hide"
def paint(tile,name,face,base,glow,seed):
    w,h=tile.size;d=ImageDraw.Draw(tile);c,style=material(name,base,glow)
    light={"up":1.18,"down":.63,"north":1.02,"south":.87,"east":.94,"west":.82}[face]
    for y in range(h):
        for x in range(w):
            noise=((x*19+y*37+seed+(x*y*3))%9-4)*.018
            edge=min(x,y,w-1-x,h-1-y)
            v=light*(.78+.22*(1-y/max(1,h-1))+noise)
            if edge==0:v*=.68
            if edge==1 and (x==1 or y==1):v*=1.18
            d.point((x,y),shade(c,v))
    if style=="plate":
        d.line((2,2,w-3,2),fill=shade(c,1.5*light));d.line((2,h-3,w-3,h-3),fill=shade(c,.55*light))
        if w>7 and h>7:
            for x in (3,w-4):d.point((x,4),fill=shade(glow,.7))
    elif style=="cloth":
        for x in range(2,w-1,4):d.line((x,1,x,h-2),fill=shade(c,light*.75))
    elif style=="energy":
        d.polygon([(w//2,1),(w-2,h//2),(w//2,h-2),(1,h//2)],fill=shade(glow,1.10))
        d.line((w//2,2,w//2,h-3),fill=shade(glow,1.5))
    elif style=="bone":
        d.line((w//2,2,w//2,h-3),fill=shade(c,1.18*light))
    elif "head" in name or "skull" in name or "muzzle" in name:
        if face=="north":
            y=max(2,h//3);d.rectangle((2,y,max(2,w//3),min(h-2,y+1)),fill=shade(glow,1.25))
            d.rectangle((min(w-3,w*2//3),y,w-3,min(h-2,y+1)),fill=shade(glow,1.25))
    elif style=="hide":
        for y in range(3,h-2,4):
            d.line((2,y,min(w-3,5+y%5),y),fill=shade(c,light*.82))
def build_atlases(stage):
    roster=parse_generated(stage);assert set(roster)==set(PALETTES),(set(roster),set(PALETTES))
    output=stage/"assets/rustpg/textures/entity"; report={}
    # 32px padded tile: a one-pixel gutter keeps bilinear/atlas filtering off neighbouring faces.
    cell=34
    rs=["// generated from actual creature meshes and atlas face rectangles",
        "pub fn uv(kind:&str,index:usize)->Option<[[f32;4];6]>{",
        "let rows:&[[[f32;4];6]]=match kind {"]
    tables=[]
    for kind,rig in sorted(roster.items()):
        cubes=rig["cubes"];assert cubes
        count=len(cubes)*6;cols=16;rows=(count+cols-1)//cols
        size=1
        while size<max(cols*cell,rows*cell):size*=2
        atlas=Image.new("RGBA",(size,size),(0,0,0,0)); mappings=[];base,glow=PALETTES[kind]
        for i,cube in enumerate(cubes):
            face_uv=[]
            for j,face in enumerate(DIRS):
                index=i*6+j;x=index%cols*cell+1;y=index//cols*cell+1
                tile=Image.new("RGBA",(32,32))
                seed=int(hashlib.sha256((kind+cube["name"]+face).encode()).hexdigest()[:8],16)
                paint(tile,cube["name"],face,base,glow,seed)
                atlas.paste(tile,(x,y))
                # Extrude outer pixels into gutter to keep UV seams from sampling blank pixels.
                atlas.paste(tile.crop((0,0,1,32)),(x-1,y))
                atlas.paste(tile.crop((31,0,32,32)),(x+32,y))
                atlas.paste(tile.crop((0,0,32,1)),(x,y-1))
                atlas.paste(tile.crop((0,31,32,32)),(x,y+32))
                face_uv.append([x*16/size,y*16/size,(x+32)*16/size,(y+32)*16/size])
            mappings.append(face_uv)
        atlas.save(output/(kind+"_uv.png"))
        key=kind.upper()
        rs.append(f' "rustpg:{kind}"=>{key},')
        def fmt(v):return f"{v:.8f}f32"
        table="const "+key+":&[[[f32;4];6]]=&[\n"
        for uv in mappings:table+=" ["+",".join("["+",".join(map(fmt,r))+"]" for r in uv)+"],\n"
        tables.append(table+"];\n")
        report[kind]={"cubes":len(cubes),"faces":count,"atlas":[size,size],"uv":mappings}
    rs+=["_=>return None};rows.get(index).copied()}"]+tables
    (stage/"src/rustpg_creature_uv_generated.rs").write_text("\n".join(rs))
    (stage/"uv_atlas_report.json").write_text(json.dumps(report,indent=2))
    lib=stage/"src/lib.rs";s=lib.read_text();assert "mod rustpg_native_entities;" in s
    lib.write_text(s.replace("mod rustpg_native_entities;","mod rustpg_native_entities;\nmod rustpg_creature_uv_generated;",1))
    p=stage/"src/rustpg_native_entities.rs";s=p.read_text()
    old="for(part,path)in spec.cubes.iter().zip(spec.chains.iter()){"
    assert old in s;s=s.replace(old,"for(index,(part,path))in spec.cubes.iter().zip(spec.chains.iter()).enumerate(){",1)
    old='let face=Some(CubeFace::new(spec.texture));\n            let _=draw.cube(part.from,part.to,[face;6],part_tint(self.kind,part.name));'
    assert old in s
    s=s.replace(old,'''let texture=format!("{}_uv",spec.texture);
            let Some(uv)=crate::rustpg_creature_uv_generated::uv(self.kind,index) else {
                draw.pop();draw.pop();return Err("Missing authored UV mapping".into());
            };
            let faces=std::array::from_fn(|side|Some(CubeFace::new(&texture).with_uv(uv[side])));
            let _=draw.cube(part.from,part.to,faces,0xFFFFFF);''',1)
    p.write_text(s)
