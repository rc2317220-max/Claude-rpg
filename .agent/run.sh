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
import bpy, os, json, numpy as np, io, base64
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
def extract(k,name,budget):
    t=tri[lab[tri[:,0]]==k]; l=loops[lab[tri[:,0]]==k]
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
spec=[(4,'Head_A',14000),(6,'Head_B',14000),(8,'Head_C',14000),(3,'Accessory_sheet',18000),(0,'Front_avatar',30000),(7,'Back_avatar',30000),(11,'Threequarter_avatar',30000),(13,'Cape_candidate',10000),(5,'Side_avatar',18000),(9,'Lower_accessory',10000)]
parts=[extract(*x) for x in spec]
bpy.data.objects.remove(o,do_unlink=True)
# Separate candidate scene, preserving UV and shared texture.
bpy.ops.file.pack_all()
bpy.ops.wm.save_as_mainfile(filepath=str(out/'avatar-candidate-parts.blend'),compress=True)
scene=bpy.context.scene
scene.render.engine='CYCLES';scene.cycles.samples=8;scene.cycles.use_denoising=True
scene.render.resolution_x=512;scene.render.resolution_y=512;scene.render.resolution_percentage=100
scene.world=bpy.data.worlds.new('Studio');scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.25,.25,.25,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.7
scene.view_settings.view_transform='Standard'
def aim(o,t):o.rotation_euler=(t-o.location).to_track_quat('-Z','Y').to_euler()
lights=[]
for name,d,power in [('Key',(-3,-4,5),1000),('Fill',(4,-2,2),500),('Rim',(0,4,5),800)]:
    data=bpy.data.lights.new(name,'AREA');data.shape='DISK';l=bpy.data.objects.new(name,data);scene.collection.objects.link(l);lights.append((l,d,power))
cam=bpy.data.cameras.new('Camera');camera=bpy.data.objects.new('Camera',cam);scene.collection.objects.link(camera);scene.camera=camera;cam.type='ORTHO'
sheet=Image.new('RGB',(1200,len(parts)*320),(28,28,28));draw=ImageDraw.Draw(sheet)
for i,obj in enumerate(parts):
    for p in parts:p.hide_render=p!=obj
    points=[obj.matrix_world@Vector(v) for v in obj.bound_box]
    lo=Vector([min(v[j] for v in points) for j in range(3)]);hi=Vector([max(v[j] for v in points) for j in range(3)])
    c=(lo+hi)/2;span=max(hi-lo)
    for l,d,power in lights:
        l.location=c+Vector(d)*span;l.data.energy=power*span**2/9;l.data.size=span*1.3;aim(l,c)
    cam.ortho_scale=span*1.15;cam.clip_start=max(span/10000,.00001);cam.clip_end=span*100
    for j,(name,d) in enumerate([('front',(0,-1,0)),('back',(0,1,0)),('side',(1,0,0)),('top3q',(1,-1,2))]):
        camera.location=c+Vector(d).normalized()*span*4;aim(camera,c)
        path=out/f'{obj.name}_{name}.png';scene.render.filepath=str(path);bpy.ops.render.render(write_still=True)
        im=Image.open(path).convert('RGB');im.thumbnail((300,296));sheet.paste(im,(j*300,i*320));draw.text((j*300+5,i*320+298),obj.name+' '+name,fill='white')
    obj.hide_render=False
    print('PART VIEWS',obj.name,flush=True)
sheet.save(out/'candidate-contact-sheet.jpg',quality=90)
# Send smaller separate contact sheets in log for actual visual review.
for i in range(len(parts)):
    buf=io.BytesIO();sheet.crop((0,i*320,1200,(i+1)*320)).save(buf,format='JPEG',quality=65)
    data=base64.b64encode(buf.getvalue()).decode()
    for j in range(0,len(data),3000):print('PART_IMAGE',i,j//3000,data[j:j+3000],flush=True)
print('CANDIDATES COMPLETE',flush=True)
PY
blender -b -t 4 --python-exit-code 1 -P /tmp/inspect_avatar.py
