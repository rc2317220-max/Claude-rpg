use glam::{Mat4, Quat, Vec3};
use horizon_assets::{
    bake::bake_flat_face,
    model::{Direction, Material},
};
use horizon_client_render::{
    gfx::entity_pixel_test,
    mesher::Vertex,
};
use std::{fs, io::BufWriter, path::{Path, PathBuf}};

#[path = "../../../project/src/rustpg_generated_models.rs"]
mod rustpg_generated_models;
#[path = "../../../project/src/rustpg_generated_hunter.rs"]
mod rustpg_generated_hunter;
#[path = "../../../project/src/rustpg_creatures_generated.rs"]
mod rustpg_creatures_generated;

use rustpg_generated_models::{
    GeneratedBone, GeneratedCube, GeneratedKey,
    SENTINEL_BONES, SENTINEL_CUBES, SENTINEL_CHAINS, SENTINEL_KEYS,
};
use rustpg_generated_hunter::{
    HUNTER_BONES, HUNTER_CUBES, HUNTER_CHAINS, HUNTER_KEYS,
};
use rustpg_creatures_generated::*;

#[derive(Clone, Copy)]
struct Rig {
    id: &'static str,
    texture: &'static str,
    bones: &'static [GeneratedBone],
    cubes: &'static [GeneratedCube],
    chains: &'static [&'static [usize]],
    keys: &'static [GeneratedKey],
    idle: &'static str,
    walk: &'static str,
    walk_duration: f32,
    action: &'static str,
    action_time: f32,
    model_scale: f32,
}

fn rigs() -> [Rig; 12] {
    [
        Rig { id:"ashen_sentinel", texture:"ashen_sentinel", bones:SENTINEL_BONES, cubes:SENTINEL_CUBES, chains:SENTINEL_CHAINS, keys:SENTINEL_KEYS, idle:"animation.ashen_sentinel.idle", walk:"animation.ashen_sentinel.walk", walk_duration:1.0, action:"animation.ashen_sentinel.slam", action_time:0.55, model_scale:1.5 },
        Rig { id:"rift_hunter", texture:"rift_hunter", bones:HUNTER_BONES, cubes:HUNTER_CUBES, chains:HUNTER_CHAINS, keys:HUNTER_KEYS, idle:"animation.rift_hunter.idle", walk:"animation.rift_hunter.run", walk_duration:0.8, action:"animation.rift_hunter.lunge", action_time:0.45, model_scale:1.0 },
        Rig { id:"mana_hound", texture:"mana_hound", bones:MANA_HOUND_BONES, cubes:MANA_HOUND_CUBES, chains:MANA_HOUND_CHAINS, keys:MANA_HOUND_KEYS, idle:"animation.mana_hound.idle", walk:"animation.mana_hound.run", walk_duration:0.7, action:"animation.mana_hound.pounce", action_time:0.45, model_scale:1.0 },
        Rig { id:"grave_knight", texture:"grave_knight", bones:GRAVE_KNIGHT_BONES, cubes:GRAVE_KNIGHT_CUBES, chains:GRAVE_KNIGHT_CHAINS, keys:GRAVE_KNIGHT_KEYS, idle:"animation.grave_knight.idle", walk:"animation.grave_knight.walk", walk_duration:1.0, action:"animation.grave_knight.slash", action_time:0.48, model_scale:1.05 },
        Rig { id:"rift_mantis", texture:"rift_mantis", bones:RIFT_MANTIS_BONES, cubes:RIFT_MANTIS_CUBES, chains:RIFT_MANTIS_CHAINS, keys:RIFT_MANTIS_KEYS, idle:"animation.rift_mantis.idle", walk:"animation.rift_mantis.scuttle", walk_duration:0.7, action:"animation.rift_mantis.reap", action_time:0.35, model_scale:1.0 },
        Rig { id:"crystal_wyrm", texture:"crystal_wyrm", bones:CRYSTAL_WYRM_BONES, cubes:CRYSTAL_WYRM_CUBES, chains:CRYSTAL_WYRM_CHAINS, keys:CRYSTAL_WYRM_KEYS, idle:"animation.crystal_wyrm.idle", walk:"animation.crystal_wyrm.slither", walk_duration:1.6, action:"animation.crystal_wyrm.bite", action_time:0.42, model_scale:1.0 },
        Rig { id:"eclipse_ogre", texture:"eclipse_ogre", bones:ECLIPSE_OGRE_BONES, cubes:ECLIPSE_OGRE_CUBES, chains:ECLIPSE_OGRE_CHAINS, keys:ECLIPSE_OGRE_KEYS, idle:"animation.eclipse_ogre.idle", walk:"animation.eclipse_ogre.walk", walk_duration:1.3, action:"animation.eclipse_ogre.smash", action_time:0.60, model_scale:1.35 },
        Rig { id:"dusk_revenant", texture:"dusk_revenant", bones:DUSK_REVENANT_BONES, cubes:DUSK_REVENANT_CUBES, chains:DUSK_REVENANT_CHAINS, keys:DUSK_REVENANT_KEYS, idle:"animation.dusk_revenant.idle", walk:"animation.dusk_revenant.glide", walk_duration:1.1, action:"animation.dusk_revenant.soul_rake", action_time:0.55, model_scale:1.1 },
        Rig { id:"storm_gryphon", texture:"storm_gryphon", bones:STORM_GRYPHON_BONES, cubes:STORM_GRYPHON_CUBES, chains:STORM_GRYPHON_CHAINS, keys:STORM_GRYPHON_KEYS, idle:"animation.storm_gryphon.idle", walk:"animation.storm_gryphon.flight", walk_duration:0.9, action:"animation.storm_gryphon.thunder_dive", action_time:0.40, model_scale:1.3 },
        Rig { id:"rune_colossus", texture:"rune_colossus", bones:RUNE_COLOSSUS_BONES, cubes:RUNE_COLOSSUS_CUBES, chains:RUNE_COLOSSUS_CHAINS, keys:RUNE_COLOSSUS_KEYS, idle:"animation.rune_colossus.idle", walk:"animation.rune_colossus.walk", walk_duration:1.5, action:"animation.rune_colossus.earthshatter", action_time:0.60, model_scale:1.8 },
        Rig { id:"ember_imp", texture:"ember_imp", bones:EMBER_IMP_BONES, cubes:EMBER_IMP_CUBES, chains:EMBER_IMP_CHAINS, keys:EMBER_IMP_KEYS, idle:"animation.ember_imp.idle", walk:"animation.ember_imp.hover", walk_duration:0.5, action:"animation.ember_imp.ember_slam", action_time:0.45, model_scale:0.75 },
        Rig { id:"abyss_weaver", texture:"abyss_weaver", bones:ABYSS_WEAVER_BONES, cubes:ABYSS_WEAVER_CUBES, chains:ABYSS_WEAVER_CHAINS, keys:ABYSS_WEAVER_KEYS, idle:"animation.abyss_weaver.idle", walk:"animation.abyss_weaver.skitter", walk_duration:0.6, action:"animation.abyss_weaver.venom_bite", action_time:0.40, model_scale:1.1 },
    ]
}

fn sample(keys:&[GeneratedKey], clip:&str, bone:&str, channel:&str, time:f32, default:[f32;3]) -> [f32;3] {
    let mut left:Option<&GeneratedKey>=None;
    let mut right:Option<&GeneratedKey>=None;
    for key in keys {
        if key.clip!=clip || key.bone!=bone || key.channel!=channel { continue; }
        if key.time<=time && left.is_none_or(|p| key.time>p.time) { left=Some(key); }
        if key.time>=time && right.is_none_or(|p| key.time<p.time) { right=Some(key); }
    }
    match (left,right) {
        (Some(a),Some(b)) if b.time>a.time => {
            let f=((time-a.time)/(b.time-a.time)).clamp(0.0,1.0);
            std::array::from_fn(|i| a.value[i]+(b.value[i]-a.value[i])*f)
        }
        (Some(a),_) => a.value,
        (_,Some(b)) => b.value,
        _ => default,
    }
}

fn rotate_about(axis:Vec3, degrees:f32, pivot:Vec3)->Mat4 {
    Mat4::from_translation(pivot)
        * Mat4::from_quat(Quat::from_axis_angle(axis.normalize_or_zero(), degrees.to_radians()))
        * Mat4::from_translation(-pivot)
}

fn abgr(rgb:u32)->u32 {
    let (r,g,b)=((rgb>>16)&255,(rgb>>8)&255,rgb&255);
    0xFF00_0000 | b<<16 | g<<8 | r
}

fn tint(kind:&str,name:&str)->u32 {
    match kind {
        "mana_hound" => if name.contains("mana_spine"){0x78F1FF}else if name.contains("fang"){0xF4FFFF}else{0xF5FAFF},
        "grave_knight" => if name.contains("visor")||name.contains("horn")||name.contains("crest")||name.contains("shield_cross")||name.contains("shield_boss"){0xD4A8FF}else if name.contains("blade")||name.contains("sword_guard"){0xF3F7FF}else{0xEEF0F7},
        "rift_mantis" => if name.contains("blade"){0xFF8AF5}else if name.contains("crest"){0x8BFFE8}else{0xF0EFFF},
        "crystal_wyrm" => if name.contains("crystal")||name.contains("horn"){0x8EF4FF}else if name.contains("fang"){0xFFFFFF}else{0xF2FCFF},
        "eclipse_ogre" => if name.contains("core"){0xFF7358}else if name.contains("horn")||name.contains("club_head"){0xFFD27A}else{0xF6D6CD},
        "ashen_sentinel" => if name=="core_crystal"{0xFFE0A8}else if name=="faceplate"{0xFFC783}else if name.contains("horn"){0xE0B0FF}else{0xFFFFFF},
        "rift_hunter" => if name.contains("fang"){0xE6FFFA}else if name.contains("talon"){0xC9F6FA}else if name=="muzzle"{0xD9FFF8}else{0xFFFFFF},
        "dusk_revenant" => if name.contains("face_void"){0x4D4964}else if name.contains("core")||name.contains("crown")||name.contains("tendril"){0xE0C2FF}else{0xF0E9FF},
        "storm_gryphon" => if name.contains("feather")||name.contains("crest"){0xA5E7FF}else if name.contains("beak")||name.contains("talon")||name.contains("toe"){0xFFF0B8}else{0xF4FAFF},
        "rune_colossus" => if name.contains("rune")||name.contains("heart")||name.contains("sigil"){0x95FFE1}else{0xF0F6F1},
        "ember_imp" => if name.contains("wing")||name.contains("fire")||name.contains("eye")||name.contains("tail"){0xFFC071}else if name.contains("horn"){0xFFE29A}else{0xFFE0C8},
        "abyss_weaver" => if name.contains("spine")||name.contains("eye")||name.contains("fang")||name.contains("stinger"){0xE5A0FF}else{0xEEE6FF},
        _=>0xFFFFFF,
    }
}

fn build_vertices(rig:Rig, idle_time:f32, walk_time:f32, walking:f32,
    action:Option<(&str,f32)>, yaw:f32) -> Vec<Vertex> {
    let ground=rig.cubes.iter().map(|c|c.from[1]).fold(f32::INFINITY,f32::min);
    let mut root=Mat4::from_scale(Vec3::splat(rig.model_scale));
    if ground.is_finite() {
        root *= Mat4::from_translation(Vec3::new(0.0,-ground/16.0,0.0));
    }
    let material=Material{sprite:String::new(),force_translucent:false};
    let mut staged:Vec<(Vec3,[f32;2],u32,Vec3)>=Vec::new();

    for (cube,chain) in rig.cubes.iter().zip(rig.chains.iter()) {
        let mut pose=root;
        for &index in *chain {
            let bone=&rig.bones[index];
            let idle=sample(rig.keys,rig.idle,bone.name,"rotation",idle_time,[0.0;3]);
            let moving=sample(rig.keys,rig.walk,bone.name,"rotation",walk_time,[0.0;3]);
            let action_rot=action.map(|(clip,time)|
                sample(rig.keys,clip,bone.name,"rotation",time,[0.0;3])).unwrap_or([0.0;3]);
            let rotation:[f32;3]=std::array::from_fn(|a|
                bone.rotation[a]+idle[a]+moving[a]*walking+action_rot[a]);
            let pivot=Vec3::from_array(bone.pivot)/16.0;
            for (axis,degrees) in [(Vec3::Z,rotation[2]),(Vec3::Y,rotation[1]),(Vec3::X,rotation[0])] {
                if degrees.abs()>0.0001 { pose*=rotate_about(axis,degrees,pivot); }
            }
            let idle_size=sample(rig.keys,rig.idle,bone.name,"scale",idle_time,[1.0;3]);
            let action_size=action.map(|(clip,time)|
                sample(rig.keys,clip,bone.name,"scale",time,[1.0;3])).unwrap_or([1.0;3]);
            let size:[f32;3]=std::array::from_fn(|i|idle_size[i]*action_size[i]);
            if size.iter().any(|v|(v-1.0).abs()>0.0001) {
                pose*=Mat4::from_translation(pivot)*Mat4::from_scale(Vec3::from_array(size))*Mat4::from_translation(-pivot);
            }
        }
        let pivot=Vec3::from_array(cube.origin)/16.0;
        for (axis,degrees) in [(Vec3::Z,cube.rotation[2]),(Vec3::Y,cube.rotation[1]),(Vec3::X,cube.rotation[0])] {
            if degrees.abs()>0.0001 { pose*=rotate_about(axis,degrees,pivot); }
        }

        for (direction, uv) in Direction::ALL.into_iter().zip(cube.uv) {
            let Ok(quad)=bake_flat_face(cube.from,cube.to,uv,direction,material.clone(),-1) else {continue};
            let points=quad.positions.map(|p|pose.transform_point3(Vec3::from_array(p)));
            let mut normal=(points[1]-points[0]).cross(points[2]-points[0]).normalize_or_zero();
            if normal==Vec3::ZERO { normal=Vec3::Y; }
            for i in 0..4 {
                staged.push((points[i],quad.uv[i],abgr(tint(rig.id,cube.name)),normal));
            }
        }
    }

    let camera=Mat4::from_rotation_x((-10.0f32).to_radians())*Mat4::from_rotation_y(yaw.to_radians());
    for row in &mut staged { row.0=camera.transform_point3(row.0); row.3=camera.transform_vector3(row.3).normalize_or_zero(); }
    let mut min=Vec3::splat(f32::INFINITY);
    let mut max=Vec3::splat(f32::NEG_INFINITY);
    for (p,_,_,_) in &staged { min=min.min(*p); max=max.max(*p); }
    let center=(min+max)*0.5;
    let span=max-min;
    let scale=1.62/span.x.max(span.y).max(0.001);

    staged.into_iter().map(|(p,uv,color,n)|{
        let p=(p-center)*scale;
        Vertex{
            pos:[p.x,p.y,0.55+p.z*0.08],
            uv,
            layer:0,
            color,
            light:0xF0F0,
            normal:[
                (n.x*127.0).round().clamp(-127.0,127.0) as i8,
                (n.y*127.0).round().clamp(-127.0,127.0) as i8,
                (n.z*127.0).round().clamp(-127.0,127.0) as i8,
                -127,
            ],
            glint_uv:[0.0,0.0],
        }
    }).collect()
}

fn load_rgba(path:&Path)->Result<([u32;2],Vec<u8>),Box<dyn std::error::Error>> {
    let mut decoder=png::Decoder::new(std::io::BufReader::new(fs::File::open(path)?));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader=decoder.read_info()?;
    let mut buf=vec![0;reader.output_buffer_size().unwrap_or(0)];
    let info=reader.next_frame(&mut buf)?;
    let src=&buf[..info.buffer_size()];
    let rgba=match info.color_type {
        png::ColorType::Rgba=>src.to_vec(),
        png::ColorType::Rgb=>src.chunks_exact(3).flat_map(|p|[p[0],p[1],p[2],255]).collect(),
        png::ColorType::Grayscale=>src.iter().flat_map(|&v|[v,v,v,255]).collect(),
        png::ColorType::GrayscaleAlpha=>src.chunks_exact(2).flat_map(|p|[p[0],p[0],p[0],p[1]]).collect(),
        other=>return Err(format!("unsupported PNG color type {other:?}").into()),
    };
    Ok(([info.width,info.height],rgba))
}

fn save_rgba(path:&Path,size:u32,pixels:&[u8])->Result<(),Box<dyn std::error::Error>> {
    let file=fs::File::create(path)?;
    let mut enc=png::Encoder::new(BufWriter::new(file),size,size);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w=enc.write_header()?;
    w.write_image_data(pixels)?;
    Ok(())
}

fn main()->Result<(),Box<dyn std::error::Error>> {
    let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
    let out=std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(||root.join(".agent/artifacts/rustpg-entities"));
    fs::create_dir_all(&out)?;
    let size=std::env::var("RUSTPG_ENTITY_LAB_SIZE").ok().and_then(|v|v.parse().ok()).unwrap_or(256u32);
    let views=[("front",0.0f32),("three_quarter",38.0),("side",90.0)];
    let poses=[("idle",0.65f32,0.0f32,0.0f32),("move",0.3f32,0.33f32,1.0f32)];
    let mut manifest=String::from("{\n  \"renderer\": \"horizon-client-render::gfx::entity_pixel_test\",\n  \"creatures\": [\n");
    let all=rigs();
    for (ri,rig) in all.into_iter().enumerate() {
        let texture=root.join("project/assets/rustpg/textures/entity").join(format!("{}.png",rig.texture));
        let (atlas_size,atlas)=load_rgba(&texture)?;
        let dir=out.join(rig.id);
        fs::create_dir_all(&dir)?;
        for (pose_name,idle_t,walk_t,walking) in poses {
            for (view_name,yaw) in views {
                let vertices=build_vertices(rig,idle_t,walk_t*rig.walk_duration,walking,None,yaw);
                let pixels=entity_pixel_test::render(&vertices,atlas_size,&atlas,Mat4::IDENTITY,false)
                    .map_err(|e|format!("{} {pose_name} {view_name}: {e}",rig.id))?;
                save_rgba(&dir.join(format!("{pose_name}-{view_name}.png")),size,&pixels)?;
            }
        }
        // Authoring-only action preview. Public EntityRenderState does not expose
        // attack progress, so this validates the authored clip without claiming
        // runtime attack dispatch is already wired.
        let vertices=build_vertices(rig,0.35,0.0,0.0,Some((rig.action,rig.action_time)),38.0);
        let pixels=entity_pixel_test::render(&vertices,atlas_size,&atlas,Mat4::IDENTITY,false)
            .map_err(|e|format!("{} action three_quarter: {e}",rig.id))?;
        save_rgba(&dir.join("action-three_quarter.png"),size,&pixels)?;
        manifest.push_str(&format!("    {{\"id\":\"{}\",\"cubes\":{},\"bones\":{}}}{}\n",
            rig.id,rig.cubes.len(),rig.bones.len(),if ri+1==12{""}else{","}));
    }
    manifest.push_str("  ]\n}\n");
    fs::write(out.join("manifest.json"),manifest)?;
    println!("RustPG entity visual lab wrote {}",out.display());
    Ok(())
}
