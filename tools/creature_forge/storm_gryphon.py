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
box("barrel","body",[-4.3,10,-2],[4.3,17.8,9],"fur")
box("loin","body",[-3.7,10.5,6],[3.7,16.5,11.1],"fur")
box("dorsal_coat","body",[-3.8,17,-1.7],[3.8,18.5,8.6],"slate")
box("belly","body",[-3.6,9.3,-1],[3.6,11.3,7.2],"cream")
box("breast","chest",[-4.9,10,-9],[4.9,20,-2],"cream")
box("breast_shield","chest",[-3.8,12,-10.2],[3.8,18.7,-8.5],"ivory")
segment("keel","chest",[0,11,-9.4],[0,18,-10],2.6,1.6,"cream")
box("neck_base","neck",[-3.4,17,-12],[3.4,22.2,-6.9],"ivory")
box("throat","neck",[-2.7,17.2,-13.3],[2.7,22.3,-11],"cream")
box("skull","head",[-3.1,20.4,-15.1],[3.1,26.2,-10.4],"ivory")
box("crown","head",[-2.7,25.5,-14],[2.7,27.1,-11],"slate")
box("upper_beak","head",[-1.85,21.5,-18.6],[1.85,24.1,-14.6],"horn")
segment("beak_hook","head",[0,22.8,-18.1],[0,20,-19.6],2.1,1.75,"horn")
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
    bx("cheek","head",[2.2,20.8,-14.5],[3.7,24.7,-10.8],"ivory",[0,0,-10],[2.8,23,-12])
    bx("eye_socket","head",[2.75,23,-15.1],[3.65,24.65,-12.6],"slate")
    bx("iris","head",[3.4,23.35,-14.6],[3.8,24.1,-13.5],"electric")
    bx("pupil","head",[3.78,23.47,-14.25],[3.94,24.0,-13.89],"joint")
    sg("brow","head",[1.9,25,-15.3],[3.8,24.6,-12.2],1.1,1.0,"slate")
    crest="crest_"+side;group(crest,mir([2.4,25.5,-10.7]),"head")
    for j in range(3):sg("crest_plume"+str(j),crest,[1.4+j*.45,25.5,-11.2+j*.35],[2.7+j*.75,29.6-j*.9,-8.7+j*.55],1.05,.78,"feather" if j%2 else "slate")
    for j in range(4):
        sg("neck_hackle"+str(j),"neck",[2.2+j*.2,21.5-j*1.1,-10+j*.75],[3.9+j*.3,18.8-j*.65,-6.8+j*.65],1.9,1.3,"ivory" if j<2 else "feather")
    # Thick feline thigh and hock; raptor front leg with independent toes.
    for typ,A,B,F in [("front",[3.6,14,-6.7],[4.2,7.8,-7.9],[4.1,1.35,-6.2]),("hind",[3.7,13.6,7.7],[4.4,7.1,4.6],[4.2,1.35,8.1])]:
        leg=typ+"_leg_"+side;shin=typ+"_shin_"+side;foot=typ+"_foot_"+side
        group(leg,mir(A),"chest" if typ=="front" else "body");group(shin,mir(B),leg);group(foot,mir(F),shin)
        sg(typ+"_upper",leg,A,B,3.5 if typ=="front" else 4.2,3.4 if typ=="front" else 4.6,"ivory" if typ=="front" else "fur")
        sg(typ+"_lower",shin,B,F,1.75 if typ=="front" else 2.65,1.9 if typ=="front" else 2.8,"horn" if typ=="front" else "fur")
        if typ=="front":
            for j in range(3):
                p=[4.1+(j-1)*.55,6.6-j*.9,-8.1+j*.2];sg("scale"+str(j),shin,p,[p[0],p[1]-.55,p[2]+.1],1.6,.7,"darkhorn")
        bx(typ+"_pad",foot,[F[0]-1.45,.18,F[2]-1.6],[F[0]+1.45,1.7,F[2]+1.05],"horn" if typ=="front" else "fur")
        for j in range(3):
            x=F[0]+(j-1)*.88;toe=typ+"_toe_"+side+"_"+str(j);group(toe,mir([x,1.05,F[2]-.7]),foot)
            sg(typ+"_digit"+str(j),toe,[x,1.0,F[2]-.6],[x,.7,F[2]-2.0],.7,.72,"horn" if typ=="front" else "fur")
            sg(typ+"_claw"+str(j),toe,[x,.75,F[2]-1.9],[x,.29,F[2]-2.9],.48,.49,"talon")
        group(typ+"_talon_fx_"+side,mir([F[0],.75,F[2]-2.6]),foot)
    # Four linked wing sections. Shape is a tapered swept feather fan, not a flat slab.
    A=[4.4,18.3,-5.2];B=[11.4,19.3,-4.5];D=[18.6,19.0,-1.8];E=[25.0,18.5,2.5]
    w="wing_"+side;el="wing_elbow_"+side;wr="wing_wrist_"+side;tip="wing_tip_"+side
    for n,p,parent in [(w,A,"chest"),(el,B,w),(wr,D,el),(tip,E,wr)]:group(n,mir(p),parent)
    sg("wing_humerus",w,A,B,3.5,4.3,"slate");sg("wing_forearm",el,B,D,2.85,3.4,"slate");sg("wing_hand",wr,D,E,1.85,2.8,"slate")
    # Feather roots overlap as on a real wing. Leading edge small coverts, trailing edge long vanes.
    for j in range(6):
        x=5.5+j*1.07;z=-4.5+j*.13
        sg("secondary_"+str(j),w,[x,18.5,z],[x+.6,17.6,z+8.7-j*.28],2.2,.75,"feather" if j%2==0 else "slate")
        sg("covert_"+str(j),w,[x,19.5,z],[x+.4,19,z+4.1],2.0,.72,"ivory")
    for j in range(6):
        x=11.4+j*1.05;z=-4.2+j*.36
        sg("outer_secondary_"+str(j),el,[x,19.0,z],[x+1.4,18,z+10.5+j*.24],2.15,.72,"feather")
        sg("outer_covert_"+str(j),el,[x,19.65,z],[x+1,19.1,z+4.3],1.95,.65,"slate")
    for j in range(7):
        x=18.1+j*.88;z=-1.8+j*.58;end=[x+5.2+j*.23,17.7,z+9.5-j*.65]
        feather="primary_"+side+"_"+str(j);group(feather,mir([x,18.8,z]),wr)
        mid=[(x+end[0])/2,18.2,(z+end[2])/2]
        sg("primary_root"+str(j),feather,[x,18.8,z],mid,2.15,.7,"slate")
        sg("primary_vane"+str(j),feather,mid,end,1.5,.58,"feather")
        p=[end[0]-.9,end[1]+.07,end[2]-1.15]
        sg("primary_tip"+str(j),feather,p,end,.85,.53,"electric" if j%3==0 else "ivory")
    for j in range(3):sg("wingtip_quill"+str(j),tip,E,[27.8+j*.68,18.0,5.1+j*1.8],1.2,.55,"slate")
    group("wing_fx_"+side,mir([23,18,5]),wr)
palette={"fur":(85,103,125),"cream":(170,180,184),"ivory":(201,211,216),"slate":(49,68,97),"feather":(92,128,166),"horn":(187,140,66),"darkhorn":(114,78,36),"talon":(51,57,67),"joint":(24,32,44),"electric":(92,206,249)}
def tile(w,h,mat,n,f):
    im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im);rng=random.Random(int(hashlib.sha256((n+"/"+f).encode()).hexdigest()[:16],16))
    base=palette[mat]
    for y in range(h):
        for x in range(w):
            v=y/max(1,h-1);u=x/max(1,w-1);rim=(x==0 or y==0)
            shade=int(13*(1-v)-10*v+7*math.sin(math.pi*u))+rng.choice([-3,-1,0,0,2,3])
            if mat in ("fur","ivory","slate","feather") and rim:shade+=13
            if mat=="joint":shade+=-7 if y%3==0 else 3
            if mat=="feather":shade+=int(18*math.sin((u*.7+v)*math.pi))
            if mat=="electric":shade+=18 if (x+y)%3==0 else -4
            d.point((x,y),fill=tuple(max(0,min(255,c+shade)) for c in base)+(255,))
    if mat in ("fur","ivory","slate") and w>=5 and h>=6:
        d.line([(1,h-2),(w//2,h//2),(w-2,1)],fill=(43,62,89,255))
        d.line([(1,h-1),(w//2,h//2+1),(w-2,2)],fill=(157,186,211,255))
    if mat=="crystal" and w>=4 and h>=5:
        d.line([(0,h-1),(w//2,h//2),(w-1,0)],fill=(153,194,227,255))
        d.line([(w//2,0),(w//2,h-1)],fill=(76,109,145,255))
    if mat=="electric" and w>=3:d.line([(w//2,0),(w//2,h-1)],fill=(212,248,255,255))
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
    if mat in ("electric",):extrude(em,t,x,y)
    uv=[x,y,x+w,y+h];c.setdefault("faces",{})[f]=dict(uv=uv,texture=atlas_name)
    layout.append(dict(part=c["name"],face=f,material=mat,uv=uv));x+=w+2;row=max(row,h)
atlas.save(out/atlas_name);em.save(out/(name+"_emissive.png"));(out/"uv-layout.json").write_text(json.dumps(layout,indent=2))
# Flight and ground animations are separate; all numbers are model units and seconds.
clips=[("idle",2.4,True),("walk",1.12,True),("gallop",.72,True),("takeoff",1.5,False),("flight",1.05,True),("glide",2.0,True),("land",1.55,False),("talon_left",1.3,False),("talon_right",1.3,False),("beak_strike",1.15,False),("wing_gust",1.8,False),("storm_breath",2.4,False),("dive",2.2,False),("stagger",.7,False),("death",2.5,False)]
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
event("takeoff",.55,"vfx","ground_fx","gust");event("land",1.0,"vfx","ground_fx","impact");event("death",1.7,"vfx","ground_fx","dust")
spec=dict(schema=1,creature=name,front="-Z",authority="server",runtime_integrated=False,boss=dict(optional=True,portrait=False,phase_threshold=.35),attacks=attacks,events=events)
(out/"events.json").write_text(json.dumps(spec,indent=2))
self_test();rig=Rig(G);rest={b:rig.pivot(b) for b in rig.groups};keys=[];checks=[];pose_data={};eye=np.eye(3)
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
        else:root[1]+=.12*math.sin(math.tau*u);neck=2*math.sin(math.tau*u)
        rig.p["root"]=root;rig.r["body"]=[pitch,yaw,roll];rig.r["chest"]=[pitch*.12,0,-roll*.2]
        rig.r["neck"]=[neck,0,0];rig.r["head"]=[-neck*.26,3*math.sin(math.tau*u) if clip=="idle" else 0,0];rig.r["jaw"]=[-jaw,0,0]
        rig.r["tail"]=[-pitch*.65,5*math.sin(math.tau*u),-roll*.6]
        for j in range(3):rig.r["tail_"+str(j)]=[3*math.sin(math.tau*u-j*.6),4*math.sin(math.tau*u-j*.8),0]
        rig.r["tail_fan"]=[2*math.sin(math.tau*u),0,0]
        for side,s in [("l",-1),("r",1)]:
            rig.r["crest_"+side]=[4*math.sin(math.tau*u+s*.6),0,s*2]
            # Wing folds back with a positive Z trailing direction; mirrored flapping lifts both wings.
            rig.r["wing_"+side]=[0,-s*54*fold,s*(8*fold+flap)]
            rig.r["wing_elbow_"+side]=[0,-s*47*fold,s*(-flap*.38-7*fold)]
            rig.r["wing_wrist_"+side]=[0,s*25*fold,s*(-flap*.25+7*fold)]
            rig.r["wing_tip_"+side]=[0,s*12*fold,s*(-flap*.15)]
            for j in range(7):rig.r["primary_"+side+"_"+str(j)]=[0,s*(j-3)*2.1*(1-fold),s*(flap*.13+1.8*math.sin(math.tau*u-j*.35))]
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
pts=np.concatenate([p[0] for p in rest_polys]);center=(pts.min(0)+pts.max(0))/2;extent=float(np.linalg.norm(pts.max(0)-pts.min(0)))*1.3;center[1]+=2.0
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
sheet=Image.new("RGB",(1024,1056),(21,25,33));d=ImageDraw.Draw(sheet);d.text((12,8),"STORM GRYPHON | OFFLINE ACTUAL MODEL",fill="white")
for i,(view,angle) in enumerate([("front",(8,180,0)),("three-quarter",(18,140,0)),("side",(8,90,0)),("back",(15,0,0))]):
    im=frame(rest_polys,tex,angle,512,center,extent);im.save(out/(view+".png"));sheet.paste(im,(i%2*512,32+i//2*512))
sheet.save(out/"model-sheet.png")
portrait=frame(rest_polys,tex,(10,145,0),384,np.array([0,23,-12]),32,True);portrait.save(out/"portrait-bestiary.png");portrait.resize((128,128),Image.Resampling.LANCZOS).save(out/"portrait-target.png")
frames=out/"frames";frames.mkdir(exist_ok=True);idx=0;times=[];segments=[]
poses=Image.new("RGB",(4*320,math.ceil(len(clips)/4)*344),(21,25,33));pd=ImageDraw.Draw(poses)
for i,(clip,L,_) in enumerate(clips):
    full="animation."+name+"."+clip;start=idx
    hit=next((e["at"] for e in events if e["clip"]==clip and e["kind"]=="hit"),L*.5)
    px=i%4*320;py=i//4*344;poses.paste(frame(polygons(data,full,hit),tex,(16,140,0),320,center,extent),(px,py+24));pd.text((px+6,py+5),clip,fill="white")
    for t in np.arange(0,L,1/16):
        tick=time.perf_counter();poly=polygons(data,full,float(t));canvas=Image.new("RGB",(704,384),(21,25,33))
        for j,angle in enumerate([(12,145,0),(6,90,0)]):
            im=frame(poly,tex,angle,352,center,extent).convert("RGBA")
            # Draw the authored VFX at the actual saved animated socket, with same camera.
            for e in events:
                age=float(t)-e["at"]
                if e["clip"]!=clip or e["kind"]!="vfx" or not 0<=age<.65:continue
                p=(socket(full,float(t),e["socket"])-center)@rotation(angle).T
                pos=(352/2+p[0]*352*.8/extent,352/2-p[1]*352*.8/extent)
                overlay(im,out,e["effect"],age,pos)
            canvas.paste(im.convert("RGB"),(j*352,32))
        dr=ImageDraw.Draw(canvas);dr.text((8,8),clip.upper()+"  |  FRONT -Z  |  OFFLINE VFX PREVIEW",fill=(223,232,242))
        canvas.save(frames/f"{idx:04}.png");idx+=1;times.append((time.perf_counter()-tick)*1000)
    segments.append(dict(clip=clip,first=start,last=idx-1))
poses.save(out/"pose-sheet.png")
subprocess.run(["ffmpeg","-hide_banner","-loglevel","error","-y","-framerate","16","-i",str(frames/"%04d.png"),"-c:v","libx264","-pix_fmt","yuv420p",str(out/"animation-preview.mp4")],check=True)
subprocess.run(["rustc","--edition=2021","--test",str(out/"storm_gryphon_generated.rs"),"-o",str(out/"rust-tests")],check=True);subprocess.run([str(out/"rust-tests")],check=True);(out/"rust-tests").unlink()
(out/"summary.json").write_text(json.dumps(dict(kind="offline_asset_candidate_not_game",bones=len(G),cubes=len(C),faces=len(layout),clips=len(clips),attacks=len(attacks),frames=idx,uv={k:v for k,v in uv.items() if k!="faces_detail"},events=event_report,median_cpu_frame_ms=float(np.median(times)),runtime_integrated=False,segments=segments),indent=2))
print((out/"summary.json").read_text())
