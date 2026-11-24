#!/usr/bin/env python3
"""Offline CPU 3-D creature inspector. No game, GPU, account or proprietary assets.
Runtime mode deliberately reproduces the current renderer's full-texture-per-face
policy; authored mode respects explicit Blockbench UV rectangles. Orthographic
projection, depth-tested textured triangles, bone pivots and linear keyframes.
This is independent evidence, not a Sunrise screenshot.
"""
import argparse, json, math, re
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw

ROOT=Path(__file__).resolve().parents[1]
FACES={"east":(1,5,6,2),"west":(4,0,3,7),"up":(3,2,6,7),"down":(4,5,1,0),"south":(5,4,7,6),"north":(0,1,2,3)}
def vec(s): return np.array([float(v.strip().removesuffix("f32")) for v in s.split(",")])
def rotation(a):
    x,y,z=np.radians(a); c,s=np.cos,np.sin
    return np.array([[c(z),-s(z),0],[s(z),c(z),0],[0,0,1]])@np.array([[c(y),0,s(y)],[0,1,0],[-s(y),0,c(y)]])@np.array([[1,0,0],[0,c(x),-s(x)],[0,s(x),c(x)]])
def around(a,p):
    m=np.eye(4);m[:3,:3]=rotation(a);m[:3,3]=p-m[:3,:3]@p
    return m
def apply(m,pts): return pts@m[:3,:3].T+m[:3,3]
def parse_generated(project):
    sources=[project/"src"/n for n in ("rustpg_generated_models.rs","rustpg_generated_hunter.rs","rustpg_creatures_generated.rs")]
    result={}
    for path in sources:
        if not path.exists(): continue
        for name,typ,body in re.findall(r'pub const (\w+)_(BONES|CUBES|KEYS)\s*:[^=]+?=\s*&\[(.*?)\];',path.read_text(),re.S):
            key={"SENTINEL":"ashen_sentinel","HUNTER":"rift_hunter"}.get(name,name.lower())
            data=result.setdefault(key,{"bones":{},"cubes":[],"keys":[]})
            if typ=="BONES":
                for n,p,o,r in re.findall(r'GeneratedBone\s*\{\s*name:\s*"([^"]+)",\s*parent:\s*(None|Some\("[^"]+"\)),\s*pivot:\s*\[([^\]]+)\],\s*rotation:\s*\[([^\]]+)\]',body):
                    data["bones"][n]=(None if p=="None" else p[6:-2],vec(o),vec(r))
            elif typ=="CUBES":
                for n,b,f,t,o,r in re.findall(r'GeneratedCube\s*\{\s*name:\s*"([^"]+)",\s*bone:\s*"([^"]+)",\s*from:\s*\[([^\]]+)\],\s*to:\s*\[([^\]]+)\],\s*origin:\s*\[([^\]]+)\],\s*rotation:\s*\[([^\]]+)\]',body):
                    data["cubes"].append(dict(name=n,bone=b,lo=vec(f),hi=vec(t),origin=vec(o),rot=vec(r)))
            else:
                for clip,b,ch,t,v in re.findall(r'GeneratedKey\s*\{\s*clip:\s*"([^"]+)",\s*bone:\s*"([^"]+)",\s*channel:\s*"([^"]+)",\s*time:\s*([^,]+),\s*value:\s*\[([^\]]+)\]',body):
                    data["keys"].append((clip,b,ch,float(t.strip().removesuffix("f32")),vec(v)))
    return result
def authored(path):
    data=json.loads(path.read_text()); bones={}; cube_bones={}
    def walk(nodes,parent=None):
        for x in nodes:
            if isinstance(x,str): cube_bones[x]=parent
            else:
                if not isinstance(x,dict): raise ValueError("unsupported outliner entry")
                name=x.get("uuid",x.get("name")); bones[name]=(parent,np.array(x.get("origin",[0,0,0])),np.array(x.get("rotation",[0,0,0])))
                walk(x.get("children",[]),name)
    walk(data.get("outliner",[]))
    res=data["resolution"]; cubes=[]
    for x in data["elements"]:
        if x.get("type","cube")!="cube": raise ValueError("only cuboid models are supported")
        if x.get("visibility",True) is False: continue
        f,t=np.array(x["from"]),np.array(x["to"]); inf=float(x.get("inflate",0))
        uv={k:[v["uv"][0]/res["width"],v["uv"][1]/res["height"],v["uv"][2]/res["width"],v["uv"][3]/res["height"]] for k,v in x["faces"].items() if v.get("texture") is not None}
        if any(v.get("rotation",0) for v in x["faces"].values()): raise ValueError("rotated UV faces not yet supported")
        cubes.append(dict(name=x["name"],bone=cube_bones.get(x["uuid"]),lo=f-inf,hi=t+inf,origin=np.array(x.get("origin",(f+t)/2)),rot=np.array(x.get("rotation",[0,0,0])),uv=uv))
    return dict(bones=bones,cubes=cubes,keys=[])
def sample(keys,b,ch,clip,t,default):
    rows=sorted((k[3],k[4]) for k in keys if k[:3]==(clip,b,ch))
    if not rows:return np.array(default)
    if t<=rows[0][0]:return rows[0][1]
    for (ta,a),(tb,v) in zip(rows,rows[1:]):
        if t<=tb:return a+(v-a)*(t-ta)/max(tb-ta,1e-8)
    return rows[-1][1]
def geometry(model,clip,t,mode):
    bones=model["bones"]; keys=model["keys"]; cache={}; visiting=set()
    def matrix(b):
        if b is None:return np.eye(4)
        if b in cache:return cache[b]
        if b in visiting:raise ValueError("cycle in rig")
        visiting.add(b);parent,p,r=bones[b]
        m=matrix(parent)@around(r+sample(keys,b,"rotation",clip,t,[0]*3),p)
        scale=sample(keys,b,"scale",clip,t,[1]*3); sm=np.eye(4);sm[:3,:3]=np.diag(scale);sm[:3,3]=p-scale*p
        m=m@sm;visiting.remove(b);cache[b]=m
        return m
    polys=[]
    for c in model["cubes"]:
        x,y,z=c["lo"]; X,Y,Z=c["hi"]
        pts=np.array([[x,y,z],[X,y,z],[X,Y,z],[x,Y,z],[x,y,Z],[X,y,Z],[X,Y,Z],[x,Y,Z]])
        pts=apply(matrix(c["bone"])@around(c["rot"],c["origin"]),pts)
        for face,indices in FACES.items():
            uv=c.get("uv",{}).get(face) if mode=="authored" else [0,0,1,1]
            if uv is None:continue
            a,b,d,e=uv
            polys.append((pts[list(indices)],np.array([[a,e],[d,e],[d,b],[a,b]]),face))
    if not polys:raise ValueError("no drawable faces")
    return polys
def triangle(buf,depth,points,uv,texture,shade):
    h,w=depth.shape
    lo=np.maximum(np.floor(points[:,:2].min(0)).astype(int),[0,0]);hi=np.minimum(np.ceil(points[:,:2].max(0)).astype(int),[w-1,h-1])
    if np.any(hi<lo):return
    yy,xx=np.mgrid[lo[1]:hi[1]+1,lo[0]:hi[0]+1]; a,b,c=points
    v0=b[:2]-a[:2]; v1=c[:2]-a[:2]; det=v0[0]*v1[1]-v0[1]*v1[0]
    if abs(det)<1e-8:return
    dx=xx+.5-a[0];dy=yy+.5-a[1]; u=(dx*v1[1]-dy*v1[0])/det;v=(dy*v0[0]-dx*v0[1])/det; q=1-u-v
    z=q*a[2]+u*b[2]+v*c[2]; old=depth[yy,xx]
    mask=(q>=-1e-6)&(u>=-1e-6)&(v>=-1e-6)&(z>old)
    if not mask.any():return
    tex=q[...,None]*uv[0]+u[...,None]*uv[1]+v[...,None]*uv[2]
    ix=np.clip((tex[...,0]*texture.shape[1]).astype(int),0,texture.shape[1]-1);iy=np.clip((tex[...,1]*texture.shape[0]).astype(int),0,texture.shape[0]-1)
    rgba=texture[iy,ix];mask &= rgba[...,3]>=128
    buf[yy[mask],xx[mask]]=np.clip(rgba[mask,:3]*shade,0,255).astype(np.uint8);depth[yy[mask],xx[mask]]=z[mask]
def render(polys,texture,angles,size=512):
    mat=rotation(angles); allpts=np.concatenate([p[0] for p in polys]); center=(allpts.min(0)+allpts.max(0))/2
    scale=size*.78/max(np.linalg.norm(allpts.max(0)-allpts.min(0)),.01)
    buf=np.zeros((size,size,3),dtype=np.uint8);buf[:]=[21,25,33];depth=np.full((size,size),-np.inf)
    light=np.array([-.35,.55,.75]);light/=np.linalg.norm(light)
    for pts,uv,face in polys:
        p=(pts-center)@mat.T
        normal=np.cross(p[1]-p[0],p[2]-p[0]);length=np.linalg.norm(normal)
        if length<1e-9:continue
        shade=.56+.44*abs(np.dot(normal/length,light))
        p[:,0]=p[:,0]*scale+size/2;p[:,1]=-p[:,1]*scale+size/2
        for tri in ([0,1,2],[0,2,3]):triangle(buf,depth,p[tri],uv[tri],texture,shade)
    if not np.isfinite(depth).any():raise ValueError("blank render")
    return Image.fromarray(buf),int(np.isfinite(depth).sum())
def main():
    ap=argparse.ArgumentParser();ap.add_argument("--project",type=Path,default=ROOT/"project");ap.add_argument("--model",type=Path);ap.add_argument("--texture",type=Path);ap.add_argument("--creature");ap.add_argument("--clip");ap.add_argument("--time",type=float,default=0);ap.add_argument("--output",type=Path,default=Path("creature-inspection"));a=ap.parse_args()
    if bool(a.model)!=bool(a.texture):ap.error("--model and --texture must be supplied together")
    a.output.mkdir(parents=True,exist_ok=True); jobs=[]
    if a.model:jobs.append((a.model.stem,authored(a.model),a.texture,"authored"))
    else:
        roster=parse_generated(a.project)
        if not roster:raise ValueError("no generated Rust rigs")
        for name,data in roster.items():
            if a.creature and name!=a.creature:continue
            texture=a.project/"assets/rustpg/textures/entity"/(name+".png")
            jobs.append((name,data,texture,"runtime"))
            model=a.project/"models/creatures"/(name+".bbmodel")
            if model.exists():jobs.append((name,authored(model),texture,"authored"))
    if not jobs:raise ValueError("no matching model")
    reports=[]
    for name,data,path,mode in jobs:
        texture=np.array(Image.open(path).convert("RGBA")); clips=sorted(set(k[0] for k in data["keys"]))
        frames=[("rest",None,0)]
        if mode=="runtime":
            for clip in ([a.clip] if a.clip else clips):
                length=max(k[3] for k in data["keys"] if k[0]==clip)
                frames.extend((clip.split(".")[-1]+"-"+str(i),clip,t) for i,t in enumerate([a.time] if a.clip else [0,length*.5]))
        for label,clip,t in frames:
            polys=geometry(data,clip,t,mode)
            views=[];counts=[]
            for view,angle in [("front",(10,180,0)),("three-quarter",(15,135,0)),("back",(10,0,0)),("side",(10,90,0))]:
                img,count=render(polys,texture,angle);views.append(img);counts.append(count)
                img.save(a.output/f"{name}-{mode}-{label}-{view}.png")
            sheet=Image.new("RGB",(1024,1060),(21,25,33));d=ImageDraw.Draw(sheet);d.text((12,8),f"{name} | {mode} | {label} | independent mesh preview",fill="white")
            for i,img in enumerate(views):sheet.paste(img,((i%2)*512,36+(i//2)*512))
            sheet.save(a.output/f"{name}-{mode}-{label}-sheet.png")
            reports.append(dict(creature=name,mode=mode,clip=clip,time=t,cubes=len(data["cubes"]),bones=len(data["bones"]),visible_pixels=counts,texture_size=list(texture.shape[:2][::-1]),texture=str(path),full_texture_per_face=mode=="runtime"))
    (a.output/"report.json").write_text(json.dumps({"kind":"offline_mesh_preview_not_game","reports":reports},indent=2))
    print("Rendered",len(reports),"pose sheets")
if __name__=="__main__":main()
