//! Server-authoritative action-combat cadence for RustPG 6.0.
//! The design follows the uploaded reference mod's data-driven action windows,
//! but never accepts a client-reported hit: Horizon's real attack/damage hooks remain authority.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Timing{pub windup:u32,pub active:u32,pub recovery:u32,pub chain:u32}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct ActionTicket{pub step:u8,pub timing:Timing,pub stamina_mult:f64,pub damage_mult:f32}
#[derive(Clone,Debug,Default)]
pub struct CombatState{
    weapon:String,step:u8,next_input:u64,chain_until:u64,last_accept:u64,last_damage_mult:f32,
}
fn base(weapon:&str,step:u8)->(u32,u32,u32,u32,f64){
    match weapon{
        "unarmed"=>(2,2,4,12,0.72),
        "sword"=>(3,2,5,14,0.90),
        "axe"=>(5,2,9,18,1.22),
        "mace"=>(6,2,11,20,1.35),
        "spear"|"trident"=>(4,3,7,17,1.05),
        _=>(3,2,6,14,1.0),
    }
}
fn max_steps(weapon:&str)->u8{match weapon{"unarmed"=>4,"sword"=>3,"axe"|"mace"=>2,"spear"|"trident"=>3,_=>2}}
/// Mastery compresses anticipation/recovery, not the authoritative active slice.
pub fn timing(weapon:&str,step:u8,mastery:f64)->Timing{
    let (w,a,r,c,_)=base(weapon,step);let m=if mastery.is_finite(){mastery.clamp(0.0,1.0)}else{0.0};
    let scale=1.0-0.28*m;
    Timing{windup:((w as f64*scale).round() as u32).max(1),active:a,
        recovery:((r as f64*scale).round() as u32).max(2),chain:((c as f64*(1.0+0.18*m)).round() as u32).max(6)}
}
impl CombatState{
    pub fn attempt(&mut self,weapon:&str,tick:u64,mastery:f64)->Result<ActionTicket,&'static str>{
        if tick<self.next_input{return Err("ação ainda em recuperação");}
        if self.weapon==weapon && tick<=self.chain_until{self.step=(self.step%max_steps(weapon))+1}else{self.step=1;self.weapon=weapon.to_owned();}
        let mastery=if mastery.is_finite(){mastery.clamp(0.0,1.0)}else{0.0};
        let t=timing(weapon,self.step,mastery);let (_,_,_,_,cost)=base(weapon,self.step);
        // Only a small bounded combo increase; regular RPG stats still own most scaling.
        let dmg=(1.0+0.025*(self.step.saturating_sub(1) as f32)+(0.06*mastery.clamp(0.0,1.0)) as f32).min(1.18);
        self.next_input=tick.saturating_add(u64::from(t.windup+t.active+t.recovery));
        self.chain_until=tick.saturating_add(u64::from(t.chain.max(t.windup+t.active+t.recovery)));
        self.last_accept=tick;self.last_damage_mult=dmg;
        Ok(ActionTicket{step:self.step,timing:t,stamina_mult:cost,damage_mult:dmg})
    }
    pub fn boost_last(&mut self,mult:f32){if mult.is_finite(){self.last_damage_mult=(self.last_damage_mult*mult.clamp(1.0,1.35)).min(1.40);}}
    pub fn damage_multiplier(&self,tick:u64)->f32{if tick.saturating_sub(self.last_accept)<=2{self.last_damage_mult.max(1.0)}else{1.0}}
    pub fn reset_if_idle(&mut self,tick:u64){if tick>self.chain_until.saturating_add(20){self.step=0;self.weapon.clear();self.last_damage_mult=1.0;}}
    pub fn step(&self)->u8{self.step}
}
#[cfg(test)]mod tests{use super::*;
#[test]fn mastery_shortens_recovery_not_active(){let a=timing("sword",1,0.0);let b=timing("sword",1,1.0);assert_eq!(a.active,b.active);assert!(b.windup<=a.windup&&b.recovery<a.recovery);}
#[test]fn combo_is_server_timed_and_bounded(){let mut s=CombatState::default();let a=s.attempt("sword",100,0.5).unwrap();assert_eq!(a.step,1);assert!(s.attempt("sword",101,0.5).is_err());let ready=100+u64::from(a.timing.windup+a.timing.active+a.timing.recovery);assert!(s.attempt("sword",ready-1,0.5).is_err());let b=s.attempt("sword",ready,0.5).unwrap();assert_eq!(b.step,2);assert!(b.damage_mult<=1.18);assert!(s.damage_multiplier(ready)>1.0);assert_eq!(s.damage_multiplier(120),1.0);}
}

#[cfg(test)]mod recovery_regression{use super::*;#[test]fn nonfinite_mastery_is_neutral(){let mut a=CombatState::default();let mut b=CombatState::default();assert_eq!(a.attempt("mace",100,f64::NAN),b.attempt("mace",100,0.0));}}

/// One paid swing, shared by the server's critical, pre-damage and post-damage hooks.
#[derive(Clone, Debug)]
pub struct MeleeHit {
    pub victim: u32,
    kind: String,
    tick: u64,
    pub weapon: &'static str,
    pub sharpness: u64,
    critical: bool,
    damage_tick: Option<u64>,
}

impl MeleeHit {
    pub fn new(victim: u32, kind: &str, tick: u64, weapon: &'static str, sharpness: u64) -> Self {
        Self { victim, kind: kind.into(), tick, weapon, sharpness, critical: false, damage_tick: None }
    }
    pub fn observe_critical(&mut self, kind: &str, tick: u64, critical: bool) {
        if self.tick == tick && self.kind == kind && self.damage_tick.is_none() {
            self.critical = critical;
        }
    }
    pub fn begin(&mut self, victim: u32, tick: u64) -> bool {
        if victim != self.victim || !self.fresh(tick) || self.damage_tick.is_some() {
            return false;
        }
        self.damage_tick = Some(tick);
        true
    }
    pub fn matches_post(&self, victim: u32, tick: u64) -> bool {
        self.victim == victim && self.damage_tick == Some(tick)
    }
    pub fn fresh(&self, tick: u64) -> bool {
        tick.checked_sub(self.tick).is_some_and(|age| age <= 2)
    }
    pub fn critical(&self) -> bool { self.critical }
}

#[cfg(test)]
mod melee_hit_tests {
    use super::*;

    #[test]
    fn only_the_paid_target_can_consume_each_stage() {
        let mut hit = MeleeHit::new(7, "minecraft:zombie", 20, "sword", 4);
        hit.observe_critical("minecraft:spider", 20, true);
        hit.observe_critical("minecraft:zombie", 19, true);
        assert!(!hit.critical());
        hit.observe_critical("minecraft:zombie", 20, true);
        assert!(!hit.begin(8, 20));
        assert!(!hit.begin(7, 19));
        assert!(!hit.begin(7, 23));
        assert!(hit.begin(7, 22));
        assert!(!hit.begin(7, 22));
        hit.observe_critical("minecraft:zombie", 22, false);
        assert!(hit.critical());
        assert!(!hit.matches_post(8, 22));
        assert!(!hit.matches_post(7, 23));
        assert!(hit.matches_post(7, 22));
    }
}
