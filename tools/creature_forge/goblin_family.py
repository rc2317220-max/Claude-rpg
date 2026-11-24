#!/usr/bin/env python3
"""Goblin family production candidate. Shared anatomy/atlas, separate weapon rigs,
pose-driven role combat and depth-tested FX. Actual MCP writes; offline only."""
import argparse,hashlib,json,math,os,subprocess,sys,time
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/"tools/asset_studio"),str(ROOT/"tools")]
from art_brain import Sculpt,atlas
from motion_lab import Rig,align,euler,rot,curve,self_test
from studio_v2 import load,polygons,uv_check,validate_events,rust_export
from creature_inspector import rotation,around
import world_fx
P=argparse.ArgumentParser();P.add_argument("--out",type=Path,required=True);P.add_argument("--role",choices=["knife","spear","archer"],required=True);P.add_argument("--preflight",action="store_true")
a=P.parse_args();out=a.out.resolve();out.mkdir(parents=True,exist_ok=True);role=a.role;name="goblin_"+role
brief=json.loads((ROOT/"docs/creatures/GOBLIN_FAMILY_BRIEF.json").read_text());S=Sculpt();g,b,r,v=S.group,S.box,S.rod,S.volume
# Squat, broad, crouched scavenger. Thick rib cage and hunched back, huge angular head,
# strong forearms, broad feet and layered irregular gear. All rest pivots in absolute units.
g("root",[0,0,0]);g("pelvis",[0,10.2,0],"root");g("torso",[0,12,0],"pelvis")
b("pelvic_mass","pelvis",[-3.75,9.1,-2.5],[3.75,12.2,2.8],"skin_shadow")
v("ribcage","torso",[0,11.6,0],[0,18.9,.65],[6.4,8.6,9.6,7.8],[4.9,5.5,6.1,4.8],"skin")
b("hunched_back","torso",[-3.8,15,1.8],[3.8,19.7,3.8],"skin_shadow",[12,0,0],[0,17,2])
b("lower_belly","torso",[-3.25,11.3,-3.15],[3.25,14.5,-2.0],"skin_light")
for j in range(3):
    b("rib_fold_"+str(j),"torso",[-3.3+j*.20,14.1+j*.88,-3.22],[3.3-j*.20,14.46+j*.88,-2.6],"skin_shadow")
g("neck",[0,18.3,-.35],"torso");v("neck","neck",[0,18,-.25],[0,20.9,-1.5],[4.5,4.1],[4.3,4.0],"skin_shadow")
g("head",[0,21,-1.8],"neck");g("jaw",[0,20.45,-3.25],"head")
# Angular skull, strong cheekbones and jutting chin rather than a single rectangular head.
v("skull","head",[0,20.5,-1.8],[0,27.4,-1.65],[5.8,8.2,8.8,7.5],[5.1,6.0,6.2,5.15],"skin")
b("forehead_plate","head",[-3.25,25,-5.0],[3.25,26.65,-3.7],"skin_light")
b("forehead_crease","head",[-2.6,25.1,-5.13],[2.6,25.4,-4.75],"skin_shadow")
b("lower_jaw","jaw",[-3.05,19.5,-4.2],[3.05,21.25,-.7],"skin_shadow")
b("chin","jaw",[-2.5,19.65,-4.85],[2.5,20.8,-3.65],"skin")
b("mouth_dark","head",[-2.65,21.05,-4.92],[2.65,21.75,-4.4],"lip")
b("lower_lip","jaw",[-2.35,20.85,-5.08],[2.35,21.25,-4.52],"lip")
# Long crooked nose, two nostrils, a hooked pointed end.
v("nose","head",[0,24.1,-4.25],[0,22.05,-7.15],[2.1,1.85,1.45],[1.9,1.7,1.35],"skin")
b("nose_tip","head",[-.84,21.8,-7.55],[.84,22.9,-6.45],"skin_light")
for j in range(5):
    x=(j-2)*.87;b("upper_tooth_"+str(j),"head",[x-.31,21.0-(j%2)*.14,-5.1],[x+.32,21.64,-4.65],"teeth")
for j in range(3):
    x=(j-1)*1.02;b("lower_tooth_"+str(j),"jaw",[x-.31,21.05,-5.07],[x+.31,21.54+(j%2)*.10,-4.55],"teeth")
# Scalp cropped mohawk, asymmetric mended gear.
for j in range(5):
    z=-3+j*.85;r("hair_tuft_"+str(j),"head",[0,26.2,z],[.2*(j%2),28.1-(j%3)*.35,z+1.0],1.0,.84,"hair")
b("vest_back","torso",[-4.5,12.7,2.65],[4.5,18.9,3.55],"leather_dark")
b("vest_front","torso",[-3.75,12.25,-3.45],[3.75,14.2,-2.8],"leather")
r("diagonal_harness","torso",[-3.9,18.25,-3.02],[3.1,12.3,-3.25],1.25,.48,"leather_dark")
r("harness_edge","torso",[-3.8,18.2,-3.34],[3.2,12.3,-3.52],.22,.22,"stitch")
b("belt","pelvis",[-4.05,10.8,-3.1],[4.05,12.15,3.15],"leather_dark")
b("buckle","pelvis",[-1.1,10.87,-3.52],[1.1,12.04,-3.05],"metal")
b("buckle_inset","pelvis",[-.68,11.16,-3.65],[.68,11.77,-3.47],"leather_dark")
g("loincloth",[0,11,-2.8],"pelvis")
for j in range(3):
    x=(j-1)*1.66;b("rag_panel_"+str(j),"loincloth",[x-.81,7.7+(j%2)*.48,-3.12],[x+.81,11.15,-2.62],"cloth_red" if j==1 else "cloth")
    for k in range(3):b("rag_fray_"+str(j)+"_"+str(k),"loincloth",[x-.74+k*.49,7.36+(j%2)*.48+(k%2)*.25,-3.13],[x-.45+k*.49,7.9+(j%2)*.48,-2.62],"cloth")
g("pouch",[3.75,11.4,1.0],"pelvis")
b("pouch_body","pouch",[3.28,9.3,-.1],[5.32,11.8,2.2],"leather")
b("pouch_lid","pouch",[3.12,11.25,-.24],[5.42,12.0,2.29],"leather_dark")
b("pouch_button","pouch",[5.24,10.85,.65],[5.54,11.25,1.13],"bone")
for j in range(4):r("pouch_stitch_"+str(j),"pouch",[5.43,9.65+j*.46,.1],[5.43,9.82+j*.46,.46],.16,.16,"stitch")
for side,sgn in [("l",-1),("r",1)]:
    def m(p):return [sgn*p[0],p[1],p[2]]
    def mb(n,bone,A,B,mat,ang=None,piv=None):
        A,B=m(A),m(B);b(side+"_"+n,bone,np.minimum(A,B),np.maximum(A,B),mat,[ang[0],sgn*ang[1],sgn*ang[2]] if ang else None,m(piv) if piv else None)
    def mr(n,bone,A,B,w,d,mat):r(side+"_"+n,bone,m(A),m(B),w,d,mat)
    def mv(n,bone,A,B,ws,ds,mat):v(side+"_"+n,bone,m(A),m(B),ws,ds,mat)
    # Face expression: angular brow lowered toward center, cheek fat, pupils and eye rim.
    mb("cheek","head",[2.5,21.65,-4.8],[4.45,24.0,-2.1],"skin_shadow",[0,0,-13],[3.5,22.9,-3.1])
    mb("socket","head",[.95,23.0,-5.06],[3.68,24.68,-4.42],"skin_shadow")
    mb("eye","head",[1.27,23.34,-5.22],[3.28,24.15,-4.98],"eye")
    mb("pupil","head",[1.87,23.38,-5.32],[2.36,24.13,-5.19],"pupil")
    mr("brow","head",[.77,24.10,-5.43],[3.65,24.98,-4.70],1.05,.9,"skin_light")
    mr("cheek_wrinkle","head",[1.4,22.5,-5.0],[3.6,23.0,-4.65],.30,.31,"skin_shadow")
    mb("nostril","head",[.5,22.16,-7.31],[.94,22.54,-6.9],"skin_shadow")
    mr("lower_tusk","jaw",[2.3,20.5,-4.66],[2.67,22.11,-5.05],.62,.64,"teeth")
    ear="ear_"+side;g(ear,m([3.7,24,-1.1]),"head")
    # Thick root, thin taper, jagged notched right ear; visible inner surface.
    E=[3.65,24.1,-1.1];M=[6.0,24.9,-.45];T=[8.2 if side=="l" else 7.6,26.6 if side=="l" else 25.8,.7]
    mr("ear_root",ear,E,M,2.1,1.45,"skin")
    mr("ear_mid",ear,M,[(M[j]+T[j])/2 for j in range(3)],1.65,.95,"skin")
    mr("ear_point",ear,[(M[j]+T[j])/2 for j in range(3)],T,.78,.55,"skin_light")
    mr("inner_ear",ear,[4.15,24.2,-1.87],[6.58,25.5,-.91],.84,.35,"ear")
    if side=="l":
        for j in range(3):mr("ear_ring_"+str(j),ear,[5.05+j*.25,23.85+j*.13,-1.17],[5.18+j*.25,23.31+j*.13,-1.03],.26,.29,"metal")
    else:
        mr("scar_1","head",[2.4,25.0,-5.02],[3.48,23.0,-5.09],.29,.28,"scar")
        mr("scar_2","head",[3.48,23,-5.09],[3.82,22.65,-4.89],.22,.24,"scar")
    # Arm joints and expressive hands. Broad shoulders and bulging, tapered forearms.
    A=[4.9,18.3,.05];E=[6.0,13.65,.1];H=[6.0,9.1,-.6]
    arm="arm_"+side;el="elbow_"+side;hand="hand_"+side
    g(arm,m(A),"torso");g(el,m(E),arm);g(hand,m(H),el)
    mv("upper_arm",arm,A,E,[3.6,3.1,2.55],[3.3,3.0,2.6],"skin")
    mv("forearm",el,E,H,[2.85,3.15,2.05],[2.9,2.8,2.1],"skin")
    mb("knuckle_mass",hand,[4.95,7.7,-1.75],[7.04,9.65,.5],"skin_shadow")
    for j in range(4):
        x=5.13+j*.52;finger="finger_"+side+"_"+str(j);g(finger,m([x,8.38,-1.34]),hand)
        mr("finger_"+str(j),finger,[x,8.6,-1.35],[x,7.31,-1.8],.48,.55,"skin")
        mr("nail_"+str(j),finger,[x,7.49,-1.98],[x,7.15,-2.08],.34,.28,"bone")
    thumb="thumb_"+side;g(thumb,m([5.02,9.15,-1.25]),hand)
    mr("thumb",thumb,[5.02,9.15,-1.25],[4.7,8.15,-1.87],.66,.65,"skin")
    # Asymmetric shoulder pad, stitched bracer, wraps.
    if side=="l":
        mb("shoulder_pad",arm,[3.7,17.5,-1.88],[7.1,19.2,1.81],"leather_dark",[0,0,-14],[5,18,0])
        mb("shoulder_plate",arm,[4.45,18.2,-2.02],[6.98,19.35,1.2],"metal",[0,0,-14],[5,18,0])
        for j in range(3):mb("shoulder_rivet_"+str(j),arm,[4.65+j*.8,18.56,-2.15],[4.94+j*.8,18.9,-1.9],"bone")
    else:
        for j in range(3):mb("arm_wrap_"+str(j),arm,[4.38,15.5+j*.51,-1.62],[7.04,15.9+j*.51,1.58],"cloth_red",[0,0,-6],[6,16,0])
    for j in range(3):
        mb("bracer_"+str(j),el,[4.52,10.0+j*.52,-1.92],[7.54,10.41+j*.52,1.3],"leather" if j!=1 else "leather_dark")
    # Short, bent legs and oversized feet. Hip-to-foot IK has slack for lunges.
    A=[2.6,10.7,.4];K=[3.1,5.9,-1.0];F=[3.5,1.25,.35]
    leg="leg_"+side;shin="shin_"+side;foot="foot_"+side
    g(leg,m(A),"pelvis");g(shin,m(K),leg);g(foot,m(F),shin)
    mv("thigh",leg,A,K,[3.7,3.5,2.7],[3.8,3.45,3.0],"skin")
    mv("shin",shin,K,F,[2.7,2.45,2.2],[2.9,2.55,2.25],"skin_shadow")
    mb("knee_pad",shin,[1.65,5.35,-2.55],[4.55,7.12,-.65],"leather" if side=="l" else "cloth")
    for j in range(2):mb("ankle_wrap_"+str(j),shin,[2.13,2.0+j*.62,-1.1],[4.77,2.43+j*.62,1.75],"cloth")
    mb("foot_pad",foot,[1.8,.1,-2.35],[5.22,1.75,1.75],"leather_dark" if side=="l" else "skin_shadow")
    for j in range(3):
        x=2.16+j*.97;toe="toe_"+side+"_"+str(j);g(toe,m([x,1.05,-1.64]),foot)
        mr("toe_"+str(j),toe,[x,.92,-1.8],[x,.58,-3.15],.8,.85,"leather" if side=="l" else "skin")
        if side=="r":mr("toe_nail_"+str(j),toe,[x,.55,-2.98],[x,.32,-3.48],.58,.47,"bone")
g("dust_fx",[0,.2,0],"root")
# Add all equipment to one atlas, then select role subtree. Weapons in absolute rest coordinates;
# grip orientation local axes are explicit: spear/knife forward -Z, bow vertical +Y.
H=np.array([6.,9.1,-.6]);L=np.array([-6.,9.1,-.6])
def weapon_group(n,p,parent):g(n,p,parent)
weapon_group("knife_gear",H,"hand_r")
r("knife_handle","knife_gear",H+[0,0,1.35],H+[0,0,-1.1],.84,.9,"wood")
for j in range(3):b("knife_wrap_"+str(j),"knife_gear",H+[-.53,-.53,.75-j*.58],H+[.53,.53,1.04-j*.58],"leather_dark")
b("knife_guard","knife_gear",H+[-1.38,-.4,-1.34],H+[1.38,.4,-.92],"rust")
v("knife_blade","knife_gear",H+[0,0,-1.1],H+[0,0,-6.15],[1.65,1.48,.96,.35],[.55,.48,.36,.25],"metal")
r("knife_cutting_edge","knife_gear",H+[.72,0,-1.25],H+[.16,0,-6.0],.23,.30,"edge")
g("knife_tip",H+[0,0,-6.15],"knife_gear");g("knife_grip",H,"knife_gear")
weapon_group("spear_gear",H,"hand_r")
r("spear_shaft","spear_gear",H+[0,0,6.1],H+[0,0,-11.5],.82,.84,"wood")
for j in range(5):b("spear_binding_"+str(j),"spear_gear",H+[-.52,-.52,-.4-j*.42],H+[.52,.52,-.15-j*.42],"rope")
v("spear_head","spear_gear",H+[0,0,-10.5],H+[0,0,-15.4],[1.2,1.75,1.1,.32],[.68,.64,.47,.22],"metal")
r("spear_edge","spear_gear",H+[.72,0,-11.5],H+[.05,0,-15.1],.22,.29,"edge")
g("spear_tip",H+[0,0,-15.4],"spear_gear");g("spear_grip",H,"spear_gear");g("spear_support",H+[0,0,-3.1],"spear_gear")
weapon_group("bow_gear",L,"hand_l")
# Recurve shape: tips trail +Z, belly extends -Z. Rest string is a straight vertical line.
for sign,tag in [(-1,"low"),(1,"high")]:
    pts=[L+[0,0,0],L+[0,sign*3.0,-.9],L+[0,sign*5.4,-.20],L+[0,sign*7.0,1.8]]
    for j,(u,w) in enumerate(zip(pts,pts[1:])):r("bow_"+tag+"_limb_"+str(j),"bow_gear",u,w,[1.05,.78,.48][j],[1.05,.73,.48][j],"wood")
    g("string_"+tag,pts[-1],"bow_gear")
    r("bow_string_"+tag,"string_"+tag,pts[-1],L+[0,0,1.8],.16,.16,"rope")
    g("string_tip_"+tag,pts[-1],"bow_gear")
b("bow_wrap","bow_gear",L+[-.64,-1.0,-.58],L+[.64,1.0,.58],"leather_dark")
g("bow_forward",L+[0,0,-.85],"bow_gear");g("bow_nock",L+[0,0,1.8],"bow_gear")
g("quiver_gear",[0,16,3.4],"torso")
r("quiver","quiver_gear",[2,11.8,3.2],[-1.7,19.7,4.5],2.4,2.5,"leather")
for j in range(4):
    x=-1.6+j*.5;r("quiver_arrow_"+str(j),"quiver_gear",[x,15.4,4.15],[x-.45,22.6+(j%2)*.75,4.9],.26,.28,"wood")
    r("quiver_fletch_"+str(j),"quiver_gear",[x-.36,20.7+(j%2)*.75,4.8],[x-.45,22.6+(j%2)*.75,4.9],.72,.37,"fletch")
g("archer_draw",H,"hand_r")
# Atlas is deterministic and shared across separate models; body geometry and pixels identical.
tex,em=atlas(S,out)
def subtree(root):
    seen={root}
    for row in S.groups:
        if row.get("parent") in seen:seen.add(row["name"])
    return seen
gear={"knife":subtree("knife_gear"),"spear":subtree("spear_gear"),"archer":subtree("bow_gear")|subtree("quiver_gear")|subtree("archer_draw")}
allgear=set.union(*gear.values());exclude=allgear-gear[role]
G=[g for g in S.groups if g["name"] not in exclude];C=[c for c,mat in S.cubes if c["parent"] not in exclude]
body=[c for c,mat in S.cubes if c["parent"] not in allgear]
body_hash=hashlib.sha256(json.dumps(body,sort_keys=True).encode()).hexdigest()
atlas_hash=hashlib.sha256((out/"goblin_shared.png").read_bytes()).hexdigest()
common=[("idle",1.8,True),("walk",1.0,True),("run",.72,True),("threat",1.4,False),("stagger",.52,False),("death",1.8,False)]
specific={"knife":[("slash",.98,False),("stab",.95,False),("double_cut",1.55,False),("rush",.70,False)],
"spear":[("thrust",1.15,False),("long_thrust",1.35,False),("retreat_poke",1.30,False),("backstep",.8,False)],
"archer":[("aimed_shot",1.65,False),("quick_shot",1.0,False),("retreat_shot",1.55,False),("backstep",.8,False)]}
clips=common+specific[role];events=[];attacks=[]
def ev(c,t,k,s,fx,source=None):
    e=dict(clip=c,at=t,kind=k,socket=s,effect=fx,id=c+"_"+str(len(events)))
    if source:e["direction_from"]=source
    events.append(e)
def attack(c,wind,end,cd,shape,reach,mul,hits,tip,fx,source):
    attacks.append(dict(clip=c,windup=wind,active_end=end,cooldown=cd,shape=shape,range=reach,damage_multiplier=mul))
    for h in hits:
        ev(c,h,"hit",tip,shape,source);ev(c,h,"vfx",tip,fx,source);ev(c,h,"sound",tip,role+"_"+fx)
if role=="knife":
    attack("slash",.37,.65,1.1,"arc",1.9,1.0,[.49],"knife_tip","knife_cut","knife_grip")
    attack("stab",.34,.64,1.1,"pierce",2.0,1.2,[.46],"knife_tip","knife_cut","knife_grip")
    attack("double_cut",.34,1.14,2.2,"arc",1.9,.8,[.46,1.04],"knife_tip","knife_cut","knife_grip")
    ev("rush",.12,"vfx","dust_fx","dust")
elif role=="spear":
    for c,h,w,end,cd,reach,mul in [("thrust",.58,.40,.76,1.6,3.2,1.0),("long_thrust",.70,.50,.88,2.3,4.2,1.3),("retreat_poke",.65,.45,.86,1.9,3.1,.9)]:
        attack(c,w,end,cd,"pierce",reach,mul,[h],"spear_tip","spear_thrust","spear_grip")
    ev("backstep",.12,"vfx","dust_fx","dust")
    ev("death",.85,"state","spear_support","release_support_grip")
else:
    for c,h,w,end,cd,reach,mul in [("aimed_shot",1.02,.80,1.21,2.4,18,1.3),("quick_shot",.54,.38,.73,1.4,13,.8),("retreat_shot",.90,.67,1.12,2.2,16,1.0)]:
        attack(c,w,end,cd,"projectile",reach,mul,[h],"bow_forward","arrow","bow_nock")
    ev("backstep",.12,"vfx","dust_fx","dust")
ev("death",1.2,"vfx","dust_fx","dust")
spec=dict(schema=2,creature=name,authority="server",runtime_integrated=False,front="-Z",role=brief["roles"][role],attacks=attacks,events=events,body_sha256=body_hash,atlas_sha256=atlas_hash)
(out/"events.json").write_text(json.dumps(spec,indent=2))
self_test();world_fx.self_test();rig=Rig(G);rest={b:rig.pivot(b) for b in rig.groups};eye=np.eye(3);keys=[];checks=[];poses={}
def scalar(t,rows):return float(curve([(x,[y]) for x,y in rows],t)[0])
def orient_forward(direction):return align([0,0,-1],direction)
def hand(side,target,orientation):
    return rig.ik("arm_"+side,"elbow_"+side,"hand_"+side,target,[-1 if side=="l" else 1,.2,.45],orientation)
def coords(n):return rig.pivot(n)
def weapon_world(n,point):
    R,T=rig.world(n);return R@np.asarray(point)+T
for c,Lt,loop in clips:
    full="animation."+name+"."+c;keys.append(dict(op="add_animation",name=full,length=Lt,loop="loop" if loop else "once",snapping=24))
    ts=sorted({round(i*Lt/math.ceil(Lt*20),5) for i in range(math.ceil(Lt*20)+1)}|{e["at"] for e in events if e["clip"]==c})
    tracks={};previous={};poses[c]=len(ts)
    hit_times=[e["at"] for e in events if e["clip"]==c and e["kind"]=="hit"]
    for t in ts:
        u=t/Lt;rig.reset();root=np.array([0.,-1.35,0.]);pitch=-8.;yaw=0.;roll=0.;headpitch=4.;jaw=3.;q=0.;draw=0.
        footgoals={};active_foot=None
        if c in ("walk","run","rush","backstep"):
            step={"walk":2.5,"run":3.5,"rush":3.8,"backstep":-2.6}[c]
            root[1]+=.17*math.cos(math.tau*u*2);pitch=-12 if c=="walk" else -20 if c in ("run","rush") else 3
            for side in ("l","r"):
                phase=(u+(.5 if side=="r" else 0))%1;p=rest["foot_"+side].copy()
                if phase<.60:p[2]+=-step/2+step*phase/.60
                else:
                    f=(phase-.60)/.4;p[2]+=step/2-step*f;p[1]+=1.15*math.sin(math.pi*f)
                footgoals[side]=p
        elif c=="threat":
            q=scalar(t,[(0,0),(.45,1),(.95,.75),(Lt,0)]);jaw=28*q;pitch=-11*q-8;root[1]-=.8*q
        elif c=="stagger":q=math.sin(math.pi*u);pitch=12*q-8;roll=9*q;root[1]-=.65*q
        elif c=="death":
            q=scalar(t,[(0,0),(.5,.25),(1.4,1),(Lt,1)]);root[1]-=5.5*q;pitch=-65*q-8;roll=18*q;jaw=13*q
        if hit_times:
            # Distinct anticipation/release/recovery timing; no generic rotation-only clip.
            hit=min(hit_times,key=lambda h:abs(h-t));start=max(0,hit-.40);wind=hit-.13;end=min(Lt,hit+.33)
            if start<=t<=end:
                q=scalar(t,[(start,0),(wind,-1),(hit,1),(end,0)])
                root[1]-=.55*abs(q);pitch=-8-7*max(0,q);yaw=12*q if role=="knife" else -4*q
                if c in ("rush","long_thrust","stab","retreat_poke"):
                    root[2]=-1.0*max(0,q) if c!="retreat_poke" else .7*max(0,q)
        if role=="archer" and hit_times:
            h=hit_times[0]
            draw=scalar(t,[(0,0),(h*.38,.25),(h-.05,1),(h,1),(min(Lt,h+.06),0),(Lt,0)])
            yaw=-12*draw;pitch=-5;root[1]-=.32*draw
        rig.p["root"]=root;rig.r["pelvis"]=[pitch*.25,yaw*.3,roll*.3];rig.r["torso"]=[pitch,yaw*.7,roll]
        rig.r["neck"]=[-pitch*.55,0,0];rig.r["head"]=[headpitch+3*math.sin(math.tau*u),-yaw*.5,0];rig.r["jaw"]=[-jaw,0,0]
        rig.r["loincloth"]=[8*math.sin(math.tau*u) if c in ("walk","run","rush") else -3*q,0,0]
        rig.r["pouch"]=[4*math.sin(math.tau*u),0,3*math.sin(math.tau*u)]
        for side,sign in [("l",-1),("r",1)]:
            rig.r["ear_"+side]=[3*math.sin(math.tau*u+sign*.8),0,sign*3*math.sin(math.tau*u)]
            p=footgoals.get(side,rest["foot_"+side].copy())
            if c=="death":p[0]+=sign*1.2*u
            err=rig.ik("leg_"+side,"shin_"+side,"foot_"+side,p,[sign*.12,0,-1],eye)
            if c not in ("death","walk","run","rush","backstep"):checks.append(dict(clip=c,t=t,kind="stance",error=round(err,4),passed=bool(err<.5)))
        # Weapon-specific body language and grips.
        if role=="knife":
            right=np.array([5.5,13.2,-2.9]);direction=np.array([.10,.12,-1.])
            if hit_times and q:
                if c=="stab":right=np.array([5.0,15.2,-3.1])+np.array([-1.3,.2,-3.6])*max(0,q)+np.array([1.4,.4,2.5])*max(0,-q);direction=np.array([-.05,.02,-1])
                else:
                    reverse=c=="double_cut" and t>.77
                    right=np.array([5.4,14.2,-3.1])+np.array([-3.1 if not reverse else 1.2,.6,-3.25])*max(0,q)+np.array([1.25 if not reverse else -2.7,1.4,1.25])*max(0,-q)
                    direction=np.array([(-.70 if not reverse else .70)*q,.1,-1.])
            if c=="run" or c=="rush":right[2]+=.95*math.sin(math.tau*u)
            hand("r",right,orient_forward(direction))
            left=np.array([-5.5,12.7,-2.9])+np.array([-.55,1.1,1.8])*max(0,q)
            hand("l",left,orient_forward([-.3,0,-1]))
        elif role=="spear":
            right=np.array([1.3,14.4,-1.3]);direction=np.array([-.45,.05,-1.])
            if hit_times:
                right+=np.array([-.25,.3,-3.25])*max(0,q)+np.array([.45,.2,2.1])*max(0,-q)
                direction=np.array([-.45,.04,-1.])
            # During death the off-hand releases the shaft deliberately. The weapon
            # stays attached to the right hand, which follows the falling torso.
            # Live two-handed poses retain the original grip gate.
            release=scalar(t,[(0,0),(.85,0),(1.22,1),(Lt,1)]) if c=="death" else 0.
            if release>0:
                R,T=rig.world("torso")
                fallen=R@np.array([4.8,11.1,-1.0])+T
                right=right*(1-release)+fallen*release
                direction=np.asarray(direction)*(1-release)+(R@np.array([0.,0.,-1.]))*release
            hand("r",right,orient_forward(direction))
            support=coords("spear_support")
            target=support.copy()
            if release>0:
                R,T=rig.world("torso")
                relaxed=R@np.array([-6.0,10.0,-1.4])+T
                target=support*(1-release)+relaxed*release
            err=hand("l",target,orient_forward(direction) if release<.5 else eye)
            kind="support_grip" if release==0 else "released_hand_target"
            checks.append(dict(clip=c,t=t,kind=kind,error=round(err,4),passed=bool(err<1.8)))
            if release>=.999:
                gap=float(np.linalg.norm(coords("hand_l")-support))
                checks.append(dict(clip=c,t=t,kind="released_hand_separation",gap=round(gap,4),passed=bool(gap>1.8)))
        else:
            # Bow held to the left, right hand pulls along the arrow axis. String bends
            # toward the actual right-hand draw point; release snaps back.
            left=np.array([-4.7,15.2,-4.1])+np.array([.7,.9,-1.3])*draw
            bowR=orient_forward([.12,.02,-1])
            hand("l",left,bowR)
            nock0=coords("bow_nock");axis=bowR@np.array([0,0,1.])
            nock=nock0+axis*3.55*draw
            hand("r",nock,orient_forward([0,0,-1]))
            # Bow nock socket moves with the drawn string, not with the camera.
            parentR,_=rig.world("bow_gear");rig.p["bow_nock"]=parentR.T@(nock-nock0)
            for tag in ("high","low"):
                bone="string_"+tag;pt=coords("string_tip_"+tag);restvec=rest["bow_nock"]-rest["string_tip_"+tag]
                worldvec=nock-pt;parentR,_=rig.world("bow_gear")
                rig.r[bone]=euler(parentR.T@align(restvec,worldvec))
                rig.s[bone]=[1,float(np.linalg.norm(worldvec)/np.linalg.norm(restvec)),1]
        for side in ("l","r"):
            grasp=side=="r" if role=="knife" else not (role=="spear" and side=="l" and c=="death" and t>=.85)
            for j in range(4):rig.r["finger_"+side+"_"+str(j)]=[(-42 if grasp else -12)+(j-1.5)*3,0,0]
            rig.r["thumb_"+side]=[-22 if grasp else -7,0,0]
        for e in events:
            if e["clip"]==c and e["kind"]=="hit" and abs(t-e["at"])<1e-5:
                tip=coords(e["socket"]);source=coords(e["direction_from"]);d=tip-source
                checks.append(dict(clip=c,t=t,kind="direction",tip=tip.tolist(),axis=d.tolist(),passed=bool(d[2]<-.5 and tip[2]<-4)))
        for ch,vals in [("rotation",rig.r),("position",rig.p),("scale",rig.s)]:
            for bone,val in vals.items():
                val=np.asarray(val,float);k=(bone,ch)
                if ch=="rotation" and k in previous:val=previous[k]+(val-previous[k]+180)%360-180
                previous[k]=val;tracks.setdefault(k,[]).append((t,val))
    for (bone,ch),rows in tracks.items():
        # All channels here have entries on every sample except bow strings, which are continuous.
        chosen=rows if any(np.linalg.norm(v-rows[0][1])>.0001 for _,v in rows[1:]) else [rows[0],rows[-1]]
        for t,val in chosen:keys.append(dict(op="set_keyframe",animation=full,bone=bone,channel=ch,time=t,value=[round(float(x),5) for x in val],interpolation="linear"))
bad=[x for x in checks if not x["passed"]]
report=dict(role=role,bones=len(G),cubes=len(C),shared_body_cubes=len(body),atlas_sha256=atlas_hash,body_sha256=body_hash,clips=len(clips),keyframes=sum(x["op"]=="set_keyframe" for x in keys),checks=len(checks),failed=bad,pose_samples=poses)
(out/"motion-report.json").write_text(json.dumps(report,indent=2))
print(json.dumps({k:v for k,v in report.items() if k!="failed"}),flush=True)
if bad:raise ValueError("pose gates failed: "+str(bad[:8]))
if a.preflight:sys.exit(0)
env=dict(os.environ,RUSTPG_ASSET_ROOT=str(out));model=name+".bbmodel"
def call(tool,args,log):
    f=out/(log+"-args.json");f.write_text(json.dumps(args))
    q=subprocess.run(["bash",str(ROOT/"tools/asset_studio/bb.sh"),tool,"@"+str(f)],env=env,capture_output=True,text=True)
    (out/(log+".json")).write_text(q.stdout)
    if q.returncode:raise RuntimeError(tool+": "+q.stdout[-2000:]+" "+q.stderr[-2000:])
    return json.loads(q.stdout)
rev=call("bbmodel_create",dict(file=model,name="Goblin "+role.title(),format="free",box_uv=False,resolution=dict(width=1024,height=1024)),"01-create")["revision"]
rev=call("bbmodel_add_texture",dict(file=model,expected_revision=rev,image="goblin_shared.png",name="goblin_shared.png"),"02-texture")["revision"]
for i,batch in enumerate([G,C]+[keys[j:j+350] for j in range(0,len(keys),350)]):
    rev=call("bbmodel_edit",dict(file=model,expected_revision=rev,operations=batch),"03-edit-"+str(i))["revision"]
call("bbmodel_validate",dict(file=model,self_test=True),"04-geometry");call("bbmodel_validate_animations",dict(file=model),"05-animation")
doc,data,names=load(out/model);qa=uv_check(doc,tex,em,out);er=validate_events(doc,spec,names);rust_export(doc,data,names,spec,out)
(out/"ashen_sentinel_v3_generated.rs").rename(out/(name+"_generated.rs"))
# Index exported keys once. Independent FK includes scale and translation; used for real socket
# and depth-aware VFX from the actual saved BBModel, not authoring data.
idx={}
for cl,bone,ch,t,val in data["keys"]:idx.setdefault((cl,bone,ch),[]).append((t,val))
for rows in idx.values():rows.sort(key=lambda x:x[0])
def sam(cl,b,ch,t,default):
    rows=idx.get((cl,b,ch),[])
    if not rows:return np.array(default,float)
    if t<=rows[0][0]:return rows[0][1]
    for (a,u),(z,v) in zip(rows,rows[1:]):
        if t<=z:return u+(v-u)*(t-a)/max(z-a,1e-8)
    return rows[-1][1]
byname={v:k for k,v in names.items()}
from functools import lru_cache
@lru_cache(maxsize=8192)
def mat(cl,t,bone):
    if bone is None:return np.eye(4)
    parent,p,r=data["bones"][bone];tr=np.eye(4);tr[:3,3]=sam(cl,bone,"position",t,[0]*3)
    sc=np.eye(4);v=sam(cl,bone,"scale",t,[1]*3);sc[:3,:3]=np.diag(v);sc[:3,3]=p-v*p
    return mat(cl,t,parent)@tr@around(r+sam(cl,bone,"rotation",t,[0]*3),p)@sc
def socket(cl,t,n):
    bone=byname[n];return (mat(cl,float(t),bone)@np.append(data["bones"][bone][1],1))[:3]
saved=[]
for e in events:
    if e["kind"]!="hit":continue
    full="animation."+name+"."+e["clip"];tip=socket(full,e["at"],e["socket"]);source=socket(full,e["at"],e["direction_from"]);d=tip-source
    effect=next(v for v in events if v["kind"]=="vfx" and v["clip"]==e["clip"] and v["at"]==e["at"])
    paths=world_fx.effect_paths(effect,e["at"]+.08,socket,full)
    saved.append(dict(clip=e["clip"],t=e["at"],tip=tip.tolist(),axis=d.tolist(),paths=len(paths),passed=bool(d[2]<-.5 and tip[2]<-4 and paths)))
(out/"saved-direction-report.json").write_text(json.dumps(saved,indent=2))
assert all(e["passed"] for e in saved)
texarr=np.array(tex);rest_poly=polygons(data,None,0)
def render(poly,angle,size,center,extent,fx=()):
    buf,dep=world_fx.scene(poly,texarr,angle,size,center,extent)
    world_fx.draw_fx(buf,dep,fx,angle,size,center,extent)
    return Image.fromarray(buf)
def fit(poly,angle,size):
    pts=np.concatenate([p[0] for p in poly]);R=rotation(angle);q=pts@R.T;ctr=((q.min(0)+q.max(0))/2)@R
    return render(poly,angle,size,ctr,max(np.ptp(q[:,0]),np.ptp(q[:,1]))*1.05)
idle="animation."+name+".idle";idlepoly=polygons(data,idle,.45)
sheet=Image.new("RGB",(1024,1056),(24,27,32));d=ImageDraw.Draw(sheet);d.text((12,8),name.upper()+" | ACTUAL OFFLINE MODEL",fill="white")
for i,(n,ang) in enumerate([("front",(3,180,0)),("three-quarter",(5,145,0)),("side",(4,90,0)),("back",(4,0,0))]):
    im=fit(idlepoly,ang,512);im.save(out/(n+".png"));sheet.paste(im,(i%2*512,32+i//2*512))
sheet.save(out/"model-sheet.png")
h=socket(idle,.45,"head");render(idlepoly,(3,145,0),640,h+np.array([0,.5,-1.8]),13).save(out/"face-closeup.png")
frames=out/"frames";frames.mkdir(exist_ok=True);frame_i=0;segments=[]
pose_sheet=Image.new("RGB",(4*256,math.ceil(len(clips)/4)*280),(24,27,32));pd=ImageDraw.Draw(pose_sheet)
for i,(c,Lt,_) in enumerate(clips):
    full="animation."+name+"."+c;first=frame_i;hit=next((e["at"] for e in events if e["clip"]==c and e["kind"]=="hit"),Lt*.55)
    pose_sheet.paste(fit(polygons(data,full,hit),(8,145,0),256),(i%4*256,i//4*280+24));pd.text((i%4*256+6,i//4*280+6),c,fill="white")
    pts=np.concatenate([p[0] for t in np.linspace(0,Lt,7) for p in polygons(data,full,float(t))]);center=(pts.min(0)+pts.max(0))/2;extent=max(np.ptp(pts[:,j]) for j in range(3))*1.18
    # Extend forward framing for ranged projectile so it visibly travels away from the bow.
    if role=="archer" and any(e["clip"]==c and e["kind"]=="hit" for e in events):center[2]-=6;extent+=12
    for t in np.arange(0,Lt,1/16):
        t=float(t);poly=polygons(data,full,t);paths=[]
        for e in events:
            if e["clip"]==c and e["kind"]=="vfx":paths.extend(world_fx.effect_paths(e,t,socket,full))
        canvas=Image.new("RGB",(832,448),(24,27,32))
        for j,ang in enumerate([(8,145,0),(4,90,0)]):canvas.paste(render(poly,ang,416,center,extent,paths),(j*416,32))
        ImageDraw.Draw(canvas).text((10,8),role.upper()+" | "+c.upper()+" | WORLD-SPACE FX | OFFLINE",fill=(232,222,199))
        canvas.save(frames/f"{frame_i:04}.png");frame_i+=1
    segments.append(dict(clip=c,first=first,last=frame_i-1))
pose_sheet.save(out/"pose-sheet.png")
subprocess.run(["ffmpeg","-hide_banner","-loglevel","error","-y","-framerate","16","-i",str(frames/"%04d.png"),"-c:v","libx264","-pix_fmt","yuv420p",str(out/"animation-preview.mp4")],check=True)
subprocess.run(["rustc","--edition=2021","--test",str(out/(name+"_generated.rs")),"-o",str(out/"rust-tests")],check=True);subprocess.run([str(out/"rust-tests")],check=True);(out/"rust-tests").unlink()
(out/"summary.json").write_text(json.dumps(dict(kind="offline_asset_candidate",runtime_integrated=False,role=role,bones=len(G),cubes=len(C),body_sha256=body_hash,atlas_sha256=atlas_hash,clips=len(clips),attacks=len(attacks),frames=frame_i,events=er,uv={k:v for k,v in qa.items() if k!="faces_detail"},segments=segments),indent=2))
print((out/"summary.json").read_text())
