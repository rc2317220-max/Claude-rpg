//! Typed, engine-independent combat pipeline for Claude RPG extension authors.
//! The Starling bridge owns entity identity/world/permissions. Stages only
//! calculate bounded damage; no plugin can mutate player HP directly.
use std::collections::BTreeSet;
use super::crpg_relic_engine::RelicEffects;

pub const COMBAT_ABI:u16=1;
pub const MAX_STAGES:usize=32;
pub const MAX_DAMAGE:f32=1.0e9;

#[derive(Clone,Copy,Debug)]
pub struct CombatContext {
    pub base_damage:f32,
    pub attacker_exhausted:bool,
    pub attacker:RelicEffects,
    pub defender:RelicEffects,
}
#[derive(Clone,Copy,Debug)]
pub struct StageInfo {pub id:&'static str,pub abi:u16}

pub trait DamageStage:Send+Sync {
    fn info(&self)->StageInfo;
    /// No ServerHost or LevelRandomHost is ever exposed to user extensions.
    fn apply(&self,context:&CombatContext,amount:f32)->f32;
}
#[derive(Debug,PartialEq,Eq)]
pub enum RegisterError {BadId,WrongAbi,Duplicate,Full}
pub struct CombatPipeline {
    stages:Vec<Box<dyn DamageStage>>,
    ids:BTreeSet<&'static str>,
}
impl CombatPipeline {
    pub fn new()->Self {Self{stages:Vec::new(),ids:BTreeSet::new()}}
    pub fn register(&mut self,stage:Box<dyn DamageStage>)->Result<(),RegisterError> {
        let info=stage.info();
        if info.abi!=COMBAT_ABI {return Err(RegisterError::WrongAbi);}
        if info.id.len()>64 || !info.id.contains(':') || !info.id.bytes().all(|b|
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b,b'_'|b'-'|b'.'|b':')) {
            return Err(RegisterError::BadId);
        }
        if self.ids.contains(info.id) {return Err(RegisterError::Duplicate);}
        if self.stages.len()>=MAX_STAGES {return Err(RegisterError::Full);}
        self.ids.insert(info.id);self.stages.push(stage);Ok(())
    }
    pub fn resolve(&self,context:CombatContext)->f32 {
        if !context.base_damage.is_finite(){return 0.0;}
        let mut amount=context.base_damage.clamp(0.0,MAX_DAMAGE);
        // A plugin returning NaN/negative/inf is ignored. This guard must
        // remain in the host registry, not in the plugin implementation.
        for stage in &self.stages {
            let proposed=stage.apply(&context,amount);
            if proposed.is_finite() && proposed>=0.0 {
                amount=proposed.min(MAX_DAMAGE);
            }
        }
        amount
    }
    pub fn count(&self)->usize {self.stages.len()}
}

struct FatigueStage;
impl DamageStage for FatigueStage {
    fn info(&self)->StageInfo {StageInfo{id:"claude_rpg:fatigue",abi:COMBAT_ABI}}
    fn apply(&self,c:&CombatContext,d:f32)->f32 {
        if c.attacker_exhausted {d*0.75}else{d}
    }
}
struct AttackRelics;
impl DamageStage for AttackRelics {
    fn info(&self)->StageInfo {StageInfo{id:"claude_rpg:relic_attack",abi:COMBAT_ABI}}
    fn apply(&self,c:&CombatContext,d:f32)->f32 {d*(1.0+c.attacker.attack_bonus)}
}
struct GuardRelics;
impl DamageStage for GuardRelics {
    fn info(&self)->StageInfo {StageInfo{id:"claude_rpg:relic_guard",abi:COMBAT_ABI}}
    fn apply(&self,c:&CombatContext,d:f32)->f32 {d*(1.0-c.defender.guard)}
}

pub fn builtins()->CombatPipeline {
    let mut pipe=CombatPipeline::new();
    pipe.register(Box::new(FatigueStage)).expect("valid static hook");
    pipe.register(Box::new(AttackRelics)).expect("valid static hook");
    pipe.register(Box::new(GuardRelics)).expect("valid static hook");
    pipe
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn matches_original_damage_formula() {
        let mut a=RelicEffects::default();a.attack_bonus=0.18;
        let mut d=RelicEffects::default();d.guard=0.24;
        let ctx=CombatContext{base_damage:100.0,attacker_exhausted:true,
            attacker:a,defender:d};
        let final_damage=builtins().resolve(ctx);
        assert!((final_damage-a.final_hit(100.0,true,d)).abs()<0.001);
    }
    struct Broken;
    impl DamageStage for Broken {
        fn info(&self)->StageInfo {StageInfo{id:"example:broken",abi:COMBAT_ABI}}
        fn apply(&self,_:&CombatContext,_:f32)->f32 {f32::NAN}
    }
    #[test] fn invalid_plugin_outputs_cannot_break_damage() {
        let mut pipe=builtins();
        assert_eq!(pipe.register(Box::new(Broken)),Ok(()));
        assert_eq!(pipe.register(Box::new(Broken)),Err(RegisterError::Duplicate));
        let c=CombatContext{base_damage:22.0,attacker_exhausted:false,
            attacker:RelicEffects::default(),defender:RelicEffects::default()};
        assert_eq!(pipe.resolve(c),22.0);
    }
}
