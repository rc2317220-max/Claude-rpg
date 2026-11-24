"""Original PNG VFX flipbooks, boss skin and mock (not in-game UI)."""
import json,math,random
from pathlib import Path
from PIL import Image,ImageDraw
def build(out):
    out=Path(out);v=out/"vfx";v.mkdir(exist_ok=True);size=64;frames=12
    for name in ["charge","swipe","shockwave","eruption","sparks","dust"]:
        sheet=Image.new("RGBA",(size,size*frames))
        for i in range(frames):
            im=Image.new("RGBA",(size,size));d=ImageDraw.Draw(im);t=i/(frames-1);alpha=int(255*(1-t*.9));rng=random.Random(320+i)
            if name in ("charge","shockwave"):
                r=int(4+26*t) if name=="shockwave" else int(28-19*t)
                d.ellipse((32-r,32-r,32+r,32+r),outline=(255,111,26,alpha),width=3)
                d.ellipse((33-r,33-r,31+r,31+r),outline=(255,224,140,alpha),width=1)
            elif name=="swipe":
                d.arc((5,5,59,59),int(30+120*t),int(160+120*t),fill=(255,136,45,alpha),width=5)
                d.arc((9,9,55,55),int(30+120*t),int(160+120*t),fill=(255,238,159,alpha),width=2)
            else:
                for j in range(24):
                    angle=rng.uniform(0,math.tau);r=rng.uniform(5,26)*t; x=32+math.cos(angle)*r;y=32+math.sin(angle)*r-(14*t if name=="eruption" else 0)
                    c=(155,135,109,alpha//2) if name=="dust" else (255,145,44,alpha)
                    d.rectangle((int(x),int(y),int(x)+1,int(y)+1),fill=c)
            sheet.paste(im,(0,i*size))
        sheet.save(v/(name+".png"))
    (v/"manifest.json").write_text(json.dumps(dict(frame_width=64,frame_height=64,frames=12,fps=18,layout="vertical",effects=["charge","swipe","shockwave","eruption","sparks","dust"]),indent=2))
    w,h=320,24;frame=Image.new("RGBA",(w,h));d=ImageDraw.Draw(frame)
    d.polygon([(0,12),(13,1),(307,1),(319,12),(307,23),(13,23)],fill=(40,35,31,255),outline=(155,112,64,255),width=2)
    d.rectangle((18,6,301,17),fill=(23,20,18,255))
    for x in [12,306]:d.polygon([(x,4),(x+5,12),(x,20),(x-5,12)],fill=(246,160,65,255))
    frame.save(out/"bossbar-frame.png")
    fill=Image.new("RGBA",(284,12));d=ImageDraw.Draw(fill)
    for y in range(12):d.line((0,y,283,y),fill=(max(100,230-y*8),max(30,118-y*6),30,255))
    fill.save(out/"bossbar-fill.png")
    mock=Image.new("RGB",(640,120),(24,25,30));d=ImageDraw.Draw(mock);d.text((320,25),"ASHEN SENTINEL  |  GUARDIAN OF THE ASHES",anchor="mm",fill=(230,206,168))
    mock.paste(frame.resize((480,36),Image.Resampling.NEAREST),(80,43),frame.resize((480,36),Image.Resampling.NEAREST));f=fill.crop((0,0,191,12)).resize((286,18));mock.paste(f,(107,52),f)
    d.text((320,94),"PHASE I   /   67%   |   UI ASSET MOCK - NOT GAME",anchor="mm",fill=(201,176,145));mock.save(out/"bossbar-preview.png")
