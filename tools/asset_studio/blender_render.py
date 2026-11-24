"""CPU Blender render of explicit quad geometry. Run with blender -b -t 2 -P ... -- mesh.json texture.png output.png."""
import bpy,json,sys,math,os
from mathutils import Vector
from pathlib import Path
meshfile,texture,output=sys.argv[sys.argv.index("--")+1:]
data=json.loads(Path(meshfile).read_text())
bpy.ops.wm.read_factory_settings(use_empty=True)
verts=[];faces=[];uvs=[]
for f in data["faces"]:
    start=len(verts);verts.extend((x,-z,y) for x,y,z in f["vertices"])
    faces.append(tuple(range(start,start+4)));uvs.extend((u,1-v) for u,v in f["uv"])
mesh=bpy.data.meshes.new("AssetMesh");mesh.from_pydata(verts,[],faces);mesh.update()
obj=bpy.data.objects.new("Asset",mesh);bpy.context.collection.objects.link(obj)
uv=mesh.uv_layers.new(name="UV")
for i,v in enumerate(uvs):uv.data[i].uv=v
material=bpy.data.materials.new("Pixel texture");material.use_nodes=True
nodes=material.node_tree.nodes;links=material.node_tree.links
tex=nodes.new("ShaderNodeTexImage");tex.image=bpy.data.images.load(str(Path(texture).resolve()));tex.interpolation="Closest"
bsdf=nodes.get("Principled BSDF");links.new(tex.outputs["Color"],bsdf.inputs["Base Color"]);links.new(tex.outputs["Alpha"],bsdf.inputs["Alpha"]);bsdf.inputs["Roughness"].default_value=.85
obj.data.materials.append(material)
lo=Vector(tuple(min(v[i] for v in verts) for i in range(3)));hi=Vector(tuple(max(v[i] for v in verts) for i in range(3)));center=(hi+lo)/2
span=max((hi-lo).length,1)
scene=bpy.context.scene;scene.render.engine="CYCLES";scene.cycles.device="CPU";scene.cycles.samples=12
scene.world=bpy.data.worlds.new("Studio");scene.world.use_nodes=True;scene.world.node_tree.nodes["Background"].inputs[0].default_value=(.13,.16,.22,1);scene.world.node_tree.nodes["Background"].inputs[1].default_value=.6
bpy.ops.object.camera_add(location=center+Vector(tuple(map(float,os.getenv('RUSTPG_CAMERA','1.3,-1.8,.8').split(','))))*span)
cam=bpy.context.object;cam.rotation_euler=(center-cam.location).to_track_quat("-Z","Y").to_euler();cam.data.type="ORTHO";cam.data.ortho_scale=span*1.15;scene.camera=cam
for position,energy in [((-.7,-1.2,1.8),55),((1.5,.8,1.1),35)]:
    bpy.ops.object.light_add(type="AREA",location=center+Vector(position)*span)
    light=bpy.context.object;light.rotation_euler=(center-light.location).to_track_quat("-Z","Y").to_euler();light.data.energy=energy*span*span;light.data.shape="DISK";light.data.size=span
scene.render.resolution_x=512;scene.render.resolution_y=512;scene.render.resolution_percentage=100
scene.render.image_settings.file_format="PNG";scene.render.filepath=str(Path(output).resolve())
bpy.ops.render.render(write_still=True)
