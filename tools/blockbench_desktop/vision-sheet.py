import sys
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw
mode=sys.argv[1]
if mode=="reference":
    im=Image.open(sys.argv[2]).convert("RGB")
    im.thumbnail((672,504))
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
elif mode=="custom":
    import json, math
    labels=json.loads(sys.argv[3]);imgs=sys.argv[4:]
    if len(labels)!=len(imgs) or not 1<=len(imgs)<=12: raise ValueError("view count")
    cols=3 if len(imgs)<=9 else 4
    rows=math.ceil(len(imgs)/cols)
    w,h=224,224
    sheet=Image.new("RGB",(cols*w,rows*h),"#20252d")
    draw=ImageDraw.Draw(sheet)
    for i,(p,label) in enumerate(zip(imgs,labels)):
        im=ImageOps.contain(Image.open(p).convert("RGB"),(w-8,h-24))
        x=(i%cols)*w;y=(i//cols)*h
        sheet.paste(im,(x+(w-im.width)//2,y+20))
        draw.text((x+8,y+4),label,fill="white")
    sheet.save(sys.argv[2],quality=88)
else: raise ValueError("unknown mode")
