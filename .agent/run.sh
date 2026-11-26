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
sudo apt-get install -y -qq blender python3-numpy
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
import bpy, os, json, math
from mathutils import Vector
from pathlib import Path
out=Path(os.environ['AVATAR_OUT'])
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=os.environ['AVATAR_SRC'],merge_vertices=True)
objs=[o for o in bpy.context.scene.objects if o.type=='MESH']
assert objs,'No imported meshes'
def bounds(o):
    p=[o.matrix_world @ Vector(v) for v in o.bound_box]
    return [[round(min(v[i] for v in p),6) for i in range(3)],[round(max(v[i] for v in p),6) for i in range(3)]]
def inventory():
    return [{'name':o.name,'vertices':len(o.data.vertices),'faces':len(o.data.polygons),'bounds':bounds(o),'materials':[m.name if m else None for m in o.data.materials],'uv_layers':[u.name for u in o.data.uv_layers]} for o in bpy.context.scene.objects if o.type=='MESH']
report={'source':inventory(),'blender':bpy.app.version_string}
# Save an untouched, packed, editable import before any separation.
bpy.ops.file.pack_all()
bpy.ops.wm.save_as_mainfile(filepath=str(out/'avatar-import-original.blend'))
# Separate genuine disconnected islands only on the working copy. No arbitrary anatomical cuts.
for o in objs:
    if not o.data.polygons: continue
    bpy.ops.object.select_all(action='DESELECT')
    o.select_set(True); bpy.context.view_layer.objects.active=o
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.separate(type='LOOSE')
    bpy.ops.object.mode_set(mode='OBJECT')
parts=[o for o in bpy.context.scene.objects if o.type=='MESH']
report['loose_parts']=inventory()
(out/'mesh-inventory.json').write_text(json.dumps(report,indent=2))
print('MESH INVENTORY',json.dumps({'source':report['source'],'parts_count':len(parts)}),flush=True)
for x in sorted(report['loose_parts'],key=lambda x:x['faces'],reverse=True)[:40]: print('PART',json.dumps(x),flush=True)
bpy.ops.file.pack_all()
bpy.ops.wm.save_as_mainfile(filepath=str(out/'avatar-separated-candidate.blend'))
# Export individually selectable genuine mesh islands, with original UV/materials.
parts_dir=out/'pieces'; parts_dir.mkdir(exist_ok=True)
for i,o in enumerate(sorted(parts,key=lambda o:len(o.data.polygons),reverse=True)):
    if len(o.data.polygons)<100 or i>=16: continue
    bpy.ops.object.select_all(action='DESELECT')
    o.select_set(True); bpy.context.view_layer.objects.active=o
    bpy.ops.export_scene.gltf(filepath=str(parts_dir/f'part_{i:03d}.glb'),export_format='GLB',use_selection=True)
# Record exact mapping for selecting reusable pieces.
(out/'pieces-map.json').write_text(json.dumps([{'index':i, 'file':f'part_{i:03d}.glb' if len(o.data.polygons)>=100 and i<16 else None,'name':o.name,'faces':len(o.data.polygons),'bounds':bounds(o)} for i,o in enumerate(sorted(parts,key=lambda o:len(o.data.polygons),reverse=True))],indent=2))
# Render the actual imported geometry with consistent orthographic framing.
points=[o.matrix_world @ Vector(v) for o in parts for v in o.bound_box]
lo=Vector([min(v[i] for v in points) for i in range(3)])
hi=Vector([max(v[i] for v in points) for i in range(3)])
center=(lo+hi)/2; span=max(hi-lo)
scene=bpy.context.scene
scene.render.engine='CYCLES'
scene.cycles.device='CPU'
scene.cycles.samples=8
scene.cycles.use_denoising=True
scene.render.resolution_x=640; scene.render.resolution_y=640; scene.render.resolution_percentage=100
scene.world=bpy.data.worlds.new('Neutral studio')
scene.world.use_nodes=True
scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.22,.22,.22,1)
scene.world.node_tree.nodes['Background'].inputs[1].default_value=.55
def aim(o,t): o.rotation_euler=(t-o.location).to_track_quat('-Z','Y').to_euler()
for name,d,power in [('Key',(-3,-4,5),1000),('Fill',(4,-2,2),500),('Rim',(0,4,5),800)]:
    data=bpy.data.lights.new(name,'AREA'); data.energy=power*span**2/9; data.shape='DISK';data.size=span*1.3
    o=bpy.data.objects.new(name,data);scene.collection.objects.link(o);o.location=center+Vector(d)*span;aim(o,center)
cam=bpy.data.cameras.new('Audit Camera');camera=bpy.data.objects.new('Audit Camera',cam);scene.collection.objects.link(camera);scene.camera=camera
cam.type='ORTHO';cam.ortho_scale=span*1.35;cam.clip_end=span*100;cam.clip_start=max(span/10000,.00001)
scene.view_settings.view_transform='Standard'
for name,d in [('front',(0,-1,0)),('back',(0,1,0)),('right',(1,0,0)),('left',(-1,0,0)),('top',(0,-.001,1)),('bottom',(0,-.001,-1)),('front_3q',(1,-2,.5)),('rear_3q',(-1,2,.5)),('upper_3q',(1,-1,2))]:
    camera.location=center+Vector(d).normalized()*span*4
    aim(camera,center)
    scene.render.filepath=str(out/(name+'.png'))
    bpy.ops.render.render(write_still=True)
    print('RENDERED',name,flush=True)
bpy.ops.wm.save_as_mainfile(filepath=str(out/'avatar-audit-scene.blend'))
PY
blender -b -t 4 --python-exit-code 1 -P /tmp/inspect_avatar.py
