#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
OUT="$PWD/.agent/artifacts/portrait-cape"
mkdir -p "$OUT"
export OUT
cat Avatar_Toon_V17.zip.00{1,2} > /tmp/avatar-v17.zip
sudo apt-get update -qq
sudo apt-get install -y -qq python3-pil
/usr/bin/python3 - <<'PY'
import zipfile,json,pathlib,base64,os,math,hashlib
from PIL import Image
from io import BytesIO
out=pathlib.Path(os.environ['OUT'])
root=pathlib.Path('/tmp/portrait-cape')
(root/'src').mkdir(parents=True,exist_ok=True)
with zipfile.ZipFile('/tmp/avatar-v17.zip') as z:
    assert z.testzip() is None
    m=json.loads(z.read(next(n for n in z.namelist() if n.endswith('.bbmodel'))))
g=next(g for g in m['groups'] if g['name']=='Cape')
def walk(a):
    for n in a:
        if isinstance(n,dict):
            yield n
            yield from walk(n.get('children',[]))
node=next(n for n in walk(m['outliner']) if n['uuid']==g['uuid'])
def children(n):
    for c in n.get('children',[]):
        if isinstance(c,str):yield c
        else:yield from children(c)
ids=set(children(node))
elements=[e for e in m['elements'] if e['uuid'] in ids]
assert elements and all(e['name'].startswith('cape_') for e in elements)
print('CAPE_ELEMENTS',[(e['name'],len(e['faces'])) for e in elements],flush=True)
textures=m['textures']; texmap={t['uuid']:i for i,t in enumerate(textures)}
def texindex(f):
    t=f.get('texture')
    if isinstance(t,int):return t
    return texmap[t]
used=sorted({texindex(f) for e in elements for f in e['faces'].values() if f.get('texture') is not None})
def fl(x):
    assert math.isfinite(x)
    return f'{float(x):.7f}f32'
def arr(x):return '['+','.join(fl(v) for v in x)+']'
def rot(v,r):
    x,y,z=v
    for a,t in enumerate(r):
        c,s=math.cos(math.radians(t)),math.sin(math.radians(t))
        if a==0:y,z=y*c-z*s,y*s+z*c
        if a==1:x,z=x*c+z*s,-x*s+z*c
        if a==2:x,y=x*c-y*s,x*s+y*c
    return [x,y,z]
pivot=g['origin']
lines=['use mod_api::{ModelPart,ModelPolygon,ModelVertex,PartPose};','pub fn parts() -> Vec<(&\'static str,ModelPart)> { vec![']
report=[]
for ti in used:
    t=textures[ti]
    raw=base64.b64decode(t['source'].split(',',1)[1])
    im=Image.open(BytesIO(raw)).convert('RGBA')
    w=t.get('uv_width') or t.get('width') or m['resolution']['width']
    h=t.get('uv_height') or t.get('height') or m['resolution']['height']
    assert w>0 and h>0
    # Retain the original picture, downsample atlas only for mobile GPU memory.
    im.thumbnail((2048,2048),Image.Resampling.LANCZOS)
    asset=root/'assets/portrait_cape/textures/entity'/f'cape_{ti}.png'
    asset.parent.mkdir(parents=True,exist_ok=True);im.save(asset)
    lines.append(f'("portrait_cape:entity/cape_{ti}",ModelPart::new(vec![')
    count=0
    for e in elements:
        for f in e['faces'].values():
            if f.get('texture') is None or texindex(f)!=ti:continue
            keys=f['vertices']
            if len(keys)<3:continue
            coords=[]
            for k in keys:
                p=rot(e['vertices'][k],e.get('rotation',[0,0,0]))
                p=[p[j]+e['origin'][j]-pivot[j] for j in range(3)]
                u,v=f['uv'][k];coords.append((p,[u/w,v/h]))
            # Native draw accepts quads. Fan triangulate n-gons, duplicate triangle corner.
            polys=[coords] if len(coords)==4 else [[coords[0],coords[j],coords[j+1],coords[j+1]] for j in range(1,len(coords)-1)]
            for poly in polys:
                a,b,c=[q[0] for q in poly[:3]]
                ab=[b[j]-a[j] for j in range(3)];ac=[c[j]-a[j] for j in range(3)]
                n=[ab[1]*ac[2]-ab[2]*ac[1],ab[2]*ac[0]-ab[0]*ac[2],ab[0]*ac[1]-ab[1]*ac[0]]
                norm=math.sqrt(sum(x*x for x in n))
                if norm<1e-9:continue
                vs=','.join('ModelVertex::new('+arr(p)+','+fl(uv[0])+','+fl(uv[1])+')' for p,uv in poly)
                lines.append('ModelPolygon::new(['+vs+'],'+arr([x/norm for x in n])+'),')
                count+=1
    lines.append('],vec![],PartPose::ZERO)),')
    report.append(dict(texture=t['name'],source_sha256=hashlib.sha256(raw).hexdigest(),output_size=im.size,faces=count))
lines.append('] }')
(root/'src/geometry.rs').write_text('\n'.join(lines))
(out/'cape-extraction.json').write_text(json.dumps({'pivot':pivot,'elements':[e['name'] for e in elements],'textures':report},indent=2))
# Portable cape-only Blockbench source, same geometry and original texture bytes.
m['elements']=elements;m['groups']=[g];m['outliner']=[node];m['name']='Portrait Cape Only'
m['model_identifier']='portrait_cape'
remap={old:new for new,old in enumerate(used)}
for e in elements:
    for f in e['faces'].values():
        if f.get('texture') is not None:f['texture']=remap[texindex(f)]
m['textures']=[textures[i] for i in used]
for i,t in enumerate(m['textures']):t['id']=str(i)
(out/'Portrait_Cape.bbmodel').write_text(json.dumps(m,separators=(',',':')))
print('CAPE_EXTRACTED',report,flush=True)
PY
cat >/tmp/portrait-cape/Cargo.toml <<'EOF'
[package]
name = "portrait-cape"
version = "1.0.0"
edition = "2024"
[dependencies]
sunrise-mod-api = { path = ".sunrise/sdk/api/crates/sunrise-mod-api" }
EOF
cat >/tmp/portrait-cape/sunrise-mod.toml <<'EOF'
id = "portrait_cape"
name = "Portrait Cape"
version = "1.0.0"
description = "Local cosmetic cape only, original portrait and custom geometry."
api = "0.2"
side = "client"
[native]
game = "26.3"
sunrise = "cac7bf64590fb38b4badee90f1f09848ce7f8daa"
EOF
cat >/tmp/portrait-cape/src/lib.rs <<'EOF'
use mod_api::{self, Hooks, ClientHost, RenderPlayerPostEvent, ModelPart};
use std::sync::OnceLock;
mod geometry;
static CAPE:OnceLock<Vec<(&'static str,ModelPart)>>=OnceLock::new();
#[derive(Default)]
struct PortraitCape;
#[mod_api::sunrise_mod]
impl Hooks for PortraitCape {
    fn render_player_post(&self, _host:&dyn ClientHost, event:&mut RenderPlayerPostEvent<'_>) {
        if !event.who.local || event.player.death_time>0 {return;}
        let Some(model)=event.model else {return;};
        let Some(body)=model.part("body") else {return;};
        if !body.visible {return;}
        let motion=event.player.walk_speed.clamp(0.0,1.0);
        let age=event.player.age;
        let swing=0.05+motion*0.35+(age*0.16).sin()*(0.018+motion*0.035);
        for (sprite, original) in CAPE.get_or_init(geometry::parts) {
            let mut part=original.clone();
            part.x=0.0;part.y=24.0;part.z=-2.5;
            part.x_rot=swing;
            let _=event.draw.model_part(&part,sprite,0xFFFFFF);
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn extracted_mesh_valid() {
        let parts=super::geometry::parts();
        assert!(!parts.is_empty());
        for (_,p) in parts {
            assert!(!p.polygons.is_empty());
            for f in p.polygons {
                for v in f.vertices {
                    assert!(v.pos.iter().all(|v|v.is_finite()));
                    assert!(v.u.is_finite()&&v.v.is_finite());
                }
            }
        }
    }
}
EOF
git clone --filter=blob:none --no-checkout https://github.com/starling-client/starling.git "$RUNNER_TEMP/sunrise"
git -C "$RUNNER_TEMP/sunrise" fetch --depth=1 origin cac7bf64590fb38b4badee90f1f09848ce7f8daa
git -C "$RUNNER_TEMP/sunrise" checkout --detach cac7bf64590fb38b4badee90f1f09848ce7f8daa
(cd "$RUNNER_TEMP/sunrise" && bash tools/ci/rustup_install.sh --profile minimal --component rust-src)
cp -a /tmp/portrait-cape /tmp/portrait-cape-check
sed -i "s|.sunrise/sdk/api/crates/sunrise-mod-api|$RUNNER_TEMP/sunrise/crates/sunrise-mod-api|" /tmp/portrait-cape-check/Cargo.toml
cargo check --manifest-path /tmp/portrait-cape-check/Cargo.toml --lib 2>&1 | tee "$OUT/check.log"
cargo test --manifest-path /tmp/portrait-cape-check/Cargo.toml --lib 2>&1 | tee "$OUT/tests.log"
python3 - <<'PY'
import pathlib,zipfile,os
root=pathlib.Path('/tmp/portrait-cape')
out=pathlib.Path(os.environ['OUT'])
with zipfile.ZipFile(out/'Portrait-Cape-1.0.0-Preview4.smod','w',zipfile.ZIP_DEFLATED) as z:
    for p in root.rglob('*'):
        if p.is_file():z.write(p,p.relative_to(root).as_posix())
with zipfile.ZipFile(out/'Portrait-Cape-1.0.0-Preview4.smod') as z:assert z.testzip() is None
print('CAPE_SMOD_PACKAGED')
PY
