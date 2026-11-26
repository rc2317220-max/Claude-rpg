import bpy, os, json, math, base64, io, pathlib
from mathutils import Vector, Euler
from PIL import Image, ImageDraw
out=pathlib.Path(os.environ['AVATAR_OUT'])
basepath=pathlib.Path(os.environ['AVATAR_BASE'])
hybrid=out/'Avatar_Toon_V18_Hybrid.bbmodel'
bpy.ops.wm.read_factory_settings(use_empty=True)
def build(path):
    data=json.loads(path.read_text())
    mats=[]; texdims=[]; texids={}
    for i,t in enumerate(data['textures']):
        source=t.get('source','')
        target=out/('texture_%02d.png'%i)
        if source.startswith('data:'):
            target.write_bytes(base64.b64decode(source.split(',',1)[1]))
        else:
            candidates=[basepath.parent/t.get('name',''),basepath.parent/t.get('relative_path',''),out/t.get('name','')]
            src=next(p for p in candidates if p.is_file())
            target.write_bytes(src.read_bytes())
        img=bpy.data.images.load(str(target),check_existing=False)
        img.pack()
        m=bpy.data.materials.new('Texture '+str(i)+' '+t.get('name',''));m.use_nodes=True
        nodes=m.node_tree.nodes;nt=nodes.new('ShaderNodeTexImage');nt.image=img;nt.interpolation='Linear'
        bs=nodes.get('Principled BSDF');m.node_tree.links.new(nt.outputs['Color'],bs.inputs['Base Color']);bs.inputs['Roughness'].default_value=.84
        mats.append(m)
        texdims.append((t.get('uv_width') or t.get('width') or data['resolution']['width'],t.get('uv_height') or t.get('height') or data['resolution']['height']))
        for k in (t.get('uuid'),str(t.get('id',i))):texids[k]=i
    objects=[]
    for e in data['elements']:
        if e.get('visibility') is False:continue
        if e.get('type')!='mesh':
            raise ValueError('Unexpected non-mesh element: '+str(e.get('name')))
        verts=e['vertices']; keys=list(verts);index={k:i for i,k in enumerate(keys)}
        origin=Vector(e.get('origin',[0,0,0]));r=Euler([math.radians(x) for x in e.get('rotation',[0,0,0])],'XYZ').to_matrix()
        xyz=[r@Vector(verts[k])+origin for k in keys]
        faces=[]; face_meta=[]
        for f in e['faces'].values():
            vs=f['vertices']
            if len(vs)<3 or len(set(vs))<3:continue
            tex=f.get('texture',0); ti=tex if isinstance(tex,int) else texids.get(tex,0)
            if not isinstance(ti,int) or ti<0 or ti>=len(mats):ti=0
            faces.append([index[k] for k in vs]);face_meta.append((vs,ti,f.get('uv',{})))
        if not faces:continue
        mesh=bpy.data.meshes.new(e['name']);mesh.from_pydata([(v.x,-v.z,v.y) for v in xyz],[],faces);mesh.update()
        for m in mats:mesh.materials.append(m)
        uv=mesh.uv_layers.new(name='UVMap')
        for poly,(vs,ti,uvs) in zip(mesh.polygons,face_meta):
            poly.material_index=ti
            w,h=texdims[ti]
            for loop,k in zip(poly.loop_indices,vs):
                u,v=uvs.get(k,[0,0]);uv.data[loop].uv=(u/w,1-v/h)
        obj=bpy.data.objects.new(e['name'],mesh);bpy.context.scene.collection.objects.link(obj);objects.append(obj)
    return objects,data
objs,data=build(hybrid)
print('HYBRID_RENDER_MESHES',len(objs),'FACES',sum(len(o.data.polygons) for o in objs),flush=True)
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=12;scene.cycles.use_denoising=True
scene.render.resolution_x=720;scene.render.resolution_y=720;scene.render.resolution_percentage=100
scene.world=bpy.data.worlds.new('Studio');scene.world.use_nodes=True
scene.world.node_tree.nodes['Background'].inputs[0].default_value=(.25,.25,.25,1);scene.world.node_tree.nodes['Background'].inputs[1].default_value=.7
scene.view_settings.view_transform='Standard'
center=Vector((0,0,19));span=43
def aim(o,t):o.rotation_euler=(t-o.location).to_track_quat('-Z','Y').to_euler()
for name,d,power in [('Key',(-3,-4,5),1000),('Fill',(4,-2,2),500),('Rim',(0,4,5),800)]:
    ld=bpy.data.lights.new(name,'AREA');ld.shape='DISK';l=bpy.data.objects.new(name,ld);scene.collection.objects.link(l);l.location=center+Vector(d)*span;ld.energy=power*span**2/9;ld.size=span*1.3;aim(l,center)
cam=bpy.data.cameras.new('Camera');camera=bpy.data.objects.new('Camera',cam);scene.collection.objects.link(camera);scene.camera=camera;cam.type='ORTHO';cam.ortho_scale=43;cam.clip_end=500
views=[('front',(0,-1,0)),('back',(0,1,0)),('right',(1,0,0)),('left',(-1,0,0)),('front3q',(1,-2,.5)),('rear3q',(-1,2,.5)),('top',(0,-.001,1)),('bottom',(0,-.001,-1)),('upper3q',(1,-1,2))]
sheet=Image.new('RGB',(1200,1260),(28,28,28));draw=ImageDraw.Draw(sheet)
for i,(name,d) in enumerate(views):
    camera.location=center+Vector(d).normalized()*120;aim(camera,center)
    path=out/f'hybrid_{name}.png';scene.render.filepath=str(path);bpy.ops.render.render(write_still=True)
    im=Image.open(path).convert('RGB');im.thumbnail((400,400));x=i%3*400;y=i//3*420;sheet.paste(im,(x,y));draw.text((x+5,y+401),name,fill='white')
    print('HYBRID_VIEW',name,flush=True)
sheet.save(out/'hybrid-contact-sheet.jpg',quality=90)
buf=io.BytesIO();sheet.save(buf,format='JPEG',quality=65);s=base64.b64encode(buf.getvalue()).decode()
for i in range(0,len(s),3000):print('HYBRID_IMAGE',i//3000,s[i:i+3000],flush=True)
bpy.ops.wm.save_as_mainfile(filepath=str(out/'Avatar_Toon_V18_Hybrid.blend'),compress=True)
print('HYBRID_AUDIT_COMPLETE',flush=True)
