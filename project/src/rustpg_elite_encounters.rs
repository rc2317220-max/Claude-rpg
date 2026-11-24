//! Scaled hostile combat passives and health-based boss phases.
//! Reproduces the behavior *pattern* of Remnant Mobs and Remnant Bosses, not
//! their proprietary mob entities, animation assets or AI code.
use crate::rustpg_scaling::{MobScale,Tier};

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Snapshot {
    pub tier:Tier,
    pub mutation:Option<&'static str>,
    pub resistance:f64,
    pub health_fraction:f64,
    pub regen:f64,
}
impl Snapshot {
    pub fn from_scale(scale:&MobScale)->Self {
        Self{tier:scale.tier,mutation:scale.mutation,resistance:scale.resistance,
             health_fraction:1.0,regen:scale.regen}
    }
    pub fn update_health(&mut self,health:f32,max:f32){
        if health.is_finite() && max.is_finite() && max>0.0 {
            self.health_fraction=f64::from((health/max).clamp(0.0,1.0));
        }
    }
    pub fn boss_phase(self)->u8 {
        if self.tier<Tier::MiniBoss {1}else{
            crate::rustpg_scaling::boss_phase(1.0-self.health_fraction.clamp(0.0,1.0))
        }
    }
    /// Boss phases increase aggression as life drops; first phase leaves
    /// vanilla/other mod multipliers untouched. Never re-multiply base stats.
    pub fn outgoing_multiplier(self)->f32 {
        let phase=if self.tier>=Tier::Boss {self.boss_phase()}else{1};
        let phase_mult=match phase{3=>1.35,2=>1.15,_=>1.0};
        let berserk=crate::rustpg_evil_att::berserk_factor(self.mutation,
            1.0-self.health_fraction.clamp(0.0,1.0)).min(1.5);
        (phase_mult*berserk).min(2.0) as f32
    }
    /// Resistances from the saved MobScale were not previously used by combat.
    /// Armor/toughness already apply natively and must not be counted again.
    pub fn incoming_multiplier(self,damage_type:&str)->f32 {
        let source=damage_type.rsplit(':').next().unwrap_or(damage_type);
        if matches!(source,"out_of_world"|"generic_kill"|"starve"){return 1.0;}
        (1.0-self.resistance.clamp(0.0,0.75)) as f32
    }
    /// Regeneration is absolute health per second, limited by maximum HP.
    /// No healing from death; caller checks current health before applying.
    pub fn heal_per_second(self,max_health:f32)->f32 {
        if !max_health.is_finite() || max_health<=0.0 {return 0.0}
        if self.tier<Tier::Fortified && !matches!(self.mutation,Some("regenerative"|"vampiric")) {return 0.0}
        (self.regen.max(0.0).min(1000.0) as f32).min(max_health*0.02+0.5)
    }
}
pub fn resolve(initial:f32,attacker:Option<Snapshot>,defender:Option<Snapshot>,kind:&str)->f32 {
    if !initial.is_finite() {return 0.0}
    let offense=attacker.map_or(1.0,Snapshot::outgoing_multiplier);
    let defense=defender.map_or(1.0,|s|s.incoming_multiplier(kind));
    (initial.max(0.0)*offense*defense).clamp(0.0,1.0e9)
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn boss_enrage_has_real_hp_thresholds(){
        let base=crate::rustpg_scaling::stats_for(20,Tier::Boss);
        let mut snap=Snapshot::from_scale(&base);
        assert_eq!(snap.boss_phase(),1); assert_eq!(snap.outgoing_multiplier(),1.0);
        snap.update_health(55.0,100.0);assert_eq!(snap.boss_phase(),2);
        snap.update_health(15.0,100.0);assert_eq!(snap.boss_phase(),3);
        assert!(snap.outgoing_multiplier()>1.3);
    }
    #[test]fn normal_vampiric_mob_can_regenerate(){
        let scale=crate::rustpg_scaling::with_mutation(crate::rustpg_scaling::stats_for(20,Tier::Normal),Some("vampiric"));
        assert!(Snapshot::from_scale(&scale).heal_per_second(20.0)>0.0);
    }
    #[test]fn resistance_does_not_negate_void_and_regen_has_cap(){
        let mut snap=Snapshot::from_scale(&crate::rustpg_scaling::stats_for(50,Tier::Elite));
        snap.resistance=1.0;
        assert_eq!(snap.incoming_multiplier("minecraft:out_of_world"),1.0);
        assert!((snap.incoming_multiplier("minecraft:mob_attack")-0.25).abs()<1e-6);
        snap.regen=10000.0;
        assert!(snap.heal_per_second(100.0)<=2.5);
    }
}
