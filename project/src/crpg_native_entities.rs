//! Native Starling 0.9 entity presentation and AI.
//! Original procedural models; no GeckoLib models, animations or textures are copied.
use mod_api::{self,BlockEntityDrawContext,CubeFace,EntityRenderState,EntityRenderer,EntityPart,
    Goal,GoalContext,GoalFlags,GoalRegistrar,RandomSource};
use std::sync::{Arc,OnceLock};
use crate::crpg_skeletal::{Clip,Key,Pose,Track,Vec3};

pub const SENTINEL:&str="claude_rpg:ashen_sentinel";
pub const HUNTER:&str="claude_rpg:rift_hunter";

fn rig()->&'static Clip {
    static CLIP:OnceLock<Clip>=OnceLock::new();
    CLIP.get_or_init(||{
        let rotation=|deg:f32|Pose{rotation:Vec3{x:deg,y:0.0,z:0.0},..Pose::identity()};
        Clip{name:"breathing_walk".into(),length:40.0,looping:true,tracks:vec![
            Track{bone:"left_arm".into(),keys:vec![Key{at:0.0,pose:rotation(12.0)},Key{at:20.0,pose:rotation(-19.0)},Key{at:40.0,pose:rotation(12.0)}]},
            Track{bone:"right_arm".into(),keys:vec![Key{at:0.0,pose:rotation(-20.0)},Key{at:20.0,pose:rotation(13.0)},Key{at:40.0,pose:rotation(-20.0)}]},
            Track{bone:"head".into(),keys:vec![Key{at:0.0,pose:rotation(-4.0)},Key{at:20.0,pose:rotation(4.0)},Key{at:40.0,pose:rotation(-4.0)}]},
        ]}
    })
}

struct NativeRig {boss:bool}
fn cube(draw:&mut dyn BlockEntityDrawContext,from:[f32;3],to:[f32;3],tex:&str,tint:u32){
    let face=Some(CubeFace::new(tex));
    let _=draw.cube(from,to,[face;6],tint);
}
impl EntityRenderer for NativeRig {
    fn submit(&self,entity:&EntityRenderState,draw:&mut dyn BlockEntityDrawContext)->Result<(),String>{
        let Some(living)=entity.living.as_ref() else{return Ok(())};
        let scaled=living.scale * living.age_scale;
        if !scaled.is_finite() || scaled<=0.0{return Ok(())}
        let pose=rig().sample(entity.age).ok_or("Invalid Claude RPG rig")?;
        let walk=(living.walk_animation_speed.clamp(0.0,1.0)*30.0) as f32;
        draw.push();
        draw.rotate_about([0.0,1.0,0.0],-living.body_rot,[0.0;3]);
        let boss_scale=if self.boss{1.5}else{1.0};
        draw.scale([scaled*boss_scale;3]);
        let texture=if self.boss{"block/netherite_block"}else{"block/deepslate"};
        let metal=if self.boss{0xC8A8FF}else{0x77C8D7};
        // 1 unit in this geometry = 1/16 block; the art is entirely procedural.
        cube(draw,[-5.0,12.0,-3.0],[5.0,24.0,3.0],texture,metal);
        cube(draw,[-4.0,24.0,-4.0],[4.0,32.0,4.0],texture,0xF3F0FF);
        draw.push();
        draw.rotate_about([1.0,0.0,0.0],pose["head"].rotation.x+living.x_rot*0.25,[0.0,28.0,0.0]);
        cube(draw,[-3.5,25.0,-4.2],[3.5,31.0,-3.8],"block/amethyst_block",0xFBE8FF);
        draw.pop();
        for (left,key) in [(true,"left_arm"),(false,"right_arm")] {
            let side=if left{-1.0}else{1.0};
            draw.push();
            draw.rotate_about([1.0,0.0,0.0],pose[key].rotation.x+walk*side,[side*6.5,22.0,0.0]);
            cube(draw,[side*6.5-2.0,10.0,-2.0],[side*6.5+2.0,24.0,2.0],texture,metal);
            draw.pop();
        }
        for side in [-1.0f32,1.0] {
            draw.push();
            draw.rotate_about([1.0,0.0,0.0],-walk*side,[side*2.5,12.0,0.0]);
            cube(draw,[side*2.5-2.0,0.0,-2.5],[side*2.5+2.0,12.0,2.5],texture,0x9296B9);
            draw.pop();
        }
        if self.boss {
            let pulse=0.6+0.4*(entity.age*0.16).sin().abs();
            let tint=if pulse>0.8{0xE2B1FF}else{0x9B78D8};
            cube(draw,[-2.5,17.0,-3.5],[2.5,21.5,-3.0],"block/amethyst_block",tint);
            cube(draw,[-6.0,31.0,-1.0],[-3.0,36.0,1.0],"block/amethyst_block",0xEBD2FF);
            cube(draw,[3.0,31.0,-1.0],[6.0,36.0,1.0],"block/amethyst_block",0xEBD2FF);
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
    #[test]fn registered_rig_has_separate_bone_tracks(){
        assert!(rig().valid());
        assert_eq!(rig().sample(12.0).unwrap().len(),3);
    }
}
