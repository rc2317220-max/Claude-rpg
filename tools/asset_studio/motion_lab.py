"""Asset Studio v3: explicit -Z front, two-bone IK, pose curves, contact gates.
Works in BBModel absolute rest coordinates. Bake numeric linear samples for MCP.
No claim of runtime movement, collision or combat integration."""
import math,json
from pathlib import Path
import numpy as np

FRONT=np.array([0.,0.,-1.])
def unit(v):
    v=np.asarray(v,float);n=np.linalg.norm(v)
    if n<1e-9:raise ValueError("zero vector")
    return v/n
def rot(a):
    x,y,z=np.radians(a);c,s=np.cos,np.sin
    return np.array([[c(z),-s(z),0],[s(z),c(z),0],[0,0,1]])@np.array([[c(y),0,s(y)],[0,1,0],[-s(y),0,c(y)]])@np.array([[1,0,0],[0,c(x),-s(x)],[0,s(x),c(x)]])
def euler(m):
    y=math.asin(float(np.clip(-m[2,0],-1,1)))
    if abs(math.cos(y))>1e-7:x=math.atan2(m[2,1],m[2,2]);z=math.atan2(m[1,0],m[0,0])
    else:x=math.atan2(-m[1,2],m[1,1]);z=0
    return np.degrees([x,y,z])
def align(a,b):
    a,b=unit(a),unit(b);v=np.cross(a,b);c=float(np.dot(a,b));s=np.linalg.norm(v)
    if s<1e-8:
        if c>0:return np.eye(3)
        v=unit(np.cross(a,[1,0,0] if abs(a[0])<.8 else [0,1,0]));return 2*np.outer(v,v)-np.eye(3)
    k=np.array([[0,-v[2],v[1]],[v[2],0,-v[0]],[-v[1],v[0],0]])
    return np.eye(3)+k+k@k*((1-c)/(s*s))
def curve(rows,t):
    if t<=rows[0][0]:return np.array(rows[0][1],float)
    for (a,u),(b,v) in zip(rows,rows[1:]):
        if t<=b:
            q=(t-a)/(b-a);q=q*q*(3-2*q)
            return np.array(u,float)*(1-q)+np.array(v,float)*q
    return np.array(rows[-1][1],float)
class Rig:
    def __init__(self,groups):
        self.groups={g["name"]:(g.get("parent"),np.array(g["origin"],float)) for g in groups}
        self.reset()
    def reset(self):self.r={};self.p={};self.s={}
    def world(self,b):
        if b is None:return np.eye(3),np.zeros(3)
        parent,o=self.groups[b];pr,pt=self.world(parent)
        r=rot(self.r.get(b,[0,0,0]));return pr@r,pt+pr@(o-r@o+self.p.get(b,np.zeros(3)))
    def pivot(self,b):
        r,t=self.world(b);return r@self.groups[b][1]+t
    def ik(self,a,b,c,target,pole,end_rotation=None):
        """Solve joint location and endpoint in world coords, then local Euler.
        Pole selects elbow/knee plane. Clamp unreachable targets, report error."""
        parent,_=self.groups[a];p0=self.pivot(a);d1=self.groups[b][1]-self.groups[a][1];d2=self.groups[c][1]-self.groups[b][1]
        l1,l2=np.linalg.norm(d1),np.linalg.norm(d2);v=np.asarray(target,float)-p0
        dist=np.linalg.norm(v);axis=unit(v);reach=float(np.clip(dist,abs(l1-l2)+.05,l1+l2-.04))
        # Avoid overflexion: internal angle >= 25 degrees.
        reach=max(reach,math.sqrt(l1*l1+l2*l2-2*l1*l2*math.cos(math.radians(25))))
        pp=np.asarray(pole,float);pp=pp-axis*np.dot(pp,axis)
        if np.linalg.norm(pp)<1e-7:pp=np.cross(axis,[1,0,0] if abs(axis[0])<.8 else [0,1,0])
        along=(l1*l1-l2*l2+reach*reach)/(2*reach);off=math.sqrt(max(0,l1*l1-along*along))
        elbow=p0+axis*along+unit(pp)*off;end=p0+axis*reach
        pr,_=self.world(parent);ar=align(d1,elbow-p0);self.r[a]=euler(pr.T@ar)
        br=align(ar@d2,end-elbow)@ar;self.r[b]=euler(ar.T@br)
        self.r[c]=euler(br.T@(br if end_rotation is None else end_rotation))
        return float(np.linalg.norm(self.pivot(c)-target))
def limb(rig,side,kind,target,orient=None):
    sign=-1 if side=="left" else 1
    if kind=="arm":
        return rig.ik(side+"_arm",side+"_forearm",side+"_hand",target,[sign,.15,.45],orient)
    return rig.ik(side+"_leg",side+"_shin",side+"_foot",target,[0,0,-1],np.eye(3))
def bake(groups,clips,spec,out):
    rig=Rig(groups);keys=[];checks=[];pose_recipes=[];rest={b:rig.pivot(b) for b in rig.groups}
    zero=[0,0,0];eye=np.eye(3)
    attack_hits={a["clip"]:[e["at"] for e in spec["events"] if e["clip"]==a["clip"] and e["kind"]=="hit"] for a in spec["attacks"]}
    for clip,length in clips:
        name=clip.split(".")[-1];loop=name in ("dormant","idle","walk","heavy_walk","run")
        keys.append(dict(op="add_animation",name=clip,length=length,loop="loop" if loop else "once",snapping=24))
        times=sorted(set(round(float(x),5) for x in np.linspace(0,length,math.ceil(length*12)+1))|{round(float(e["at"]),5) for e in spec["events"] if e["clip"]==name})
        samples=[];previous={}
        for t in times:
            rig.reset();u=t/length;root=np.array([0.,-1.0,0.]);pitch=0.;yaw=0.;roll=0.;head=zero
            # Weight shifts and anticipations are authored as timed poses, not mirrored cycles.
            arm_targets={};arm_ori={};feet={s:rest[s+"_foot"].copy() for s in ("left","right")}
            if name in ("walk","heavy_walk","run"):
                step={"walk":4.5,"heavy_walk":5.0,"run":6.0}[name];lift={"walk":1.6,"heavy_walk":1.9,"run":2.8}[name]
                root[1]=-2.0+.2*math.cos(4*math.pi*u);pitch=-5 if name!="run" else -15;roll=2*math.sin(2*math.pi*u)
                for i,s in enumerate(("left","right")):
                    phase=(u+i*.5)%1;sign=-1 if s=="left" else 1
                    # Stance moves backward relative to root at constant velocity.
                    if phase<.60:z=-step/2+step*phase/.60;y=2.
                    else:
                        v=(phase-.60)/.40;z=step/2-step*(v*v*(3-2*v));y=2+lift*math.sin(math.pi*v)
                    feet[s]=np.array([rest[s+"_foot"][0],y,z])
                    arm_targets[s]=np.array([sign*10,12.,-z*.8]);arm_ori[s]=eye
            elif name in ("left_punch","right_punch","combo"):
                seq=[("left",.58),("right",1.25)] if name=="combo" else [(name.split("_")[0],.63)]
                for side,hit in seq:
                    start=max(0,hit-.45);wind=hit-.13;end=min(length,hit+.48);sign=-1 if side=="left" else 1
                    if start<=t<=end:
                        a=curve([(start,[sign*10,12,0]),(wind,[sign*12,20,3]),(hit,[sign*8,21,-12.6]),(hit+.10,[sign*8,20,-11.5]),(end,[sign*10,12,0])],t)
                        arm_targets[side]=a;arm_ori[side]=rot([90,0,0])
                        yaw=float(curve([(start,[0]),(wind,[sign*17]),(hit,[-sign*12]),(end,[0])],t)[0])
                        root[1]=float(curve([(start,[-1]),(wind,[-2.5]),(hit,[-1.5]),(end,[-1])],t)[0])
                        pitch=-8*math.sin(math.pi*(t-start)/(end-start))
            elif name=="slam":
                root[1]=float(curve([(0,[-1]),(.80,[-1]),(1.02,[-7.8]),(1.35,[-7.3]),(length,[-1])],t)[0])
                pitch=float(curve([(0,[0]),(.80,[12]),(1.02,[-48]),(1.35,[-42]),(length,[0])],t)[0])
                for s,sign in [("left",-1),("right",1)]:
                    arm_targets[s]=curve([(0,[sign*10,12,0]),(.80,[sign*10,35,-3]),(1.02,[sign*10,4.0,-11]),(1.35,[sign*10,4.2,-10]),(length,[sign*10,12,0])],t)
                    arm_ori[s]=eye
                    feet[s][2]=2.0
            elif name=="eruption":
                root[1]=float(curve([(0,[-1]),(1.10,[-5]),(1.35,[-2]),(length,[-1])],t)[0])
                pitch=float(curve([(0,[0]),(1.1,[-25]),(1.35,[12]),(length,[0])],t)[0])
                for s,sign in [("left",-1),("right",1)]:
                    arm_targets[s]=curve([(0,[sign*10,12,0]),(1.1,[sign*5,21,-7]),(1.35,[sign*16,24,-6]),(length,[sign*10,12,0])],t)
            elif name in ("guard","guard_break"):
                end=length;hold=.35 if name=="guard" else .20
                for s,sign in [("left",-1),("right",1)]:
                    goal=[sign*3,22,-8] if name=="guard" else [sign*15,23,2]
                    arm_targets[s]=curve([(0,[sign*10,12,0]),(hold,goal),(end*.75,goal),(end,[sign*10,12,0])],t)
                    arm_ori[s]=rot([80,0,0]) if name=="guard" else eye
                root[1]=-1-1.5*math.sin(math.pi*u);pitch=-12*math.sin(math.pi*u)
            elif name in ("dormant","awaken"):
                q=1 if name=="dormant" else float(curve([(0,[1]),(.65,[1]),(length,[0])],t)[0])
                root[1]=-1-3*q;pitch=-25*q;head=[-15*q,0,0]
                for s,sign in [("left",-1),("right",1)]:arm_targets[s]=[sign*10,12-5*q,-4*q];arm_ori[s]=eye
            elif name=="stagger":
                pitch=float(curve([(0,[0]),(.16,[18]),(.4,[-8]),(length,[0])],t)[0]);roll=7*math.sin(math.pi*u)
            elif name=="enraged":
                root[1]=-1-2*math.sin(math.pi*u);pitch=12*math.sin(math.tau*u)
                for s,sign in [("left",-1),("right",1)]:arm_targets[s]=curve([(0,[sign*10,12,0]),(.8,[sign*16,24,-2]),(length,[sign*10,12,0])],t)
            elif name in ("death","collapse"):
                q=float(curve([(0,[0 if name=="death" else 1]),(length,[1])],t)[0])
                root[1]=-1-8*q;pitch=-82*q;head=[-15*q,0,0]
                for s,sign in [("left",-1),("right",1)]:
                    arm_targets[s]=[sign*(10+2*q),12-8*q,-7*q];arm_ori[s]=eye;feet[s][2]=3*q
            else:
                root[1]=-1+.14*math.sin(math.tau*u);roll=1.2*math.sin(math.tau*u);head=[0,4*math.sin(math.tau*u),0]
            rig.p["root"]=root;rig.r["torso"]=[pitch,yaw,roll];rig.r["head"]=head
            for side in ("left","right"):
                sign=-1 if side=="left" else 1
                limb(rig,side,"leg",feet[side])
                target=arm_targets.get(side,np.array([sign*10,12+root[1],-1.0]))
                limb(rig,side,"arm",target,arm_ori.get(side,eye))
                rig.r[side+"_shoulder_plates"]=[float(rig.r[side+"_arm"][0])*.08,0,0]
                for j in range(3):rig.r[side+"_finger_"+str(j)]=[18 if name in attack_hits else 6,0,0]
            # Evaluate the actual FK hand socket and foot positions at impact.
            for hit in attack_hits.get(name,[]):
                if abs(t-hit)<1e-4:
                    s="left" if name=="left_punch" or name=="combo" and hit<1 else "right"
                    if name in ("left_punch","right_punch","combo"):
                        hand=rig.pivot(s+"_hand_fx");advance=float(np.dot(hand-rest[s+"_hand_fx"],FRONT))
                        checks.append(dict(clip=name,time=t,kind="forward_hit",advance=advance,passed=advance>6))
                    elif name=="slam":
                        heights=[float(rig.pivot(s+"_hand_fx")[1]) for s in ("left","right")]
                        checks.append(dict(clip=name,time=t,kind="ground_impact",heights=heights,passed=max(heights)<4.0))
            if name not in ("death","collapse"):
                for side in ("left","right"):
                    error=float(np.linalg.norm(rig.pivot(side+"_foot")-feet[side]))
                    if error>.45:checks.append(dict(clip=name,time=t,kind="foot_contact",error=error,passed=False))
            row={}
            for channel,data in [("rotation",rig.r),("position",rig.p)]:
                for b,v in data.items():
                    v=np.array(v,float)
                    if channel=="rotation" and (b,channel) in previous:v=previous[b,channel]+(v-previous[b,channel]+180)%360-180
                    previous[b,channel]=v;row[b,channel]=v
            samples.append((t,row))
        # Bake only nonconstant tracks; keep endpoints; wrap discontinuities were unwrapped.
        tracks=sorted(set(k for _,row in samples for k in row))
        for b,ch in tracks:
            vals=[row.get((b,ch),np.zeros(3)) for _,row in samples]
            selected=range(len(samples)) if any(np.linalg.norm(v-vals[0])>.001 for v in vals[1:]) else [0,len(samples)-1]
            for i in selected:keys.append(dict(op="set_keyframe",animation=clip,bone=b,channel=ch,time=samples[i][0],value=[round(float(x),5) for x in vals[i]],interpolation="linear"))
        pose_recipes.append(dict(clip=name,duration=length,samples=len(samples),tracks=len(tracks)))
    report=dict(front_axis="-Z",solver="analytic two-bone IK, baked smoothed poses",checks=checks,clips=pose_recipes,failed=sum(not x["passed"] for x in checks))
    (out/"motion-gates.json").write_text(json.dumps(report,indent=2))
    # Attack gates must pass before the candidate is called validated; foot errors remain visible.
    if any(not c["passed"] for c in checks if c["kind"] in ("forward_hit","ground_impact")):raise ValueError("attack contact gate failed; see motion-gates.json")
    return keys

def self_test():
    r=Rig([dict(name="a",origin=[0,10,0]),dict(name="b",parent="a",origin=[0,5,0]),dict(name="c",parent="b",origin=[0,0,0])])
    assert r.ik("a","b","c",[0,8,-8],[1,0,0])<1e-5
    assert r.pivot("c")[2]<-7.99
    assert np.allclose(rot(euler(rot([37,23,-18]))),rot([37,23,-18]))
    assert np.dot(rot([90,0,0])@np.array([0,-1,0]),FRONT)>.999
if __name__=="__main__":self_test()
