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

p=argparse.ArgumentParser();p.add_argument("--out",type=Path,required=True);a=p.parse_args()
out=a.out.resolve();out.mkdir(parents=True,exist_ok=True)
env=dict(os.environ,RUSTPG_ASSET_ROOT=str(out))
model="ashen_sentinel_v2.bbmodel"
atlas_name="ashen_sentinel_v2.png"
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

# Allocate each visible face separately with uniform pixels per world unit.
# Pack by decreasing height with a 1-pixel extruded border; no shared face art.
N=256;density=2;faces=[]
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
    if mat=="basalt" and w>=5 and h>=6:
        path=[(w//3,0),(w//2,h//3),(max(0,w//2-1),h//2),(min(w-1,w*2//3),h-1)]
        d.line(path,fill=(24,26,29,255))
        if face=="north" and ("horn" not in name):d.line(path[:2],fill=(143,59,28,255))
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
atlas.save(out/atlas_name);glow.save(out/"ashen_sentinel_v2_emissive.png")
(out/"uv-layout.json").write_text(json.dumps(layout,indent=2))

def anim(name,length,loop,tracks):
    name="animation.ashen_sentinel."+name;clips.append((name,length))
    keys.append(dict(op="add_animation",name=name,length=length,loop=loop,snapping=24))
    for bone,channel,points in tracks:
        for t,v in points:keys.append(dict(op="set_keyframe",animation=name,bone=bone,channel=channel,time=t,value=v,interpolation="linear"))
def track(b,rows):return b,"rotation",rows
def swing(b,amp,duration=1.2):
    return track(b,[(0,[amp,0,0]),(duration/2,[-amp,0,0]),(duration,[amp,0,0])])
anim("idle",2,"loop",[track("torso",[(0,[0,0,-1]),(1,[0,0,1]),(2,[0,0,-1])]),track("head",[(0,[0,-4,0]),(1,[0,4,0]),(2,[0,-4,0])]),("core","scale",[(0,[1,1,1]),(1,[1.10,1.10,1.10]),(2,[1,1,1])])])
anim("walk",1.2,"loop",[swing("left_leg",20),swing("right_leg",-20),swing("left_arm",-15),swing("right_arm",15),track("left_shin",[(0,[0,0,0]),(.3,[15,0,0]),(.6,[0,0,0]),(1.2,[0,0,0])]),track("right_shin",[(0,[0,0,0]),(.6,[0,0,0]),(.9,[15,0,0]),(1.2,[0,0,0])])])
anim("slam",1.8,"once",[track("torso",[(0,[0,0,0]),(.65,[12,0,0]),(.85,[-22,0,0]),(1.15,[-18,0,0]),(1.8,[0,0,0])]),*[track(side+"_arm",[(0,[0,0,0]),(.65,[-142,0,0]),(.85,[-30,0,0]),(1.15,[-30,0,0]),(1.8,[0,0,0])]) for side in ("left","right")],*[track(side+"_forearm",[(0,[0,0,0]),(.65,[-30,0,0]),(.85,[0,0,0]),(1.8,[0,0,0])]) for side in ("left","right")]])
anim("guard",1.3,"once",[track("left_arm",[(0,[0,0,0]),(.35,[-65,0,12]),(.95,[-65,0,12]),(1.3,[0,0,0])]),track("right_arm",[(0,[0,0,0]),(.35,[-65,0,-12]),(.95,[-65,0,-12]),(1.3,[0,0,0])]),*[track(side+"_forearm",[(0,[0,0,0]),(.35,[-45,0,0]),(.95,[-45,0,0]),(1.3,[0,0,0])]) for side in ("left","right")]])
anim("stagger",.7,"once",[track("torso",[(0,[0,0,0]),(.15,[14,0,6]),(.4,[-5,0,-2]),(.7,[0,0,0])]),track("head",[(0,[0,0,0]),(.15,[12,0,0]),(.7,[0,0,0])])])
def call(tool,args,log):
    path=out/(log+"-args.json");path.write_text(json.dumps(args))
    r=subprocess.run(["bash",str(ROOT/"tools/asset_studio/bb.sh"),tool,"@"+str(path)],env=env,capture_output=True,text=True,check=False)
    (out/(log+".json")).write_text(r.stdout)
    if r.returncode:raise RuntimeError(tool+": "+r.stdout[-3000:]+" "+r.stderr[-3000:])
    return json.loads(r.stdout)
rev=call("bbmodel_create",dict(file=model,name="Ashen Sentinel V2",format="free",box_uv=False,resolution=dict(width=N,height=N)),"01-create")["revision"]
rev=call("bbmodel_add_texture",dict(file=model,expected_revision=rev,image=atlas_name,name=atlas_name),"02-texture")["revision"]
rev=call("bbmodel_edit",dict(file=model,expected_revision=rev,operations=groups+[c for c,_ in cubes]+keys),"03-build")["revision"]
info=call("bbmodel_info",dict(file=model),"04-info")
call("bbmodel_validate",dict(file=model,self_test=True),"05-geometry")
call("bbmodel_validate_animations",dict(file=model),"06-animations")
for clip,duration in clips:call("bbmodel_sample_pose",dict(file=model,animation=clip,time=duration*.5),"pose-"+clip.split(".")[-1])
doc=json.loads((out/model).read_text());assert len(doc["elements"])==len(cubes) and len(doc["groups"])==len(groups) and len(doc["animations"])==len(clips)
# Render actual saved BBModel, not a concept picture. Import numeric keys from
# its animators; the inspector supports rotations and scales in this candidate.
rig=authored(out/model);tex=np.array(atlas)
for clip in doc["animations"]:
    for bone,animator in clip["animators"].items():
        if bone not in rig["bones"]:continue
        for key in animator["keyframes"]:
            d=key["data_points"][0];rig["keys"].append((clip["name"],bone,key["channel"],float(key["time"]),np.array([float(d[k]) for k in "xyz"])))
assert rig["keys"]
views=[("front",(8,180,0)),("three-quarter",(12,145,0)),("back",(8,0,0)),("side",(8,90,0))]
sheet=Image.new("RGB",(1024,1056),(21,25,33));draw=ImageDraw.Draw(sheet);draw.text((12,8),"ASHEN SENTINEL V2 | Blockbench model / offline UV preview",fill="white")
for i,(name,angle) in enumerate(views):
    im,count=render(geometry(rig,None,0,"authored"),tex,angle);assert count>1000
    im.save(out/(name+".png"));sheet.paste(im,((i%2)*512,32+(i//2)*512))
sheet.save(out/"model-sheet.png")
frame_dir=out/"frames";frame_dir.mkdir(exist_ok=True)
idx=0;segments=[]
for clip,duration in clips:
    short=clip.split(".")[-1];start=idx
    for i in range(math.ceil(duration*12)):
        im,count=render(geometry(rig,clip,i/12,"authored"),tex,(10,150,0),384)
        assert count>1000
        ImageDraw.Draw(im).text((10,10),short.upper(),fill="white")
        im.save(frame_dir/f"{idx:04}.png");idx+=1
    segments.append(dict(clip=short,first_frame=start,last_frame=idx-1))
subprocess.run(["ffmpeg","-hide_banner","-loglevel","error","-y","-framerate","12","-i",str(frame_dir/"%04d.png"),"-c:v","libx264","-pix_fmt","yuv420p",str(out/"animation-preview.mp4")],check=True)
subprocess.run([sys.executable,str(ROOT/"tools/asset_studio/export_mesh.py"),"--model",str(out/model),"--output",str(out/"mesh.json")],check=True)
# The Blender bridge renders the actual geometry and atlas; camera uses the
# front (-Z in Blockbench), not the rear of the creature.
subprocess.run(["blender","-b","-t","2","--python-exit-code","1","-P",str(ROOT/"tools/asset_studio/blender_render.py"),"--",str(out/"mesh.json"),str(out/atlas_name),str(out/"blender.png")],env=dict(env,RUSTPG_CAMERA="1.3,1.8,0.8"),check=True)
(out/"summary.json").write_text(json.dumps(dict(kind="offline_asset_candidate_not_game",bones=len(groups),cubes=len(cubes),unique_faces=len(layout),texture_size=[N,N],animations=segments,keyframes=len(rig["keys"]),frame_count=idx,revision=rev,geometry_report="05-geometry.json",animation_report="06-animations.json",runtime_integrated=False),indent=2))
print((out/"summary.json").read_text())
