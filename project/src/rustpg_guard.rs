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
#[derive(Default)]
pub struct Engine {entries:HashMap<(String,u32),Entry>}
impl Engine {
    pub fn activate_costed(&mut self,dim:&str,entity:u32,stance:Stance,now:u64,
                    stamina:&mut Stamina,cost:f64)->Result<(),String> {
        if dim.is_empty() || entity==0 {return Err("Entidade inválida".into());}
        let key=(dim.to_owned(),entity);
        if self.entries.get(&key).is_some_and(|v|now<v.cooldown) {
            return Err("Ação defensiva em recarga".into());
        }
        if stamina.exhausted || !stamina.current.is_finite() || stamina.current < cost {
            return Err("Stamina insuficiente para defender".into());
        }
        if !stamina.consume(cost) {return Err("Stamina insuficiente".into());}
        self.entries.insert(key,Entry{stance,started:now,until:now.saturating_add(stance.window()),
                                     cooldown:now.saturating_add(stance.cooldown()),unused:true,
                                     counter_ready:false,counter_until:0});
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
        let damage_type=kind.rsplit(':').next().unwrap_or(kind);
        let blocked=matches!(damage_type,"fall"|"fire"|"in_fire"|"on_fire"|
            "lava"|"wither"|"starve"|"drown"|"freeze"|"out_of_world"|
            "magic"|"indirect_magic"|"thorns"|"cactus"|"sweet_berry_bush");
        if blocked {return 1.0;}
        let ranged=matches!(damage_type,"arrow"|"trident"|"fireball"|"mob_projectile"|
            "unattributed_fireball"|"explosion"|"player_explosion");
        if entry.stance==Stance::Parry && ranged{return 1.0;}
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
pub fn activate_costed(dim:&str,entity:u32,stance:Stance,stamina:&mut Stamina,cost:f64)->Result<String,String>{
    let tick=TICK.load(Ordering::Relaxed);
    let cost=if cost.is_finite(){cost.clamp(0.05,stance.cost())}else{stance.cost()};
    inner().lock().unwrap_or_else(|e|e.into_inner())
        .activate_costed(dim,entity,stance,tick,stamina,cost)?;
    Ok(format!("{}: janela de {} ticks, recarga {} ticks",stance.name(),
        stance.window(),stance.cooldown()))
}
pub fn activate(dim:&str,entity:u32,stance:Stance,stamina:&mut Stamina)->Result<String,String>{
    activate_costed(dim,entity,stance,stamina,stance.cost())
}
/// Prepare a defense without creating a live damage window.
pub fn prepare_persistent(stance:Stance,p:&mut crate::rustpg_profile::Profile,cost:f64)->Result<String,String>{
    let cost=if cost.is_finite(){cost.clamp(0.05,stance.cost())}else{stance.cost()};
    if p.guard_cooldown>0{return Err("Ação defensiva em recarga".into());}
    if p.stamina.exhausted || !p.stamina.current.is_finite() || p.stamina.current<cost{return Err("Vigor insuficiente".into());}
    if !p.stamina.consume(cost){return Err("Vigor insuficiente".into());}
    p.guard_cooldown=stance.cooldown() as u32;
    Ok(format!("{}: janela de {} ticks, recarga {} ticks",stance.name(),stance.window(),stance.cooldown()))
}
pub fn publish(dim:&str,id:u32,stance:Stance){
    if dim.is_empty() || id==0{return;}
    let now=current_tick();
    inner().lock().unwrap_or_else(|e|e.into_inner()).entries.insert((dim.to_owned(),id),
        Entry{stance,started:now,until:now.saturating_add(stance.window()),cooldown:now.saturating_add(stance.cooldown()),unused:true,counter_ready:false,counter_until:0});
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
