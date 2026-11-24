import sys
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw
mode=sys.argv[1]
if mode=="reference":
    im=Image.open(sys.argv[2]).convert("RGB")
    im.thumbnail((1000,800))
    im.save(sys.argv[3],quality=90)
elif mode=="views":
    out=sys.argv[2]
    labels=["FRONT","RIGHT","BACK","LEFT","3/4 LEFT","3/4 RIGHT","TOP"]
    imgs=sys.argv[3:]
    if len(imgs)!=len(labels): raise ValueError("incorrect view count")
    sheet=Image.new("RGB",(1024,768),"#20252d")
    draw=ImageDraw.Draw(sheet)
    for i,(p,label) in enumerate(zip(imgs,labels)):
        im=ImageOps.contain(Image.open(p).convert("RGB"),(250,230))
        x=(i%4)*256;y=(i//4)*256
        sheet.paste(im,(x+(256-im.width)//2,y+16))
        draw.text((x+8,y+3),label,fill="white")
    sheet=sheet.crop((0,0,1024,512))
    sheet.save(out,quality=90)
else: raise ValueError("unknown mode")
