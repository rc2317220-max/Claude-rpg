#!/usr/bin/env python3
"""Generate the original RustPG rift-forged UI sprite set.

Standard-library only.  Sprites are intentionally modest (64-768 px source
atlases) because they are nine-sliced/tinted by Horizon on Android.
"""
from pathlib import Path
import struct, zlib, math

ROOT=Path(__file__).resolve().parents[1]
GUI=ROOT/"project/assets/rustpg/textures/gui"

VOID=(9,13,19,205)
SURFACE=(16,22,31,205)
SURFACE_HI=(23,31,42,220)
LINE=(91,123,136,140)
AETHER=(105,213,208,235)
MOON=(215,227,231,220)
GOLD=(230,189,102,235)
HP=(224,82,97,240)
STAMINA=(101,197,138,240)
MANA=(138,120,207,235)
MUTED=(124,139,146,170)

def chunk(kind,data):
    return struct.pack(">I",len(data))+kind+data+struct.pack(">I",zlib.crc32(kind+data)&0xffffffff)

class Canvas:
    def __init__(self,w,h):
        self.w=w;self.h=h
        self.p=[0]*(w*h*4)
    def px(self,x,y,c):
        if not(0<=x<self.w and 0<=y<self.h):return
        i=(y*self.w+x)*4
        a=c[3]/255.0
        oa=self.p[i+3]/255.0
        outa=a+oa*(1-a)
        if outa<=0:return
        for k in range(3):
            self.p[i+k]=int((c[k]*a+self.p[i+k]*oa*(1-a))/outa)
        self.p[i+3]=int(outa*255)
    def fill(self,x,y,w,h,c):
        x0=max(0,int(x));y0=max(0,int(y));x1=min(self.w,int(x+w));y1=min(self.h,int(y+h))
        for yy in range(y0,y1):
            for xx in range(x0,x1):self.px(xx,yy,c)
    def line(self,x0,y0,x1,y1,c,width=1):
        x0=int(x0);y0=int(y0);x1=int(x1);y1=int(y1)
        dx=abs(x1-x0);sx=1 if x0<x1 else -1;dy=-abs(y1-y0);sy=1 if y0<y1 else -1;err=dx+dy
        while True:
            r=max(0,width//2)
            for yy in range(y0-r,y0+r+1):
                for xx in range(x0-r,x0+r+1):self.px(xx,yy,c)
            if x0==x1 and y0==y1:break
            e=2*err
            if e>=dy:err+=dy;x0+=sx
            if e<=dx:err+=dx;y0+=sy
    def frame(self,inset,c,width=2,corner=10):
        x=inset;y=inset;w=self.w-2*inset;h=self.h-2*inset
        self.line(x+corner,y,x+w-corner,y,c,width)
        self.line(x+corner,y+h,x+w-corner,y+h,c,width)
        self.line(x,y+corner,x,y+h-corner,c,width)
        self.line(x+w,y+corner,x+w,y+h-corner,c,width)
        self.line(x,y+corner,x+corner,y,c,width)
        self.line(x+w-corner,y,x+w,y+corner,c,width)
        self.line(x,y+h-corner,x+corner,y+h,c,width)
        self.line(x+w-corner,y+h,x+w,y+h-corner,c,width)
    def save(self,path):
        path.parent.mkdir(parents=True,exist_ok=True)
        raw=bytearray()
        for y in range(self.h):
            raw.append(0)
            raw.extend(self.p[y*self.w*4:(y+1)*self.w*4])
        data=b"\x89PNG\r\n\x1a\n"
        data+=chunk(b"IHDR",struct.pack(">IIBBBBB",self.w,self.h,8,6,0,0,0))
        data+=chunk(b"IDAT",zlib.compress(bytes(raw),9))
        data+=chunk(b"IEND",b"")
        path.write_bytes(data)

def material(c,accent=AETHER,secondary=False):
    # Quiet vertical material gradient; no full-screen checker/grid.
    for y in range(c.h):
        t=y/max(1,c.h-1)
        a=int((118 if secondary else 150)*(1-0.30*t))
        c.fill(0,y,c.w,1,(SURFACE[0],SURFACE[1],SURFACE[2],a))
    # Fine rift striations only at the edges.
    step=max(48,c.w//9)
    for x in range(step//2,c.w,step):
        c.line(x,0,max(0,x-24),min(c.h-1,28),(accent[0],accent[1],accent[2],18),1)
    c.frame(max(2,c.w//128),(LINE[0],LINE[1],LINE[2],110),max(1,c.w//384),max(5,c.w//48))
    rail=max(28,min(120,c.w//5))
    c.line(max(8,c.w//32),1,max(8,c.w//32)+rail,1,(accent[0],accent[1],accent[2],190),2)

def make_background():
    c=Canvas(512,512)
    # Almost transparent; geometry hints at a broken rift seal, not a neon grid.
    cx=256;cy=256
    for r,a in [(205,14),(154,18),(104,22)]:
        pts=[]
        for i in range(8):
            ang=math.pi/8+i*math.pi/4
            pts.append((cx+math.cos(ang)*r,cy+math.sin(ang)*r))
        for p,q in zip(pts,pts[1:]+pts[:1]):
            c.line(*p,*q,(AETHER[0],AETHER[1],AETHER[2],a),2)
    for i in range(7):
        y=54+i*67
        c.line(20,y,78,y-18,(MOON[0],MOON[1],MOON[2],10),1)
        c.line(434,y+10,492,y-8,(GOLD[0],GOLD[1],GOLD[2],8),1)
    c.save(GUI/"ui/panels/ui_background_holographic.png")

def make_panel(path,w,h,accent=AETHER,secondary=False):
    c=Canvas(w,h);material(c,accent,secondary)
    # Four small corner sigils make the large surface intentional without
    # making the middle noisy behind text.
    m=max(10,w//64);k=max(16,w//32)
    for x,s in [(m,1),(w-m,-1)]:
        for y,t in [(m,1),(h-m,-1)]:
            c.line(x,y,x+s*k,y,(accent[0],accent[1],accent[2],72),2)
            c.line(x,y,x,y+t*k,(accent[0],accent[1],accent[2],72),2)
    c.save(GUI/path)

def make_frame(path,w,h,accent=AETHER,strong=False):
    c=Canvas(w,h)
    c.frame(max(2,w//128),(accent[0],accent[1],accent[2],210 if strong else 145),max(1,w//256),max(6,w//42))
    inset=max(8,w//40)
    c.line(inset,2,min(w-inset,inset+w//5),2,(GOLD[0],GOLD[1],GOLD[2],135),2)
    c.save(GUI/path)

def make_title():
    c=Canvas(512,96)
    c.fill(12,15,488,66,(SURFACE[0],SURFACE[1],SURFACE[2],150))
    c.line(24,15,168,15,(AETHER[0],AETHER[1],AETHER[2],200),2)
    c.line(344,80,488,80,(GOLD[0],GOLD[1],GOLD[2],85),1)
    c.line(12,31,12,66,(LINE[0],LINE[1],LINE[2],145),2)
    c.line(500,31,500,66,(LINE[0],LINE[1],LINE[2],145),2)
    c.save(GUI/"ui/frames/ui_frame_title.png")

def make_tab(name,state):
    c=Canvas(256,64)
    fill={"normal":(SURFACE[0],SURFACE[1],SURFACE[2],150),
          "hover":(24,35,42,190),"pressed":(26,31,37,220),
          "selected":(24,43,46,218),"disabled":(10,13,18,118)}[state]
    edge={"normal":LINE,"hover":AETHER,"pressed":MOON,"selected":AETHER,"disabled":MUTED}[state]
    c.fill(4,8,248,48,fill)
    c.line(4,8,252,8,(edge[0],edge[1],edge[2],90),1)
    if state=="selected":
        c.line(18,55,238,55,(AETHER[0],AETHER[1],AETHER[2],235),3)
        c.line(88,9,168,9,(GOLD[0],GOLD[1],GOLD[2],80),1)
    elif state=="hover":
        c.line(22,55,110,55,(AETHER[0],AETHER[1],AETHER[2],150),2)
    c.save(GUI/f"ui/tabs/ui_tab_{name}.png")

def make_node(name,state):
    c=Canvas(256,128)
    edge={"locked":MUTED,"available":AETHER,"owned":STAMINA,"maxed":GOLD}[state]
    fill=(SURFACE[0],SURFACE[1],SURFACE[2],120 if state=="locked" else 175)
    c.fill(32,16,192,96,fill)
    # Faceted hex-ish frame.
    pts=[(48,16),(208,16),(224,32),(224,96),(208,112),(48,112),(32,96),(32,32),(48,16)]
    for p,q in zip(pts,pts[1:]):c.line(*p,*q,(edge[0],edge[1],edge[2],200 if state!="locked" else 105),2)
    if state in ("available","maxed"):
        c.line(70,22,186,22,(edge[0],edge[1],edge[2],95),2)
    if state=="maxed":
        c.line(96,104,160,104,(GOLD[0],GOLD[1],GOLD[2],220),3)
    c.save(GUI/f"ui/panels/ui_tree_node_{name}.png")

def make_meter(folder,prefix,color):
    # Full source remains 512x64 for compatibility with asset tooling.
    frame=Canvas(512,64)
    frame.fill(8,15,496,34,(7,10,14,185))
    frame.line(12,14,500,14,(LINE[0],LINE[1],LINE[2],120),2)
    frame.line(12,50,500,50,(LINE[0],LINE[1],LINE[2],80),1)
    frame.line(8,21,8,44,(color[0],color[1],color[2],185),2)
    frame.save(GUI/f"hud/{folder}/{prefix}_frame.png")
    fill=Canvas(512,64)
    for y in range(18,48):
        t=(y-18)/30
        col=tuple(min(255,int(color[i]*(1.05-0.22*t))) for i in range(3))+(235,)
        fill.fill(8,y,496,1,col)
    fill.line(12,18,500,18,(MOON[0],MOON[1],MOON[2],105),1)
    fill.save(GUI/f"hud/{folder}/{prefix}_fill.png")

    # The runtime deliberately uses trimmed meters so HP/SP can be shorter and
    # thicker than the XP rail without stretching 64px of transparent padding.
    # Keep these exact dimensions in sync with lib.rs: frame 506x52, fill 468x24.
    trim=Canvas(506,52)
    trim.frame(1,(38,54,64,225),2,8)
    trim.frame(5,(color[0],color[1],color[2],180),1,6)
    trim.line(13,7,493,7,(MOON[0],MOON[1],MOON[2],72),1)
    trim.line(17,44,489,44,(color[0],color[1],color[2],88),1)
    # Short end-caps make each resource immediately identifiable at phone size.
    trim.line(7,16,7,35,(color[0],color[1],color[2],225),2)
    trim.line(498,16,498,35,(color[0],color[1],color[2],110),1)
    trim.save(GUI/f"hud/{folder}/{prefix}_frame_trim.png")

    ftrim=Canvas(468,24)
    for y in range(24):
        t=y/23
        gain=1.12-0.30*t
        col=tuple(max(0,min(255,int(color[i]*gain))) for i in range(3))+(238,)
        ftrim.fill(0,y,468,1,col)
    ftrim.line(2,1,465,1,(MOON[0],MOON[1],MOON[2],120),1)
    ftrim.line(2,22,465,22,(0,0,0,72),1)
    # Sparse diagonal facets survive downsampling without looking striped.
    for x in range(42,468,84):
        ftrim.line(x,3,min(467,x+14),20,(MOON[0],MOON[1],MOON[2],28),1)
    ftrim.save(GUI/f"hud/{folder}/{prefix}_fill_trim.png")

make_background()
make_panel("ui/panels/ui_panel_main.png",768,512,AETHER,False)
make_panel("ui/panels/ui_panel_main_compact.png",512,512,AETHER,False)
make_panel("ui/panels/ui_panel_secondary.png",512,512,GOLD,True)
make_panel("ui/panels/ui_panel_notification.png",512,256,AETHER,True)
make_frame("ui/frames/ui_frame_main.png",512,512,AETHER,True)
make_frame("ui/frames/ui_frame_main_compact.png",512,512,AETHER,True)
make_frame("ui/frames/ui_frame_card.png",512,512,AETHER,False)
make_frame("ui/frames/ui_frame_target.png",512,256,HP,False)
make_frame("ui/frames/ui_frame_bestiary.png",512,512,GOLD,False)
make_frame("ui/frames/ui_frame_cyan.png",512,512,AETHER,False)
make_frame("ui/frames/ui_tree_node_focus.png",256,128,GOLD,True)
make_title()
for state in ("normal","hover","pressed","selected","disabled"):make_tab(state,state)
for state in ("locked","available","owned","maxed"):make_node(state,state)
make_meter("health","hud_health",HP)
make_meter("stamina","hud_stamina",STAMINA)
make_meter("mana","hud_mana",MANA)
make_meter("experience","hud_experience",GOLD)
print("generated RustPG rift-forged UI assets")
