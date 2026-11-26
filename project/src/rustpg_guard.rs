//! Server-authoritative, latency-tolerant defensive action windows.
//! Inspired by defensive timings in Epic Fight / Better Combat, not a port of
//! either animation engine. No client can decide damage prevention.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, atomic::{AtomicU64,Ordering}};
use crate::rustpg_engine_stamina::Stamina;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Stance { Parry, Dodge }
impl Stance {
    pub fn name(self)->&'static str {match self {Self::Parry=>"aparar",Self::Dodge=>"esquivar"}}
    pub fn cost(self)->f64 {match self {Self::Parry=>14.0,Self::Dodge=>20.0}}
    pub fn window(self)->u64 {match self {Self::Parry=>8,Self::Dodge=>11}}
    pub fn cooldown(self)->u64 {match self {Self::Parry=>36,Self::Dodge=>58}}
    pub fn multiplier(self)->f32 {match self {Self::Parry=>0.40,Self::Dodge=>0.30}}
}
#[derive(Clone,Copy,Debug)]
struct Entry {stance:Stance,started:u64,until:u64,cooldown:u64,unused:bool,
    counter_ready:bool,counter_until:u64}
/// A prepared stance has no effect until published after profile persistence.
pub struct Pending {key:(String,u32),entry:Entry}
#[derive(Default)]
pub struct Engine {entries:HashMap<(String,u32),Entry>}
impl Engine {
    fn prepare_costed(&self,dim:&str,entity:u32,stance:Stance,now:u64,
                    stamina:&mut Stamina,cost:f64)->Result<Pending,String> {
        if dim.is_empty() || entity==0 {return Err("Entidade inválida".into());}
        let key=(dim.to_owned(),entity);
        if self.entries.get(&key).is_some_and(|v|now<v.cooldown) {
            return Err("Ação defensiva em recarga".into());
        }
        if stamina.exhausted || !stamina.current.is_finite() || stamina.current < cost {
            return Err("Stamina insuficiente para defender".into());
        }
        if !stamina.consume(cost) {return Err("Stamina insuficiente".into());}
        Ok(Pending{key,entry:Entry{stance,started:now,until:now.saturating_add(stance.window()),
                                     cooldown:now.saturating_add(stance.cooldown()),unused:true,
                                     counter_ready:false,counter_until:0}})
    }
    fn publish(&mut self,pending:Pending){self.entries.insert(pending.key,pending.entry);}
    pub fn activate_costed(&mut self,dim:&str,entity:u32,stance:Stance,now:u64,
                    stamina:&mut Stamina,cost:f64)->Result<(),String> {
        let pending=self.prepare_costed(dim,entity,stance,now,stamina,cost)?;
        self.publish(pending);
        Ok(())
    }
    pub fn activate(&mut self,dim:&str,entity:u32,stance:Stance,now:u64,
                    stamina:&mut Stamina)->Result<(),String> {
        self.activate_costed(dim,entity,stance,now,stamina,stance.cost())
    }
    /// A successful reduction consumes the window; fire, starvation, void,
    /// poison and unrelated sources cannot accidentally consume a parry.
    pub fn on_hit(&mut self,dim:&str,entity:u32,kind:&str,now:u64)->f32 {
        let Some(entry)=self.entries.get_mut(&(dim.to_owned(),entity))else{return 1.0};
        if !entry.unused || now>entry.until {return 1.0;}
        // Unknown and ambient damage do not consume a combat defense.
        let damage_type=kind.strip_prefix("minecraft:").unwrap_or(kind);
        let melee=matches!(damage_type,"player_attack"|"mob_attack"|"mob_attack_no_aggro"|"mace_smash"|"sting");
        let ranged=matches!(damage_type,"arrow"|"trident"|"fireball"|"mob_projectile"|
            "wither_skull"|"spit"|"thrown"|"wind_charge"|"fireworks"|"explosion"|"player_explosion");
        if !(melee || (entry.stance==Stance::Dodge && ranged)){return 1.0;}
        entry.unused=false;
        let elapsed=now.saturating_sub(entry.started);
        if entry.stance==Stance::Parry && elapsed<=3 {
            entry.counter_ready=true;entry.counter_until=now.saturating_add(20);
            0.15
        } else if entry.stance==Stance::Dodge && elapsed<=4 {
            0.0
        } else {entry.stance.multiplier()}
    }
    pub fn take_counter(&mut self,dim:&str,entity:u32,now:u64)->bool {
        let Some(entry)=self.entries.get_mut(&(dim.to_owned(),entity)) else{return false;};
        if entry.counter_ready && now<=entry.counter_until {entry.counter_ready=false;true}else{false}
    }
    pub fn remove(&mut self,dim:&str,entity:u32){self.entries.remove(&(dim.to_owned(),entity));}
    pub fn cleanup(&mut self,now:u64){self.entries.retain(|_,v|now<v.cooldown);}
}
static INSTANCE:OnceLock<Mutex<Engine>>=OnceLock::new();
static TICK:AtomicU64=AtomicU64::new(0);
fn inner()->&'static Mutex<Engine>{INSTANCE.get_or_init(||Mutex::new(Engine::default()))}
pub fn current_tick()->u64 {TICK.load(Ordering::Relaxed)}
pub fn set_tick(tick:u64){TICK.store(tick,Ordering::Relaxed);if tick%40==0 {
    inner().lock().unwrap_or_else(|e|e.into_inner()).cleanup(tick);
}}
pub fn prepare_costed(dim:&str,entity:u32,stance:Stance,stamina:&mut Stamina,cost:f64)->Result<(String,Pending),String>{
    let tick=TICK.load(Ordering::Relaxed);
    let cost=if cost.is_finite(){cost.clamp(0.05,stance.cost())}else{stance.cost()};
    let pending=inner().lock().unwrap_or_else(|e|e.into_inner())
        .prepare_costed(dim,entity,stance,tick,stamina,cost)?;
    let message=format!("{}: janela de {:.2} s, recarga {:.2} s",stance.name(),
        stance.window() as f64/20.0,stance.cooldown() as f64/20.0).replace('.',",");
    Ok((message,pending))
}
pub fn publish(pending:Pending){inner().lock().unwrap_or_else(|e|e.into_inner()).publish(pending);}
pub fn activate_costed(dim:&str,entity:u32,stance:Stance,stamina:&mut Stamina,cost:f64)->Result<String,String>{
    let (message,pending)=prepare_costed(dim,entity,stance,stamina,cost)?;
    publish(pending);Ok(message)
}
pub fn activate(dim:&str,entity:u32,stance:Stance,stamina:&mut Stamina)->Result<String,String>{
    activate_costed(dim,entity,stance,stamina,stance.cost())
}
pub fn on_hit(dim:&str,entity:u32,kind:&str)->f32 {
    let tick=TICK.load(Ordering::Relaxed);
    inner().lock().unwrap_or_else(|e|e.into_inner()).on_hit(dim,entity,kind,tick)
}
pub fn take_counter(dim:&str,entity:u32)->bool {
    let tick=TICK.load(Ordering::Relaxed);
    inner().lock().unwrap_or_else(|e|e.into_inner()).take_counter(dim,entity,tick)
}
pub fn remove(dim:&str,entity:u32){inner().lock().unwrap_or_else(|e|e.into_inner()).remove(dim,entity);}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn only_authorized_hits_reduce_and_reductions_are_one_shot(){
        let mut e=Engine::default();let mut s=Stamina::new(100.0,0.0,0);
        e.activate("overworld",7,Stance::Parry,100,&mut s).unwrap();
        assert_eq!(e.on_hit("overworld",7,"minecraft:arrow",102),1.0);
        assert_eq!(e.on_hit("overworld",7,"minecraft:mob_attack",103),0.15);
        assert!(e.take_counter("overworld",7,104));
        assert_eq!(e.on_hit("overworld",7,"minecraft:mob_attack",104),1.0);
        assert!(e.activate("overworld",7,Stance::Dodge,110,&mut s).is_err());
        e.activate("overworld",7,Stance::Dodge,136,&mut s).unwrap();
        assert_eq!(e.on_hit("overworld",7,"minecraft:arrow",140),0.0);
        assert_eq!(e.on_hit("overworld",7,"minecraft:lava",141),1.0);
    }
    #[test] fn prepared_guard_is_inert_until_profile_save(){
        let mut e=Engine::default();let saved=Stamina::new(100.0,0.0,0);
        let mut candidate=saved;
        let pending=e.prepare_costed("overworld",7,Stance::Parry,100,&mut candidate,14.0).unwrap();
        assert_eq!(saved.current,100.0);
        assert_eq!(candidate.current,86.0);
        assert_eq!(e.on_hit("overworld",7,"minecraft:mob_attack",101),1.0);
        e.publish(pending);
        assert_eq!(e.on_hit("overworld",7,"minecraft:mob_attack",101),0.15);
    }
    #[test] fn cost_is_never_spent_for_rejected_requests(){
        let mut e=Engine::default();let mut s=Stamina::new(100.0,0.0,0);
        e.activate("overworld",7,Stance::Parry,100,&mut s).unwrap();
        let balance=s.current;
        assert!(e.activate("overworld",7,Stance::Parry,102,&mut s).is_err());
        assert_eq!(s.current,balance);
    }
}

#[cfg(test)] mod reduced_cost_tests {
    use super::*;
    #[test] fn reduced_dodge_cost_is_consumed_by_server_engine() {
        let mut e=Engine::default();let mut s=Stamina::new(100.0,0.0,0);
        e.activate_costed("overworld",42,Stance::Dodge,1,&mut s,5.0).unwrap();
        assert!((s.current-95.0).abs()<0.0001);
    }
}

#[cfg(test)]mod source_regression{
    use super::*;
    #[test]fn environmental_and_unknown_damage_do_not_consume_defense(){
        let mut e=Engine::default();let mut s=Stamina::new(100.0,0.0,0);
        e.activate("overworld",7,Stance::Parry,100,&mut s).unwrap();
        for kind in ["minecraft:generic_kill","minecraft:in_wall","minecraft:hot_floor","minecraft:cramming","mod:mob_attack","minecraft:fireworks"]{
            assert_eq!(e.on_hit("overworld",7,kind,101),1.0);
        }
        assert_eq!(e.on_hit("overworld",7,"minecraft:player_attack",102),0.15);
    }
}
