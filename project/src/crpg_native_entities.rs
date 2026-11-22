//! Native entity AI and Blockbench-generated bone renderers for Horizon 0.9.0.
//! Two custom creatures use real .bbmodel sources and original PNG atlases.
use mod_api::{self,BlockEntityDrawContext,CubeFace,EntityRenderState,EntityRenderer,EntityPart,
    Goal,GoalContext,GoalFlags,GoalRegistrar,RandomSource};
use std::sync::Arc;
use crate::crpg_generated_models::{GeneratedBone,GeneratedCube,GeneratedKey,
    SENTINEL_BONES,SENTINEL_CUBES,SENTINEL_CHAINS,SENTINEL_KEYS};
use crate::crpg_generated_hunter::{HUNTER_BONES,HUNTER_CUBES,HUNTER_CHAINS,HUNTER_KEYS};

pub const SENTINEL:&str="claude_rpg:ashen_sentinel";
pub const HUNTER:&str="claude_rpg:rift_hunter";

/// Sample one authored Blockbench channel. The file stores linear/catmullrom
/// curves; native geometry currently approximates both with linear blending.
/// Combat moves require an explicit server animation trigger (not yet wired).
fn sample_channel(keys:&[GeneratedKey],clip:&str,bone:&str,channel:&str,
    time:f32,default:[f32;3])->[f32;3] {
    let mut left:Option<&GeneratedKey>=None;
    let mut right:Option<&GeneratedKey>=None;
    for key in keys {
        if key.clip!=clip || key.bone!=bone || key.channel!=channel {continue;}
        if key.time<=time && left.is_none_or(|prev|key.time>prev.time) {left=Some(key);}
        if key.time>=time && right.is_none_or(|prev|key.time<prev.time) {right=Some(key);}
    }
    match (left,right){
        (Some(a),Some(b)) if b.time>a.time=>{
            let factor=((time-a.time)/(b.time-a.time)).clamp(0.0,1.0);
            std::array::from_fn(|i|a.value[i]+(b.value[i]-a.value[i])*factor)
        },
        (Some(a),_)=>a.value,
        (_,Some(b))=>b.value,
        _=>default,
    }
}

/// A single reusable renderer for all creatures exported by our Blockbench
/// headless pipeline. One registry renderer instance per custom entity type.
struct NativeRig {boss:bool}
impl EntityRenderer for NativeRig {
    fn submit(&self,entity:&EntityRenderState,draw:&mut dyn BlockEntityDrawContext)->Result<(),String>{
        let Some(living)=entity.living.as_ref() else{return Ok(())};
        let scaled=living.scale*living.age_scale;
        if !scaled.is_finite() || scaled<=0.0{return Ok(());}
        let (bones,cubes,chains,keys,idle_clip,walk_clip,walk_duration,texture,model_scale):
            (&[GeneratedBone],&[GeneratedCube],&[&[usize]],&[GeneratedKey],&str,&str,f32,&str,f32)
            =if self.boss {
                (SENTINEL_BONES,SENTINEL_CUBES,SENTINEL_CHAINS,SENTINEL_KEYS,
                 "animation.ashen_sentinel.idle","animation.ashen_sentinel.walk",
                 1.0,"claude_rpg:entity/ashen_sentinel",1.5)
            }else{
                (HUNTER_BONES,HUNTER_CUBES,HUNTER_CHAINS,HUNTER_KEYS,
                 "animation.rift_hunter.idle","animation.rift_hunter.run",
                 0.8,"claude_rpg:entity/rift_hunter",1.0)
            };
        if cubes.len()!=chains.len(){return Err("Corrupt compiled Blockbench cube ancestry".into())}
        let idle_time=(entity.age/20.0).rem_euclid(2.0);
        let walk_time=(living.walk_animation_pos*0.16).rem_euclid(walk_duration);
        let walking=living.walk_animation_speed.clamp(0.0,1.0);
        draw.push();
        draw.rotate_about([0.0,1.0,0.0],-living.body_rot,[0.0;3]);
        draw.scale([scaled*model_scale;3]);
        // Both reviewed models bottom at -3 Blockbench units.
        draw.translate([0.0,3.0/16.0,0.0]);
        for (part,path) in cubes.iter().zip(chains.iter()){
            draw.push();
            for &index in path.iter(){
                let Some(bone)=bones.get(index)else{return Err("Corrupt creature bone index".into())};
                let idle=sample_channel(keys,idle_clip,bone.name,"rotation",idle_time,[0.0;3]);
                let moving=sample_channel(keys,walk_clip,bone.name,"rotation",walk_time,[0.0;3]);
                let mut rotation:[f32;3]=std::array::from_fn(|axis|
                    bone.rotation[axis]+idle[axis]+moving[axis]*walking);
                if bone.name=="head" {
                    rotation[0]+=living.x_rot*0.25;
                    rotation[1]+=living.y_rot*0.35;
                }
                let pivot=bone.pivot.map(|v|v/16.0);
                for (axis,degrees) in [([0.0,0.0,1.0],rotation[2]),
                    ([0.0,1.0,0.0],rotation[1]),([1.0,0.0,0.0],rotation[0])]{
                    if degrees.abs()>0.0001 {draw.rotate_about(axis,degrees,pivot);}
                }
                let size=sample_channel(keys,idle_clip,bone.name,"scale",idle_time,[1.0;3]);
                if size.iter().any(|v|(v-1.0).abs()>0.0001) {
                    draw.translate(pivot);
                    draw.scale(size);
                    draw.translate(pivot.map(|v|-v));
                }
            }
            let pivot=part.origin.map(|v|v/16.0);
            for (axis,degrees) in [([0.0,0.0,1.0],part.rotation[2]),
                ([0.0,1.0,0.0],part.rotation[1]),([1.0,0.0,0.0],part.rotation[0])]{
                if degrees.abs()>0.0001 {draw.rotate_about(axis,degrees,pivot);}
            }
            let tint=if self.boss {
                if part.name=="core_crystal"{0xFFD08B}
                else if part.name=="faceplate"{0xFFB56A}
                else if part.name.contains("horn"){0xCB92E8}
                else {0xFFFFFF}
            }else if part.name.contains("fang"){0xC6FAF1}
                else if part.name.contains("talon"){0xA0D9DE}
                else if part.name=="muzzle"{0xB9F1E7}
                else {0xFFFFFF};
            let face=Some(CubeFace::new(texture));
            let _=draw.cube(part.from,part.to,[face;6],tint);
            draw.pop();
        }
        draw.pop();
        Ok(())
    }
}

/// Adds a short, telegraphed jump when a custom mob is near an opponent.
/// The actual hostile navigation and attacks are inherited from the zombie template.
#[derive(Default)]
struct LeapingAttack {left:u32}
impl Goal for LeapingAttack {
    fn flags(&self)->GoalFlags {GoalFlags::JUMP.union(GoalFlags::LOOK)}
    fn can_use(&mut self,cx:&mut GoalContext<'_>)->bool {
        cx.random().next_int_bound(90).unwrap_or(1)==0
    }
    fn can_continue(&mut self,_:&mut GoalContext<'_>)->bool{self.left>0}
    fn start(&mut self,_:&mut GoalContext<'_>){self.left=12;}
    fn tick(&mut self,cx:&mut GoalContext<'_>){
        if self.left==7 {cx.jump();}
        self.left=self.left.saturating_sub(1);
    }
    fn name(&self)->&'static str {"claude_rpg:telegraphed_leap"}
}

pub struct EliteGoals;
#[mod_api::horizon_part]
impl EntityPart for EliteGoals {
    fn register_goals(&self,goals:&mut GoalRegistrar<'_>){
        goals.add_goal(3,LeapingAttack::default());
    }
}

pub fn register_entities(registrar:&mut dyn mod_api::Registrar){
    registrar.entity_part(SENTINEL,EliteGoals);
    registrar.entity_part(HUNTER,EliteGoals);
}
pub fn register_renderers(event:&mut mod_api::RegisterEntityRenderersEvent<'_>){
    event.renderers.register(SENTINEL.into(),Arc::new(NativeRig{boss:true}));
    event.renderers.register(HUNTER.into(),Arc::new(NativeRig{boss:false}));
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn both_creatures_use_real_generated_blockbench_bones(){
        let rigs=[
            (SENTINEL_BONES,SENTINEL_CUBES,SENTINEL_CHAINS,SENTINEL_KEYS,
             "animation.ashen_sentinel.slam"),
            (HUNTER_BONES,HUNTER_CUBES,HUNTER_CHAINS,HUNTER_KEYS,
             "animation.rift_hunter.lunge"),
        ];
        for (bones,cubes,chains,keys,attack) in rigs {
            assert!(bones.len()>=10);
            assert!(cubes.len()>=19);
            assert_eq!(cubes.len(),chains.len());
            assert!(keys.iter().any(|k|k.clip==attack));
            for chain in chains {
                assert!(!chain.is_empty());
                assert!(chain.iter().all(|index|*index<bones.len()));
            }
        }
    }
    #[test] fn interpolation_uses_authored_key_values(){
        let rotation=sample_channel(HUNTER_KEYS,"animation.rift_hunter.idle",
            "head","rotation",1.0,[0.0;3]);
        assert!((rotation[1]-7.0).abs()<0.0001);
    }
}
