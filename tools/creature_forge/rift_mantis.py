#!/usr/bin/env python3
"""Rift Mantis production candidate, original cuboid art and articulated hexapod.
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
G=[];C=[];name="rift_mantis";model=name+".bbmodel";atlas_name=name+".png";N=512
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
group("root",[0,0,0]);group("abdomen",[0,8.2,4],"root");group("thorax",[0,9.5,-1],"root")
group("neck",[0,12.5,-4.1],"thorax");group("head",[0,14,-7],"neck")
box("ventral_thorax","thorax",[-2.6,7.2,-4.6],[2.6,11.7,2.1],"joint")
box("thorax_carapace","thorax",[-3.1,10,-4.5],[3.1,13.4,1.8],"shell")
box("pronotum_crown","thorax",[-2.6,13.2,-3.9],[2.6,14.1,.9],"edge")
# Overlapping abdominal tergites taper to a pointed stinger, each pivot follows its predecessor.
prev="abdomen"
for i,(z,w,h) in enumerate([(3.6,6.6,4.8),(6.7,7.2,5.2),(9.6,6.4,4.7),(12.1,5.0,4),(14.3,3.2,3.0)]):
    b="tergite_"+str(i);group(b,[0,8,z-1.6],prev)
    box(b+"_shell",b,[-w/2,8-h/2,z-1.8],[w/2,8+h/2,z+1.8],"shell",[(-4+i*2),0,0],[0,8,z])
    box(b+"_ridge",b,[-w*.29,8+h/2-.2,z-1.5],[w*.29,8+h/2+.55,z+1.45],"edge")
    for sign in [-1,1]:
        x=sign*w*.43;box(b+"_spiracle_"+str(sign),b,[x-.36,7.4,z-.9],[x+.36,8.2,z+.65],"violet")
    prev=b
group("tail_tip",[0,8,15.4],prev)
segment("tail_stinger","tail_tip",[0,8,15.1],[0,7.3,18],1.4,1.4,"crystal")
box("neck_ring","neck",[-1.4,11.6,-6],[1.4,13.4,-3.4],"joint")
box("skull","head",[-2.3,12.7,-9.3],[2.3,15.9,-6],"shell")
box("crown_plate","head",[-1.55,15.7,-8.8],[1.55,16.7,-6.6],"edge")
box("lower_face","head",[-1.4,12.2,-9.6],[1.4,13.5,-8.2],"shell")
# Narrow facial profile, independent mandibles, antenna joints and compound eyes.
for side,sgn in [("l",-1),("r",1)]:
    def m(v):return [sgn*v[0],v[1],v[2]]
    def sb(n,b,A,B,w,d,mat):segment(side+"_"+n,b,m(A),m(B),w,d,mat)
    def mb(n,b,A,B,mat,angle=None,pivot=None):
        A,B=m(A),m(B);lo=[min(x,y) for x,y in zip(A,B)];hi=[max(x,y) for x,y in zip(A,B)]
        box(side+"_"+n,b,lo,hi,mat,([angle[0],sgn*angle[1],sgn*angle[2]] if angle else None),m(pivot) if pivot else None)
    mb("cheek","head",[1.7,12.8,-9],[3.1,15.2,-6.9],"shell",[0,0,-18],[2,14,-8])
    mb("compound_eye","head",[1.85,13.5,-9.5],[3.1,15.2,-7.9],"eye",[0,sgn*0,0])
    jaw="jaw_"+side;group(jaw,m([1.15,12.5,-9.1]),"head")
    sb("mandible",jaw,[1.15,12.5,-9.1],[2.1,11.4,-10.1],.85,.85,"edge")
    sb("mandible_tip",jaw,[2.1,11.4,-10.1],[.7,11.7,-11.4],.55,.55,"ivory")
    ant="antenna_"+side;ant2=ant+"_tip";group(ant,m([1.25,16,-7.3]),"head")
    group(ant2,m([2.7,19.8,-7.5]),ant)
    sb("antenna_stem",ant,[1.25,16,-7.3],[2.7,19.8,-7.5],.45,.45,"edge")
    sb("antenna_tip",ant2,[2.7,19.8,-7.5],[4.5,21,-9.7],.32,.32,"violet")
    # Six weight-bearing legs, with two main joints and an independently orientable tarsus.
    for i,z in enumerate([-2.6,3.0,8.5]):
        hip=[2.55,8.8 if i==0 else 8.0,z];knee=[8.1,9.2-i*.5,z+(-1.2 if i==0 else 1.0 if i==2 else 0)]
        foot=[11.3,1.05,z+(-3 if i==0 else 3 if i==2 else 0)]
        l="leg"+str(i+1)+"_"+side;k="shin"+str(i+1)+"_"+side;f="foot"+str(i+1)+"_"+side
        group(l,m(hip),"thorax" if i==0 else "root");group(k,m(knee),l);group(f,m(foot),k)
        sb("coxa"+str(i),l,hip,[4.3,hip[1]+.45,z],1.8,1.6,"joint")
        sb("femur"+str(i),l,[3.65,hip[1]+.3,z],knee,1.5,1.45,"shell")
        sb("femur_keel"+str(i),l,[4.1,hip[1]+.8,z],[knee[0]-.4,knee[1]+.85,knee[2]],.72,.85,"edge")
        mb("knee"+str(i),k,[knee[0]-.85,knee[1]-.8,knee[2]-.75],[knee[0]+.85,knee[1]+.8,knee[2]+.75],"joint")
        sb("tibia"+str(i),k,knee,foot,1.02,1.0,"shell")
        mid=[(knee[j]+foot[j])/2 for j in range(3)]
        sb("tibial_edge"+str(i),k,knee,mid,.58,1.2,"edge")
        mb("tarsus"+str(i),f,[foot[0]-.65,.28,foot[2]-.85],[foot[0]+.65,1.35,foot[2]+.8],"joint")
        sb("claw"+str(i),f,[foot[0],.75,foot[2]-.35],[foot[0]+.45,.42,foot[2]-1.9],.56,.52,"ivory")
    # Raptorial limbs: substantial upper segment, jointed lower segment, long serrated sickle.
    A=[3.0,12.5,-3.5];B=[7.3,14,-5.5];D=[8.0,11.3,-10.4]
    arm="scythe_"+side;elbow="elbow_"+side;blade="blade_"+side;tip="tip_"+side
    group(arm,m(A),"thorax");group(elbow,m(B),arm);group(blade,m(D),elbow)
    sb("raptor_coxa",arm,A,B,2.35,2.15,"shell")
    sb("raptor_armor",arm,[3.5,13,-3.6],[7,14.6,-5.4],1.65,1.2,"edge")
    mb("elbow_joint",elbow,[6.2,13,-6.5],[8.4,15,-4.5],"joint")
    sb("raptor_tibia",elbow,B,D,2.05,1.75,"shell")
    sb("tibia_ridge",elbow,[7,14,-6],[7.7,12,-9.8],1.25,.85,"violet")
    # Blade face is continuous and curves through three tapered, overlapping sections.
    path=[D,[8.7,10.3,-14.4],[7.8,10,-17.6],[5.9,10.2,-20.0],[3.9,10.7,-21.0]]
    for j,(u,v) in enumerate(zip(path,path[1:])):
        sb("blade_spine"+str(j),blade,u,v,[2.25,1.8,1.25,.65][j],[1.25,1,.8,.48][j],"crystal")
        off=[.0,-.48,0];u=[x+y for x,y in zip(u,off)];v=[x+y for x,y in zip(v,off)]
        sb("cutting_edge"+str(j),blade,u,v,[1.45,1.2,.8,.42][j],[.72,.58,.45,.3][j],"glow")
    for j in range(4):
        p=[8.25-j*.7,9.85,-12.7-j*1.65]
        sb("serration"+str(j),blade,p,[p[0]-1.25,p[1]-.15,p[2]-.7],.55,.6,"ivory")
    group(tip,m(path[-1]),blade)
    group("blade_fx_"+side,m(path[2]),blade)
group("mouth_fx",[0,12.5,-10.5],"head");group("ground_fx",[0,0,-9],"root")
# Intentional texture palette, each material painted at anatomical face scale.
palette={"shell":(36,48,54),"edge":(63,94,97),"joint":(23,29,38),"crystal":(81,43,110),"glow":(157,94,222),"violet":(106,58,144),"eye":(75,200,174),"ivory":(154,185,185)}
def tile(w,h,mat,n,f):
    im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im);rng=random.Random(int(hashlib.sha256((n+"/"+f).encode()).hexdigest()[:16],16))
    base=palette[mat]
    for y in range(h):
        for x in range(w):
            v=y/max(1,h-1);u=x/max(1,w-1);rim=(x==0 or y==0)
            shade=int(13*(1-v)-10*v+7*math.sin(math.pi*u))+rng.choice([-3,-1,0,0,2,3])
            if mat in ("shell","edge","crystal") and rim:shade+=13
            if mat=="joint":shade+=-7 if y%3==0 else 3
            if mat=="crystal":shade+=int(18*math.sin((u*.7+v)*math.pi))
            if mat=="eye":shade+=18 if (x+y)%3==0 else -4
            d.point((x,y),fill=tuple(max(0,min(255,c+shade)) for c in base)+(255,))
    if mat in ("shell","edge") and w>=5 and h>=6:
        d.line([(1,h-2),(w//2,h//2),(w-2,1)],fill=(24,34,43,255))
        d.line([(1,h-1),(w//2,h//2+1),(w-2,2)],fill=(84,112,112,255))
    if mat=="crystal" and w>=4 and h>=5:
        d.line([(0,h-1),(w//2,h//2),(w-1,0)],fill=(158,103,207,255))
        d.line([(w//2,0),(w//2,h-1)],fill=(105,66,153,255))
    if mat=="glow" and w>=3:d.line([(w//2,0),(w//2,h-1)],fill=(222,194,255,255))
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
    if mat in ("glow","violet","eye"):extrude(em,t,x,y)
    uv=[x,y,x+w,y+h];c.setdefault("faces",{})[f]=dict(uv=uv,texture=atlas_name)
    layout.append(dict(part=c["name"],face=f,material=mat,uv=uv));x+=w+2;row=max(row,h)
atlas.save(out/atlas_name);em.save(out/(name+"_emissive.png"));(out/"uv-layout.json").write_text(json.dumps(layout,indent=2))
# Attacks have separate telegraphs and active windows. All server dispatch is future work.
clips=[("idle",2.0,True),("scuttle",.84,True),("sprint",.6,True),("stalk",1.5,True),("left_reap",1.25,False),("right_reap",1.25,False),("cross_cut",1.8,False),("impale",1.75,False),("rift_burst",2.35,False),("parry",1.15,False),("stagger",.65,False),("death",2.0,False)]
attacks=[];events=[]
def event(clip,at,kind,socket,effect):
    events.append(dict(clip=clip,at=at,kind=kind,socket=socket,effect=effect,id=clip+"_"+str(len(events))))
def attack(clip,wind,end,cd,shape,range_,mult,hit,sockets,fx):
    attacks.append(dict(clip=clip,windup=wind,active_end=end,cooldown=cd,shape=shape,range=range_,damage_multiplier=mult))
    for t,s in zip(hit,sockets):
        event(clip,max(.04,t-.32),"vfx",s,"charge");event(clip,t,"hit",s,shape);event(clip,t,"vfx",s,fx);event(clip,t,"sound",s,"rift_slice")
attack("left_reap",.54,.76,2.2,"arc",3.3,1.12,[.62],["tip_l"],"slash")
attack("right_reap",.54,.76,2.2,"arc",3.3,1.12,[.62],["tip_r"],"slash")
attack("cross_cut",.56,1.29,3.8,"arc",3.6,.88,[.63,1.17],["tip_l","tip_r"],"slash")
attack("impale",.87,1.12,4.5,"pierce",4.5,1.6,[.98],["tip_l"],"lance")
attack("rift_burst",1.16,1.45,6.0,"projectile_fan",7,1.15,[1.30],["mouth_fx"],"shards")
event("parry",.3,"state","mouth_fx","guard_on");event("parry",.85,"state","mouth_fx","guard_off");event("death",1.2,"vfx","ground_fx","dust")
spec=dict(schema=1,creature=name,front="-Z",authority="server",runtime_integrated=False,boss=dict(optional=True,portrait=False,phase_threshold=.35),attacks=attacks,events=events)
(out/"events.json").write_text(json.dumps(spec,indent=2))
self_test();rig=Rig(G);rest={b:rig.pivot(b) for b in rig.groups};keys=[];checks=[];pose_data={}
zero=np.zeros(3);eye=np.eye(3)
for clip,L,loop in clips:
    full="animation."+name+"."+clip;keys.append(dict(op="add_animation",name=full,length=L,loop="loop" if loop else "once",snapping=24))
    ts=sorted({round(i*L/math.ceil(L*16),5) for i in range(math.ceil(L*16)+1)}|{e["at"] for e in events if e["clip"]==clip})
    tracks={};prev={}
    for t in ts:
        u=t/L;rig.reset();root=np.array([0.,-.8,0.]);thor=np.array([0.,0.,0.]);walk=clip in ("scuttle","sprint","stalk");dead=clip=="death"
        if walk:root[1]+=.12*math.cos(4*math.pi*u);thor[2]=1.2*math.sin(2*math.pi*u)
        if clip=="stalk":root[1]-=1.1;thor[0]=-5
        # Individually authored attack timing with different silhouettes, not reused punches.
        armgoals={};tipdirs={}
        seq=[("l",.62)] if clip=="left_reap" else [("r",.62)] if clip=="right_reap" else [("l",.63),("r",1.17)] if clip=="cross_cut" else []
        for side,hit in seq:
            sign=-1 if side=="l" else 1;start=max(0,hit-.42);wind=hit-.16;end=min(L,hit+.36)
            if start<=t<=end:
                armgoals[side]=curve([(start,[sign*7.6,10.7,-9.3]),(wind,[sign*10,13,-6.3]),(hit,[sign*4.4,11,-12.2]),(hit+.08,[sign*1.5,10.5,-11.2]),(end,[sign*7.6,10.7,-9.3])],t)
                tipdirs[side]=curve([(start,[-sign*.36,-.06,-1]),(wind,[sign*.85,.14,-.5]),(hit,[-sign*.65,-.13,-1]),(hit+.08,[-sign*.92,-.14,-.45]),(end,[-sign*.36,-.06,-1])],t)
                thor[1]=float(curve([(start,[0]),(wind,[sign*14]),(hit,[-sign*16]),(end,[0])],t)[0])
                root[1]-=.6*math.sin(math.pi*(t-start)/(end-start))
        if clip=="impale":
            q=float(curve([(0,[0]),(.75,[1]),(.98,[0]),(L,[0])],t)[0])
            root[1]-=2.2*q;thor[0]=float(curve([(0,[0]),(.75,[-13]),(.98,[-5]),(L,[0])],t)[0])
            for side,sign in [("l",-1),("r",1)]:
                armgoals[side]=curve([(0,[sign*7.6,10.7,-9.3]),(.75,[sign*5.3,10,-7]),(.98,[sign*4,11,-13.0]),(L,[sign*7.6,10.7,-9.3])],t)
                tipdirs[side]=[0,0,-1]
        if clip=="rift_burst":
            q=float(curve([(0,[0]),(1.02,[1]),(1.3,[.7]),(L,[0])],t)[0]);thor[0]=12*q;root[1]-=.7*q
            for side,sign in [("l",-1),("r",1)]:armgoals[side]=[sign*(7.6+2*q),10.7+2*q,-9.3+3*q];tipdirs[side]=[sign*.6,.1,-.85]
        if clip=="parry":
            q=float(curve([(0,[0]),(.3,[1]),(.85,[1]),(L,[0])],t)[0]);root[1]-=q
            for side,sign in [("l",-1),("r",1)]:armgoals[side]=[sign*(7.6-4*q),10.7+q*2,-9.3];tipdirs[side]=[-sign*(.36+.8*q),.3*q,-1+.7*q]
        if clip=="stagger":thor[2]=10*math.sin(math.pi*u);root[1]-=1.3*math.sin(math.pi*u)
        if dead:
            q=float(curve([(0,[0]),(.5,[.2]),(1.4,[1]),(L,[1])],t)[0]);root[1]-=5.6*q;thor[2]=18*q;thor[0]=-20*q
            for side,sign in [("l",-1),("r",1)]:armgoals[side]=[sign*(7.6+2*q),10.7-7*q,-9.3+2*q];tipdirs[side]=[sign*.2,-.1,-1]
        rig.p["root"]=root;rig.r["thorax"]=thor;rig.r["head"]=[-4 if clip=="stalk" else 3*math.sin(math.tau*u),3*math.sin(math.tau*u),0]
        rig.r["abdomen"]=[2*math.sin(math.tau*u),0,2*math.sin(math.tau*u)]
        for j in range(5):rig.r["tergite_"+str(j)]=[1.4*math.sin(math.tau*u-j*.5),0,0]
        rig.r["tail_tip"]=[2*math.sin(math.tau*u),0,0]
        for side,sign in [("l",-1),("r",1)]:
            rig.r["antenna_"+side]=[6*math.sin(math.tau*u+sign*.8),0,sign*3];rig.r["antenna_"+side+"_tip"]=[9*math.sin(math.tau*u+sign*.8-.5),0,0]
            opening=24*math.sin(math.pi*u) if clip=="rift_burst" else 3*math.sin(math.tau*u)
            rig.r["jaw_"+side]=[0,sign*opening,0]
            for j in range(3):
                leg="leg"+str(j+1)+"_"+side;shin="shin"+str(j+1)+"_"+side;foot="foot"+str(j+1)+"_"+side
                target=rest[foot].copy();stance=True
                if walk:
                    phase=(u+((j+(side=="r"))%2)*.5)%1;stride={"scuttle":3.6,"sprint":4.5,"stalk":2.4}[clip]
                    if phase<.64:target[2]+=-stride/2+stride*phase/.64
                    else:
                        q=(phase-.64)/.36;target[2]+=stride/2-stride*q;target[1]+=1.45*math.sin(math.pi*q);stance=False
                if dead:target[0]+=sign*1.5*u
                error=rig.ik(leg,shin,foot,target,[sign,.9,0],eye)
                if not dead and stance:checks.append(dict(clip=clip,t=t,kind="stance_contact",socket=foot,error=round(error,4),passed=bool(error<.30)))
            arm="scythe_"+side;elb="elbow_"+side;blade="blade_"+side
            goal=armgoals.get(side,[sign*7.6,10.7+root[1],-9.3])
            direction=tipdirs.get(side,[-sign*.36,-.06,-1]);d=rest["tip_"+side]-rest[blade]
            rig.ik(arm,elb,blade,goal,[sign,1,.2],align(d,direction))
        for e in events:
            if e["clip"]==clip and e["kind"]=="hit" and abs(t-e["at"])<.0001:
                pos=rig.pivot(e["socket"]);checks.append(dict(clip=clip,t=t,kind="attack_forward",socket=e["socket"],position=pos.tolist(),passed=bool(pos[2]<(-9 if e["socket"]=="mouth_fx" else -15))))
        row={}
        for ch,data in [("rotation",rig.r),("position",rig.p)]:
            for b,v in data.items():
                v=np.asarray(v,float);k=(b,ch)
                if ch=="rotation" and k in prev:v=prev[k]+(v-prev[k]+180)%360-180
                prev[k]=v;tracks.setdefault(k,[]).append((t,v))
    for (b,ch),rows in tracks.items():
        for t,v in rows:keys.append(dict(op="set_keyframe",animation=full,bone=b,channel=ch,time=t,value=[round(float(x),5) for x in v],interpolation="linear"))
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
rev=call("bbmodel_create",dict(file=model,name="Rift Mantis",format="free",box_uv=False,resolution=dict(width=N,height=N)),"01-create")["revision"]
rev=call("bbmodel_add_texture",dict(file=model,expected_revision=rev,image=atlas_name,name=atlas_name),"02-texture")["revision"]
# Keep each animation's creation and keys in order. Batches are actual MCP edits.
for i,batch in enumerate([G,[c for c,_ in C]]+[keys[j:j+350] for j in range(0,len(keys),350)]):
    rev=call("bbmodel_edit",dict(file=model,expected_revision=rev,operations=batch),"03-edit-"+str(i))["revision"]
call("bbmodel_validate",dict(file=model,self_test=True),"04-geometry")
call("bbmodel_validate_animations",dict(file=model),"05-animation")
doc,data,names=load(out/model);uv=uv_check(doc,atlas,em,out);event_report=validate_events(doc,spec,names);rust_export(doc,data,names,spec,out)
(out/"ashen_sentinel_v3_generated.rs").rename(out/"rift_mantis_generated.rs")
# Use saved model, not the geometry recipe, for all evidence.
tex=np.array(atlas);rest_polys=polygons(data,None,0)
pts=np.concatenate([p[0] for p in rest_polys]);center=(pts.min(0)+pts.max(0))/2;extent=float(np.linalg.norm(pts.max(0)-pts.min(0)))*1.3
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
        saved_checks.append(dict(clip=e["clip"],socket=e["socket"],position=pos.tolist(),passed=bool(pos[2]<(-9 if e["socket"]=="mouth_fx" else -15))))
(out/"saved-contact-report.json").write_text(json.dumps(saved_checks,indent=2))
assert all(e["passed"] for e in saved_checks)
from mantis_fx import build,overlay
build(out)
sheet=Image.new("RGB",(1024,1056),(21,25,33));d=ImageDraw.Draw(sheet);d.text((12,8),"RIFT MANTIS | OFFLINE ACTUAL MODEL",fill="white")
for i,(view,angle) in enumerate([("front",(8,180,0)),("three-quarter",(18,140,0)),("side",(8,90,0)),("back",(15,0,0))]):
    im=frame(rest_polys,tex,angle,512,center,extent);im.save(out/(view+".png"));sheet.paste(im,(i%2*512,32+i//2*512))
sheet.save(out/"model-sheet.png")
portrait=frame(rest_polys,tex,(10,145,0),384,np.array([0,13,-8]),26,True);portrait.save(out/"portrait-bestiary.png");portrait.resize((128,128),Image.Resampling.LANCZOS).save(out/"portrait-target.png")
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
subprocess.run(["rustc","--edition=2021","--test",str(out/"rift_mantis_generated.rs"),"-o",str(out/"rust-tests")],check=True);subprocess.run([str(out/"rust-tests")],check=True);(out/"rust-tests").unlink()
(out/"summary.json").write_text(json.dumps(dict(kind="offline_asset_candidate_not_game",bones=len(G),cubes=len(C),faces=len(layout),clips=len(clips),attacks=len(attacks),frames=idx,uv={k:v for k,v in uv.items() if k!="faces_detail"},events=event_report,median_cpu_frame_ms=float(np.median(times)),runtime_integrated=False,segments=segments),indent=2))
print((out/"summary.json").read_text())
