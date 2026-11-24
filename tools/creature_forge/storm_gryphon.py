#!/usr/bin/env python3
"""Storm Gryphon production candidate: original avian/feline anatomy and flight poses.
Uses pinned Blockbench MCP, existing IK math, pixel QA, fixed camera and Rust export.
All evidence is offline, not an Android screenshot or implemented game behavior."""
import argparse,hashlib,json,math,os,random,subprocess,sys,time
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/"tools/asset_studio"),str(ROOT/"tools")]
from motion_lab import Rig,align,euler,rot,curve,self_test
from studio_v2 import load,polygons,frame,uv_check,validate_events,rust_export
from creature_inspector import rotation,sample,around
P=argparse.ArgumentParser();P.add_argument("--out",type=Path,required=True);P.add_argument("--preflight",action="store_true")
a=P.parse_args();out=a.out.resolve();out.mkdir(parents=True,exist_ok=True)
G=[];C=[];name="storm_gryphon";model=name+".bbmodel";atlas_name=name+".png";N=512
def group(n,o,parent=None):
    g=dict(op="add_group",name=n,origin=list(o))
    if parent:g["parent"]=parent
    G.append(g)
def box(n,b,lo,hi,mat,angle=None,pivot=None):
    c=dict(op="add_cube",name=n,parent=b,**{"from":list(lo),"to":list(hi)},box_uv=False)
    if angle is not None:c["rotation"]=[float(v) for v in angle]
    if pivot is not None:c["origin"]=list(pivot)
    C.append((c,mat))
def segment(n,b,A,B,width,depth,mat):
    A=np.array(A,float);B=np.array(B,float);mid=(A+B)/2;length=np.linalg.norm(B-A)
    half=np.array([width/2,length/2,depth/2]);ang=euler(align([0,1,0],B-A))
    box(n,b,mid-half,mid+half,mat,ang,mid)
# Body axes: front -Z, up +Y. Feline rear quarters, raptor forelimbs.
group("root",[0,0,0]);group("body",[0,13,0],"root");group("chest",[0,15,-5],"body")
group("neck",[0,18,-8],"chest");group("head",[0,22,-12],"neck")
box("barrel","body",[-5.1,9.8,-2.8],[5.1,17.4,8.4],"fur")
box("loin","body",[-3.8,10.2,6],[3.8,16.3,11.1],"fur")
box("dorsal_coat","body",[-4.45,16.6,-2.3],[4.45,18.1,8.6],"slate")
box("belly","body",[-3.8,9.1,-1],[3.8,11,7.2],"fur")
box("breast","chest",[-5.9,9.9,-9.8],[5.9,19,-1.6],"slate")
box("breast_shield","chest",[-4.2,12,-11],[4.2,18.6,-8.8],"cream")
segment("keel","chest",[0,11,-10.0],[0,17.9,-10.9],2.3,1.45,"slate")
box("neck_base","neck",[-3.9,16.8,-12.8],[3.9,21.9,-6.9],"slate")
box("throat","neck",[-2.95,17.0,-13.6],[2.95,22.0,-11],"ivory")
box("skull","head",[-3.4,20.7,-15.8],[3.4,25.7,-10.1],"slate")
box("crown","head",[-2.95,25.2,-15.4],[2.95,26.3,-10.4],"obsidian")
box("upper_beak","head",[-1.95,21.5,-18.1],[1.95,24,-15.0],"darkhorn")
box("beak_ridge","head",[-1.22,23.5,-18.05],[1.22,24.35,-15.4],"horn")
segment("beak_hook","head",[0,22.65,-17.8],[0,20.4,-19.25],2.05,1.65,"darkhorn")
group("jaw",[0,21,-14.6],"head")
box("lower_beak","jaw",[-1.5,20.1,-18.1],[1.5,21.5,-14.6],"darkhorn")
group("beak_fx",[0,22,-19.4],"head")
group("chest_fx",[0,16,-10.4],"chest");group("ground_fx",[0,0,-6],"root")
group("tail",[0,14,9.5],"body")
prev="tail"
for i,(A,B,w) in enumerate([([0,14,9.5],[0,13.3,14.4],2.2),([0,13.3,14.4],[0,13.6,19],1.5),([0,13.6,19],[0,15.2,23],1.15)]):
    b="tail_"+str(i);group(b,A,prev);segment(b+"_shaft",b,A,B,w,w,"fur");prev=b
group("tail_fan",[0,15.2,23],prev)
for i in range(5):
    segment("tail_quill_"+str(i),"tail_fan",[0,15.2,22.6],[(i-2)*1.4,16.4,27-abs(i-2)*.7],1.4,.62,"slate" if i%2 else "feather")
for side,s in [("l",-1),("r",1)]:
    def mir(v):return [s*v[0],v[1],v[2]]
    def sg(n,b,A,B,w,d,mat):segment(side+"_"+n,b,mir(A),mir(B),w,d,mat)
    def bx(n,b,A,B,mat,ang=None,piv=None):
        A,B=mir(A),mir(B);lo=[min(x,y) for x,y in zip(A,B)];hi=[max(x,y) for x,y in zip(A,B)]
        box(side+"_"+n,b,lo,hi,mat,[ang[0],s*ang[1],s*ang[2]] if ang else None,mir(piv) if piv else None)
    bx("cheek","head",[2.35,20.65,-15.15],[3.95,23.7,-10.35],"ivory",[0,0,-12],[2.8,23,-12])
    bx("eye_socket","head",[2.9,23,-15.65],[3.83,24.35,-12.8],"obsidian")
    bx("iris","head",[3.78,23.35,-15.1],[4.02,23.91,-13.35],"electric",[0,0,0])
    bx("pupil","head",[4.0,23.35,-14.55],[4.08,23.91,-14.13],"joint")
    sg("brow","head",[2.95,23.9,-15.95],[3.9,25.05,-12.5],1.25,1.1,"obsidian")
    bx("nostril","head",[1.91,23.0,-16.95],[2.04,23.4,-16.32],"joint")
    crest="crest_"+side;group(crest,mir([2.4,25.5,-10.7]),"head")
    for j in range(3):sg("crest_plume"+str(j),crest,[1.5+j*.45,25.5,-11.2+j*.35],[2.9+j*.66,28.15-j*.58,-6.7+j*.5],1.12,.79,"slate" if j%2 else "obsidian")
    for j in range(5):
        sg("neck_hackle"+str(j),"neck",[2.5+j*.20,22.0-j*.92,-11.4+j*.61],[4.3+j*.29,18.2-j*.76,-7.2+j*.69],1.75,1.06,"ivory" if j<2 else "slate")
    for j in range(3):
        sg("shoulder_plume"+str(j),"chest",[4.45,18.0-j*.96,-7.9+j*.42],[6.05,14.2-j*.93,-4.5+j*.7],2.05,1.5,"slate" if j!=1 else "feather")
    # Thick feline thigh and hock; raptor front leg with independent toes.
    for typ,A,B,F in [("front",[3.6,14,-6.7],[4.2,7.8,-7.9],[4.1,1.35,-6.2]),("hind",[3.7,13.6,7.7],[4.4,7.1,4.6],[4.2,1.35,8.1])]:
        leg=typ+"_leg_"+side;shin=typ+"_shin_"+side;foot=typ+"_foot_"+side
        group(leg,mir(A),"chest" if typ=="front" else "body");group(shin,mir(B),leg);group(foot,mir(F),shin)
        sg(typ+"_upper",leg,A,B,4.1 if typ=="front" else 4.8,4.0 if typ=="front" else 4.8,"slate" if typ=="front" else "fur")
        sg(typ+"_lower",shin,B,F,2.18 if typ=="front" else 2.8,2.2 if typ=="front" else 2.9,"darkhorn" if typ=="front" else "fur")
        if typ=="front":
            for j in range(3):
                p=[4.1+(j-1)*.55,6.6-j*.9,-8.1+j*.2];sg("scale"+str(j),shin,p,[p[0],p[1]-.55,p[2]+.1],1.6,.7,"darkhorn")
        bx(typ+"_pad",foot,[F[0]-1.65,.18,F[2]-1.6],[F[0]+1.65,1.8,F[2]+1.22],"darkhorn" if typ=="front" else "fur")
        for j in range(3):
            x=F[0]+(j-1)*1.02;toe=typ+"_toe_"+side+"_"+str(j);group(toe,mir([x,1.05,F[2]-.7]),foot)
            sg(typ+"_digit"+str(j),toe,[x,1.15,F[2]-.6],[x,.8,F[2]-2.0],.85,.88,"darkhorn" if typ=="front" else "fur")
            sg(typ+"_claw"+str(j),toe,[x,.9,F[2]-1.85],[x,.48,F[2]-3.05],.68,.65,"talon")
            sg(typ+"_claw_hook"+str(j),toe,[x,.48,F[2]-3.02],[x,.24,F[2]-3.45],.43,.42,"talon")
        group(typ+"_talon_fx_"+side,mir([F[0],.75,F[2]-2.6]),foot)
    # Four linked wing sections. Shape is a tapered swept feather fan, not a flat slab.
    A=[4.4,18.3,-5.2];B=[11.4,19.3,-4.5];D=[18.6,19.0,-1.8];E=[25.0,18.5,2.5]
    w="wing_"+side;el="wing_elbow_"+side;wr="wing_wrist_"+side;tip="wing_tip_"+side
    for n,p,parent in [(w,A,"chest"),(el,B,w),(wr,D,el),(tip,E,wr)]:group(n,mir(p),parent)
    sg("wing_humerus",w,A,B,3.5,4.3,"slate");sg("wing_forearm",el,B,D,2.85,3.4,"slate");sg("wing_hand",wr,D,E,1.85,2.8,"slate")
    # Feather architecture: layered coverts and tapered vanes, with narrower root spacing.
    # Flat face patterning supplies small detail; avoid a forest of duplicate overlapping rods.
    for j in range(5):
        x=5.5+j*1.23;z=-4.6+j*.14
        sg("secondary_"+str(j),w,[x,18.8,z],[x+.75,17.9,z+10.1-j*.24],1.62,.72,"feather")
        sg("covert_"+str(j),w,[x,19.65,z],[x+.5,19.15,z+4.8],1.48,.62,"slate")
    for j in range(5):
        x=11.65+j*1.3;z=-4.2+j*.40
        sg("outer_secondary_"+str(j),el,[x,19,z],[x+1.4,17.9,z+11.6+j*.24],1.7,.76,"feather")
        sg("outer_covert_"+str(j),el,[x,19.8,z],[x+.7,19.0,z+5.0],1.42,.62,"slate")
    # Swept seven-finger silhouette. Three tapered sections per primary; luminescence
    # painted at the ends rather than adding separate little blocks.
    endpoints=[[23,17.7,12.5],[26.6,17.65,13.4],[29.7,17.55,12.4],[32.3,17.45,10.8],[34.2,17.5,8.4],[35.0,17.6,5.4],[34.4,17.75,2.7]]
    for j,end in enumerate(endpoints):
        x=18.05+j*.82;z=-1.9+j*.36;start=[x,18.8,z]
        feather="primary_"+side+"_"+str(j);group(feather,mir(start),wr)
        mid=[start[k]+(end[k]-start[k])*.56 for k in range(3)]
        cut=[start[k]+(end[k]-start[k])*.87 for k in range(3)]
        sg("primary_root"+str(j),feather,start,mid,1.72,.73,"slate")
        sg("primary_vane"+str(j),feather,mid,cut,1.32,.59,"feather")
        sg("primary_tip"+str(j),feather,cut,end,.76,.43,"stormtip")
    # Wing tip is still articulated; a short leading feather follows that group.
    sg("wingtip_quill",tip,E,[31.3,18.0,3.1],1.04,.63,"obsidian")
    group("wing_fx_"+side,mir([23,18,5]),wr)
# Lower the neck and head, move the entire sub-rig forward. Do not scale or
# stretch keyframed bones. Art and sockets are transformed together in rest space.
def descendants(names):
    result=set(names)
    for g in G:
        if g.get("parent") in result:result.add(g["name"])
    return result
headset=descendants(["head"]);neckset=descendants(["neck"])
# Shift the skull 1.5 units farther forward and lower the full neck assembly.
for g in G:
    if g["name"] in neckset:
        delta=np.array([0,-2.0,-2.1 if g["name"] in headset else -.75])
        g["origin"]=(np.array(g["origin"])+delta).tolist()
for c,_ in C:
    if c["parent"] in neckset:
        delta=np.array([0,-2.0,-2.1 if c["parent"] in headset else -.75])
        for k in ("from","to","origin"):
            if k in c:c[k]=(np.array(c[k])+delta).tolist()
# Real texture direction: feather vanes use shafts and tapered barb stripes;
# feline coat has clustered fur strokes. Avoid the old identical diagonal stripe on every cube.
palette={"fur":(38,44,56),"cream":(105,119,132),"ivory":(187,203,214),"slate":(30,38,56),"feather":(47,63,90),"obsidian":(19,24,38),"stormtip":(74,132,185),"horn":(141,121,84),"darkhorn":(75,69,62),"talon":(24,31,42),"joint":(16,21,31),"electric":(101,218,255)}
def tile(w,h,mat,n,f):
    im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im)
    rng=random.Random(int(hashlib.sha256((n+"/"+f).encode()).hexdigest()[:16],16));base=palette[mat]
    for y in range(h):
        for x in range(w):
            u=x/max(1,w-1);v=y/max(1,h-1)
            shade=round(10*(1-v)-6*v+7*math.sin(math.pi*u))+rng.choice([-3,-1,0,0,1,3])
            if mat in ("slate","feather","stormtip","obsidian"):
                shade+=int(11*(1-abs(u-.5)*2))-int(10*v)
                if f in ("north","south","east","west") and h>=5:
                    # feather barb pattern slants toward center shaft
                    if (y+int(abs(x-(w-1)/2)*.80))%4==0:shade-=8
                if mat=="stormtip":shade+=int(24*v)
            if mat in ("fur","cream","ivory"):
                shade+=4 if ((x//2+y//3)%3)==0 else -2
                if y%4==0 and rng.random()<.35:shade-=7
            if mat in ("horn","darkhorn","talon"):shade+=7 if x==w//2 else -4 if x in (0,w-1) else 0
            if mat=="electric":shade+=24
            d.point((x,y),fill=tuple(max(0,min(255,k+shade)) for k in base)+(255,))
    if mat in ("slate","feather","stormtip") and h>=6 and w>=3:
        shaft=tuple(min(255,k+21) for k in base)+(255,)
        d.line((w//2,0,w//2,h-1),fill=shaft)
        if mat=="stormtip":
            d.line((w//2,h//2,w//2,h-1),fill=(132,208,247,255))
    if mat=="electric" and w>=2:
        d.line((w//2,0,w//2,h-1),fill=(209,246,255,255))
    return im
def extrude(im,t,x,y):
    w,h=t.size;im.paste(t,(x,y))
    for dx,dy,rect in [(0,-1,(0,0,w,1)),(0,h,(0,h-1,w,h)),(-1,0,(0,0,1,h)),(w,0,(w-1,0,w,h))]:im.paste(t.crop(rect),(x+dx,y+dy))
    for dx,dy,tx,ty in [(-1,-1,0,0),(w,-1,w-1,0),(-1,h,0,h-1),(w,h,w-1,h-1)]:im.putpixel((x+dx,y+dy),t.getpixel((tx,ty)))
atlas=Image.new("RGBA",(N,N));em=Image.new("RGBA",(N,N));faces=[];layout=[]
for c,mat in C:
    x,y,z=[v-u for u,v in zip(c["from"],c["to"])]
    for f,(w,h) in dict(north=(x,y),south=(x,y),east=(z,y),west=(z,y),up=(x,z),down=(x,z)).items():faces.append((max(2,math.ceil(w*2)),max(2,math.ceil(h*2)),c,mat,f))
x=y=1;row=0
for w,h,c,mat,f in sorted(faces,key=lambda e:(-e[1],-e[0],e[2]["name"],e[4])):
    if x+w>=N:x=1;y+=row+2;row=0
    if y+h>=N:raise ValueError("atlas overflow")
    t=tile(w,h,mat,c["name"],f);extrude(atlas,t,x,y)
    if mat=="electric":extrude(em,t,x,y)
    elif mat=="stormtip":
        mask=Image.new("RGBA",(w,h));md=ImageDraw.Draw(mask)
        md.line((w//2,h//2,w//2,h-1),fill=(116,194,243,175),width=1);extrude(em,mask,x,y)
    uv=[x,y,x+w,y+h];c.setdefault("faces",{})[f]=dict(uv=uv,texture=atlas_name)
    layout.append(dict(part=c["name"],face=f,material=mat,uv=uv));x+=w+2;row=max(row,h)
atlas.save(out/atlas_name);em.save(out/(name+"_emissive.png"));(out/"uv-layout.json").write_text(json.dumps(layout,indent=2))
# Flight and ground animations are separate; all numbers are model units and seconds.
clips=[("idle",2.4,True),("threat_display",2.6,False),("walk",1.12,True),("gallop",.72,True),("takeoff",1.5,False),("flight",1.05,True),("glide",2.0,True),("land",1.55,False),("talon_left",1.3,False),("talon_right",1.3,False),("beak_strike",1.15,False),("wing_gust",1.8,False),("storm_breath",2.4,False),("dive",2.2,False),("stagger",.7,False),("death",2.5,False)]
attacks=[];events=[]
def event(clip,at,kind,socket,effect):
    events.append(dict(clip=clip,at=at,kind=kind,socket=socket,effect=effect,id=clip+"_"+str(len(events))))
def attack(clip,wind,end,cd,shape,r,mult,hit,sockets,fx):
    attacks.append(dict(clip=clip,windup=wind,active_end=end,cooldown=cd,shape=shape,range=r,damage_multiplier=mult))
    for t,s in zip(hit,sockets):
        event(clip,max(.04,t-.38),"vfx",s,"charge");event(clip,t,"hit",s,shape);event(clip,t,"vfx",s,fx);event(clip,t,"sound",s,"gryphon_"+fx)
attack("talon_left",.54,.80,2.1,"arc",3.2,1.15,[.65],["front_talon_fx_l"],"claw")
attack("talon_right",.54,.80,2.1,"arc",3.2,1.15,[.65],["front_talon_fx_r"],"claw")
attack("beak_strike",.45,.69,2.0,"pierce",3.5,1.3,[.55],["beak_fx"],"claw")
attack("wing_gust",.82,1.06,5.0,"cone",5.5,.85,[.92],["chest_fx"],"gust")
attack("storm_breath",1.08,1.70,7.5,"lightning_cone",9,1.45,[1.25,1.55],["beak_fx","beak_fx"],"lightning")
attack("dive",1.03,1.35,6.5,"impact",4.5,1.9,[1.18],["front_talon_fx_l"],"impact")
event("threat_display",.64,"vfx","wing_fx_l","charge");event("threat_display",.64,"vfx","wing_fx_r","charge");event("threat_display",1.15,"vfx","chest_fx","gust");event("threat_display",1.2,"sound","beak_fx","gryphon_roar");event("takeoff",.55,"vfx","ground_fx","gust");event("land",1.0,"vfx","ground_fx","impact");event("death",1.7,"vfx","ground_fx","dust")
spec=dict(schema=1,creature=name,front="-Z",authority="server",runtime_integrated=False,boss=dict(optional=True,portrait=False,phase_threshold=.35),attacks=attacks,events=events)
(out/"events.json").write_text(json.dumps(spec,indent=2))
self_test();rig=Rig(G);rest={b:rig.pivot(b) for b in rig.groups};keys=[];checks=[];pose_data={};eye=np.eye(3)
# Cache transformed cuboid corners to constrain the death clip against the floor.
corner_cache=[]
for c,_ in C:
    lo=np.array(c["from"]);hi=np.array(c["to"]);o=np.array(c.get("origin",(lo+hi)/2))
    verts=np.array([[x,y,z] for x in (lo[0],hi[0]) for y in (lo[1],hi[1]) for z in (lo[2],hi[2])])
    verts=(verts-o)@rot(c.get("rotation",[0,0,0])).T+o
    corner_cache.append((c["parent"],verts))
def floor_height():
    matrices={}
    minimum=1e9
    for bone,verts in corner_cache:
        if bone not in matrices:matrices[bone]=rig.world(bone)
        r,t=matrices[bone];minimum=min(minimum,float((verts@r.T+t)[:,1].min()))
    return minimum
def scalar(t,rows):return float(curve([(x,[y]) for x,y in rows],t)[0])
for clip,L,loop in clips:
    full="animation."+name+"."+clip;keys.append(dict(op="add_animation",name=full,length=L,loop="loop" if loop else "once",snapping=24))
    ts=sorted({round(i*L/math.ceil(L*16),5) for i in range(math.ceil(L*16)+1)}|{e["at"] for e in events if e["clip"]==clip});tracks={};prev={}
    for t in ts:
        u=t/L;rig.reset()
        root=np.array([0.,-1.7,0.]);pitch=0.;roll=0.;yaw=0.;fold=1.;flap=0.;neck=0.;jaw=0.;air=0.
        footgoals={};active_leg=None
        if clip in ("walk","gallop"):
            stride=2.8 if clip=="walk" else 4.5;root[1]+=.12*math.cos(4*math.pi*u);pitch=1.4*math.sin(math.tau*u)
            for side in ("l","r"):
                for typ in ("front","hind"):
                    # Diagonal walking pairs, gallop with fore/hind pairs offset.
                    phase=(u+(.5 if (side=="r")!=(typ=="hind") else 0))%1 if clip=="walk" else (u+(.40 if typ=="hind" else 0)+(.12 if side=="r" else 0))%1
                    p=rest[typ+"_foot_"+side].copy()
                    if phase<.64:p[2]+=-stride/2+stride*phase/.64
                    else:
                        v=(phase-.64)/.36;p[2]+=stride/2-stride*v;p[1]+=1.65*math.sin(math.pi*v)
                    footgoals[typ,side]=p
        elif clip=="threat_display":
            q=scalar(t,[(0,0),(.48,.25),(1.18,1),(1.8,1),(L,0)])
            root[1]-=1.45*q;pitch=-6*q;neck=-11*q;fold=1-.82*q;flap=37*q;jaw=29*q
        elif clip in ("flight","glide"):
            air=1;fold=0;root[1]=4.5+.4*math.sin(math.tau*u);pitch=-3;flap=42*math.sin(math.tau*u) if clip=="flight" else 5*math.sin(math.tau*u)
        elif clip=="takeoff":
            air=scalar(t,[(0,0),(.48,0),(.9,1),(L,1)])
            root[1]=scalar(t,[(0,-1.7),(.42,-3.4),(.75,.2),(L,4.5)])
            fold=1-scalar(t,[(0,0),(.4,.3),(.85,1),(L,1)])
            flap=scalar(t,[(0,0),(.42,34),(.76,-46),(1.05,36),(L,0)]);pitch=scalar(t,[(0,0),(.5,15),(L,-3)])
        elif clip=="land":
            air=scalar(t,[(0,1),(.7,1),(1.05,0),(L,0)])
            root[1]=scalar(t,[(0,4.5),(.7,1.2),(1.0,-2.7),(L,-1.7)])
            fold=scalar(t,[(0,0),(.95,.15),(L,1)]);flap=scalar(t,[(0,12),(.5,42),(.85,-24),(L,0)]);pitch=scalar(t,[(0,-3),(.8,13),(L,0)])
        elif clip.startswith("talon_"):
            active_leg="l" if clip.endswith("left") else "r";s=-1 if active_leg=="l" else 1
            q=scalar(t,[(0,0),(.48,1),(.65,.75),(1.0,0),(L,0)])
            root[1]-=.5*q;roll=-s*4*q;pitch=-3*q;fold=1-.25*q
            p=rest["front_foot_"+active_leg]
            footgoals["front",active_leg]=curve([(0,p),(.48,[s*5.5,6,-3.7]),(.65,[s*3.8,6,-12.8]),(.80,[s*1.8,3.1,-11.2]),(L,p)],t)
        elif clip=="beak_strike":
            q=scalar(t,[(0,0),(.38,-1),(.55,1),(.75,.5),(L,0)])
            neck=-19*q;pitch=-8*q;root[1]-=.4*max(0,q);jaw=12*max(0,q)
        elif clip=="wing_gust":
            fold=1-scalar(t,[(0,0),(.68,1),(.98,1),(L,0)])
            flap=scalar(t,[(0,0),(.68,50),(.92,-35),(1.13,-15),(L,0)])
            pitch=scalar(t,[(0,0),(.68,9),(.92,-12),(L,0)]);root[1]-=.6*math.sin(math.pi*u)
        elif clip=="storm_breath":
            q=scalar(t,[(0,0),(.93,1),(1.55,.8),(L,0)])
            neck=9*q;pitch=4*q;fold=1-.55*q;flap=12*q;jaw=24*q
        elif clip=="dive":
            air=scalar(t,[(0,1),(1.0,1),(1.35,0),(L,0)])
            root[1]=scalar(t,[(0,5.0),(.8,6.0),(1.18,-1.0),(1.45,-2.5),(L,-1.7)])
            pitch=scalar(t,[(0,-4),(.8,-34),(1.18,-12),(L,0)])
            fold=scalar(t,[(0,0),(.8,.62),(1.18,.18),(L,1)])
            flap=scalar(t,[(0,5),(.8,4),(1.18,-30),(L,0)])
            for sgn,side in [(-1,"l"),(1,"r")]:
                p=rest["front_foot_"+side]
                footgoals["front",side]=curve([(0,[sgn*4.2,10,-5]),(.75,[sgn*4.2,7,-6]),(1.18,[sgn*4.8,2,-11]),(1.45,p),(L,p)],t)
        elif clip=="stagger":root[1]-=1.3*math.sin(math.pi*u);roll=6*math.sin(math.pi*u);neck=10*math.sin(math.pi*u)
        elif clip=="death":
            q=scalar(t,[(0,0),(.5,.2),(1.6,1),(L,1)]);root[1]-=7*q;roll=20*q;pitch=-9*q;fold=1-.5*q;neck=-25*q
        else:root[1]-=.30+.17*math.sin(math.tau*u);neck=-5+2*math.sin(math.tau*u);fold=.90;flap=4*math.sin(math.tau*u)
        rig.p["root"]=root;rig.r["body"]=[pitch,yaw,roll];rig.r["chest"]=[pitch*.12,0,-roll*.2]
        rig.r["neck"]=[neck,0,0];rig.r["head"]=[-neck*.26,3*math.sin(math.tau*u) if clip=="idle" else 0,0];rig.r["jaw"]=[-jaw,0,0]
        rig.r["tail"]=[-pitch*.65,5*math.sin(math.tau*u),-roll*.6]
        for j in range(3):rig.r["tail_"+str(j)]=[3*math.sin(math.tau*u-j*.6),4*math.sin(math.tau*u-j*.8),0]
        rig.r["tail_fan"]=[2*math.sin(math.tau*u),0,0]
        for side,s in [("l",-1),("r",1)]:
            rig.r["crest_"+side]=[4*math.sin(math.tau*u+s*.6),0,s*2]
            # Wing folds back with a positive Z trailing direction; mirrored flapping lifts both wings.
            rig.r["wing_"+side]=[0,-s*52*fold,s*(18*fold+flap)]
            rig.r["wing_elbow_"+side]=[0,-s*45*fold,s*(-flap*.42-24*fold)]
            rig.r["wing_wrist_"+side]=[0,s*23*fold,s*(-flap*.20+5*fold)]
            rig.r["wing_tip_"+side]=[0,s*12*fold,s*(-flap*.15)]
            for j in range(7):rig.r["primary_"+side+"_"+str(j)]=[0,s*(j-3)*1.8*(1-fold),s*(flap*.16+1.4*math.sin(math.tau*u-j*.35))]
            for typ in ("front","hind"):
                l=typ+"_leg_"+side;k=typ+"_shin_"+side;f=typ+"_foot_"+side;target=footgoals.get((typ,side),rest[f].copy())
                if air and (typ,side) not in footgoals:
                    tucked=np.array([s*4.2,root[1]+6.2, -5.5 if typ=="front" else 9.0])
                    target=target*(1-air)+tucked*air
                if clip=="death":target[0]+=s*1.7*u
                err=rig.ik(l,k,f,target,[s*.25,0,-1 if typ=="hind" else 1],eye)
                if air<.001 and clip not in ("death","dive") and (typ,side) not in footgoals:
                    checks.append(dict(clip=clip,t=t,kind="stance_contact",socket=f,error=round(err,4),passed=bool(err<.35)))
                for j in range(3):rig.r[typ+"_toe_"+side+"_"+str(j)]=[(-18*air if typ=="front" else -7*air),0,0]
        if clip=="death":
            low=floor_height()
            if low<.05:rig.p["root"][1]+=.05-low
            checks.append(dict(clip=clip,t=t,kind="death_floor",height=round(floor_height(),4),passed=bool(floor_height()>=-.001)))
        for e in events:
            if e["clip"]==clip and e["kind"]=="hit" and abs(t-e["at"])<.0001:
                pos=rig.pivot(e["socket"])
                limit=-8 if "talon" in e["socket"] else -8 if e["socket"]=="chest_fx" else -15
                checks.append(dict(clip=clip,t=t,kind="attack_forward",socket=e["socket"],position=pos.tolist(),passed=bool(pos[2]<limit)))
        for ch,data in [("rotation",rig.r),("position",rig.p)]:
            for b,v in data.items():
                v=np.asarray(v,float);k=(b,ch)
                if ch=="rotation" and k in prev:v=prev[k]+(v-prev[k]+180)%360-180
                prev[k]=v;tracks.setdefault(k,[]).append((t,v))
    # Constant channels are encoded with only two keys. Dynamic ones preserve the sampled curves.
    for (b,ch),rows in tracks.items():
        selected=rows if any(np.linalg.norm(v-rows[0][1])>.001 for _,v in rows[1:]) else [rows[0],rows[-1]]
        for t,v in selected:keys.append(dict(op="set_keyframe",animation=full,bone=b,channel=ch,time=t,value=[round(float(x),5) for x in v],interpolation="linear"))
    pose_data[clip]=len(ts)
fail=[x for x in checks if not x["passed"]]
report=dict(front="-Z",bones=len(G),cubes=len(C),faces=len(layout),clips=len(clips),keyframes=sum(k["op"]=="set_keyframe" for k in keys),pose_samples=pose_data,checks=len(checks),failures=fail,attack_checks=[x for x in checks if x["kind"]=="attack_forward"],max_stance_error=max(x["error"] for x in checks if x["kind"]=="stance_contact"))
(out/"motion-report.json").write_text(json.dumps(report,indent=2))
print(json.dumps({k:v for k,v in report.items() if k!="failures"},indent=2),flush=True)
if fail:raise ValueError("motion gate failures: "+str(fail[:4]))
if a.preflight:sys.exit(0)
env=dict(os.environ,RUSTPG_ASSET_ROOT=str(out))
def call(tool,args,log):
    f=out/(log+"-args.json");f.write_text(json.dumps(args));r=subprocess.run(["bash",str(ROOT/"tools/asset_studio/bb.sh"),tool,"@"+str(f)],env=env,capture_output=True,text=True)
    (out/(log+".json")).write_text(r.stdout)
    if r.returncode:raise RuntimeError(tool+": "+r.stdout[-2500:]+" "+r.stderr[-2500:])
    return json.loads(r.stdout)
rev=call("bbmodel_create",dict(file=model,name="Storm Gryphon",format="free",box_uv=False,resolution=dict(width=N,height=N)),"01-create")["revision"]
rev=call("bbmodel_add_texture",dict(file=model,expected_revision=rev,image=atlas_name,name=atlas_name),"02-texture")["revision"]
# Keep each animation's creation and keys in order. Batches are actual MCP edits.
for i,batch in enumerate([G,[c for c,_ in C]]+[keys[j:j+350] for j in range(0,len(keys),350)]):
    rev=call("bbmodel_edit",dict(file=model,expected_revision=rev,operations=batch),"03-edit-"+str(i))["revision"]
call("bbmodel_validate",dict(file=model,self_test=True),"04-geometry")
call("bbmodel_validate_animations",dict(file=model),"05-animation")
doc,data,names=load(out/model);uv=uv_check(doc,atlas,em,out);event_report=validate_events(doc,spec,names);rust_export(doc,data,names,spec,out)
(out/"ashen_sentinel_v3_generated.rs").rename(out/"storm_gryphon_generated.rs")
# Use saved model, not the geometry recipe, for all evidence.
tex=np.array(atlas);rest_polys=polygons(data,None,0)
pts=np.concatenate([p[0] for p in rest_polys]);center=(pts.min(0)+pts.max(0))/2;extent=float(max(np.ptp(pts[:,0]),np.ptp(pts[:,1]),np.ptp(pts[:,2])))*1.08;center[1]+=2.0
# Independent posed socket projection, same transform convention as offline renderer.
byname={v:k for k,v in names.items()}
def socket(clip,t,n):
    def mat(b):
        if b is None:return np.eye(4)
        parent,p,r=data["bones"][b];tr=np.eye(4);tr[:3,3]=sample(data["keys"],b,"position",clip,t,[0]*3)
        return mat(parent)@tr@around(r+sample(data["keys"],b,"rotation",clip,t,[0]*3),p)
    b=byname[n];return (mat(b)@np.append(data["bones"][b][1],1))[:3]
# Validate exported animation sockets again independently of the authoring solver.
saved_checks=[]
for e in events:
    if e["kind"]=="hit":
        pos=socket("animation."+name+"."+e["clip"],e["at"],e["socket"])
        saved_checks.append(dict(clip=e["clip"],socket=e["socket"],position=pos.tolist(),passed=bool(pos[2]<(-8 if "talon" in e["socket"] or e["socket"]=="chest_fx" else -15))))
(out/"saved-contact-report.json").write_text(json.dumps(saved_checks,indent=2))
assert all(e["passed"] for e in saved_checks)
from gryphon_fx import build,overlay
build(out)
# Fit every still to the actual projected silhouette (no tiny creatures in giant sheets).
def fitted(poly,angle,size=512,margin=1.08):
    pp=np.concatenate([f[0] for f in poly]);m=rotation(angle);q=pp@m.T
    mid=(q.min(0)+q.max(0))/2;ctr=mid@m
    span=max(np.ptp(q[:,0]),np.ptp(q[:,1]))*margin
    return frame(poly,tex,angle,size,ctr,span)
idle_polys=polygons(data,"animation."+name+".idle",.65)
hero_polys=polygons(data,"animation."+name+".threat_display",1.35)
sheet=Image.new("RGB",(1024,1080),(21,25,33));d=ImageDraw.Draw(sheet);d.text((12,8),"STORM GRYPHON: TEMPEST | ACTUAL OFFLINE MODEL",fill="white")
for i,(view,angle,poly) in enumerate([("hero",(8,145,0),hero_polys),("ground",(7,135,0),idle_polys),("side",(8,90,0),idle_polys),("wings",(48,180,0),rest_polys)]):
    im=fitted(poly,angle);im.save(out/(view+".png"))
    x=i%2*512;y=28+i//2*526;sheet.paste(im,(x,y));d.text((x+12,y+8),view.upper(),fill=(208,228,250))
sheet.save(out/"model-sheet.png")
# Head-only closeup uses the actual posed head location and full mesh; no invented portrait.
h=socket("animation."+name+".idle",.65,"head")
portrait=frame(idle_polys,tex,(6,135,0),384,h+np.array([0,0,-1.8]),17,True)
portrait.save(out/"portrait-bestiary.png");portrait.resize((128,128),Image.Resampling.LANCZOS).save(out/"portrait-target.png")
headshot=frame(idle_polys,tex,(6,135,0),640,h+np.array([0,0,-1.8]),17)
headshot.save(out/"head-closeup.png")
fitted(hero_polys,(4,145,0),768,1.02).save(out/"hero.png")
frames=out/"frames";frames.mkdir(exist_ok=True);idx=0;times=[];segments=[]
poses=Image.new("RGB",(4*320,math.ceil(len(clips)/4)*344),(21,25,33));pd=ImageDraw.Draw(poses)
for i,(clip,L,_) in enumerate(clips):
    full="animation."+name+"."+clip;start=idx
    hit=next((e["at"] for e in events if e["clip"]==clip and e["kind"]=="hit"),L*.5)
    px=i%4*320;py=i//4*344;poses.paste(fitted(polygons(data,full,hit),(16,140,0),320),(px,py+24));pd.text((px+6,py+5),clip,fill="white")
    # Per-clip fixed camera bounds, not automatic per-frame zoom; frame body and wings together.
    points=np.concatenate([p[0] for tt in np.linspace(0,L,9) for p in polygons(data,full,float(tt))]);clip_center=(points.min(0)+points.max(0))/2
    clip_extent=max(np.ptp(points[:,0]),np.ptp(points[:,1]),np.ptp(points[:,2]))*1.10
    for t in np.arange(0,L,1/16):
        tick=time.perf_counter();poly=polygons(data,full,float(t));canvas=Image.new("RGB",(896,480),(21,25,33))
        for j,angle in enumerate([(12,145,0),(6,90,0)]):
            im=frame(poly,tex,angle,448,clip_center,clip_extent).convert("RGBA")
            # Draw the authored VFX at the actual saved animated socket, with same camera.
            for e in events:
                age=float(t)-e["at"]
                if e["clip"]!=clip or e["kind"]!="vfx" or not 0<=age<.65:continue
                p=(socket(full,float(t),e["socket"])-clip_center)@rotation(angle).T
                pos=(448/2+p[0]*448*.8/clip_extent,448/2-p[1]*448*.8/clip_extent)
                overlay(im,out,e["effect"],age,pos)
            canvas.paste(im.convert("RGB"),(j*448,32))
        dr=ImageDraw.Draw(canvas);dr.text((8,8),clip.upper()+"  |  FRONT -Z  |  OFFLINE VFX PREVIEW",fill=(223,232,242))
        canvas.save(frames/f"{idx:04}.png");idx+=1;times.append((time.perf_counter()-tick)*1000)
    segments.append(dict(clip=clip,first=start,last=idx-1))
poses.save(out/"pose-sheet.png")
subprocess.run(["ffmpeg","-hide_banner","-loglevel","error","-y","-framerate","16","-i",str(frames/"%04d.png"),"-c:v","libx264","-pix_fmt","yuv420p",str(out/"animation-preview.mp4")],check=True)
subprocess.run(["rustc","--edition=2021","--test",str(out/"storm_gryphon_generated.rs"),"-o",str(out/"rust-tests")],check=True);subprocess.run([str(out/"rust-tests")],check=True);(out/"rust-tests").unlink()
(out/"summary.json").write_text(json.dumps(dict(kind="offline_asset_candidate_not_game",bones=len(G),cubes=len(C),faces=len(layout),clips=len(clips),attacks=len(attacks),frames=idx,uv={k:v for k,v in uv.items() if k!="faces_detail"},events=event_report,median_cpu_frame_ms=float(np.median(times)),runtime_integrated=False,segments=segments),indent=2))
print((out/"summary.json").read_text())
