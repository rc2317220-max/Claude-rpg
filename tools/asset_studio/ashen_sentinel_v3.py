#!/usr/bin/env python3
"""Original Ashen Sentinel asset candidate. All model edits use real Blockbench MCP.
No stable RustPG asset or renderer is modified; output is offline art evidence.
"""
import argparse,base64,hashlib,json,math,os,random,subprocess,sys
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"tools"))
from creature_inspector import authored,geometry,render

p=argparse.ArgumentParser();p.add_argument("--out",type=Path,required=True);p.add_argument('--baseline',type=Path);a=p.parse_args()
out=a.out.resolve();out.mkdir(parents=True,exist_ok=True)
env=dict(os.environ,RUSTPG_ASSET_ROOT=str(out))
model="ashen_sentinel_v3.bbmodel"
atlas_name="ashen_sentinel_v3.png"
groups=[];cubes=[];keys=[];clips=[]
def group(name,origin,parent=None):
    g=dict(op="add_group",name=name,origin=origin)
    if parent:g["parent"]=parent
    groups.append(g)
def cube(name,parent,lo,hi,material,rot=None,pivot=None):
    c=dict(op="add_cube",name=name,parent=parent,**{"from":lo,"to":hi},box_uv=False)
    if rot:c["rotation"]=rot
    if pivot:c["origin"]=pivot
    cubes.append((c,material))
group("root",[0,0,0])
group("pelvis",[0,13,0],"root")
group("torso",[0,17,0],"pelvis")
group("core",[0,22,-4],"torso")
group("neck",[0,28,0],"torso")
group("head",[0,29,0],"neck")
cube("pelvis_basalt","pelvis",[-4,12,-3],[4,16,3],"basalt")
cube("belt_front","pelvis",[-4.5,13,-3.8],[4.5,15,-3],"bronze")
cube("belt_clasp","pelvis",[-1.5,12.5,-4.4],[1.5,15.5,-3.8],"magma")
cube("torso_foundation","torso",[-4.5,16,-3.5],[4.5,27,3.5],"basalt")
cube("abdomen_plate","torso",[-3.5,16,-4.3],[3.5,19,-3.5],"iron")
cube("back_carapace","torso",[-5.5,19,3.5],[5.5,27,4.8],"iron")
cube("back_spine","torso",[-1,19,4.8],[1,28,5.8],"bronze")
cube("collar","torso",[-6,26,-4.5],[6,28,4.5],"bronze")
cube("core_socket","core",[-2.9,19.1,-4.8],[2.9,24.9,-3.5],"iron")
cube("core_ember","core",[-2.1,19.9,-5.3],[2.1,24.1,-4.8],"magma")
cube("core_keystone","core",[-.9,21.1,-5.7],[.9,22.9,-5.3],"gold",[0,0,45],[0,22,-5.5])
cube("neck_joint","neck",[-2,28,-2],[2,30,2],"joint")
cube("helm_shell","head",[-3.5,29,-3.3],[3.5,35,3.3],"basalt")
cube("helm_crown","head",[-3.8,34.8,-3.6],[3.8,36,3.6],"iron")
cube("face_mask","head",[-3.2,29.8,-3.9],[3.2,34,-3.3],"iron")
cube("eye_slit","head",[-2.5,32.1,-4.15],[2.5,33.1,-3.9],"magma")
cube("nose_ridge","head",[-.4,30.8,-4.45],[.4,34,-4.15],"bronze")
cube("chin_guard","head",[-2.8,29,-4],[2.8,30.1,-3.3],"bronze")
for side,sgn in [("left",-1),("right",1)]:
    def bounds(lo,hi):
        if sgn==1:return lo,hi
        return [-hi[0],lo[1],lo[2]],[-lo[0],hi[1],hi[2]]
    def part(n,b,lo,hi,mat,rot=None,pivot=None):
        lo,hi=bounds(lo,hi)
        if rot:rot=[rot[0],rot[1]*sgn,rot[2]*sgn]
        if pivot:pivot=[pivot[0]*sgn,pivot[1],pivot[2]]
        cube(side+"_"+n,b,lo,hi,mat,rot,pivot)
    part("chest_plate","torso",[3,19,-4.5],[5.7,26,-3.5],"iron")
    part("chest_trim","torso",[2.9,24.8,-5],[5.9,26,-4.5],"bronze")
    part("horn_base","head",[3.4,33,-1.7],[5.4,36,1.7],"bronze",[0,0,-27],[3.4,33,0])
    part("horn_tip","head",[4.5,35,-1.2],[6.1,39,1.2],"basalt",[0,0,-22],[4.5,35,0])
    arm=side+"_arm";fore=side+"_forearm";hand=side+"_hand"
    group(arm,[sgn*6,25,0],"torso")
    group(fore,[sgn*8,18,0],arm)
    group(hand,[sgn*8,11,0],fore)
    part("shoulder_joint",arm,[5,23,-2.5],[8,26,2.5],"joint")
    part("pauldron",arm,[6,22,-3.8],[10,27,3.8],"iron")
    part("pauldron_crest",arm,[7,26,-3.2],[10.5,28,3.2],"bronze")
    part("upper_arm",arm,[6,18,-2.4],[9,23,2.4],"basalt")
    part("elbow",fore,[6.4,17,-2.2],[9.6,19,2.2],"joint")
    part("gauntlet",fore,[6,12,-3],[10,17.5,3],"iron")
    part("gauntlet_rune",fore,[6.7,13,-3.4],[9.3,16.5,-3],"magma")
    part("wrist",hand,[6.5,11,-2.2],[9.5,12.5,2.2],"joint")
    part("fist",hand,[5.8,8,-3],[10.2,11.5,3],"basalt")
    part("knuckle_plate",hand,[5.7,8.5,-3.6],[10.3,10.4,-3],"bronze")
    leg=side+"_leg";shin=side+"_shin";foot=side+"_foot"
    group(leg,[sgn*2.8,13,0],"pelvis")
    group(shin,[sgn*2.8,7,0],leg)
    group(foot,[sgn*2.8,2,0],shin)
    part("thigh",leg,[.8,7,-2.6],[4.8,13,2.6],"basalt")
    part("thigh_plate",leg,[1,8,-3.2],[4.6,12.5,-2.6],"iron")
    part("knee",shin,[1.1,6,-3.1],[4.5,8,2.8],"joint")
    part("kneecap",shin,[.9,6,-3.8],[4.7,8.5,-3.1],"bronze")
    part("shin",shin,[1,2,-2.5],[4.6,7,2.5],"iron")
    part("boot",foot,[.7,0,-4.5],[4.9,2.5,2.8],"basalt")
    part("toe_cap",foot,[.7,.4,-5],[4.9,2.4,-4.5],"iron")

# V3 redesign: broad upper body, compressed skull, angled plates, heavy hands,
# separate articulated armor and finger groups. Original V2 topology is not copied
# from any reference mod. Preserve bone names for future runtime migration.
for g in groups:
    x,y,z=g["origin"];g["origin"]=[x*1.24 if y>=8 else x*1.10,y if y<29 else 29+(y-29)*.75,z]
for c,mat in cubes:
    def reshape(v):
        x,y,z=v
        return [x*(1.24 if y>=8 else 1.10),y if y<29 else 29+(y-29)*.75,z*(1.12 if y>=16 else 1.0)]
    c["from"]=reshape(c["from"]);c["to"]=reshape(c["to"])
    if "origin" in c:c["origin"]=reshape(c["origin"])
    if "pauldron" in c["name"] and "crest" not in c["name"]:
        c["rotation"]=[0,0,-12 if c["name"].startswith("right") else 12]
    if "horn_tip" in c["name"]:c["rotation"]=[-16,0,-35 if c["name"].startswith("right") else 35]
# Additional tiered armour, independently hinged fingers and VFX sockets.
for side,s in [("left",-1),("right",1)]:
    def mirrored(lo,hi):
        return (lo,hi) if s==1 else ([-hi[0],lo[1],lo[2]],[-lo[0],hi[1],hi[2]])
    def piece(n,b,lo,hi,mat,rot=None,piv=None):
        lo,hi=mirrored(lo,hi)
        if rot:rot=[rot[0],rot[1]*s,rot[2]*s]
        if piv:piv=[piv[0]*s,piv[1],piv[2]]
        cube(side+"_"+n,b,lo,hi,mat,rot,piv)
    arm=side+"_arm";fore=side+"_forearm";hand=side+"_hand"
    shoulder=side+"_shoulder_plates";group(shoulder,[s*9.3,25,0],arm)
    piece("outer_pauldron",shoulder,[10.7,23.0,-3.8],[14.0,27.0,3.8],"basalt",[0,0,-24],[10.7,25,0])
    piece("upper_pauldron",shoulder,[7.8,27.0,-3.5],[12.2,29.0,3.5],"iron",[0,0,-17],[9.5,27,0])
    piece("lower_pauldron",shoulder,[10.9,20.2,-3.2],[13.1,23.8,3.2],"iron",[0,0,16],[11.3,23,0])
    piece("pauldron_spike",shoulder,[10.1,28.5,-1.4],[12.2,32,1.4],"basalt",[0,0,-27],[11,29,0])
    piece("pauldron_rim",shoulder,[8.1,26,-4.8],[12.3,27.2,-4.1],"bronze",[0,0,-15],[9,26,0])
    piece("forearm_outer",fore,[12.3,12,-2.9],[14,17.5,2.9],"basalt",[0,0,8],[12.5,15,0])
    piece("forearm_fins",fore,[12.2,16.5,-2.6],[14.7,19.1,2.6],"iron",[0,0,-20],[13,17,0])
    piece("forearm_trim",fore,[7.2,12.5,-3.8],[12.5,13.8,-3.1],"bronze")
    piece("hip_skirt","pelvis",[4.4,10,-3.4],[6.2,15.0,3.4],"iron",[0,0,-12],[4.4,14,0])
    piece("shin_outer",side+"_shin",[5.0,2.4,-2.4],[6.1,7.0,2.4],"basalt",[0,0,10],[5.1,5,0])
    piece("heel_spur",side+"_foot",[1.4,.4,2.2],[4.5,2.1,4.0],"basalt")
    piece("toe_ridge",side+"_foot",[1.1,.7,-5.3],[4.9,1.7,-4.9],"bronze")
    piece("cheek_guard","head",[2.7,29.7,-3.9],[4.5,32.6,-2.3],"bronze",[0,0,-20],[3,31,-3])
    piece("horn_hook","head",[6.4,35.8,-.8],[7.6,38.0,.8],"basalt",[0,0,32],[6.8,36,0])
    for j in range(3):
        bx=7.5+j*1.55;finger=side+"_finger_"+str(j)
        group(finger,[s*(bx+.55),9.2,-3.0],hand)
        piece("finger_"+str(j),finger,[bx,7.0,-3.5],[bx+1.1,9.5,-1.4],"basalt")
        piece("claw_"+str(j),finger,[bx+.05,6.2,-3.7],[bx+1.05,7.4,-2.2],"iron",[18,0,0],[bx+.55,7.1,-3])
    group(side+"_hand_fx",[s*10,8,-3],hand)
    group(side+"_foot_fx",[s*3,0,0],side+"_foot")
for i in range(3):
    y=20+i*2.6
    group("spine_plate_"+str(i),[0,y,4.9],"torso")
    cube("dorsal_shard_"+str(i),"spine_plate_"+str(i),[-1.8,y,5],[1.8,y+2.2,7.5],"basalt",[28,0,0],[0,y,5])
group("core_fx",[0,22,-6],"core");group("ground_fx",[0,0,0],"root")
cube("core_cage_top","core",[-2.6,24.4,-5.8],[2.6,25.2,-4.8],"bronze")
cube("core_cage_bottom","core",[-2.6,19.0,-5.8],[2.6,19.8,-4.8],"bronze")

# Allocate each visible face separately with uniform pixels per world unit.
# Pack by decreasing height with a 1-pixel extruded border; no shared face art.
N=512;density=2;faces=[]
for c,mat in cubes:
    x,y,z=[b-a for a,b in zip(c["from"],c["to"])]
    for f,(w,h) in {"north":(x,y),"south":(x,y),"east":(z,y),"west":(z,y),"up":(x,z),"down":(x,z)}.items():
        faces.append((max(2,math.ceil(w*density)),max(2,math.ceil(h*density)),c,mat,f))
atlas=Image.new("RGBA",(N,N),(0,0,0,0));glow=Image.new("RGBA",(N,N),(0,0,0,0))
layout=[];palette={"basalt":(47,51,53),"iron":(78,84,85),"bronze":(139,96,50),"joint":(31,34,36),"magma":(223,76,24),"gold":(255,180,60)}
def draw_tile(w,h,mat,name,face):
    rng=random.Random(int(hashlib.sha256((name+face).encode()).hexdigest()[:16],16))
    base=palette[mat];im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im)
    for y in range(h):
        for x in range(w):
            shade=rng.choice([-8,-4,0,0,0,3,7])+int(10*(1-y/max(1,h-1)))
            if face=="down":shade-=12
            if face=="up":shade+=9
            if mat in ("iron","bronze") and (x==0 or y==0):shade+=24
            if mat in ("iron","bronze") and (x==w-1 or y==h-1):shade-=22
            d.point((x,y),fill=tuple(max(0,min(255,v+shade)) for v in base)+(255,))
    if mat in ("iron","bronze") and w>=7 and h>=7:
        for x,y in [(2,2),(w-3,2),(2,h-3),(w-3,h-3)]:
            d.point((x,y),fill=(176,146,94,255) if mat=="bronze" else (128,132,126,255))
        if face=="north" and h>=10:
            d.line([(w//2,3),(w//2,h-4)],fill=(39,42,42,255))
    if mat in ("basalt","iron") and w>=5 and h>=6:
        path=[(w//3,0),(w//2,h//3),(max(0,w//2-1),h//2),(min(w-1,w*2//3),h-1)]
        d.line(path,fill=(24,26,29,255))
        if face=="north" and ("horn" not in name):
            d.line(path[:2],fill=(153,62,25,255))
            if w>=8 and h>=8:
                d.line([(1,h//3),(w//3,h//3+1),(w//2,h//2)],fill=(24,27,29,255))
                d.line([(0,h-2),(w//3,h-3),(w//2,h-2)],fill=(93,89,76,255))
    if "rune" in name and face=="north" and w>=5 and h>=5:
        d.line([(w//2,1),(1,h//2),(w//2,h-2),(w-2,h//2),(w//2,1)],fill=(255,211,111,255))
    if mat=="bronze" and w>=6 and h>=6 and face in ("north","south"):
        d.line([(1,h-2),(w-2,h-2)],fill=(48,49,38,255))
        d.line([(2,1),(2,h//2)],fill=(119,137,99,255))
    if mat=="magma":
        for y in range(h):
            for x in range(w):
                heat=max(0,1-abs(x-(w-1)/2)/max(1,w/2))
                d.point((x,y),fill=(255,int(85+140*heat),int(20+90*heat),255))
        if w>=7 and h>=7:
            d.line([(1,h//2),(w//2,1),(w-2,h//2),(w//2,h-2),(1,h//2)],fill=(255,240,154,255))
    return im
def extrude(dst,tile,x,y):
    w,h=tile.size;dst.paste(tile,(x,y))
    dst.paste(tile.crop((0,0,w,1)),(x,y-1));dst.paste(tile.crop((0,h-1,w,h)),(x,y+h))
    dst.paste(tile.crop((0,0,1,h)),(x-1,y));dst.paste(tile.crop((w-1,0,w,h)),(x+w,y))
    for xx,yy,tx,ty in [(x-1,y-1,0,0),(x+w,y-1,w-1,0),(x-1,y+h,0,h-1),(x+w,y+h,w-1,h-1)]:
        dst.putpixel((xx,yy),tile.getpixel((tx,ty)))
x=y=1;row=0
for w,h,c,mat,f in sorted(faces,key=lambda e:(-e[1],-e[0],e[2]["name"],e[4])):
    if x+w>=N:x=1;y+=row+2;row=0
    if y+h>=N:raise RuntimeError("UV atlas overflow")
    tile=draw_tile(w,h,mat,c["name"],f);extrude(atlas,tile,x,y)
    if mat in ("magma","gold"):extrude(glow,tile,x,y)
    uv=[x,y,x+w,y+h]
    c.setdefault("faces",{})[f]={"uv":uv,"texture":atlas_name}
    layout.append(dict(part=c["name"],face=f,material=mat,uv=uv,pixels=[w,h]))
    x+=w+2;row=max(row,h)
atlas.save(out/atlas_name);glow.save(out/"ashen_sentinel_v3_emissive.png")
(out/"uv-layout.json").write_text(json.dumps(layout,indent=2))

def anim(name,length,loop,tracks):
    name="animation.ashen_sentinel."+name;clips.append((name,length))
    keys.append(dict(op="add_animation",name=name,length=length,loop=loop,snapping=24))
    for bone,channel,points in tracks:
        for t,v in points:keys.append(dict(op="set_keyframe",animation=name,bone=bone,channel=channel,time=t,value=v,interpolation="linear"))
def track(b,rows):return b,"rotation",rows
def swing(b,amp,duration=1.2):
    return track(b,[(0,[amp,0,0]),(duration/2,[-amp,0,0]),(duration,[amp,0,0])])

def tr(b,channel,rows):return b,channel,rows
def rot(b,rows):return tr(b,"rotation",rows)
def pos(b,rows):return tr(b,"position",rows)
def both(suffix,rows):return [rot(s+"_"+suffix,rows) for s in ("left","right")]
zero=[0,0,0]
anim("dormant",2,"loop",[rot("torso",[(0,[-12,0,0]),(2,[-12,0,0])]),rot("head",[(0,[-18,0,0]),(2,[-18,0,0])]),tr("core","scale",[(0,[.65,.65,.65]),(2,[.65,.65,.65])])])
anim("awaken",2.4,"once",[rot("torso",[(0,[-12,0,0]),(.8,[-18,0,0]),(1.5,[9,0,0]),(2.4,zero)]),rot("head",[(0,[-18,0,0]),(1,[16,0,0]),(2.4,zero)]),tr("core","scale",[(0,[.65]*3),(1.3,[1.2]*3),(2.4,[1]*3)])])
anim("idle",2,"loop",[rot("torso",[(0,[0,0,-1]),(1,[0,0,1]),(2,[0,0,-1])]),rot("head",[(0,[0,-4,0]),(1,[0,4,0]),(2,[0,-4,0])]),tr("core","scale",[(0,[1]*3),(1,[1.08]*3),(2,[1]*3)])])
for name,length,amp in [("walk",1.4,16),("heavy_walk",1.8,23),("run",.9,30)]:
    tracks=[swing("left_leg",amp,length),swing("right_leg",-amp,length),swing("left_arm",-amp*.7,length),swing("right_arm",amp*.7,length),pos("root",[(0,[0,.75,0]),(length*.25,[0,1.15,0]),(length*.5,[0,.75,0]),(length*.75,[0,1.15,0]),(length,[0,.75,0])])]
    for s,sign in [("left",1),("right",-1)]:
        tracks.append(rot(s+"_foot",[(0,[-amp*sign*.5,0,0]),(length/2,[amp*sign*.5,0,0]),(length,[-amp*sign*.5,0,0])]))
    anim(name,length,"loop",tracks)
for s,sign in [("left",-1),("right",1)]:
    anim(s+"_punch",1.4,"once",[rot("torso",[(0,zero),(.48,[0,sign*24,0]),(.63,[-9,-sign*26,0]),(1.4,zero)]),rot(s+"_arm",[(0,zero),(.48,[25,0,sign*14]),(.63,[-95,0,0]),(.83,[-82,0,0]),(1.4,zero)]),rot(s+"_forearm",[(0,zero),(.48,[-75,0,0]),(.63,[-8,0,0]),(1.4,zero)]),pos("root",[(0,zero),(.48,zero),(.63,[0,.4,-1.1]),(1.4,zero)])])
anim("combo",2.2,"once",[rot("torso",[(0,zero),(.4,[0,22,0]),(.58,[0,-25,0]),(1.1,[0,-25,0]),(1.25,[0,25,0]),(2.2,zero)]),rot("left_arm",[(0,zero),(.4,[25,0,0]),(.58,[-92,0,0]),(1.0,zero),(2.2,zero)]),rot("right_arm",[(0,zero),(.8,zero),(1.1,[25,0,0]),(1.25,[-95,0,0]),(2.2,zero)])])
anim("slam",2.2,"once",[rot("torso",[(0,zero),(.85,[12,0,0]),(1.02,[-35,0,0]),(1.4,[-28,0,0]),(2.2,zero)]),*both("arm",[(0,zero),(.85,[-140,0,0]),(1.02,[-12,0,0]),(1.4,[-12,0,0]),(2.2,zero)]),*both("forearm",[(0,zero),(.85,[-24,0,0]),(1.02,zero),(2.2,zero)])])
anim("eruption",2.8,"once",[rot("torso",[(0,zero),(1.1,[16,0,0]),(1.35,[-13,0,0]),(2.8,zero)]),tr("core","scale",[(0,[1]*3),(1.1,[1.27]*3),(1.35,[1.1]*3),(2.8,[1]*3)]),*both("arm",[(0,zero),(1.1,[-48,0,0]),(1.35,[-90,0,0]),(2.8,zero)])])
anim("guard",1.6,"once",[rot("left_arm",[(0,zero),(.35,[-65,0,15]),(1.2,[-65,0,15]),(1.6,zero)]),rot("right_arm",[(0,zero),(.35,[-65,0,-15]),(1.2,[-65,0,-15]),(1.6,zero)]),*both("forearm",[(0,zero),(.35,[-45,0,0]),(1.2,[-45,0,0]),(1.6,zero)])])
anim("guard_break",1.3,"once",[rot("torso",[(0,zero),(.2,[20,0,5]),(.7,[-10,0,0]),(1.3,zero)]),*both("arm",[(0,zero),(.2,[40,0,0]),(1.3,zero)])])
anim("stagger",.75,"once",[rot("torso",[(0,zero),(.16,[14,0,6]),(.4,[-5,0,-2]),(.75,zero)]),rot("head",[(0,zero),(.16,[12,0,0]),(.75,zero)])])
anim("enraged",2,"once",[rot("torso",[(0,zero),(.8,[18,0,0]),(1.2,[-8,0,0]),(2,zero)]),tr("core","scale",[(0,[1]*3),(.8,[1.3]*3),(2,[1.12]*3)]),*both("arm",[(0,zero),(.8,[-75,0,0]),(2,zero)])])
anim("death",2.7,"once",[rot("torso",[(0,zero),(.7,[12,0,0]),(1.5,[-42,0,0]),(2.7,[-74,0,0])]),pos("root",[(0,zero),(1.5,[0,1,0]),(2.7,[0,-2,0])]),tr("core","scale",[(0,[1]*3),(1.5,[.7]*3),(2.7,[.3]*3)])])
anim("collapse",1.4,"once",[rot("torso",[(0,[-74,0,0]),(1.4,[-82,0,0])]),tr("core","scale",[(0,[.3]*3),(1.4,[.08]*3)])])
# Add secondary armor and finger movement to attack clips, using their own pivots.
for clip,length in clips:
    short=clip.split(".")[-1]
    if short not in ("left_punch","right_punch","combo","slam","guard","eruption"):continue
    for side in ("left","right"):
        for bone in [side+"_shoulder_plates"]+[side+"_finger_"+str(i) for i in range(3)]:
            for t,v in [(0,zero),(length*.42,[-12 if "finger" in bone else 5,0,0]),(length,zero)]:
                keys.append(dict(op="set_keyframe",animation=clip,bone=bone,channel="rotation",time=t,value=v,interpolation="linear"))

events=[];attacks=[]
def ev(clip,at,kind,socket,effect):
    events.append(dict(clip=clip,id=clip+"_"+str(len(events)),at=at,kind=kind,socket=socket,effect=effect))
def attack(clip,windup,end,cooldown,shape,range_,mult,knockback,hit_times,socket,effect):
    attacks.append(dict(clip=clip,windup=windup,active_end=end,cooldown=cooldown,shape=shape,range=range_,damage_multiplier=mult,knockback=knockback))
    ev(clip,max(.05,windup-.35),"vfx",socket,"charge")
    for t in hit_times:
        ev(clip,t,"hit",socket,shape);ev(clip,t,"vfx",socket,effect);ev(clip,t,"sound",socket,"stone_impact")
attack("left_punch",.60,.77,2.3,"arc",3.2,1.1,.4,[.63],"left_hand_fx","swipe")
attack("right_punch",.60,.77,2.3,"arc",3.2,1.1,.4,[.63],"right_hand_fx","swipe")
attack("combo",.55,1.35,4.5,"arc",3.4,.85,.3,[.58,1.25],"core_fx","swipe")
attack("slam",.98,1.15,5.5,"radial",4.5,1.65,.8,[1.02],"ground_fx","shockwave")
attack("eruption",1.25,1.6,8,"radial",6,1.3,.65,[1.35],"ground_fx","eruption")
for clip,at,kind,socket,effect in [("awaken",1.3,"vfx","core_fx","charge"),("guard",.35,"state","core_fx","guard_on"),("guard",1.2,"state","core_fx","guard_off"),("guard_break",.2,"vfx","core_fx","sparks"),("enraged",.8,"vfx","core_fx","eruption"),("death",1.5,"vfx","ground_fx","dust"),("collapse",1.1,"vfx","core_fx","sparks"),("walk",.35,"vfx","left_foot_fx","dust"),("walk",1.05,"vfx","right_foot_fx","dust")]:ev(clip,at,kind,socket,effect)
spec=dict(schema=1,authority="server",runtime_integrated=False,boss=dict(optional=True,phase_threshold=.35,skin="bossbar",portrait=False),attacks=attacks,events=events)
(out/"events.json").write_text(json.dumps(spec,indent=2))
def call(tool,args,log):
    path=out/(log+"-args.json");path.write_text(json.dumps(args))
    r=subprocess.run(["bash",str(ROOT/"tools/asset_studio/bb.sh"),tool,"@"+str(path)],env=env,capture_output=True,text=True,check=False)
    (out/(log+".json")).write_text(r.stdout)
    if r.returncode:raise RuntimeError(tool+": "+r.stdout[-3000:]+" "+r.stderr[-3000:])
    return json.loads(r.stdout)

rev=call("bbmodel_create",dict(file=model,name="Ashen Sentinel V3",format="free",box_uv=False,resolution=dict(width=N,height=N)),"01-create")["revision"]
rev=call("bbmodel_add_texture",dict(file=model,expected_revision=rev,image=atlas_name,name=atlas_name),"02-texture")["revision"]
for i,batch in enumerate([groups,[c for c,_ in cubes],keys[:350],keys[350:700],keys[700:]]):
    if batch:rev=call("bbmodel_edit",dict(file=model,expected_revision=rev,operations=batch),"03-build-"+str(i))["revision"]
info=call("bbmodel_info",dict(file=model),"04-info")
call("bbmodel_validate",dict(file=model,self_test=True),"05-geometry")
call("bbmodel_validate_animations",dict(file=model),"06-animations")
for clip,duration in clips:call("bbmodel_sample_pose",dict(file=model,animation=clip,time=duration*.5),"pose-"+clip.split(".")[-1])
from studio_v2 import run
perf=run(out/model,out/atlas_name,out/"ashen_sentinel_v3_emissive.png",out/"events.json",out,a.baseline)
subprocess.run(["ffmpeg","-hide_banner","-loglevel","error","-y","-framerate","12","-i",str(out/"frames/%04d.png"),"-c:v","libx264","-pix_fmt","yuv420p",str(out/"animation-preview.mp4")],check=True)
subprocess.run([sys.executable,str(ROOT/"tools/asset_studio/export_mesh.py"),"--model",str(out/model),"--output",str(out/"mesh.json")],check=True)
subprocess.run(["blender","-b","-t","2","--python-exit-code","1","-P",str(ROOT/"tools/asset_studio/blender_render.py"),"--",str(out/"mesh.json"),str(out/atlas_name),str(out/"blender.png")],env=dict(env,RUSTPG_CAMERA="1.3,1.8,0.8"),check=True)
# Original transparent 2D VFX sprites + independently usable boss bar pieces.
from effects import build
build(out)
subprocess.run(["rustc","--edition=2021","--test",str(out/"ashen_sentinel_v3_generated.rs"),"-o",str(out/"generated-tests")],check=True)
subprocess.run([str(out/"generated-tests")],check=True)
(out/"generated-tests").unlink()
print((out/"summary.json").read_text())
