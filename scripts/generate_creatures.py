#!/usr/bin/env python3
"""Generate RustPG original creature rigs, textures and entity data.

Standard-library only. All geometry, palettes and animations in this file are original
RustPG content. The generated runtime files are included in RustPG.hmod.
"""
from __future__ import annotations
from pathlib import Path
import json, math, random, re, struct, zlib

REPO=Path(__file__).resolve().parents[1]
ROOT=REPO/'project' if (REPO/'project/src/lib.rs').is_file() else REPO
TEX=ROOT/'assets/rustpg/textures/entity'; TEX.mkdir(parents=True,exist_ok=True)
DATA=ROOT/'data/rustpg/horizon/entity_type'; DATA.mkdir(parents=True,exist_ok=True)
LOOT=ROOT/'data/rustpg/loot_table/entities'; LOOT.mkdir(parents=True,exist_ok=True)

def v3(x,y,z): return [float(x),float(y),float(z)]
class M:
    def __init__(self,id,display,scale,texsize=128):
        self.id=id;self.display=display;self.scale=scale;self.texsize=texsize
        self.bones=[];self.cubes=[];self.keys=[]
    def bone(self,name,parent,pivot,rot=(0,0,0)):
        self.bones.append(dict(name=name,parent=parent,pivot=list(map(float,pivot)),rotation=list(map(float,rot))))
    def cube(self,name,bone,fr,to,origin=None,rot=(0,0,0)):
        if origin is None: origin=[(fr[i]+to[i])/2 for i in range(3)]
        self.cubes.append(dict(name=name,bone=bone,from_=list(map(float,fr)),to=list(map(float,to)),origin=list(map(float,origin)),rotation=list(map(float,rot))))
    def key(self,clip,bone,ch,t,val):
        self.keys.append(dict(clip=clip,bone=bone,channel=ch,time=float(t),value=list(map(float,val))))

def legs4(m, front_x=4.0, back_x=4.0, front_z=-4.0, back_z=4.0, top=10.0, lower=4.0):
    for side,sx in [('l',-1),('r',1)]:
        for pos,z in [('f',front_z),('b',back_z)]:
            up=f'{pos}leg_{side}'; lo=f'{pos}shin_{side}'; foot=f'{pos}paw_{side}'
            x=(front_x if pos=='f' else back_x)*sx
            m.bone(up,'body',[x,top,z]);m.bone(lo,up,[x,lower+2,z]);m.bone(foot,lo,[x,0,z-1.5])
            m.cube(up+'_muscle',up,[x-1.2,lower+2,z-1.3],[x+1.2,top+1,z+1.3])
            m.cube(lo+'_bone',lo,[x-1.0,0.8,z-1.0],[x+1.0,lower+2,z+1.0])
            m.cube(foot+'_pad',foot,[x-1.35,-0.2,z-3.0],[x+1.35,1.4,z+1.0])

def add_walk4(m,clip='walk',length=0.8,amp=28):
    names=['fleg_l','fleg_r','bleg_l','bleg_r']
    phase={'fleg_l':1,'bleg_r':1,'fleg_r':-1,'bleg_l':-1}
    for b in names:
        s=phase[b]*amp
        m.key(f'animation.{m.id}.{clip}',b,'rotation',0,[s,0,0]);m.key(f'animation.{m.id}.{clip}',b,'rotation',length/2,[-s,0,0]);m.key(f'animation.{m.id}.{clip}',b,'rotation',length,[s,0,0])

def make_mana_hound():
    m=M('mana_hound','Cão de Mana',1.0,128)
    m.bone('root',None,[0,0,0]);m.bone('body','root',[0,10,1]);m.bone('chest','body',[0,12,-4]);m.bone('neck','chest',[0,13,-7]);m.bone('head','neck',[0,14,-10]);m.bone('jaw','head',[0,12.5,-12]);m.bone('tail1','body',[0,11,6]);m.bone('tail2','tail1',[0,12,10]);m.bone('tail3','tail2',[0,13,13])
    m.cube('body_hide','body',[-4,7,-1],[4,13,7]);m.cube('chest_hide','chest',[-4.7,8,-7],[4.7,15,-1]);m.cube('neck_hide','neck',[-3,10,-10],[3,15,-6]);m.cube('skull','head',[-3.7,11,-13],[3.7,17,-8]);m.cube('muzzle','jaw',[-2.5,10.5,-15],[2.5,13,-12])
    m.cube('ear_l','head',[-4.2,15,-10],[-2,20,-8]);m.cube('ear_r','head',[2,15,-10],[4.2,20,-8]);m.cube('fang_l','jaw',[-2.1,9.2,-14],[-1.2,11.5,-13]);m.cube('fang_r','jaw',[1.2,9.2,-14],[2.1,11.5,-13])
    for i,x in enumerate([-2.5,0,2.5]):m.cube(f'mana_spine_{i}','body',[x-0.55,13,0],[x+0.55,17,2],rot=(15,0,0))
    m.cube('tail_seg1','tail1',[-1.3,9,5],[1.3,13,10]);m.cube('tail_seg2','tail2',[-1.1,10,9],[1.1,13.5,13]);m.cube('tail_tip','tail3',[-0.8,11,12],[0.8,14,16])
    legs4(m)
    m.key('animation.mana_hound.idle','chest','rotation',0,[-2,0,0]);m.key('animation.mana_hound.idle','chest','rotation',1,[2,0,0]);m.key('animation.mana_hound.idle','chest','rotation',2,[-2,0,0])
    for b,a in [('tail1',12),('tail2',18),('tail3',25)]:m.key('animation.mana_hound.idle',b,'rotation',0,[0,-a,0]);m.key('animation.mana_hound.idle',b,'rotation',1,[0,a,0]);m.key('animation.mana_hound.idle',b,'rotation',2,[0,-a,0])
    add_walk4(m,'run',0.7,32)
    m.key('animation.mana_hound.pounce','body','rotation',0,[0,0,0]);m.key('animation.mana_hound.pounce','body','rotation',0.28,[-20,0,0]);m.key('animation.mana_hound.pounce','body','rotation',0.62,[14,0,0]);m.key('animation.mana_hound.pounce','body','rotation',0.9,[0,0,0])
    return m

def humanoid_base(m,heavy=False):
    m.bone('root',None,[0,0,0]);m.bone('pelvis','root',[0,11,0]);m.bone('torso','pelvis',[0,18,0]);m.bone('neck','torso',[0,24,-0.5]);m.bone('head','neck',[0,27,-0.5])
    for side,sx in [('l',-1),('r',1)]:
        m.bone(f'arm_{side}','torso',[sx*5.2,22,0]);m.bone(f'forearm_{side}',f'arm_{side}',[sx*6.4,15,0]);m.bone(f'leg_{side}','pelvis',[sx*2.6,11,0]);m.bone(f'shin_{side}',f'leg_{side}',[sx*2.6,4,0])
    m.cube('pelvis_plate','pelvis',[-4,9,-2.8],[4,15,2.8]);m.cube('torso_plate','torso',[-5.2,15,-3],[5.2,25,3]);m.cube('neck_guard','neck',[-2.4,23,-2.2],[2.4,26,2]);m.cube('helmet','head',[-4,25,-4],[4,33,4])
    for side,sx in [('l',-1),('r',1)]:
        m.cube(f'pauldron_{side}',f'arm_{side}',[sx*8 if sx<0 else sx*5.2,19,-3],[sx*5.2 if sx<0 else sx*8,25,3])
        m.cube(f'arm_guard_{side}',f'arm_{side}',[sx*7.6 if sx<0 else sx*5.2,13,-2.1],[sx*5.2 if sx<0 else sx*7.6,21,2.1])
        m.cube(f'gauntlet_{side}',f'forearm_{side}',[sx*8.2 if sx<0 else sx*5.5,9,-2.5],[sx*5.5 if sx<0 else sx*8.2,14,2.5])
        m.cube(f'thigh_{side}',f'leg_{side}',[sx*4.5 if sx<0 else sx*1.1,4.5,-2.3],[sx*1.1 if sx<0 else sx*4.5,11,2.3])
        m.cube(f'shin_plate_{side}',f'shin_{side}',[sx*4.3 if sx<0 else sx*1.2,-2,-2.0],[sx*1.2 if sx<0 else sx*4.3,5,2.0])
        m.cube(f'boot_{side}',f'shin_{side}',[sx*4.6 if sx<0 else sx*1.0,-3,-4.5],[sx*1.0 if sx<0 else sx*4.6,1,2.3])


def make_grave_knight():
    m=M('grave_knight','Cavaleiro Sepulcral',1.05,128);humanoid_base(m)
    m.bone('sword','forearm_r',[7.5,10,-1]);m.bone('shield','forearm_l',[-7.5,14,0]);m.bone('cape','torso',[0,23,3.3]);m.bone('crest','head',[0,33,0])
    m.cube('visor','head',[-3.5,27,-4.7],[3.5,30,-3.7])
    m.cube('horn_l','head',[-6.4,31,-1],[-3.4,36,1],rot=(0,0,-22));m.cube('horn_r','head',[3.4,31,-1],[6.4,36,1],rot=(0,0,22))
    m.cube('helmet_crest','crest',[-1.0,32,-1.5],[1.0,39,1.5],rot=(0,0,-8))
    m.cube('grave_blade','sword',[6.7,-3,-1.6],[8.3,14,0.2],rot=(0,0,-8));m.cube('blade_tip','sword',[6.8,-6,-1.5],[8.2,-3,0.1],rot=(0,0,-8))
    m.cube('sword_guard','sword',[4.8,11,-2.2],[10.0,13,1.0],rot=(0,0,-8))
    m.cube('shield_face','shield',[-11.2,7,-1.5],[-5.8,21,0.4],rot=(0,0,7))
    m.cube('shield_boss','shield',[-9.2,12,-2.1],[-7.2,16,-1.2],rot=(0,0,7))
    m.cube('shield_cross_v','shield',[-8.8,8,-2.0],[-7.6,20,-1.3],rot=(0,0,7))
    m.cube('shield_cross_h','shield',[-10.5,13,-2.0],[-6.1,15,-1.3],rot=(0,0,7))
    m.cube('cape_cloth','cape',[-4.6,8,3],[4.6,24,4.2],rot=(10,0,0))
    m.cube('cape_tail_l','cape',[-4.6,2,3.2],[-0.4,10,4.0],rot=(15,0,-5))
    m.cube('cape_tail_r','cape',[0.4,3,3.2],[4.6,10,4.0],rot=(15,0,5))
    m.key('animation.grave_knight.idle','head','rotation',0,[0,-4,0]);m.key('animation.grave_knight.idle','head','rotation',1,[0,4,0]);m.key('animation.grave_knight.idle','head','rotation',2,[0,-4,0])
    m.key('animation.grave_knight.idle','cape','rotation',0,[4,0,-2]);m.key('animation.grave_knight.idle','cape','rotation',1,[9,0,2]);m.key('animation.grave_knight.idle','cape','rotation',2,[4,0,-2])
    for side,s in [('l',1),('r',-1)]:
        for t,v in [(0,22*s),(0.5,-22*s),(1,22*s)]:m.key('animation.grave_knight.walk',f'leg_{side}','rotation',t,[v,0,0]);m.key('animation.grave_knight.walk',f'arm_{side}','rotation',t,[-v*0.5,0,0])
    m.key('animation.grave_knight.slash','arm_r','rotation',0,[0,0,0]);m.key('animation.grave_knight.slash','arm_r','rotation',0.32,[-95,10,12]);m.key('animation.grave_knight.slash','arm_r','rotation',0.62,[30,-18,-20]);m.key('animation.grave_knight.slash','arm_r','rotation',1.0,[0,0,0])
    return m

def make_rift_mantis():
    m=M('rift_mantis','Louva-a-Deus da Fenda',1.0,128)
    m.bone('root',None,[0,0,0]);m.bone('abdomen','root',[0,8,5]);m.bone('thorax','abdomen',[0,11,0]);m.bone('neck','thorax',[0,13,-4]);m.bone('head','neck',[0,15,-7])
    m.cube('abdomen_shell','abdomen',[-3,5,1],[3,11,9]);m.cube('thorax_shell','thorax',[-3.2,8,-4],[3.2,15,2]);m.cube('mantis_head','head',[-3,12,-10],[3,17,-5]);m.cube('jaw','head',[-2.2,11,-11],[2.2,13,-9])
    for side,sx in [('l',-1),('r',1)]:
        m.bone(f'scythe_{side}','thorax',[sx*3,13,-3]);m.bone(f'blade_{side}',f'scythe_{side}',[sx*7,11,-6])
        m.cube(f'scythe_arm_{side}',f'scythe_{side}',[sx*7.5 if sx<0 else sx*3,9,-5],[sx*3 if sx<0 else sx*7.5,14,-2])
        m.cube(f'scythe_blade_{side}',f'blade_{side}',[sx*10 if sx<0 else sx*6.5,5,-10],[sx*6.5 if sx<0 else sx*10,12,-6],rot=(0,0,sx*18))
        for idx,(z,y) in enumerate([(-1,9),(3,7),(6,6)]):
            a=f'leg{idx+1}_{side}';b=f'shin{idx+1}_{side}'
            m.bone(a,'abdomen',[sx*2.5,y,z]);m.bone(b,a,[sx*6,y-2,z])
            m.cube(a+'_seg',a,[sx*6.5 if sx<0 else sx*2.2,y-3,z-0.8],[sx*2.2 if sx<0 else sx*6.5,y+1,z+0.8],rot=(0,0,sx*(18+idx*6)))
            m.cube(b+'_seg',b,[sx*9 if sx<0 else sx*5.5,y-7,z-0.6],[sx*5.5 if sx<0 else sx*9,y-2,z+0.6],rot=(0,0,sx*(-24-idx*4)))
    m.cube('crest_l','head',[-4.5,16,-8],[-1.5,20,-6]);m.cube('crest_r','head',[1.5,16,-8],[4.5,20,-6])
    for t,a in [(0,-5),(0.8,5),(1.6,-5)]:m.key('animation.rift_mantis.idle','head','rotation',t,[0,a,0]);m.key('animation.rift_mantis.idle','abdomen','rotation',t,[0,-a*0.4,0])
    for side,sx in [('l',-1),('r',1)]:
        for idx in range(1,4):
            b=f'leg{idx}_{side}';amp=(12+idx*3)*(1 if (idx%2==0) else -1)*sx
            for t,v in [(0,amp),(0.35,-amp),(0.7,amp)]:m.key('animation.rift_mantis.scuttle',b,'rotation',t,[0,v,0])
    for side,sx in [('l',-1),('r',1)]:
        m.key('animation.rift_mantis.reap',f'scythe_{side}','rotation',0,[0,0,0]);m.key('animation.rift_mantis.reap',f'scythe_{side}','rotation',0.22,[-45,sx*30,sx*25]);m.key('animation.rift_mantis.reap',f'scythe_{side}','rotation',0.48,[35,-sx*45,-sx*30]);m.key('animation.rift_mantis.reap',f'scythe_{side}','rotation',0.8,[0,0,0])
    return m

def make_crystal_wyrm():
    m=M('crystal_wyrm','Serpe Cristalina',1.35,256)
    m.bone('root',None,[0,0,0]);parent='root'
    for i in range(9):
        name=f'spine{i}';m.bone(name,parent,[0,7+i*0.6,4-i*4])
        radius=max(1.8,4.0-i*0.25);z=4-i*4;y=7+i*0.6
        m.cube(f'segment_{i}',name,[-radius,y-radius*0.6,z-2.6],[radius,y+radius*0.6,z+2.6])
        if i in (1,3,5,7):m.cube(f'crystal_{i}',name,[-1.2,y+radius*0.4,z-0.7],[1.2,y+radius*2.2,z+0.7],rot=(-8,0,0))
        parent=name
    m.bone('neck',parent,[0,13,-34]);m.bone('head','neck',[0,15,-39]);m.bone('jaw','head',[0,13,-42])
    m.cube('neck_shell','neck',[-3,10,-39],[3,17,-33]);m.cube('wyrm_head','head',[-4.2,11,-44],[4.2,19,-37]);m.cube('wyrm_muzzle','jaw',[-3,10,-47],[3,14,-43])
    m.cube('horn_l','head',[-5.5,17,-41],[-2.5,23,-39],rot=(0,0,-18));m.cube('horn_r','head',[2.5,17,-41],[5.5,23,-39],rot=(0,0,18))
    for side,sx in [('l',-1),('r',1)]:
        m.cube(f'jaw_fang_{side}','jaw',[sx*3 if sx<0 else sx*1.5,8.7,-46],[sx*1.5 if sx<0 else sx*3,11.5,-44])
    for i in range(9):
        b=f'spine{i}';phase=i*8
        for t,a in [(0,-10+phase%16),(1,10-phase%12),(2,-10+phase%16)]:m.key('animation.crystal_wyrm.idle',b,'rotation',t,[0,a*0.45,0])
        for t,a in [(0,18*math.sin(i*0.7)),(0.8,18*math.sin(i*0.7+math.pi)),(1.6,18*math.sin(i*0.7))]:m.key('animation.crystal_wyrm.slither',b,'rotation',t,[0,a,0])
    m.key('animation.crystal_wyrm.bite','head','rotation',0,[0,0,0]);m.key('animation.crystal_wyrm.bite','head','rotation',0.3,[-28,0,0]);m.key('animation.crystal_wyrm.bite','jaw','rotation',0.3,[34,0,0]);m.key('animation.crystal_wyrm.bite','head','rotation',0.62,[18,0,0]);m.key('animation.crystal_wyrm.bite','jaw','rotation',0.62,[0,0,0]);m.key('animation.crystal_wyrm.bite','head','rotation',0.9,[0,0,0])
    return m

def make_eclipse_ogre():
    m=M('eclipse_ogre','Ogro do Eclipse',1.65,256);humanoid_base(m,True)
    m.bone('core','torso',[0,20,-3.2]);m.bone('jaw','head',[0,26,-4]);m.bone('club','forearm_r',[7.5,10,0])
    m.cube('face_mask','head',[-3.6,27,-4.8],[3.6,31,-4]);m.cube('jaw_tusks','jaw',[-3.8,23,-5],[3.8,27,-3.5])
    m.cube('horn_l','head',[-8,31,-1],[-3.5,38,1.4],rot=(0,0,-20));m.cube('horn_r','head',[3.5,31,-1],[8,38,1.4],rot=(0,0,20))
    m.cube('eclipse_core','core',[-2.2,18,-4],[2.2,22,-3.2]);m.cube('back_spike_l','torso',[-4,22,2.7],[-2,31,4],rot=(-15,0,-10));m.cube('back_spike_r','torso',[2,22,2.7],[4,31,4],rot=(-15,0,10))
    m.cube('war_club','club',[6.2,-8,-2.5],[8.8,15,2.5],rot=(0,0,-8));m.cube('club_head','club',[4.5,-12,-5],[10.5,-6,5],rot=(0,0,-8))
    for t,s in [(0,1.0),(1,1.12),(2,1.0)]:m.key('animation.eclipse_ogre.idle','core','scale',t,[s,s,s])
    m.key('animation.eclipse_ogre.idle','head','rotation',0,[0,-3,0]);m.key('animation.eclipse_ogre.idle','head','rotation',1,[0,3,0]);m.key('animation.eclipse_ogre.idle','head','rotation',2,[0,-3,0])
    for side,s in [('l',1),('r',-1)]:
        for t,v in [(0,18*s),(0.65,-18*s),(1.3,18*s)]:m.key('animation.eclipse_ogre.walk',f'leg_{side}','rotation',t,[v,0,0]);m.key('animation.eclipse_ogre.walk',f'arm_{side}','rotation',t,[-v*0.35,0,0])
    m.key('animation.eclipse_ogre.smash','arm_r','rotation',0,[0,0,0]);m.key('animation.eclipse_ogre.smash','forearm_r','rotation',0,[0,0,0]);m.key('animation.eclipse_ogre.smash','arm_r','rotation',0.45,[-115,0,12]);m.key('animation.eclipse_ogre.smash','forearm_r','rotation',0.45,[-35,0,0]);m.key('animation.eclipse_ogre.smash','arm_r','rotation',0.78,[45,0,-8]);m.key('animation.eclipse_ogre.smash','forearm_r','rotation',0.78,[20,0,0]);m.key('animation.eclipse_ogre.smash','arm_r','rotation',1.25,[0,0,0]);m.key('animation.eclipse_ogre.smash','forearm_r','rotation',1.25,[0,0,0])
    return m


# Additional original RustPG 6.0 silhouettes and animation archetypes.

def make_dusk_revenant():
    m=M('dusk_revenant','Espectro do Crepúsculo',1.1,128)
    m.bone('root',None,[0,0,0]);m.bone('body','root',[0,15,0]);m.bone('chest','body',[0,21,0])
    m.bone('hood','chest',[0,27,-1]);m.bone('head','hood',[0,28,-2]);m.bone('core','chest',[0,20,-3])
    for side,sx in [('l',-1),('r',1)]:
        m.bone(f'arm_{side}','chest',[sx*5.4,22,0]);m.bone(f'forearm_{side}',f'arm_{side}',[sx*7.0,14,-1])
        m.bone(f'tendril_{side}','body',[sx*2.6,10,2])
    m.bone('shroud_l','body',[-2.8,14,2]);m.bone('shroud_c','body',[0,14,2]);m.bone('shroud_r','body',[2.8,14,2])
    m.cube('wraith_core','body',[-3.6,12,-2.8],[3.6,22,2.8])
    m.cube('mantle','chest',[-6.2,19,-3.2],[6.2,25,3.4])
    m.cube('hood_shell','hood',[-4.4,24,-4.2],[4.4,34,3.0])
    m.cube('face_void','head',[-2.8,26,-4.8],[2.8,31,-3.8])
    m.cube('core_ember','core',[-1.7,18,-3.7],[1.7,22,-3.0])
    m.cube('crown_l','hood',[-5.2,31,-1.0],[-2.8,37,1.0],rot=(0,0,-18))
    m.cube('crown_r','hood',[2.8,31,-1.0],[5.2,37,1.0],rot=(0,0,18))
    for side,sx in [('l',-1),('r',1)]:
        m.cube(f'arm_shroud_{side}',f'arm_{side}',[sx*8.2 if sx<0 else sx*4.6,14,-2.0],[sx*4.6 if sx<0 else sx*8.2,24,2.0],rot=(0,0,sx*7))
        m.cube(f'forearm_wisp_{side}',f'forearm_{side}',[sx*9.0 if sx<0 else sx*5.7,6,-2.4],[sx*5.7 if sx<0 else sx*9.0,16,1.2],rot=(0,0,sx*10))
        m.cube(f'claw_{side}',f'forearm_{side}',[sx*9.6 if sx<0 else sx*5.8,3,-4.2],[sx*5.8 if sx<0 else sx*9.6,8,-0.2])
        m.cube(f'tendril_upper_{side}',f'tendril_{side}',[sx*4.2 if sx<0 else sx*1.8,4,1.8],[sx*1.8 if sx<0 else sx*4.2,13,3.8],rot=(8,0,sx*8))
        m.cube(f'tendril_tip_{side}',f'tendril_{side}',[sx*4.0 if sx<0 else sx*2.0,-2,2.0],[sx*2.0 if sx<0 else sx*4.0,5,3.5],rot=(15,0,-sx*12))
    for name,x0,x1,rot in [('shroud_l',-4.4,-1.0,-7),('shroud_c',-1.5,1.5,0),('shroud_r',1.0,4.4,7)]:
        m.cube(name+'_cloth',name,[x0,2,1.8],[x1,16,4.0],rot=(8,0,rot))
        m.cube(name+'_tatter',name,[x0+0.3,-4,2.0],[x1-0.2,4,3.6],rot=(14,0,rot*1.4))
    for t,a in [(0,-5),(1,5),(2,-5)]:
        m.key('animation.dusk_revenant.idle','head','rotation',t,[0,a,0]);m.key('animation.dusk_revenant.idle','chest','rotation',t,[2+a*.12,0,0])
        m.key('animation.dusk_revenant.idle','shroud_l','rotation',t,[2,0,-a*.6]);m.key('animation.dusk_revenant.idle','shroud_r','rotation',t,[2,0,a*.6])
    for side,amp in [('l',18),('r',-18)]:
        for t,a in [(0,amp),(.55,-amp),(1.1,amp)]:m.key('animation.dusk_revenant.glide',f'arm_{side}','rotation',t,[a,0,0])
        for t,a in [(0,-12),(.45,80),(.8,-45),(1.1,-12)]:m.key('animation.dusk_revenant.soul_rake',f'forearm_{side}','rotation',t,[a,0,0])
    return m


def make_storm_gryphon():
    m=M('storm_gryphon','Grifo Tempestuoso',1.35,256)
    m.bone('root',None,[0,0,0]);m.bone('body','root',[0,11,2]);m.bone('chest','body',[0,14,-4])
    m.bone('neck','chest',[0,17,-8]);m.bone('head','neck',[0,20,-12]);m.bone('beak','head',[0,19,-16]);m.bone('tail','body',[0,12,9])
    m.cube('lion_body','body',[-4.6,7,-1],[4.6,15,10]);m.cube('eagle_chest','chest',[-5.3,9,-9],[5.3,19,0])
    m.cube('neck_fur','neck',[-3.2,14,-13],[3.2,22,-6]);m.cube('eagle_head','head',[-3.7,17,-16],[3.7,24,-10])
    m.cube('hooked_beak','beak',[-2.4,16,-20],[2.4,20,-15]);m.cube('beak_tip','beak',[-1.8,15,-22],[1.8,18,-19],rot=(10,0,0))
    m.cube('tail_root','tail',[-2.0,9,7],[2.0,14,15])
    for i,(x,z) in enumerate([(0,14),(-2,15),(2,15)]):
        m.cube(f'tail_feather_{i}','tail',[x-1.2,8,z],[x+1.2,11,z+9],rot=(0,0,(i-1)*7))
    for side,sx in [('l',-1),('r',1)]:
        m.bone(f'wing_{side}','chest',[sx*4.2,18,-3]);m.bone(f'wingtip_{side}',f'wing_{side}',[sx*10.0,18,0])
        m.cube(f'wing_base_{side}',f'wing_{side}',
            [sx*10.5 if sx<0 else sx*3.0,16,-6],[sx*3.0 if sx<0 else sx*10.5,19,4],
            origin=[sx*4.2,18,-3],rot=(0,-sx*8,sx*8))
        for i in range(5):
            z=-1+i*2.0
            m.cube(f'flight_feather_{side}_{i}',f'wingtip_{side}',
                [sx*(18.5-i*.6) if sx<0 else sx*9.0,13-i*.35,z],
                [sx*9.0 if sx<0 else sx*(18.5-i*.6),16.2-i*.35,z+1.5],
                origin=[sx*10.0,18,0],rot=(0,-sx*(8+i*2),sx*(10+i*2)))
        # Eagle forelegs: slimmer, forward talons.
        b=f'front_leg_{side}';k=f'front_ankle_{side}'
        m.bone(b,'chest',[sx*3.4,9,-5]);m.bone(k,b,[sx*3.5,3,-6])
        m.cube(f'{b}_feather',b,[sx*4.8 if sx<0 else sx*2.3,3,-6.2],[sx*2.3 if sx<0 else sx*4.8,10,-3.8])
        m.cube(f'{k}_talon',k,[sx*4.9 if sx<0 else sx*2.2,0,-9],[sx*2.2 if sx<0 else sx*4.9,3.5,-4.5])
        m.cube(f'{k}_toe',k,[sx*4.8 if sx<0 else sx*2.3,-0.2,-11],[sx*2.3 if sx<0 else sx*4.8,1.3,-8])
        # Lion hind legs: heavier and set back.
        b=f'hind_leg_{side}';k=f'hind_ankle_{side}'
        m.bone(b,'body',[sx*3.4,9,6]);m.bone(k,b,[sx*3.5,3,7])
        m.cube(f'{b}_fur',b,[sx*5.0 if sx<0 else sx*2.0,3,4.5],[sx*2.0 if sx<0 else sx*5.0,11,7.8])
        m.cube(f'{k}_paw',k,[sx*5.2 if sx<0 else sx*1.9,0,5.5],[sx*1.9 if sx<0 else sx*5.2,3.5,10.0])
    m.cube('crest_left','head',[-4.4,22,-13],[-2.0,28,-10],rot=(0,0,-12));m.cube('crest_right','head',[2.0,22,-13],[4.4,28,-10],rot=(0,0,12))
    for t,a in [(0,-4),(1,4),(2,-4)]:
        m.key('animation.storm_gryphon.idle','head','rotation',t,[0,a,0]);m.key('animation.storm_gryphon.idle','tail','rotation',t,[0,a*2,0])
    for side,sx in [('l',-1),('r',1)]:
        for t,a in [(0,38),(1,44),(2,38)]:
            m.key('animation.storm_gryphon.idle',f'wing_{side}','rotation',t,[6,-sx*6,sx*a])
            m.key('animation.storm_gryphon.idle',f'wingtip_{side}','rotation',t,[0,-sx*8,sx*28])
        for t,a in [(0,-24),(.45,28),(.9,-24)]:
            m.key('animation.storm_gryphon.flight',f'wing_{side}','rotation',t,[a,0,sx*a])
            m.key('animation.storm_gryphon.flight',f'wingtip_{side}','rotation',t,[a*.2,0,-sx*a*.8])
        for t,a in [(0,0),(.25,-60),(.55,24),(.9,0)]:m.key('animation.storm_gryphon.thunder_dive',f'wing_{side}','rotation',t,[a,0,sx*35])
    return m

def make_rune_colossus():
    m=M('rune_colossus','Colosso Rúnico',1.9,256);humanoid_base(m,True)
    m.bone('core','torso',[0,20,-3.5]);m.bone('crown','head',[0,33,0]);m.bone('spine','torso',[0,21,3])
    m.cube('rune_heart','core',[-3,18,-4.1],[3,23,-3]);m.cube('forehead_sigil','head',[-2,30,-4.7],[2,32,-4])
    m.cube('crown_stone','crown',[-5.5,33,-3],[5.5,37,3]);m.cube('back_monolith','spine',[-2.2,19,3],[2.2,31,5])
    for side,sx in [('l',-1),('r',1)]:
        m.cube(f'stone_fist_{side}',f'forearm_{side}',[sx*10 if sx<0 else sx*5.1,5,-4],[sx*5.1 if sx<0 else sx*10,13,4])
        m.cube(f'runic_pauldron_{side}',f'arm_{side}',[sx*9.8 if sx<0 else sx*5.2,20,-4],[sx*5.2 if sx<0 else sx*9.8,27,4])
        m.cube(f'leg_rune_{side}',f'shin_{side}',[sx*4.6 if sx<0 else sx*1.2,0,-2.5],[sx*1.2 if sx<0 else sx*4.6,4.5,-1.8])
    for t,x in [(0,1),(1,1.15),(2,1)]:m.key('animation.rune_colossus.idle','core','scale',t,[x,x,x]);m.key('animation.rune_colossus.idle','head','rotation',t,[0,(x-1)*30,0])
    for side,x in [('l',1),('r',-1)]:
        for t,a in [(0,14*x),(.75,-14*x),(1.5,14*x)]:m.key('animation.rune_colossus.walk',f'leg_{side}','rotation',t,[a,0,0])
        for t,a in [(0,0),(.4,-120),(.8,55),(1.3,0)]:m.key('animation.rune_colossus.earthshatter',f'arm_{side}','rotation',t,[a,0,x*12])
    return m


def make_ember_imp():
    m=M('ember_imp','Diabrete das Brasas',0.78,128)
    m.bone('root',None,[0,0,0]);m.bone('body','root',[0,11,0]);m.bone('chest','body',[0,17,0]);m.bone('head','chest',[0,23,-1]);m.bone('jaw','head',[0,20,-4])
    for side,sx in [('l',-1),('r',1)]:
        m.bone(f'arm_{side}','chest',[sx*4.2,18,0]);m.bone(f'forearm_{side}',f'arm_{side}',[sx*6.0,12,-1])
        m.bone(f'leg_{side}','body',[sx*2.3,9,0]);m.bone(f'foot_{side}',f'leg_{side}',[sx*2.5,2,-1])
        m.bone(f'wing_{side}','chest',[sx*3.6,20,2.5])
    m.bone('tail1','body',[0,10,3]);m.bone('tail2','tail1',[0,9,9])
    m.cube('imp_body','body',[-3.2,8,-2.3],[3.2,17,3]);m.cube('imp_chest','chest',[-4.3,14,-3],[4.3,21,3])
    m.cube('imp_head','head',[-4.3,20,-5],[4.3,29,3]);m.cube('hot_muzzle','jaw',[-2.5,18,-6],[2.5,22,-4])
    m.cube('horn_l','head',[-5.8,26,-2],[-2.6,34,0.5],rot=(0,0,-22));m.cube('horn_r','head',[2.6,26,-2],[5.8,34,0.5],rot=(0,0,22))
    m.cube('eye_l','head',[-2.6,24,-5.4],[-1.0,26,-4.8]);m.cube('eye_r','head',[1.0,24,-5.4],[2.6,26,-4.8])
    for side,sx in [('l',-1),('r',1)]:
        m.cube(f'arm_{side}_skin',f'arm_{side}',[sx*6.5 if sx<0 else sx*3.6,11,-2],[sx*3.6 if sx<0 else sx*6.5,19,2],rot=(0,0,sx*10))
        m.cube(f'forearm_{side}_skin',f'forearm_{side}',[sx*7.6 if sx<0 else sx*5.0,6,-2.2],[sx*5.0 if sx<0 else sx*7.6,13,1.6],rot=(0,0,sx*8))
        m.cube(f'claw_{side}',f'forearm_{side}',[sx*8.3 if sx<0 else sx*5.0,3,-3.4],[sx*5.0 if sx<0 else sx*8.3,7,-0.2])
        m.cube(f'leg_{side}_skin',f'leg_{side}',[sx*3.8 if sx<0 else sx*1.2,2,-1.8],[sx*1.2 if sx<0 else sx*3.8,10,2.0],rot=(0,0,-sx*4))
        m.cube(f'foot_{side}_claw',f'foot_{side}',[sx*4.0 if sx<0 else sx*1.0,-1,-4.2],[sx*1.0 if sx<0 else sx*4.0,3,1.8])
        m.cube(f'bat_wing_{side}',f'wing_{side}',
            [sx*12.5 if sx<0 else sx*3.5,13,2.0],[sx*3.5 if sx<0 else sx*12.5,22,3.4],
            origin=[sx*3.6,20,2.5],rot=(0,0,sx*24))
        m.cube(f'wing_tip_{side}',f'wing_{side}',
            [sx*15 if sx<0 else sx*9,10,2.2],[sx*9 if sx<0 else sx*15,18,3.2],
            origin=[sx*9,17,2.5],rot=(0,0,sx*12))
    m.cube('tail_segment','tail1',[-1.1,7,2.5],[1.1,11,10]);m.cube('tail_tip','tail2',[-1.0,7,8],[1.0,11,15],rot=(0,12,0))
    m.cube('tail_fire','tail2',[-2.0,8,14],[2.0,13,18])
    for t,x in [(0,-14),(.5,14),(1,-14),(1.5,14),(2,-14)]:
        m.key('animation.ember_imp.idle','tail1','rotation',t,[0,x,0]);m.key('animation.ember_imp.idle','tail2','rotation',t,[0,x*.7,0]);m.key('animation.ember_imp.idle','head','rotation',t,[0,x*.15,0])
    for side,sx in [('l',-1),('r',1)]:
        for t,x in [(0,-18),(.25,38),(.5,-18)]:m.key('animation.ember_imp.hover',f'wing_{side}','rotation',t,[0,0,x*sx])
        for t,x in [(0,0),(.3,-60),(.6,25),(.9,0)]:m.key('animation.ember_imp.ember_slam',f'wing_{side}','rotation',t,[x,0,0])
    return m


def make_abyss_weaver():
    m=M('abyss_weaver','Tecelã Abissal',1.1,128)
    m.bone('root',None,[0,0,0]);m.bone('body','root',[0,10,0]);m.bone('abdomen','body',[0,11,5]);m.bone('head','body',[0,11,-5]);m.bone('fangs','head',[0,9,-8])
    m.cube('weaver_chitin','body',[-3.6,7,-4],[3.6,14,3]);m.cube('abdomen_core','abdomen',[-4.4,7,1],[4.4,15,11])
    m.cube('eye_cluster','head',[-2.9,9,-9],[2.9,14,-4]);m.cube('fang_l','fangs',[-2.2,6,-11],[-0.4,10,-8],rot=(0,0,-8));m.cube('fang_r','fangs',[0.4,6,-11],[2.2,10,-8],rot=(0,0,8))
    for side,sx in [('l',-1),('r',1)]:
        for i in range(4):
            z=-5+i*3.1;b=f'leg{i}_{side}';k=f'knee{i}_{side}'
            hip=[sx*3.2,10.5,z];knee=[sx*(7.1+i*.25),9.8-i*.25,z]
            m.bone(b,'body',hip);m.bone(k,b,knee)
            m.cube(f'{b}_upper',b,
                [sx*7.6 if sx<0 else sx*2.7,8.6,z-0.75],[sx*2.7 if sx<0 else sx*7.6,11.8,z+0.75],
                origin=hip,rot=(0,0,sx*(20+i*3)))
            m.cube(f'{k}_lower',k,
                [sx*10.5 if sx<0 else sx*6.6,1.5,z-0.6],[sx*6.6 if sx<0 else sx*10.5,10.2,z+0.6],
                origin=knee,rot=(0,0,-sx*(25+i*4)))
            m.cube(f'{k}_talon',k,
                [sx*11.5 if sx<0 else sx*9.1,0,z-2.2],[sx*9.1 if sx<0 else sx*11.5,2.1,z+0.4],
                origin=[sx*10.0,1.2,z],rot=(0,sx*(8+i*2),0))
            phase=(1 if i%2 else -1)*sx
            for t,x in [(0,-18),(.3,18),(.6,-18)]:m.key('animation.abyss_weaver.skitter',b,'rotation',t,[0,x*phase,0])
            for t,x in [(0,8),(.3,-10),(.6,8)]:m.key('animation.abyss_weaver.skitter',k,'rotation',t,[0,0,x*phase])
    m.cube('spine_rune','abdomen',[-1.5,15,5],[1.5,19,7]);m.cube('stinger','abdomen',[-1.1,8,10],[1.1,12,15],rot=(0,0,4))
    for t,x in [(0,-4),(1,4),(2,-4)]:m.key('animation.abyss_weaver.idle','abdomen','rotation',t,[x,0,0]);m.key('animation.abyss_weaver.idle','head','rotation',t,[0,x*.7,0])
    for t,x in [(0,0),(.25,30),(.55,-22),(.85,0)]:m.key('animation.abyss_weaver.venom_bite','fangs','rotation',t,[x,0,0])
    return m

MODELS_LIST=[make_mana_hound(),make_grave_knight(),make_rift_mantis(),make_crystal_wyrm(),make_eclipse_ogre(),make_dusk_revenant(),make_storm_gryphon(),make_rune_colossus(),make_ember_imp(),make_abyss_weaver()]

PALETTES={
 'mana_hound':((22,38,55),(55,86,110),(78,219,240),(210,252,255)),
 'grave_knight':((32,30,42),(75,70,92),(144,105,197),(225,210,247)),
 'rift_mantis':((24,27,46),(61,70,105),(203,84,221),(120,245,229)),
 'crystal_wyrm':((16,43,60),(42,90,112),(69,211,239),(208,253,255)),
 'eclipse_ogre':((47,24,24),(102,51,40),(232,77,54),(255,198,92)),
 'dusk_revenant':((24,20,44),(69,50,92),(143,101,226),(235,216,255)),
 'storm_gryphon':((21,43,68),(61,96,128),(91,197,250),(238,250,255)),
 'rune_colossus':((34,43,46),(92,108,108),(104,232,207),(224,255,245)),
 'ember_imp':((57,25,20),(126,51,33),(248,101,50),(255,230,126)),
 'abyss_weaver':((34,26,51),(80,64,102),(166,88,231),(239,194,255)),
}
def _png_chunk(kind:bytes,data:bytes)->bytes:
    return struct.pack(">I",len(data))+kind+data+struct.pack(">I",zlib.crc32(kind+data)&0xffffffff)

def _save_png(path:Path,w:int,h:int,pixels:list[tuple[int,int,int,int]])->None:
    raw=bytearray()
    for y in range(h):
        raw.append(0)
        for x in range(w): raw.extend(pixels[y*w+x])
    data=b"\x89PNG\r\n\x1a\n"
    data+=_png_chunk(b"IHDR",struct.pack(">IIBBBBB",w,h,8,6,0,0,0))
    data+=_png_chunk(b"IDAT",zlib.compress(bytes(raw),9))
    data+=_png_chunk(b"IEND",b"")
    path.write_bytes(data)

def make_texture(m:M):
    """Paint a compact material sheet that reinforces each creature archetype.

    The native generated renderer does not expose authored cube UV islands yet,
    so the texture uses scale-stable material motifs rather than pretending a
    specific face atlas exists.  Geometry remains the silhouette authority.
    """
    N=m.texsize;base,mid,accent,shine=PALETTES[m.id]
    rng=random.Random(0xBADA55+sum(map(ord,m.id)))
    pixels=[]
    for y in range(N):
      for x in range(N):
        # Directional light gives every plate/carapace a readable top-left edge
        # instead of a flat noisy colour.
        light=(1.0-x/max(1,N-1))*0.07+(1.0-y/max(1,N-1))*0.11
        wave=0.035*math.sin(x*.19)+0.025*math.cos(y*.23)
        grain=rng.randint(-3,3)
        t=max(0.34,min(0.74,0.48+light+wave))
        c=[max(0,min(255,int(base[i]*(1-t)+mid[i]*t+grain+4))) for i in range(3)]
        pixels.append([*c,255])
    def put(x,y,color,alpha=255):
        if 0<=x<N and 0<=y<N:
            old=pixels[y*N+x];t=alpha/255.0
            for i in range(3):old[i]=int(old[i]*(1-t)+color[i]*t)
    def line(x0,y0,x1,y1,color,alpha=255,width=1):
        x0=int(x0);y0=int(y0);x1=int(x1);y1=int(y1)
        dx=abs(x1-x0);sx=1 if x0<x1 else -1;dy=-abs(y1-y0);sy=1 if y0<y1 else -1;err=dx+dy
        while True:
            for oy in range(-(width//2),width//2+1):
                for ox in range(-(width//2),width//2+1):put(x0+ox,y0+oy,color,alpha)
            if x0==x1 and y0==y1:break
            e2=2*err
            if e2>=dy:err+=dy;x0+=sx
            if e2<=dx:err+=dx;y0+=sy
    def rect(x,y,w,h,color,alpha=255):
        for yy in range(max(0,int(y)),min(N,int(y+h))):
            for xx in range(max(0,int(x)),min(N,int(x+w))):put(xx,yy,color,alpha)
    def diamond(cx,cy,r,color,alpha=255):
        for yy in range(int(cy-r),int(cy+r)+1):
            span=max(0,int(r-abs(yy-cy)))
            line(cx-span,yy,cx+span,yy,color,alpha,1)

    # Broad armor/carapace plates.  The light edge and dark lower edge make
    # geometry facets survive downsampling on a phone.
    step=max(16,N//7)
    for y in range(0,N,step):
      for x in range(0,N,step):
        if (x//step+y//step)%2==0:
          line(x+2,y+3,min(N-1,x+step-3),y+3,shine,58,1)
          line(x+3,min(N-1,y+step-3),min(N-1,x+step-2),min(N-1,y+step-3),base,150,1)
          line(x+2,y+3,x+2,min(N-1,y+step-3),mid,100,1)

    # Archetype-specific material language.
    if m.id=="mana_hound":
      for y in range(N//6,N, N//4):
        for x in range(0,N-8,12):line(x,y,min(N-1,x+7),y-4,accent,150,2)
    elif m.id=="grave_knight":
      for x in range(N//8,N,N//4):
        line(x,4,x,N-5,mid,155,max(1,N//128))
        for y in range(12,N,24):diamond(x,y,2,accent,135)
    elif m.id=="rift_mantis":
      for y in range(10,N,20):
        line(N//2,y,8,min(N-1,y+10),accent,120,1)
        line(N//2,y,N-9,min(N-1,y+10),accent,120,1)
    elif m.id=="crystal_wyrm":
      for x in range(-N//3,N,N//5):
        line(x,N-1,x+N//2,0,accent,150,2)
        line(x+3,N-1,x+N//2+3,0,shine,75,1)
    elif m.id=="eclipse_ogre":
      for _ in range(max(10,N//12)):
        x=rng.randrange(N);y=rng.randrange(N);pts=[(x,y)]
        for __ in range(4):
          x=max(0,min(N-1,x+rng.randint(-N//9,N//9)));y=max(0,min(N-1,y+rng.randint(3,N//10)));pts.append((x,y))
        for a,b in zip(pts,pts[1:]):line(*a,*b,accent,185,2)
    elif m.id=="dusk_revenant":
      for x in range(8,N,18):
        line(x,N-4,x+int(5*math.sin(x)),6,accent,92,2)
        line(x+2,N-4,x+2+int(5*math.sin(x)),6,shine,35,1)
    elif m.id=="storm_gryphon":
      for y in range(8,N,16):
        for x in range((y//16%2)*8,N,16):
          line(x,y,x+7,y+8,shine,90,1);line(x+14,y,x+7,y+8,accent,115,1)
    elif m.id=="rune_colossus":
      for y in range(14,N,28):
        for x in range(14,N,28):
          diamond(x,y,5,accent,125);diamond(x,y,2,shine,105)
    elif m.id=="ember_imp":
      cx=cy=N//2
      for k in range(12):
        a=k*math.tau/12+rng.uniform(-.12,.12)
        r=N*.48
        line(cx,cy,int(cx+math.cos(a)*r),int(cy+math.sin(a)*r),accent,145,2)
      diamond(cx,cy,max(4,N//18),shine,165)
    elif m.id=="abyss_weaver":
      cx=cy=N//2
      for r in range(max(10,N//12),N//2,max(10,N//10)):
        pts=[(int(cx+math.cos(a)*r),int(cy+math.sin(a)*r)) for a in [i*math.tau/12 for i in range(12)]]
        for a,b in zip(pts,pts[1:]+pts[:1]):line(*a,*b,accent,85,1)
      for k in range(8):
        a=k*math.tau/8;line(cx,cy,int(cx+math.cos(a)*N*.48),int(cy+math.sin(a)*N*.48),shine,65,1)

    # Sparse emissive-looking fissures/runes are focal points, not noise.
    for _ in range(max(5,N//32)):
      x=rng.randrange(5,N-5);y=rng.randrange(5,N-5)
      diamond(x,y,max(1,N//128),shine,165)
      line(x-4,y,x+4,y,accent,80,1)

    out=TEX/f'{m.id}.png'
    _save_png(out,N,N,[tuple(p) for p in pixels])
    return out

def rust_f(v):
    s=f'{float(v):.5f}'.rstrip('0').rstrip('.')
    if '.' not in s:s+='.0'
    return s+'f32'
def rv(v):return '['+', '.join(rust_f(x) for x in v)+']'
def cname(id):return re.sub(r'[^A-Z0-9_]','_',id.upper())

def generated_rust(models):
    out=['// @generated from original RustPG creature sources; do not hand-edit.\n','use crate::rustpg_generated_models::{GeneratedBone,GeneratedCube,GeneratedKey};\n']
    for m in models:
        P=cname(m.id);idx={b['name']:i for i,b in enumerate(m.bones)}
        out.append(f'pub const {P}_BONES:&[GeneratedBone]=&[\n')
        for b in m.bones:
            par='None' if b['parent'] is None else f'Some({json.dumps(b["parent"])})'
            out.append(f' GeneratedBone{{name:{json.dumps(b["name"])},parent:{par},pivot:{rv(b["pivot"])},rotation:{rv(b["rotation"])}}},\n')
        out.append('];\n');out.append(f'pub const {P}_CUBES:&[GeneratedCube]=&[\n')
        for c in m.cubes:
            out.append(f' GeneratedCube{{name:{json.dumps(c["name"])},bone:{json.dumps(c["bone"])},from:{rv(c["from_"])},to:{rv(c["to"])},origin:{rv(c["origin"])},rotation:{rv(c["rotation"])}}},\n')
        out.append('];\n');out.append(f'pub const {P}_CHAINS:&[&[usize]]=&[\n')
        for c in m.cubes:
            chain=[];b=c['bone'];seen=set()
            while b is not None:
                if b in seen:raise ValueError('cycle '+b)
                seen.add(b);chain.append(idx[b]);b=m.bones[idx[b]]['parent']
            out.append(' &['+','.join(map(str,reversed(chain)))+'],\n')
        out.append('];\n');out.append(f'pub const {P}_KEYS:&[GeneratedKey]=&[\n')
        for k in m.keys:
            out.append(f' GeneratedKey{{clip:{json.dumps(k["clip"])},bone:{json.dumps(k["bone"])},channel:{json.dumps(k["channel"])},time:{rust_f(k["time"])},value:{rv(k["value"])}}},\n')
        out.append('];\n')
    return ''.join(out)

for m in MODELS_LIST:make_texture(m)
(ROOT/'src/rustpg_creatures_generated.rs').write_text(generated_rust(MODELS_LIST),encoding='utf-8')

entity_defs={
 'mana_hound':('minecraft:zombie',38.0,0.36,6.0,0.55,0.55),
 'grave_knight':('minecraft:zombie',92.0,0.27,10.0,0.62,0.65),
 'rift_mantis':('minecraft:spider',72.0,0.34,9.0,0.72,0.55),
 'crystal_wyrm':('minecraft:cave_spider',185.0,0.31,13.0,1.0,0.72),
 'eclipse_ogre':('minecraft:ravager',460.0,0.24,19.0,1.25,0.8),
 'dusk_revenant':('minecraft:wither_skeleton',78.0,0.30,10.0,0.55,0.5),
 'storm_gryphon':('minecraft:phantom',160.0,0.31,15.0,0.9,0.65),
 'rune_colossus':('minecraft:iron_golem',385.0,0.19,22.0,1.4,0.85),
 'ember_imp':('minecraft:zombie',27.0,0.39,5.0,0.4,0.45),
 'abyss_weaver':('minecraft:spider',105.0,0.31,12.0,0.9,0.6),
}
for id,(template,hp,speed,dmg,shadow,strength) in entity_defs.items():
    (DATA/f'{id}.json').write_text(json.dumps({'template':template,'shadow_radius':shadow,'shadow_strength':strength,'attributes':{'minecraft:max_health':hp,'minecraft:movement_speed':speed,'minecraft:attack_damage':dmg}},indent=2),encoding='utf-8')
loot_items={'mana_hound':'minecraft:amethyst_shard','grave_knight':'minecraft:iron_ingot','rift_mantis':'minecraft:spider_eye','crystal_wyrm':'minecraft:amethyst_shard','eclipse_ogre':'minecraft:blaze_rod','dusk_revenant':'minecraft:echo_shard','storm_gryphon':'minecraft:feather','rune_colossus':'minecraft:iron_ingot','ember_imp':'minecraft:blaze_powder','abyss_weaver':'minecraft:spider_eye'}
for id,item in loot_items.items():
    loot={'type':'minecraft:entity','pools':[{'rolls':1,'entries':[{'type':'minecraft:item','name':item}]}]}
    (LOOT/f'{id}.json').write_text(json.dumps(loot,indent=2),encoding='utf-8')

print('generated',[(m.id,len(m.bones),len(m.cubes),len(m.keys),m.texsize) for m in MODELS_LIST])
