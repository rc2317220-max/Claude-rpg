"""Storm Gryphon original electricity, air wake and impact sprite effects."""
import math,json,random
from pathlib import Path
from PIL import Image,ImageDraw
def build(out):
    out=Path(out);v=out/"vfx";v.mkdir(exist_ok=True);S=96;F=13
    names=["charge","claw","gust","lightning","impact","dust"]
    for n in names:
        sheet=Image.new("RGBA",(S,S*F))
        for i in range(F):
            u=i/(F-1);alpha=int(255*(1-u)**.65);im=Image.new("RGBA",(S,S));d=ImageDraw.Draw(im);rng=random.Random(270+i//2)
            if n=="lightning":
                # Branching electric bolt with two secondary forks and bright core.
                pts=[(10,48),(22,39),(32,56),(44,28),(58,55),(69,37),(87,49)]
                pts=[(x,y+rng.randint(-6,6)) for x,y in pts]
                for width,col in [(9,(38,85,232,alpha//3)),(5,(78,165,255,alpha)),(2,(226,250,255,alpha))]:d.line(pts,fill=col,width=width,joint="curve")
                for p,dx in [(pts[2],-7),(pts[4],12)]:
                    d.line([p,(p[0]+dx,p[1]+17),(p[0]+dx+6,p[1]+25)],fill=(140,216,255,alpha),width=2)
            elif n=="claw":
                for j in range(3):
                    x=13+j*11;r=60+6*u
                    d.arc((x,6,x+r,83),int(210+60*u),int(330+60*u),fill=(125,192,255,alpha),width=5)
                    d.arc((x+2,8,x+r-2,81),int(210+60*u),int(330+60*u),fill=(242,248,255,alpha),width=2)
            elif n in ("gust","impact","charge"):
                r=int(7+36*u) if n!="charge" else int(38-29*u)
                for j in range(3):
                    rr=max(2,r-j*5);yy=max(1,int(rr*.35)) if n=="impact" else rr
                    d.arc((48-rr,48-yy,48+rr,48+yy),int(u*120+j*35),int(u*120+j*35+260),fill=(125,202,248,alpha//(j+1)),width=3)
                if n=="charge":
                    for j in range(5):
                        a=j*math.tau/5+u*4;x=48+r*math.cos(a);y=48+r*math.sin(a);d.ellipse((x-2,y-2,x+2,y+2),fill=(238,251,255,alpha))
            else:
                for j in range(26):
                    a=rng.uniform(0,math.tau);r=rng.uniform(8,40)*u;x=48+r*math.cos(a);y=48+r*math.sin(a)
                    d.ellipse((x-2,y-1,x+2,y+1),fill=(135,147,160,alpha//2))
            sheet.paste(im,(0,S*i))
        sheet.save(v/(n+".png"))
    (v/"manifest.json").write_text(json.dumps(dict(width=S,height=S,frames=F,fps=20,layout="vertical",effects=names),indent=2))
    im=Image.new("RGBA",(320,24));d=ImageDraw.Draw(im)
    d.polygon([(0,12),(12,4),(31,1),(289,1),(308,4),(319,12),(308,20),(289,23),(31,23),(12,20)],fill=(24,34,50,255),outline=(151,181,214,255),width=2)
    d.rectangle((25,7,294,16),fill=(12,20,35,255))
    for x,s in [(11,1),(308,-1)]:
        d.polygon([(x,3),(x+s*8,9),(x+s*2,12),(x+s*8,15),(x,21),(x+s*3,12)],fill=(154,219,255,255))
    im.save(out/"optional-bossbar-frame.png")
    fill=Image.new("RGBA",(270,10));d=ImageDraw.Draw(fill)
    for y in range(10):d.line((0,y,269,y),fill=(75-y*3,175-y*6,245-y*4,255))
    fill.save(out/"optional-bossbar-fill.png")
def overlay(im,out,effect,age,pos):
    f=Path(out)/"vfx"/(effect+".png")
    if not f.exists():return
    i=min(12,int(age*20));sprite=Image.open(f).crop((0,i*96,96,(i+1)*96));im.alpha_composite(sprite,(round(pos[0]-48),round(pos[1]-48)))
