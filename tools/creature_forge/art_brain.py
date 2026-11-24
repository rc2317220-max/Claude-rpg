"""Creature Art Studio v4: reusable anatomy, semantic material painter and UV packing.
No generated assets are claimed to be hand-painted. Each face has its own original art."""
import hashlib,json,math,random
from pathlib import Path
import numpy as np
from PIL import Image,ImageDraw
from motion_lab import align,euler
PALETTE={
 "skin":(99,120,58),"skin_shadow":(65,81,37),"skin_light":(135,148,77),"scar":(137,91,58),"lip":(67,72,34),
 "ear":(131,112,61),"eye":(235,184,58),"pupil":(23,24,16),"teeth":(209,190,137),"hair":(40,39,29),
 "leather":(85,57,35),"leather_dark":(48,34,26),"cloth":(111,80,54),"cloth_red":(115,49,35),
 "stitch":(188,161,110),"rope":(156,128,78),"wood":(104,69,37),"metal":(120,131,127),
 "edge":(192,202,191),"rust":(121,65,34),"bone":(184,169,126),"fletch":(119,130,84)}
class Sculpt:
    def __init__(self):self.groups=[];self.cubes=[]
    def group(self,n,p,parent=None):
        g=dict(op="add_group",name=n,origin=list(p))
        if parent:g["parent"]=parent
        self.groups.append(g);return n
    def box(self,n,b,lo,hi,mat,rotation=None,origin=None):
        lo=np.array(lo,float);hi=np.array(hi,float)
        if np.any(hi<=lo):raise ValueError("nonpositive cube "+n)
        c=dict(op="add_cube",name=n,parent=b,**{"from":lo.tolist(),"to":hi.tolist()},box_uv=False)
        if rotation is not None:c["rotation"]=[float(v) for v in rotation]
        if origin is not None:c["origin"]=[float(v) for v in origin]
        self.cubes.append((c,mat));return n
    def rod(self,n,b,A,B,w,d,mat):
        A=np.array(A,float);B=np.array(B,float);p=(A+B)/2
        h=np.array([w/2,np.linalg.norm(B-A)/2,d/2]);self.box(n,b,p-h,p+h,mat,euler(align([0,1,0],B-A)),p)
    def volume(self,n,b,A,B,widths,depths,mat):
        """Tapered sectional volume with contiguous axes, not arbitrary piled cubes."""
        A=np.array(A,float);B=np.array(B,float)
        for j,(w,d) in enumerate(zip(widths,depths)):
            u=A+(B-A)*j/len(widths);v=A+(B-A)*(j+1)/len(widths)
            self.rod(n+"_"+str(j),b,u,v,w,d,mat)
def material_tile(w,h,mat,name,face):
    base=PALETTE[mat];seed=int(hashlib.sha256((name+"/"+face).encode()).hexdigest()[:16],16)
    rng=random.Random(seed);im=Image.new("RGBA",(w,h));d=ImageDraw.Draw(im)
    # Coherent face-aware shading. Patterns are material-specific rather than generic noise.
    face_light={"north":8,"south":-7,"up":13,"down":-14,"east":0,"west":-2}[face]
    for y in range(h):
        for x in range(w):
            u=x/max(1,w-1);v=y/max(1,h-1)
            shade=face_light+int(10*math.sin(math.pi*u)-9*v)+rng.choice([-3,-2,0,0,1,3])
            if mat.startswith("skin") or mat=="ear":
                shade+=int(7*math.sin(u*math.pi*2+v*4)+4*math.sin(v*17+u*9))
                if (x*17+y*13+seed)%31==0:shade-=15
            elif mat.startswith("leather"):
                shade+=9 if x in (0,w-1) or y in (0,h-1) else 0
                if (x+2*y)%7==0:shade-=8
            elif mat.startswith("cloth"):shade+=5 if (x+y)%2 else -4
            elif mat=="wood":shade+=int(13*math.sin(u*34+math.sin(v*4)*.65))
            elif mat in ("metal","edge","rust"):shade+=int(22*math.sin(v*2.1+u*.8))-7
            elif mat in ("bone","teeth","rope"):shade+=6 if (x+y)%4==0 else -2
            elif mat=="hair":shade+=11 if x%3==0 else -4
            col=tuple(max(0,min(255,k+shade)) for k in base)
            d.point((x,y),fill=col+(255,))
    if mat.startswith("leather") and min(w,h)>=5:
        for y in range(2,h-1,3):
            d.point((1,y),fill=(171,138,83,255));d.point((w-2,y),fill=(171,138,83,255))
    if mat.startswith("cloth") and w>=6 and h>=6:
        d.line((1,1,w-2,1),fill=(148,111,77,255));d.line((1,h-2,w-2,h-2),fill=(65,42,28,255))
    if mat=="metal" and w>=4 and h>=5:
        for _ in range(max(1,w*h//90)):
            x=rng.randrange(w);y=rng.randrange(h);d.line((x,y,min(w-1,x+1),min(h-1,y+2)),fill=(155,88,46,255))
        d.line((0,0,0,h-1),fill=(192,202,191,255))
    if mat=="skin" and "nose" in name and face=="north" and min(w,h)>=5:
        d.point((w//3,h-2),fill=(44,53,25,255));d.point((2*w//3,h-2),fill=(44,53,25,255))
    return im
def padded(dst,t,x,y):
    w,h=t.size;dst.paste(t,(x,y))
    for dx,dy,rect in [(0,-1,(0,0,w,1)),(0,h,(0,h-1,w,h)),(-1,0,(0,0,1,h)),(w,0,(w-1,0,w,h))]:dst.paste(t.crop(rect),(x+dx,y+dy))
    for dx,dy,tx,ty in [(-1,-1,0,0),(w,-1,w-1,0),(-1,h,0,h-1),(w,h,w-1,h-1)]:dst.putpixel((x+dx,y+dy),t.getpixel((tx,ty)))
def atlas(sculpt,out,name="goblin_shared",size=1024,density=3):
    """One deterministic shared atlas for all roles. Each variant selects a subset."""
    im=Image.new("RGBA",(size,size));em=Image.new("RGBA",(size,size));rows=[];faces=[]
    for c,mat in sculpt.cubes:
        x,y,z=np.array(c["to"])-np.array(c["from"])
        for f,(w,h) in dict(north=(x,y),south=(x,y),east=(z,y),west=(z,y),up=(x,z),down=(x,z)).items():
            faces.append((max(2,math.ceil(w*density)),max(2,math.ceil(h*density)),c,mat,f))
    x=y=1;row=0
    for w,h,c,mat,f in sorted(faces,key=lambda e:(-e[1],-e[0],e[2]["name"],e[4])):
        if x+w>=size:x=1;y+=row+2;row=0
        if y+h>=size:raise ValueError("atlas overflow")
        padded(im,material_tile(w,h,mat,c["name"],f),x,y)
        uv=[x,y,x+w,y+h];c.setdefault("faces",{})[f]=dict(uv=uv,texture=name+".png")
        rows.append(dict(part=c["name"],material=mat,face=f,uv=uv));x+=w+2;row=max(row,h)
    im.save(out/(name+".png"));em.save(out/(name+"_emissive.png"))
    (out/"shared-uv-layout.json").write_text(json.dumps(rows,indent=2))
    return im,em
