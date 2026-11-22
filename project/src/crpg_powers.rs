//! Server-only active powers; unlocked from actual skill-tree purchases.
//! All costs, cooldowns and durations are enforced by the server.
use std::{collections::HashMap,sync::{Mutex,OnceLock}};
use crate::crpg_profile::Profile;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum Power {Fury,Barrier,Siphon}
impl Power {
    pub fn id(self)->&'static str {match self{Self::Fury=>"power.fury",Self::Barrier=>"power.barrier",Self::Siphon=>"power.siphon"}}
    pub fn cost(self)->f64 {match self{Self::Fury=>25.0,Self::Barrier=>28.0,Self::Siphon=>32.0}}
    pub fn duration(self)->u64 {match self{Self::Fury=>80,Self::Barrier=>100,Self::Siphon=>100}}
    pub fn cooldown(self)->u64 {match self{Self::Fury=>240,Self::Barrier=>280,Self::Siphon=>320}}
}
#[derive(Clone,Copy,Debug)]
struct Active {kind:Power,until:u64}
#[derive(Default)]pub struct Engine {
    active:HashMap<(String,u32),Active>,
    cooldowns:HashMap<(String,u32,Power),u64>,
}
impl Engine {
    fn activate(&mut self,dim:&str,id:u32,kind:Power,now:u64,profile:&mut Profile)->Result<String,String>{
        if dim.is_empty()||id==0{return Err("Entidade inválida".into())}
        if profile.rank(kind.id())==0{return Err("Adquira o node na árvore Poderes".into())}
        let key=(dim.to_owned(),id,kind);
        if self.cooldowns.get(&key).is_some_and(|t|*t>now){return Err("Poder em recarga".into())}
        if profile.stamina.exhausted || profile.stamina.current<kind.cost(){
            return Err("Stamina insuficiente".into())
        }
        if !profile.stamina.consume(kind.cost()){return Err("Stamina insuficiente".into())}
        self.cooldowns.insert(key,now.saturating_add(kind.cooldown()));
        self.active.insert((dim.to_owned(),id),Active{kind,until:now.saturating_add(kind.duration())});
        Ok(format!("Poder {}: duração {} ticks; recarga {} ticks",kind.id(),kind.duration(),kind.cooldown()))
    }
    fn current(&self,dim:&str,id:u32,now:u64)->Option<Power>{
        self.active.get(&(dim.to_owned(),id)).filter(|a|now<=a.until).map(|a|a.kind)
    }
    fn cleanup(&mut self,now:u64){
        self.active.retain(|_,a|a.until>=now);
        self.cooldowns.retain(|_,until|until.saturating_add(1)>now);
    }
    fn forget(&mut self,dim:&str,id:u32){
        self.active.remove(&(dim.to_owned(),id));
        self.cooldowns.retain(|(d,e,_),_|d!=dim||*e!=id);
    }
}
static ENGINE:OnceLock<Mutex<Engine>>=OnceLock::new();
fn engine()->&'static Mutex<Engine>{ENGINE.get_or_init(||Mutex::new(Engine::default()))}
pub fn activate(dim:&str,id:u32,kind:Power,p:&mut Profile)->Result<String,String>{
    let now=crate::crpg_guard::current_tick();
    engine().lock().unwrap_or_else(|e|e.into_inner()).activate(dim,id,kind,now,p)
}
pub fn outgoing(dim:&str,id:u32)->f32 {
    let now=crate::crpg_guard::current_tick();
    let e=engine().lock().unwrap_or_else(|e|e.into_inner());
    if e.current(dim,id,now)==Some(Power::Fury){1.25}else{1.0}
}
pub fn incoming(dim:&str,id:u32)->f32 {
    let now=crate::crpg_guard::current_tick();
    let e=engine().lock().unwrap_or_else(|e|e.into_inner());
    if e.current(dim,id,now)==Some(Power::Barrier){0.72}else{1.0}
}
pub fn siphon(dim:&str,id:u32,actual:f32)->f32 {
    if !actual.is_finite()||actual<=0.0 {return 0.0}
    let now=crate::crpg_guard::current_tick();
    let e=engine().lock().unwrap_or_else(|e|e.into_inner());
    if e.current(dim,id,now)==Some(Power::Siphon){(actual*0.09).min(4.0)}else{0.0}
}
pub fn cleanup(now:u64){if now%40==0 {engine().lock().unwrap_or_else(|e|e.into_inner()).cleanup(now)}}
pub fn remove(dim:&str,id:u32){engine().lock().unwrap_or_else(|e|e.into_inner()).forget(dim,id)}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn power_requires_node_stamina_and_cooldown(){
        let mut e=Engine::default();let mut p=Profile::new();
        assert!(e.activate("overworld",10,Power::Fury,100,&mut p).is_err());
        p.ranks.insert("power.fury".into(),1);
        e.activate("overworld",10,Power::Fury,100,&mut p).unwrap();
        assert_eq!(e.current("overworld",10,180),Some(Power::Fury));
        assert!(e.activate("overworld",10,Power::Fury,180,&mut p).is_err());
        assert_eq!(e.current("overworld",10,181),None);
    }
}
