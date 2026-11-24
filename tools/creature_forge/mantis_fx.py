"""Rift Mantis cyan/violet VFX flipbooks and optional boss skin."""
import math,json,random
from pathlib import Path
from PIL import Image,ImageDraw
def build(out):
    out=Path(out);v=out/"vfx";v.mkdir(exist_ok=True);S=96;F=13
    for n in ["charge","slash","lance","shards","dust"]:
        sheet=Image.new("RGBA",(S,S*F))
        for i in range(F):
            u=i/(F-1);alpha=int(255*(1-u)**.65);im=Image.new("RGBA",(S,S));d=ImageDraw.Draw(im);rng=random.Random(904)
            if n=="slash":
                a=int(-70+u*145);r=int(26+13*u)
                d.arc((48-r,48-r,48+r,48+r),a,a+140,fill=(141,64,235,alpha),width=7)
                d.arc((49-r,49-r,47+r,47+r),a+8,a+134,fill=(184,245,247,alpha),width=2)
            elif n=="charge":
                r=int(37-26*u);d.ellipse((48-r,48-r,48+r,48+r),outline=(124,75,222,alpha),width=2)
                for j in range(6):
                    a=j*math.tau/6+u*1.8;x=48+math.cos(a)*r;y=48+math.sin(a)*r
                    d.polygon([(x,y-3),(x+2,y),(x,y+3),(x-2,y)],fill=(118,236,222,alpha))
            elif n=="lance":
                span=12+33*u;d.polygon([(48-span,48),(48,43),(48+span,48),(48,53)],fill=(128,74,235,alpha));d.line((48-span,48,48+span,48),fill=(218,255,248,alpha),width=2)
            else:
                for j in range(19):
                    a=rng.uniform(0,math.tau);r=rng.uniform(7,45)*u;x=48+math.cos(a)*r;y=48+math.sin(a)*r
                    color=(103,114,124,alpha//2) if n=="dust" else (116,230,217,alpha)
                    d.polygon([(x,y-4),(x+2,y),(x,y+3),(x-2,y)],fill=color)
            sheet.paste(im,(0,S*i))
        sheet.save(v/(n+".png"))
    (v/"manifest.json").write_text(json.dumps(dict(width=S,height=S,frames=F,fps=20,layout="vertical",effects=["charge","slash","lance","shards","dust"]),indent=2))
    im=Image.new("RGBA",(320,24));d=ImageDraw.Draw(im)
    d.polygon([(0,12),(18,1),(302,1),(319,12),(302,23),(18,23)],fill=(20,26,35,255),outline=(87,133,145,255),width=2)
    d.rectangle((20,7,299,16),fill=(12,16,25,255))
    for x in [12,307]:d.polygon([(x,3),(x+7,12),(x,21),(x-7,12)],fill=(122,85,197,255),outline=(152,234,224,255))
    im.save(out/"optional-bossbar-frame.png")
    fill=Image.new("RGBA",(280,10));d=ImageDraw.Draw(fill)
    for y in range(10):d.line((0,y,279,y),fill=(126-y*5,82+y*4,206-y*5,255))
    fill.save(out/"optional-bossbar-fill.png")
def overlay(im,out,effect,age,pos):
    f=Path(out)/"vfx"/(effect+".png")
    if not f.exists():return
    i=min(12,int(age*20));sprite=Image.open(f).crop((0,i*96,96,(i+1)*96))
    im.alpha_composite(sprite,(round(pos[0]-48),round(pos[1]-48)))
