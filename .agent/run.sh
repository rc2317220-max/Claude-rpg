#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
mkdir -p .agent/artifacts/avatar
export AVATAR_SRC="$PWD/Minecraft Avatar Design Turnaround.glb"
export AVATAR_OUT="$PWD/.agent/artifacts/avatar"
python3 tools/ai_doctor.py || true
sha256sum "$AVATAR_SRC" | tee "$AVATAR_OUT/source-sha256.txt"
python3 - <<'PY'
import os,struct,json,pathlib
p=pathlib.Path(os.environ['AVATAR_SRC'])
with p.open('rb') as f:
    magic,version,length=struct.unpack('<4sII',f.read(12))
    n,t=struct.unpack('<I4s',f.read(8))
    assert magic==b'glTF' and t==b'JSON'
    d=json.loads(f.read(n))
report={k:d.get(k) for k in ['asset','extensionsUsed','extensionsRequired','scenes','nodes','meshes','materials','images','accessors']}
pathlib.Path(os.environ['AVATAR_OUT'],'gltf-inventory.json').write_text(json.dumps(report,indent=2))
print('GLB',version,length,'extensions',d.get('extensionsUsed'),'meshes',len(d.get('meshes',[])))
PY
sudo apt-get update -qq
sudo apt-get install -y -qq blender python3-numpy python3-scipy python3-pil
blender -b --python-exit-code 1 --python-expr 'import numpy; print(numpy.__version__)'
blender --version
# Normalize compressed geometry and WebP textures before Blender import.
mkdir -p /tmp/avatar-decode; cd /tmp/avatar-decode
npm install --no-audit --no-fund @gltf-transform/core @gltf-transform/extensions meshoptimizer draco3dgltf sharp
cat > decode.mjs <<'JS'
import {NodeIO} from '@gltf-transform/core';
import {ALL_EXTENSIONS} from '@gltf-transform/extensions';
import {MeshoptDecoder, MeshoptEncoder} from 'meshoptimizer';
import draco from 'draco3dgltf';
import sharp from 'sharp';
await MeshoptDecoder.ready; await MeshoptEncoder.ready;
const io=new NodeIO().registerExtensions(ALL_EXTENSIONS).registerDependencies({
  'meshopt.decoder':MeshoptDecoder,'meshopt.encoder':MeshoptEncoder,
  'draco3d.decoder':await draco.createDecoderModule(),
  'draco3d.encoder':await draco.createEncoderModule()
});
const doc=await io.read(process.env.AVATAR_SRC);
for(const tex of doc.getRoot().listTextures()){
  if(tex.getMimeType()==='image/webp'){
    tex.setImage(new Uint8Array(await sharp(tex.getImage()).png().toBuffer())).setMimeType('image/png').setURI('');
  }
}
for(const ext of doc.getRoot().listExtensionsUsed()) if(['EXT_meshopt_compression','KHR_draco_mesh_compression','EXT_texture_webp'].includes(ext.extensionName)) ext.dispose();
await io.write('/tmp/avatar-decoded.glb',doc);
console.log('Decoded GLB; meshes:',doc.getRoot().listMeshes().length,'textures:',doc.getRoot().listTextures().length);
JS
node decode.mjs
export AVATAR_SRC=/tmp/avatar-decoded.glb
cd "$GITHUB_WORKSPACE"
cat >/tmp/inspect_avatar.py <<'PY'
import bpy, os, json, numpy as np, io, base64, math
from mathutils import Vector
from pathlib import Path
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components
from PIL import Image,ImageDraw
out=Path(os.environ['AVATAR_OUT'])
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=os.environ['AVATAR_SRC'],merge_vertices=True)
o=next(o for o in bpy.context.scene.objects if o.type=='MESH');m=o.data
co=np.empty(len(m.vertices)*3,np.float32);m.vertices.foreach_get('co',co);co=co.reshape(-1,3)
edges=np.empty(len(m.edges)*2,np.int32);m.edges.foreach_get('vertices',edges);edges=edges.reshape(-1,2)
tol=max(float(np.ptp(co,axis=0).max())*1e-6,1e-9)
_,inv=np.unique(np.rint(co/tol).astype(np.int64),axis=0,return_inverse=True)
ee=inv[edges]; n=int(inv.max())+1
_,lab=connected_components(coo_matrix((np.ones(len(ee),np.uint8),(ee[:,0],ee[:,1])),shape=(n,n)).tocsr(),directed=False)
lab=lab[inv]
m.calc_loop_triangles()
tri=np.empty(len(m.loop_triangles)*3,np.int32);m.loop_triangles.foreach_get('vertices',tri);tri=tri.reshape(-1,3)
loops=np.empty(len(m.loop_triangles)*3,np.int32);m.loop_triangles.foreach_get('loops',loops);loops=loops.reshape(-1,3)
uv=np.empty(len(m.loops)*2,np.float32);m.uv_layers.active.data.foreach_get('uv',uv);uv=uv.reshape(-1,2)
material=m.materials[0]
def extract(k,name,budget,extra=None):
    mask=lab[tri[:,0]]==k
    if extra is not None:mask &= extra
    t=tri[mask]; l=loops[mask]
    assert len(t)>0,name
    # Preserve UVs per corner while welding coincident geometric points.
    u,rev=np.unique(inv[t],return_inverse=True)
    _,first=np.unique(inv,return_index=True)
    p=co[first[u]]
    tt=rev.reshape(-1,3)
    keep=(tt[:,0]!=tt[:,1])&(tt[:,1]!=tt[:,2])&(tt[:,2]!=tt[:,0])
    tt=tt[keep]; l=l[keep]
    mesh=bpy.data.meshes.new(name)
    mesh.from_pydata(p.tolist(),[],tt.tolist());mesh.update()
    mesh.materials.append(material)
    layer=mesh.uv_layers.new(name='UVMap')
    layer.data.foreach_set('uv',uv[l].reshape(-1))
    obj=bpy.data.objects.new(name,mesh);bpy.context.scene.collection.objects.link(obj)
    obj.matrix_world=o.matrix_world.copy()
    bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
    if len(mesh.polygons)>budget:
        mod=obj.modifiers.new('Keep detailed silhouette; reduce excessive reconstruction triangles','DECIMATE')
        mod.ratio=budget/len(mesh.polygons)
        bpy.ops.object.modifier_apply(modifier=mod.name)
    print('EXTRACTED',name,'faces',len(obj.data.polygons),flush=True)
    return obj
# Real donor geometry: keep the front-facing head, not the reconstructed duplicate backs.
cent=co[tri].mean(axis=1)
def box(k):
    a=co[lab==k];return a.min(axis=0),a.max(axis=0)
hlo,hhi=box(4); hlo0=hlo.copy();hspan=hhi-hlo
head_cut=hlo[2]+.108*hspan[2]
head=extract(4,'Head_Donor',12000,cent[:,2]>=head_cut)
flo,fhi=box(0); span=fhi-flo; fc=(flo+fhi)/2
nx=(cent[:,0]-fc[0])/span[0]; nz=(cent[:,2]-flo[2])/span[2]
band=(nz>.13)&(nz<.825)
wingL=extract(0,'Wing_Left_Donor',5500,band&(nx<-.25))
wingR=extract(0,'Wing_Right_Donor',5500,band&(nx>.25))
# Keep the donor texture; use the actual base-color image rather than the PBR data map.
shader=next(n for n in material.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
links=shader.inputs['Base Color'].links
img=links[0].from_node.image if links and links[0].from_node.type=='TEX_IMAGE' else next(n.image for n in material.node_tree.nodes if n.type=='TEX_IMAGE')
tex=out/'donor_basecolor.png';img.filepath_raw=str(tex);img.file_format='PNG';img.save()
im=Image.open(tex).convert('RGBA')
if max(im.size)>4096:im.thumbnail((4096,4096),Image.Resampling.LANCZOS)
im.save(tex)
W,H=im.size
print('BASE_COLOR_TEXTURE',W,H,flush=True)
# Fit geometry to Minecraft Man coordinate system, with +Y up and +Z front in BBModel.
# Head is cosmetic overlay, not a replacement for the vanilla collision box.
hs=15.6/hspan[0]; fs=24/(span[2]*.64)
hcenter=(hlo+hhi)/2
def map_head(v):
    return [(v.x-hcenter[0])*hs,24+(v.z-head_cut)*hs,-(v.y-hcenter[1])*hs]
def map_wing(v):
    x=(v.x-fc[0])*fs
    y=4.8+((v.z-flo[2])*fs-5.1)*(23.5/26.0)
    # Only the inner upper wing is pulled toward the back mount; lower tips retain silhouette.
    near=max(0.,min(1.,(13.5-abs(x))/7.0))
    upper=max(0.,min(1.,(y-13.)/9.))
    x-=math.copysign(3.4*near*upper,x)
    z=-4.7-(v.y-fc[1])*fs*.46
    z+=.65*near*upper
    return [x,y,min(z,-1.85)]
import uuid
def uid():return str(uuid.uuid4())
texture_id=uid()
texture={'name':tex.name,'path':tex.name,'relative_path':tex.name,'id':'0','uuid':texture_id,'width':W,'height':H,'uv_width':W,'uv_height':H,'mode':'bitmap','saved':True,'visible':True,'source':'data:image/png;base64,'+base64.b64encode(tex.read_bytes()).decode()}
groups=[];elements=[];report=[]
for obj,fn,pivot,name in [(head,map_head,[0,24,0],'Head'),(wingL,map_wing,[-3,21,-3],'Wing_L'),(wingR,map_wing,[3,21,-3],'Wing_R')]:
    mesh=obj.data
    keys=[f'v{i:x}' for i in range(len(mesh.vertices))]
    xyz=[fn(v.co) for v in mesh.vertices]
    assert all(np.isfinite(v).all() for v in xyz)
    vertices={k:[round(float(v[j]-pivot[j]),5) for j in range(3)] for k,v in zip(keys,xyz)}
    faces={}
    uvdata=mesh.uv_layers.active.data
    for i,poly in enumerate(mesh.polygons):
        vs=[keys[k] for k in poly.vertices]
        if len(set(vs))<3:continue
        faces[f'f{i:x}']={'vertices':vs,'uv':{k:[round(float(uvdata[l].uv.x)*W,4),round((1-float(uvdata[l].uv.y))*H,4)] for k,l in zip(vs,poly.loop_indices)},'texture':0}
    eid=uid();gid=uid()
    elements.append({'name':obj.name,'uuid':eid,'type':'mesh','origin':pivot,'rotation':[0,0,0],'vertices':vertices,'faces':faces,'color':0,'visibility':True,'export':True,'autouv':0,'locked':False})
    groups.append({'name':name,'uuid':gid,'origin':pivot,'rotation':[0,0,0],'children':[eid],'isOpen':True,'export':True,'visibility':True})
    report.append({'name':obj.name,'faces':len(faces),'vertices':len(vertices),'bounds':[np.min(xyz,axis=0).tolist(),np.max(xyz,axis=0).tolist()]})
    # Fit the actual Blender object too, with BBModel coordinate basis converted back to Blender.
    for v,pos in zip(mesh.vertices,xyz):v.co=(pos[0],-pos[2],pos[1])
    obj.matrix_world.identity();mesh.update()
model={'meta':{'format_version':'4.10','model_format':'free','box_uv':False},'name':'Tripo donor accessories - V18.1 candidate','model_identifier':'tripo_donor_v18','resolution':{'width':W,'height':H},'elements':elements,'outliner':groups,'textures':[texture]}
(out/'Tripo_donor_V18_1.bbmodel').write_text(json.dumps(model,separators=(',',':'),default=lambda v:v.item() if isinstance(v,np.generic) else str(v)))
(out/'donor-fit-report.json').write_text(json.dumps(report,indent=2))
print('DONOR_FIT',json.dumps(report),flush=True)
parts=[head,wingL,wingR]
bpy.data.objects.remove(o,do_unlink=True)
# Standard Man sizing guide, only for preview; not exported to the donor BBModel.
guide=[]
gm=bpy.data.materials.new('Sizing guide neutral');gm.diffuse_color=(.3,.3,.33,1);gm.use_nodes=True;gm.node_tree.nodes.get('Principled BSDF').inputs['Base Color'].default_value=(.3,.3,.33,1)
for name,x,y,z,dx,dy,dz in [('Head8',0,28,0,8,8,8),('Torso8',0,18,0,8,12,4),('ArmL',-6,18,0,4,12,4),('ArmR',6,18,0,4,12,4),('LegL',-2,6,0,4,12,4),('LegR',2,6,0,4,12,4)]:
    bpy.ops.mesh.primitive_cube_add(size=1,location=(x,-z,y));g=bpy.context.object;g.name=name;g.dimensions=(dx,dz,dy);g.data.materials.append(gm);guide.append(g)
# Keep the guide head hidden in the render; it is only a sizing reference.
guide[0].hide_render=True
bpy.ops.file.pack_all()
scene=bpy.context.scene
scene.render.engine='CYCLES';scene.cycles.samples=12;scene.cycles.use_denoising=True
scene.render.resolution_x=720;scene.render.resolution_y=720;scene.render.resolution_percentage=100
scene.world=bpy.data.worlds.new('Studio');scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.22,.22,.22,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.7
scene.view_settings.view_transform='Standard'
c=Vector((0,0,19));span=42
def aim(o,t):o.rotation_euler=(t-o.location).to_track_quat('-Z','Y').to_euler()
for name,d,power in [('Key',(-3,-4,5),1000),('Fill',(4,-2,2),500),('Rim',(0,4,5),800)]:
    data=bpy.data.lights.new(name,'AREA');data.shape='DISK';l=bpy.data.objects.new(name,data);scene.collection.objects.link(l);l.location=c+Vector(d)*span;data.energy=power*span**2/9;data.size=span*1.3;aim(l,c)
cam=bpy.data.cameras.new('Camera');camera=bpy.data.objects.new('Camera',cam);scene.collection.objects.link(camera);scene.camera=camera;cam.type='ORTHO';cam.ortho_scale=43;cam.clip_end=500
sheet=Image.new('RGB',(1200,840),(28,28,28));draw=ImageDraw.Draw(sheet)
for i,(name,d) in enumerate([('front',(0,-1,0)),('back',(0,1,0)),('side',(1,0,0)),('front3q',(1,-2,.5)),('top3q',(1,-1,2)),('rear3q',(-1,2,.5))]):
    camera.location=c+Vector(d).normalized()*120;aim(camera,c)
    path=out/f'fit_{name}.png';scene.render.filepath=str(path);bpy.ops.render.render(write_still=True)
    im=Image.open(path).convert('RGB');im.thumbnail((400,400));x=i%3*400;y=i//3*420;sheet.paste(im,(x,y));draw.text((x+5,y+401),name,fill='white')
sheet.save(out/'donor-fit-contact-sheet.jpg',quality=90)
buf=io.BytesIO();sheet.save(buf,format='JPEG',quality=65);data=base64.b64encode(buf.getvalue()).decode()
for i in range(0,len(data),3000):print('FIT_IMAGE',i//3000,data[i:i+3000],flush=True)
bpy.ops.wm.save_as_mainfile(filepath=str(out/'Tripo_donor_V18.blend'),compress=True)
print('DONOR COMPLETE',flush=True)
PY
blender -b -t 4 --python-exit-code 1 -P /tmp/inspect_avatar.py
# Reassemble the user's exact split ZIP from main without modifying main.
git fetch --depth=1 origin main
mkdir -p /tmp/v17-base
git show FETCH_HEAD:Avatar_Toon_V17.zip.001 > /tmp/v17-base/Avatar_Toon_V17.zip.001
git show FETCH_HEAD:Avatar_Toon_V17.zip.002 > /tmp/v17-base/Avatar_Toon_V17.zip.002
cat /tmp/v17-base/Avatar_Toon_V17.zip.00{1,2} > /tmp/v17-base/Avatar_Toon_V17.zip
python3 - <<'PY'
import zipfile,pathlib
root=pathlib.Path('/tmp/v17-base/extracted')
with zipfile.ZipFile('/tmp/v17-base/Avatar_Toon_V17.zip') as z:
    assert z.testzip() is None, 'ZIP CRC validation failed'
    for n in z.namelist():
        assert (root/n).resolve().is_relative_to(root.resolve()),n
    z.extractall(root)
    print('V17 ZIP VERIFIED',z.namelist(),flush=True)
PY
BASE="$(find /tmp/v17-base/extracted -name '*.bbmodel' -type f -print -quit)"
test -n "$BASE"
export AVATAR_BASE="$BASE"
python3 tools/avatar_merge_donor.py "$BASE" "$AVATAR_OUT/Tripo_donor_V18_1.bbmodel" "$AVATAR_OUT/Avatar_Toon_V18_Hybrid.bbmodel"
cp tools/avatar_merge_donor.py "$AVATAR_OUT/"
blender -b -t 4 --python-exit-code 1 -P tools/avatar_hybrid_render.py
