#!/usr/bin/env python3
"""Import the approved exact BBModel archive into a disposable Preview 4 stage.
Uses public renderer APIs. Does not install AI/attack timing or mutate stable releases.
"""
from pathlib import Path
import argparse, base64, hashlib, io, json, math, zipfile

SHA256='c12cf15398f9041939284110f440ebe69faabe5135350af4e8a471bab82baf4e'
ROSTER={
 'goblin_knife':('goblin-knife-v2-final.bbmodel','minecraft:zombie',32,0.19,5,0.30),
 'goblin_spear':('goblin-spear-v2-final.bbmodel','minecraft:zombie',44,0.17,7,0.30),
 'goblin_archer':('goblin-archer-v2-final.bbmodel','minecraft:skeleton',28,0.18,None,0.30),
 'goblin_boss':('goblin-boss-v2.bbmodel','minecraft:zombie',220,0.15,13,1.0),
}
DIRS=('down','up','north','south','west','east')

def finite(n):
    n=float(n)
    if not math.isfinite(n): raise ValueError('nonfinite value')
    return n

def f(n):return f'{finite(n):.9g}' + ('f32' if '.' in f'{finite(n):.9g}' or 'e' in f'{finite(n):.9g}' else '.0f32')
def vec(v):return '['+','.join(f(x) for x in v)+']'
def q(s):return json.dumps(s,ensure_ascii=False)
def point(v):return [finite(v[k]) for k in 'xyz']
def model(d,kind):
    group={g['uuid']:g for g in d['groups']}; elements={c['uuid']:c for c in d['elements']}
    names=set();bones=[];parts=[];paths={};seen=set()
    tw,th=(finite(d['resolution'][k]) for k in ('width','height'))
    assert len(d['textures'])==1 and tw>0 and th>0
    texture=d['textures'][0];assert abs(tw-finite(texture.get('uv_width',tw)))<1e-4 and abs(th-finite(texture.get('uv_height',th)))<1e-4
    png=base64.b64decode(texture['source'].split(',',1)[1],validate=True)
    assert png[:8]==b'\x89PNG\r\n\x1a\n'
    def visit(entry,chain):
        uid=entry if isinstance(entry,str) else entry['uuid']
        assert uid not in seen,uid;seen.add(uid)
        if uid in group:
            g=group[uid];assert g['name'] not in names;names.add(g['name']);i=len(bones);paths[uid]=i
            bones.append((g['name'],g['origin'],g.get('rotation',[0]*3)))
            for c in entry['children']:visit(c,chain+[i])
        else:
            c=elements[uid]
            if c.get('type')!='cube' or c.get('export') is False:return
            assert chain and c.get('inflate',0)==0 and all(a<=b for a,b in zip(c['from'],c['to'])),c['name']
            uv=[]
            for face in DIRS:
                u=c['faces'][face];assert u.get('texture')==0 and u.get('rotation',0)==0
                a,b,e,h=map(finite,u['uv']);assert 0<=a<=e<=tw+1e-4 and 0<=b<=h<=th+1e-4
                uv.append([a*16/tw,b*16/th,e*16/tw,h*16/th])
            parts.append((c['from'],c['to'],c['origin'],c.get('rotation',[0]*3),chain,uv))
    for e in d['outliner']:visit(e,[])
    assert len(bones)==len(group) and len(parts)==sum(c.get('type')=='cube' and c.get('export') is not False for c in elements.values())
    clips=[]
    for a in d['animations']:
        name=a['name'].split('.')[-1].removesuffix('_v2')
        if kind!='goblin_boss':name=name.removeprefix(kind.removeprefix('goblin_')+'_')
        duration=finite(a['length']);assert duration>0
        tracks=[]
        for uid,anim in a['animators'].items():
            if not anim.get('keyframes'):continue
            assert uid in paths and not anim.get('rotation_global') and not anim.get('quaternion_interpolation')
            grouped={}
            for k in anim['keyframes']:
                ch=('position','rotation','scale').index(k['channel']); t=finite(k['time']);assert -1e-6<=t<=duration+1e-4
                assert len(k['data_points'])==1
                mode={'linear':0,'step':1,'catmullrom':2}[k.get('interpolation','linear')]
                grouped.setdefault(ch,[]).append((t,point(k['data_points'][0]),mode))
            for ch,ks in grouped.items():
                ks.sort(key=lambda k:k[0]);assert all(b[0]>a[0] for a,b in zip(ks,ks[1:])),(kind,name,uid,ch)
                tracks.append((paths[uid],ch,ks))
        clips.append((name,duration,tracks))
    assert len({a[0] for a in clips})==len(clips) and {'idle','walk','run','death'}<={a[0] for a in clips}
    low=[min(c[0][i] for c in parts) for i in range(3)];high=[max(c[1][i] for c in parts) for i in range(3)]
    dims=[math.hypot(high[0]-low[0],high[2]-low[2])/16,(high[1]-low[1])/16]
    return bones,parts,clips,png,dims

RUNTIME=r'''
// Authored models: per-bone animation evaluated once per entity per frame.
use super::{BlockEntityDrawContext,CubeFace,EntityRenderState,EntityRenderer};
struct Bone {pivot:[f32;3],rot:[f32;3]}
struct Part {lo:[f32;3],hi:[f32;3],pivot:[f32;3],rot:[f32;3],chain:&'static [usize],uv:[[f32;4];6]}
struct Key {t:f32,v:[f32;3],mode:u8}
struct Track {bone:usize,ch:usize,keys:&'static [Key]}
struct Clip {name:&'static str,len:f32,tracks:&'static [Track]}
struct Rig {bones:&'static [Bone],parts:&'static [Part],clips:&'static [Clip],tex:&'static str,dims:[f32;2]}
fn sample(k:&[Key],t:f32)->[f32;3] {
    let i=k.partition_point(|x|x.t<=t);
    if i==0{return k[0].v;} if i==k.len(){return k[i-1].v;}
    let a=&k[i-1];let b=&k[i];if a.mode==1{return a.v;}
    let u=((t-a.t)/(b.t-a.t)).clamp(0.0,1.0);
    if a.mode==2 || b.mode==2 {
        // Numeric Catmull-Rom approximation; editor/engine comparison required.
        let p=k[i.saturating_sub(2)].v;let n=k[(i+1).min(k.len()-1)].v;
        return std::array::from_fn(|j|0.5*(2.0*a.v[j]+(-p[j]+b.v[j])*u+(2.0*p[j]-5.0*a.v[j]+4.0*b.v[j]-n[j])*u*u+(-p[j]+3.0*a.v[j]-3.0*b.v[j]+n[j])*u*u*u));
    }
    std::array::from_fn(|j|a.v[j]+(b.v[j]-a.v[j])*u)
}
fn rotation(draw:&mut dyn BlockEntityDrawContext,r:[f32;3],p:[f32;3]) {
    for (axis,angle) in [([0.,0.,1.],r[2]),([0.,1.,0.],r[1]),([1.,0.,0.],r[0])] {
        if angle.abs()>0.0001 {draw.rotate_about(axis,angle,p);}
    }
}
fn desired_height(id:&str)->f32{if id=="rustpg:goblin_boss"{2.65}else{0.90}}
fn model_scale(id:&str,r:&Rig)->f32 {desired_height(id)/r.dims[1].max(0.01)}
pub(super) fn dimensions(id:&str)->Option<[f32;2]>{
    let r=rig(id)?;let z=model_scale(id,r);Some([r.dims[0]*z,r.dims[1]*z])
}
pub(super) struct Goblin(pub(super) &'static str);
impl EntityRenderer for Goblin {
    fn submit(&self,e:&EntityRenderState,draw:&mut dyn BlockEntityDrawContext)->Result<(),String>{
        let Some(l)=e.living.as_ref() else{return Ok(())};let Some(r)=rig(self.0) else{return Ok(())};
        // The imported BBModel already defines the goblin's adult proportions.
        // Do not inherit a random zombie baby's second 0.5x visual shrink.
        let scale=model_scale(self.0,r);
        if !scale.is_finite()||scale<=0.0{return Ok(());}
        let ground=r.parts.iter().map(|p|p.lo[1]).fold(f32::INFINITY,f32::min);
        let speed=l.walk_animation_speed.clamp(0.0,1.0);
        let name=if l.death_time>0.0{"death"}else if speed>0.65{"run"}else if speed>0.025{"walk"}else{"idle"};
        let Some(c)=r.clips.iter().find(|c|c.name==name).or_else(||r.clips.iter().find(|c|c.name=="idle"))else{return Err("missing goblin idle clip".into())};
        let t=if name=="death"{(l.death_time/20.0).clamp(0.0,1.0)*c.len}
            else if name=="idle"{(e.age/20.0).rem_euclid(c.len)}
            else{(l.walk_animation_pos*0.10603*c.len).rem_euclid(c.len)};
        let mut pose=vec![[[0.0;3],[0.0;3],[1.0;3]];r.bones.len()];
        for tr in c.tracks {pose[tr.bone][tr.ch]=sample(tr.keys,t);}
        draw.push();draw.rotate_about([0.,1.,0.],180.0-l.body_rot,[0.;3]);draw.scale([scale;3]);
        if ground.is_finite(){draw.translate([0.0,-ground/16.0,0.0]);}
        for part in r.parts {
            draw.push();
            for &index in part.chain {
                let bone=&r.bones[index];let [pos,rot,size]=pose[index];let p=bone.pivot.map(|x|x/16.0);
                draw.translate(pos.map(|x|x/16.0));rotation(draw,std::array::from_fn(|i|bone.rot[i]+rot[i]),p);
                if size.iter().any(|v|(v-1.0).abs()>0.0001) {draw.translate(p);draw.scale(size);draw.translate(p.map(|v|-v));}
            }
            rotation(draw,part.rot,part.pivot.map(|x|x/16.0));
            let faces=std::array::from_fn(|i|Some(CubeFace::new(r.tex).with_uv(part.uv[i])));
            // The Preview SDK returns false for a missing atlas sprite. All six
            // faces use the same sprite: draw a visible diagnostic material instead
            // of silently emitting no geometry (especially important for bosses).
            if !draw.cube(part.lo,part.hi,faces,0xFFFFFF) {
                let fallback=std::array::from_fn(|_|Some(CubeFace::new("minecraft:block/stone")));
                let _=draw.cube(part.lo,part.hi,fallback,0xC49B72);
            }
            draw.pop();
        }
        draw.pop();Ok(())
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn authored_sizes_are_in_blocks() {
        for id in ["rustpg:goblin_knife","rustpg:goblin_spear","rustpg:goblin_archer","rustpg:goblin_boss"] {
            let d=dimensions(id).unwrap();assert!((d[1]-desired_height(id)).abs()<0.0001);
        }
    }
}
'''
def rust_model(kind,bones,parts,clips,dims):
    p=kind.upper();lines=[f'const {p}_B:&[Bone]=&[']
    for name,pivot,rot in bones:lines.append(f'Bone{{pivot:{vec(pivot)},rot:{vec(rot)}}},')
    lines.append('];');lines.append(f'const {p}_P:&[Part]=&[')
    for lo,hi,pivot,rot,chain,uv in parts:lines.append(f'Part{{lo:{vec(lo)},hi:{vec(hi)},pivot:{vec(pivot)},rot:{vec(rot)},chain:&{chain},uv:[{",".join(vec(x) for x in uv)}]}},')
    lines.append('];');lines.append(f'const {p}_C:&[Clip]=&[')
    for name,duration,tracks in clips:
        lines.append(f'Clip{{name:{q(name)},len:{f(duration)},tracks:&[')
        for bone,ch,keys in tracks:
            lines.append(f'Track{{bone:{bone},ch:{ch},keys:&[')
            for t,v,mode in keys:lines.append(f'Key{{t:{f(t)},v:{vec(v)},mode:{mode}}},')
            lines.append(']},')
        lines.append(']},')
    lines.append('];');lines.append(f'const {p}:Rig=Rig{{bones:{p}_B,parts:{p}_P,clips:{p}_C,tex:"rustpg:entity/{kind}_authored",dims:{vec(dims)}}};')
    return '\n'.join(lines)

def replace_once(text,a,b):
    if text.count(a)!=1:raise ValueError('integration anchor changed: '+a[:100])
    return text.replace(a,b,1)

def apply(stage,archive,validate_only=False):
    stage=Path(stage);archive=Path(archive);b=archive.read_bytes()
    if archive.suffix=='.b64':b=base64.b64decode(b,validate=False)
    assert hashlib.sha256(b).hexdigest()==SHA256,'wrong asset archive revision'
    report={};code=[RUNTIME]
    with zipfile.ZipFile(io.BytesIO(b)) as z:
        assert set(z.namelist())=={v[0] for v in ROSTER.values()}
        for kind,(filename,template,hp,speed,damage,shadow) in ROSTER.items():
            d=json.loads(z.read(filename));bones,parts,clips,png,dims=model(d,kind)
            code.append(rust_model(kind,bones,parts,clips,dims))
            report[kind]={'bones':len(bones),'cubes':len(parts),'clips':{name:sum(len(ks) for _,_,ks in tracks) for name,_,tracks in clips},'texture_sha256':hashlib.sha256(png).hexdigest(),'dimensions':dims}
            if validate_only:continue
            tex=stage/'assets/rustpg/textures/entity';tex.mkdir(parents=True,exist_ok=True);(tex/f'{kind}_authored.png').write_bytes(png)
            data=stage/'data/rustpg/sunrise/entity_type';data.mkdir(parents=True,exist_ok=True)
            attr={'minecraft:max_health':hp,'minecraft:movement_speed':speed}
            if damage is not None:attr['minecraft:attack_damage']=damage
            (data/f'{kind}.json').write_text(json.dumps({'template':template,'shadow_radius':shadow,'shadow_strength':0.65,'attributes':attr},indent=2))
            loot=stage/'data/rustpg/loot_table/entities';loot.mkdir(parents=True,exist_ok=True)
            (loot/f'{kind}.json').write_text(json.dumps({'type':'minecraft:entity','pools':[{'rolls':1,'entries':[{'type':'minecraft:item','name':'minecraft:iron_nugget'}]}]}))
            names=stage/'assets/rustpg/lang';names.mkdir(parents=True,exist_ok=True)
        if not validate_only:
            for lang,labels in [('pt_br',['Goblin de Adaga','Goblin Lanceiro','Goblin Arqueiro','Chefe Goblin']),('en_us',['Knife Goblin','Spear Goblin','Goblin Archer','Goblin Chief'])]:
                p=names/f'{lang}.json';data=json.loads(p.read_text()) if p.exists() else {}
                for kind,label in zip(ROSTER,labels):data[f'entity.rustpg.{kind}']=label
                p.write_text(json.dumps(data,ensure_ascii=False,indent=2))
    code.append('fn rig(id:&str)->Option<&\'static Rig>{Some(match id{'+''.join(f'"rustpg:{k}"=>&{k.upper()},' for k in ROSTER)+'_=>return None,})}')
    if not validate_only:
        (stage/'src/rustpg_goblin_native.rs').write_text('\n'.join(code))
        p=stage/'src/rustpg_native_entities.rs';s=p.read_text()
        s=replace_once(s,'mod roster { include!("rustpg_creatures_generated.rs"); }','mod roster { include!("rustpg_creatures_generated.rs"); }\nmod goblin { include!("rustpg_goblin_native.rs"); }')
        s=replace_once(s,'pub fn preview_dimensions(kind:&str)->Option<[f32;2]> {','pub fn preview_dimensions(kind:&str)->Option<[f32;2]> {\n    if let Some(d)=goblin::dimensions(kind){return Some(d);}')
        s=replace_once(s,'pub fn register_renderers(event:&mut mod_api::RegisterEntityRenderersEvent<\'_>){','pub fn register_renderers(event:&mut mod_api::RegisterEntityRenderersEvent<\'_>){\n    for kind in ['+','.join(q('rustpg:'+k) for k in ROSTER)+']{event.renderers.register(kind.into(),Arc::new(goblin::Goblin(kind)));}')
        # Include the new family in the shared native creature catalogue, while
        # keeping the original renderer from overriding the authored renderer.
        ids=','.join(q('rustpg:'+k) for k in ROSTER)
        s=replace_once(s,'    DUSK_REVENANT,STORM_GRYPHON,RUNE_COLOSSUS,EMBER_IMP,ABYSS_WEAVER\n];','    DUSK_REVENANT,STORM_GRYPHON,RUNE_COLOSSUS,EMBER_IMP,ABYSS_WEAVER,\n    '+ids+'\n];')
        s=replace_once(s,'    for &kind in CREATURES{\n        event.renderers.register','    for &kind in CREATURES{\n        if goblin::dimensions(kind).is_some(){continue;}\n        event.renderers.register')
        s=replace_once(s,'fn roster_has_real_bones_and_cubes(){\n        for &kind in CREATURES{\n            let r=rig(kind).unwrap();','fn roster_has_real_bones_and_cubes(){\n        for &kind in CREATURES{\n            if goblin::dimensions(kind).is_some(){continue;}\n            let r=rig(kind).unwrap();')
        p.write_text(s)
        p=stage/'src/rustpg_scaling.rs';s=p.read_text()
        s=replace_once(s,'    match kind {\n        "rustpg:ashen_sentinel"','    match kind {\n        "rustpg:goblin_boss"=>return Some(Tier::Boss),\n        "rustpg:goblin_knife"|"rustpg:goblin_spear"|"rustpg:goblin_archer"=>return Some(Tier::Normal),\n        "rustpg:ashen_sentinel"')
        s=replace_once(s,'pub fn scales(kind: &str) -> bool {','pub fn scales(kind: &str) -> bool {\n    if matches!(kind,'+'|'.join(q('rustpg:'+k) for k in ROSTER)+'){return true;}')
        s=replace_once(s,'pub fn base_health(kind: &str) -> f64 {','pub fn base_health(kind: &str) -> f64 {\n'+''.join(f'    if kind=="rustpg:{k}"{{return {float(v[2])}}}\\n' for k,v in ROSTER.items()).replace('\\n','\n'))
        s+='''
#[cfg(test)]
mod goblin_scaling_tests {
    use super::*;
    #[test] fn family_is_scaled_with_authored_base_health() {
        for (id,hp,tier) in [
            ("rustpg:goblin_knife",32.0,Tier::Normal),
            ("rustpg:goblin_spear",44.0,Tier::Normal),
            ("rustpg:goblin_archer",28.0,Tier::Normal),
            ("rustpg:goblin_boss",220.0,Tier::Boss),
        ] {
            assert!(scales(id));assert_eq!(base_health(id),hp);assert_eq!(boss_tier(id),Some(tier));
        }
    }
}
'''
        p.write_text(s)
        (stage/'goblin_import_report.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2));return report

if __name__=='__main__':
    ap=argparse.ArgumentParser();ap.add_argument('--archive',type=Path,required=True);ap.add_argument('--stage',type=Path,default=Path('.'));ap.add_argument('--validate-only',action='store_true');a=ap.parse_args();apply(a.stage,a.archive,a.validate_only)
