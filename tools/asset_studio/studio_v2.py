#!/usr/bin/env python3
"""Reusable offline cuboid asset QA, animation poses, Rust export and events.
Never executes gameplay. Supports one atlas, unrotated UV, numeric linear keys."""
import argparse,collections,json,math,sys,time
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from creature_inspector import authored,around,apply,rotation,sample,FACES,triangle

def load(path):
    doc=json.loads(Path(path).read_text());rig=authored(Path(path))
    names={g["uuid"]:g["name"] for g in doc.get("groups",[])}
    for clip in doc.get("animations",[]):
        for bone,animator in clip.get("animators",{}).items():
            if bone not in rig["bones"]:continue
            for key in animator.get("keyframes",[]):
                if key["channel"] not in ("rotation","position","scale"):continue
                if key.get("interpolation","linear")!="linear":raise ValueError("Only linear keys supported by independent renderer")
                d=key["data_points"][0];v=np.array([float(d[k]) for k in "xyz"])
                if not np.isfinite(v).all():raise ValueError("Nonfinite key")
                rig["keys"].append((clip["name"],bone,key["channel"],float(key["time"]),v))
    return doc,rig,names

def polygons(rig,clip,t):
    bones=rig["bones"];keys=rig["keys"];cache={};visiting=set()
    def matrix(b):
        if b is None:return np.eye(4)
        if b in cache:return cache[b]
        if b in visiting:raise ValueError("Bone cycle")
        visiting.add(b);parent,p,r=bones[b]
        trans=np.eye(4);trans[:3,3]=sample(keys,b,"position",clip,t,[0]*3)
        s=sample(keys,b,"scale",clip,t,[1]*3);sm=np.eye(4);sm[:3,:3]=np.diag(s);sm[:3,3]=p-s*p
        cache[b]=matrix(parent)@trans@around(r+sample(keys,b,"rotation",clip,t,[0]*3),p)@sm
        visiting.remove(b);return cache[b]
    result=[]
    for c in rig["cubes"]:
        x,y,z=c["lo"];X,Y,Z=c["hi"]
        pts=np.array([[x,y,z],[X,y,z],[X,Y,z],[x,Y,z],[x,y,Z],[X,y,Z],[X,Y,Z],[x,Y,Z]])
        pts=apply(matrix(c["bone"])@around(c["rot"],c["origin"]),pts)
        for f,ix in FACES.items():
            if f not in c["uv"]:continue
            a,b,d,e=c["uv"][f];result.append((pts[list(ix)],np.array([[a,e],[d,e],[d,b],[a,b]]),f))
    return result

def frame(polys,tex,angle,size=512,center=None,extent=None,transparent=False):
    mat=rotation(angle);pts=np.concatenate([p[0] for p in polys])
    if center is None:center=(pts.min(0)+pts.max(0))/2
    if extent is None:extent=max(np.linalg.norm(pts.max(0)-pts.min(0)),.01)
    scale=size*.80/extent;buf=np.zeros((size,size,3),dtype=np.uint8);buf[:]=[21,25,33]
    depth=np.full((size,size),-np.inf);light=np.array([-.35,.55,.75]);light/=np.linalg.norm(light)
    for pts,uv,_ in polys:
        p=(pts-center)@mat.T;n=np.cross(p[1]-p[0],p[2]-p[0]);l=np.linalg.norm(n)
        if l<1e-9:continue
        shade=.60+.40*abs(np.dot(n/l,light));p[:,0]=p[:,0]*scale+size/2;p[:,1]=-p[:,1]*scale+size/2
        for tri in ([0,1,2],[0,2,3]):triangle(buf,depth,p[tri],uv[tri],tex,shade)
    mask=np.isfinite(depth)
    if not mask.any():raise ValueError("Blank pose")
    return Image.fromarray(np.dstack([buf,mask.astype(np.uint8)*255])) if transparent else Image.fromarray(buf)

def uv_check(doc,texture,emissive,out):
    w,h=texture.size;rw,rh=doc["resolution"]["width"],doc["resolution"]["height"]
    coverage=np.zeros((h,w),np.uint16);errors=[];warnings=[];rows=[];sheet=texture.copy().resize((w*2,h*2),Image.Resampling.NEAREST).convert("RGB");d=ImageDraw.Draw(sheet)
    for c in doc["elements"]:
        if c.get("type","cube")!="cube":continue
        x,y,z=[b-a for a,b in zip(c["from"],c["to"])]
        dims={"north":(x,y),"south":(x,y),"east":(z,y),"west":(z,y),"up":(x,z),"down":(x,z)}
        for face,f in c["faces"].items():
            if f.get("texture") is None:continue
            name=c["name"]+"/"+face;a,b,e,g=f["uv"];mirror=a>e or b>g
            x0,x1=sorted([round(a*w/rw),round(e*w/rw)]);y0,y1=sorted([round(b*h/rh),round(g*h/rh)])
            if not (0<=x0<x1<=w and 0<=y0<y1<=h):errors.append(name+" outside/empty UV");continue
            coverage[y0:y1,x0:x1] = coverage[y0:y1,x0:x1] + 1
            width,height=x1-x0,y1-y0;dw,dh=dims[face]
            ratio=(width/max(dw,.001))/(height/max(dh,.001))
            if ratio>2.5 or ratio<.4:warnings.append(name+" anisotropic density")
            if mirror:warnings.append(name+" mirrored UV")
            if min(width,height)<2:warnings.append(name+" tiny UV")
            rows.append(dict(part=c["name"],face=face,uv=[a,b,e,g],pixels=[width,height],density=[round(width/max(dw,.001),2),round(height/max(dh,.001),2)]))
            d.rectangle((x0*2,y0*2,x1*2-1,y1*2-1),outline=(67,220,202))
    arr=np.asarray(texture.convert("RGBA"),dtype=np.uint8);used=coverage>0;alpha=arr[:,:,3]>0
    if not rows or not np.any(used):errors.append("empty UV coverage")
    if (used&~alpha).any():errors.append("transparent pixels inside allocated opaque face")
    if (coverage>1).any():errors.append("overlapping UV pixels")
    glow=np.asarray(emissive.convert("RGBA"),dtype=np.uint8)[:,:,3]>0
    if glow.shape!=alpha.shape:raise ValueError("emissive dimensions mismatch")
    if (glow&~alpha).any():errors.append("emissive outside painted atlas")
    hashes=collections.Counter(bytes(arr[round(r["uv"][1]*h/rh):round(r["uv"][3]*h/rh),round(r["uv"][0]*w/rw):round(r["uv"][2]*w/rw)].tobytes()) for r in rows)
    report=dict(faces=len(rows),errors=errors,warnings=warnings,occupied_fraction=float(used.mean()),painted_fraction=float(alpha.mean()),emissive_fraction=float((glow&used).sum()/max(1,used.sum())),overlap_pixels=int((coverage>1).sum()),duplicate_face_images=sum(n-1 for n in hashes.values()),luminance_std=float(arr[:,:,:3][used].mean(1).std()) if used.any() else None,faces_detail=rows)
    sheet.save(out/"uv-inspector.png");(out/"uv-report.json").write_text(json.dumps(report,indent=2))
    if errors:raise ValueError(str(errors))
    return report

def validate_events(doc,spec,names):
    clips={c["name"].split(".")[-1]:float(c["length"]) for c in doc["animations"]};seen=set()
    for e in spec["events"]:
        assert e["clip"] in clips and 0<=e["at"]<=clips[e["clip"]]
        assert e["socket"] in names.values() and e["kind"] in ("hit","vfx","sound","state")
        key=(e["clip"],e["id"]);assert key not in seen;seen.add(key)
    for atk in spec["attacks"]:
        clip=atk["clip"];assert 0<atk["windup"]<atk["active_end"]<=clips[clip] and atk["cooldown"]>=clips[clip]
        hits=[e for e in spec["events"] if e["clip"]==clip and e["kind"]=="hit"]
        assert hits and all(atk["windup"]<=e["at"]<=atk["active_end"] for e in hits)
    return dict(attacks=len(spec["attacks"]),events=len(seen),policy="server-authoritative intent only; no runtime dispatch installed")

def rust_export(doc,rig,names,spec,out):
    def q(v):return json.dumps(str(v),ensure_ascii=True)
    def n(v):
        v=float(v)
        if not math.isfinite(v):raise ValueError("nonfinite export")
        return repr(v)+"f32"
    def vec(v):return "["+",".join(map(n,v))+"]"
    lines=["// Generated from actual BBModel; compile standalone. Runtime adapter not installed.","pub struct Bone {pub name:&'static str,pub parent:Option<&'static str>,pub pivot:[f32;3],pub rotation:[f32;3]}","pub struct Cube {pub name:&'static str,pub bone:&'static str,pub lo:[f32;3],pub hi:[f32;3],pub pivot:[f32;3],pub rotation:[f32;3],pub uv:[[f32;4];6],pub face_mask:u8}","pub struct Key {pub clip:&'static str,pub bone:&'static str,pub channel:&'static str,pub at:f32,pub value:[f32;3]}","pub struct Event {pub clip:&'static str,pub id:&'static str,pub kind:&'static str,pub socket:&'static str,pub at:f32,pub effect:&'static str}","pub const BONES:&[Bone]=&["]
    for id,(parent,p,r) in rig["bones"].items():
        par="None" if parent is None else "Some("+q(names[parent])+")"
        lines.append(f"Bone{{name:{q(names[id])},parent:{par},pivot:{vec(p)},rotation:{vec(r)}}},")
    lines+=["];","pub const CUBES:&[Cube]=&["]
    faces=("down","up","north","south","west","east")
    for c in rig["cubes"]:
        mask=sum(1<<i for i,f in enumerate(faces) if f in c["uv"])
        uv="["+",".join(vec(c["uv"].get(f,[0]*4)) for f in faces)+"]"
        lines.append(f"Cube{{name:{q(c['name'])},bone:{q(names[c['bone']])},lo:{vec(c['lo'])},hi:{vec(c['hi'])},pivot:{vec(c['origin'])},rotation:{vec(c['rot'])},uv:{uv},face_mask:{mask}}},")
    lines+=["];","pub const KEYS:&[Key]=&["]
    for clip,bone,ch,t,v in rig["keys"]:lines.append(f"Key{{clip:{q(clip)},bone:{q(names[bone])},channel:{q(ch)},at:{n(t)},value:{vec(v)}}},")
    lines+=["];","pub const EVENTS:&[Event]=&["]
    for e in spec["events"]:lines.append(f"Event{{clip:{q(e['clip'])},id:{q(e['id'])},kind:{q(e['kind'])},socket:{q(e['socket'])},at:{n(e['at'])},effect:{q(e['effect'])}}},")
    lines+=["];",'''// Interval dispatch for a non-looping action. Caller owns action id, authority and deduplication.
pub fn crossed(clip:&str,previous:f32,now:f32)->impl Iterator<Item=&'static Event>+'_ {
 EVENTS.iter().filter(move |e|e.clip==clip && previous<e.at && e.at<=now)
}
#[cfg(test)] mod tests {use super::*;
 #[test] fn references(){for c in CUBES{assert!(BONES.iter().any(|b|b.name==c.bone));}
 for e in EVENTS{assert!(BONES.iter().any(|b|b.name==e.socket));}}
 #[test] fn intervals(){for e in EVENTS{assert!(crossed(e.clip,e.at-0.001,e.at).any(|x|x.id==e.id));assert!(!crossed(e.clip,e.at,e.at+0.001).any(|x|x.id==e.id));}}
}''']
    (out/"ashen_sentinel_v3_generated.rs").write_text("\n".join(lines)+"\n")

def run(model,atlas,emissive,specpath,out,baseline=None):
    out.mkdir(parents=True,exist_ok=True);doc,rig,names=load(model)
    texture=Image.open(atlas).convert("RGBA");glow=Image.open(emissive).convert("RGBA");tex=np.array(texture)
    uv=uv_check(doc,texture,glow,out);spec=json.loads(Path(specpath).read_text())
    events=validate_events(doc,spec,names);rust_export(doc,rig,names,spec,out)
    rest=polygons(rig,None,0);pts=np.concatenate([f[0] for f in rest]);center=(pts.min(0)+pts.max(0))/2
    extent=float(np.linalg.norm(pts.max(0)-pts.min(0)))*1.18
    views=[("front",(8,180,0)),("three-quarter",(12,145,0)),("back",(8,0,0)),("side",(8,90,0))]
    sheet=Image.new("RGB",(1024,1056),(21,25,33));d=ImageDraw.Draw(sheet);d.text((12,8),"ASHEN SENTINEL V3 | OFFLINE ACTUAL MODEL",fill="white")
    for i,(name,angle) in enumerate(views):
        im=frame(rest,tex,angle,512,center,extent);im.save(out/(name+".png"));sheet.paste(im,(i%2*512,32+i//2*512))
    sheet.save(out/"model-sheet.png")
    portrait=frame(rest,tex,(6,150,0),512,center+np.array([0,extent*.15,0]),extent*.59,True)
    portrait.save(out/"portrait-bestiary.png");portrait.resize((128,128),Image.Resampling.LANCZOS).save(out/"portrait-target.png")
    if baseline and (baseline/"model-sheet.png").exists():
        old=Image.open(baseline/"model-sheet.png").convert("RGB");new=sheet
        comp=Image.new("RGB",(1024,560),(21,25,33));dr=ImageDraw.Draw(comp)
        dr.text((10,8),"V2",fill="white");dr.text((522,8),"V3",fill="white")
        comp.paste(old.resize((512,528)),(0,32));comp.paste(new.resize((512,528)),(512,32));comp.save(out/"visual-diff.png")
    clips=doc["animations"];pose_sheet=Image.new("RGB",(4*256,math.ceil(len(clips)/4)*282),(21,25,33));dr=ImageDraw.Draw(pose_sheet)
    times=[];frames=out/"frames";frames.mkdir(exist_ok=True);idx=0;segments=[]
    for i,c in enumerate(clips):
        name=c["name"].split(".")[-1];duration=float(c["length"])
        hit=next((e["at"] for e in spec["events"] if e["clip"]==name and e["kind"]=="hit"),duration*.5)
        pose=frame(polygons(rig,c["name"],hit),tex,(10,150,0),256,center,extent)
        px=i%4*256;py=i//4*282;pose_sheet.paste(pose,(px,py+24));dr.text((px+5,py+5),name,fill="white")
        start=idx
        for t in np.arange(0,duration,1/12):
            begin=time.perf_counter();im=frame(polygons(rig,c["name"],float(t)),tex,(10,150,0),320,center,extent);times.append((time.perf_counter()-begin)*1000)
            ImageDraw.Draw(im).text((8,8),name.upper(),fill="white");im.save(frames/f"{idx:04}.png");idx+=1
        segments.append(dict(clip=name,start=start,end=idx-1))
    pose_sheet.save(out/"pose-sheet.png")
    perf=dict(kind="offline_cpu_only_not_android",cubes=len(rig["cubes"]),bones=len(rig["bones"]),quads=len(rest),triangles=2*len(rest),texture_rgba_bytes=texture.width*texture.height*4*2,frames=idx,cpu_render_median_ms=float(np.median(times)),cpu_render_p95_ms=float(np.percentile(times,95)),android_fps=None)
    (out/"performance.json").write_text(json.dumps(perf,indent=2));(out/"summary.json").write_text(json.dumps(dict(kind="offline_candidate_not_game",uv={k:v for k,v in uv.items() if k!="faces_detail"},events=events,performance=perf,clips=segments,runtime_integrated=False),indent=2))
    return perf

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("model",type=Path);p.add_argument("atlas",type=Path);p.add_argument("emissive",type=Path);p.add_argument("events",type=Path);p.add_argument("out",type=Path);p.add_argument("--baseline",type=Path);a=p.parse_args();run(a.model,a.atlas,a.emissive,a.events,a.out,a.baseline)
