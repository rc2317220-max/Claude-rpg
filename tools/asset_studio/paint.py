#!/usr/bin/env python3
"""Deterministic RGBA pixel-art layer compositor. JSON in, PNG out."""
import argparse,json
from pathlib import Path
from PIL import Image,ImageDraw,ImageColor
def color(v):
    return ImageColor.getcolor(v,"RGBA") if isinstance(v,str) else tuple(v)
def run(spec):
    w,h=map(int,spec["size"])
    if not (1<=w<=4096 and 1<=h<=4096):raise ValueError("Invalid image size")
    im=Image.new("RGBA",(w,h),color(spec.get("background",[0,0,0,0])))
    for layer in spec["layers"]:
        sheet=Image.new("RGBA",im.size)
        d=ImageDraw.Draw(sheet)
        for op in layer["ops"]:
            k=op["type"]; c=color(op["color"]);p=op.get("xy")
            if k=="rectangle":d.rectangle(p,fill=c)
            elif k=="ellipse":d.ellipse(p,fill=c)
            elif k=="polygon":d.polygon([tuple(x) for x in p],fill=c)
            elif k=="line":d.line([tuple(x) for x in p],fill=c,width=int(op.get("width",1)))
            elif k=="point":d.point(tuple(p),fill=c)
            elif k=="gradient":
                x0,y0,x1,y1=map(int,p);end=color(op["end"])
                if not (0<=x0<=x1<w and 0<=y0<=y1<h):raise ValueError("Gradient outside image")
                for y in range(y0,y1+1):
                    t=(y-y0)/max(1,y1-y0)
                    d.line((x0,y,x1,y),fill=tuple(round(a+(b-a)*t) for a,b in zip(c,end)))
            else:raise ValueError("Unsupported operation: "+k)
        opacity=int(layer.get("opacity",255))
        if not 0<=opacity<=255:raise ValueError("Invalid opacity")
        sheet.putalpha(sheet.getchannel("A").point(lambda x:x*opacity//255))
        im=Image.alpha_composite(im,sheet)
    return im
if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("recipe",type=Path);p.add_argument("output",type=Path);a=p.parse_args()
    a.output.parent.mkdir(parents=True,exist_ok=True)
    run(json.loads(a.recipe.read_text())).save(a.output)
