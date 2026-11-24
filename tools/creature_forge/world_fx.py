"""Depth-aware world-space attack FX. No camera-facing attack sprites.
Model-space forward = -Z. The same projection and depth buffer render model and FX."""
import math,random
import numpy as np
from PIL import Image
from creature_inspector import rotation,triangle
from motion_lab import unit
def project(p,angle,size,center,extent):
    q=(np.asarray(p)-center)@rotation(angle).T
    return np.array([size/2+q[0]*size*.8/extent,size/2-q[1]*size*.8/extent,q[2]])
def scene(polys,tex,angle,size,center,extent):
    buf=np.zeros((size,size,3),np.uint8);buf[:]=[24,27,32];depth=np.full((size,size),-np.inf)
    for pts,uv,_ in polys:
        p=(pts-center)@rotation(angle).T;n=np.cross(p[1]-p[0],p[2]-p[0]);l=np.linalg.norm(n)
        if l<1e-8:continue
        shade=.62+.38*abs(np.dot(n/l,[.30,.66,.69]))
        p[:,0]=p[:,0]*size*.8/extent+size/2;p[:,1]=-p[:,1]*size*.8/extent+size/2
        for tri in ([0,1,2],[0,2,3]):triangle(buf,depth,p[tri],uv[tri],tex,shade)
    return buf,depth
def stroke(buf,depth,A,B,color,width,alpha):
    # Sample projected segment with its interpolated z. Occlude behind the model.
    n=max(2,int(np.linalg.norm(B[:2]-A[:2])*1.5))
    h,w=depth.shape;rad=max(1,int(width))
    for p in np.linspace(A,B,n):
        x,y=np.rint(p[:2]).astype(int)
        x0=max(0,x-rad);x1=min(w,x+rad+1);y0=max(0,y-rad);y1=min(h,y+rad+1)
        if x0>=x1 or y0>=y1:continue
        yy,xx=np.mgrid[y0:y1,x0:x1]
        mask=((xx-x)**2+(yy-y)**2<=rad**2)&(p[2]+.12>=depth[y0:y1,x0:x1])
        block=buf[y0:y1,x0:x1];block[mask]=np.clip(block[mask]*(1-alpha)+np.array(color)*alpha,0,255).astype(np.uint8)
def effect_paths(event,now,socket,full):
    """Return world-space polylines, each with color, radius, alpha.
    socket(clip,t,name) reads the actual exported model transforms."""
    age=now-event["at"];fx=event["effect"]
    if age<0:return []
    start=event["at"];origin=socket(full,start,event["socket"]);paths=[]
    if fx=="knife_cut" and age<.24:
        # Curved blade trail uses actual blade trajectory, not an invented screen-space arc.
        lo=max(0,start-.16);hi=min(now,start+.10)
        points=[socket(full,float(t),event["socket"]) for t in np.linspace(lo,hi,9)]
        paths.append((points,(216,221,200),2,max(.05,1-age/.24)))
        paths.append(([p+np.array([0,.35,0]) for p in points],(151,155,132),1,max(.05,.65-age*2)))
    elif fx=="spear_thrust" and age<.26:
        grip=socket(full,start,event["direction_from"]);d=unit(origin-grip)
        paths.append(([origin-d*3,origin+d*(1+age*12)],(221,227,207),1,1-age/.26))
        for j in [-1,1]:paths.append(([origin-d*2+np.array([j*.35,0,0]),origin+d*2],(174,166,139),1,.55*(1-age/.26)))
    elif fx=="arrow" and age<.65:
        rear=socket(full,start,event["direction_from"]);d=unit(origin-rear);p=origin+d*age*38
        paths.append(([p-d*4,p],(167,123,68),1,1.))
        paths.append(([p-d*.6,p+d*.65],(221,228,218),1,1.))
        paths.append(([p-d*7,p-d*4],(183,190,167),1,.40))
    elif fx in ("dust","knife_impact","spear_impact") and age<.38:
        rng=random.Random(409+len(fx));col=(151,127,91) if fx=="dust" else (205,191,148)
        for j in range(7):
            direction=np.array([rng.uniform(-1,1),rng.uniform(.15,1),rng.uniform(-.8,-.1)])
            p=origin+direction*age*8
            paths.append(([p,p+direction*.45],col,1,.65*(1-age/.38)))
    return paths
def draw_fx(buf,depth,paths,angle,size,center,extent):
    for pts,col,width,alpha in paths:
        q=[project(p,angle,size,center,extent) for p in pts]
        for a,b in zip(q,q[1:]):stroke(buf,depth,a,b,col,width,alpha)
def self_test():
    # Front and rear cameras must disagree on depth, never on physical forward.
    d=np.array([0,0,-1.]);origin=np.zeros(3)
    assert project(d,(0,180,0),100,origin,20)[2]>project(origin,(0,180,0),100,origin,20)[2]
    assert project(d,(0,0,0),100,origin,20)[2]<project(origin,(0,0,0),100,origin,20)[2]
    b=np.zeros((24,24,3),np.uint8);z=np.ones((24,24))*2
    stroke(b,z,np.array([4,12,0]),np.array([20,12,0]),(255,255,255),2,1)
    assert not b.any()
    stroke(b,z,np.array([4,12,3]),np.array([20,12,3]),(255,255,255),2,1)
    assert b.any()
