//! Server-only active powers; unlocked from actual skill-tree purchases.
//! All costs, cooldowns and durations are enforced by the server.
use std::{collections::HashMap,sync::{Mutex,OnceLock}};
use crate::rustpg_profile::Profile;
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub enum Power {Fury,Barrier,Siphon}
impl Power {
    pub fn id(self)->&'static str {match self{Self::Fury=>"power.fury",Self::Barrier=>"power.barrier",Self::Siphon=>"power.siphon"}}
    pub fn name(self)->&'static str {match self{Self::Fury=>"Fúria",Self::Barrier=>"Barreira",Self::Siphon=>"Sifão"}}
    pub fn cost(self)->f64 {match self{Self::Fury=>25.0,Self::Barrier=>28.0,Self::Siphon=>32.0}}
    pub fn duration(self)->u64 {match self{Self::Fury=>80,Self::Barrier=>100,Self::Siphon=>100}}
    pub fn cooldown(self)->u64 {match self{Self::Fury=>240,Self::Barrier=>280,Self::Siphon=>320}}
}
#[derive(Clone,Copy,Debug)]
struct Active {kind:Power,until:u64}
pub struct Pending {key:(u128,Power),entity:(String,u32),active:Active,cooldown:u64}
#[derive(Default)]pub struct Engine {
    active:HashMap<(String,u32),Active>,
    cooldowns:HashMap<(u128,Power),u64>,
}
impl Engine {
    fn prepare_costed(&self,uuid:u128,dim:&str,id:u32,kind:Power,now:u64,profile:&mut Profile,cost:f64)->Result<(String,Pending),String>{
        if dim.is_empty()||id==0{return Err("Entidade inválida".into())}
        if profile.rank(kind.id())==0{return Err("Adquira a habilidade na árvore de Poderes".into())}
        let key=(uuid,kind);
        if self.cooldowns.get(&key).is_some_and(|t|*t>now){return Err("Poder em recarga".into())}
        if profile.stamina.exhausted || profile.stamina.current<cost{
            return Err("Stamina insuficiente".into())
        }
        if !profile.stamina.consume(cost){return Err("Stamina insuficiente".into())}
        let pending=Pending{key,entity:(dim.to_owned(),id),active:Active{kind,until:now.saturating_add(kind.duration())},cooldown:now.saturating_add(kind.cooldown())};
        let message=format!("{}: duração {:.1} s; recarga {:.1} s",kind.name(),kind.duration() as f64/20.0,kind.cooldown() as f64/20.0).replace('.',",");
        Ok((message,pending))
    }
    fn publish(&mut self,pending:Pending){
        self.cooldowns.insert(pending.key,pending.cooldown);
        self.active.insert(pending.entity,pending.active);
    }
    fn activate_costed(&mut self,uuid:u128,dim:&str,id:u32,kind:Power,now:u64,profile:&mut Profile,cost:f64)->Result<String,String>{
        let (message,pending)=self.prepare_costed(uuid,dim,id,kind,now,profile,cost)?;
        self.publish(pending);Ok(message)
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
        // Cooldowns belong to the player, not the transient entity.
    }
}
static ENGINE:OnceLock<Mutex<Engine>>=OnceLock::new();
fn engine()->&'static Mutex<Engine>{ENGINE.get_or_init(||Mutex::new(Engine::default()))}
pub fn prepare_costed(uuid:u128,dim:&str,id:u32,kind:Power,p:&mut Profile,cost:f64)->Result<(String,Pending),String>{
    let now=crate::rustpg_guard::current_tick();
    let cost=if cost.is_finite(){cost.clamp(0.05,kind.cost())}else{kind.cost()};
    engine().lock().unwrap_or_else(|e|e.into_inner()).prepare_costed(uuid,dim,id,kind,now,p,cost)
}
pub fn publish(pending:Pending){engine().lock().unwrap_or_else(|e|e.into_inner()).publish(pending);}
pub fn activate_costed(uuid:u128,dim:&str,id:u32,kind:Power,p:&mut Profile,cost:f64)->Result<String,String>{
    let (message,pending)=prepare_costed(uuid,dim,id,kind,p,cost)?;publish(pending);Ok(message)
}
pub fn outgoing(dim:&str,id:u32)->f32 {
    let now=crate::rustpg_guard::current_tick();
    let e=engine().lock().unwrap_or_else(|e|e.into_inner());
    if e.current(dim,id,now)==Some(Power::Fury){1.25}else{1.0}
}
pub fn incoming(dim:&str,id:u32)->f32 {
    let now=crate::rustpg_guard::current_tick();
    let e=engine().lock().unwrap_or_else(|e|e.into_inner());
    if e.current(dim,id,now)==Some(Power::Barrier){0.72}else{1.0}
}
pub fn siphon(dim:&str,id:u32,actual:f32)->f32 {
    if !actual.is_finite()||actual<=0.0 {return 0.0}
    let now=crate::rustpg_guard::current_tick();
    let e=engine().lock().unwrap_or_else(|e|e.into_inner());
    if e.current(dim,id,now)==Some(Power::Siphon){(actual*0.09).min(4.0)}else{0.0}
}
pub fn cleanup(now:u64){if now%40==0 {engine().lock().unwrap_or_else(|e|e.into_inner()).cleanup(now)}}
pub fn remove(dim:&str,id:u32){engine().lock().unwrap_or_else(|e|e.into_inner()).forget(dim,id)}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn prepared_power_does_not_publish_effect_or_cooldown(){
        let mut e=Engine::default();let mut saved=Profile::new();
        saved.ranks.insert("power.fury".into(),1);
        let mut candidate=saved.clone();
        let (_,pending)=e.prepare_costed(123,"overworld",7,Power::Fury,100,&mut candidate,25.0).unwrap();
        assert_eq!(saved.stamina.current,100.0);
        assert_eq!(candidate.stamina.current,75.0);
        assert_eq!(e.current("overworld",7,101),None);
        assert!(e.cooldowns.is_empty());
        e.publish(pending);
        assert_eq!(e.current("overworld",7,101),Some(Power::Fury));
        assert!(e.cooldowns.contains_key(&(123,Power::Fury)));
    }
    #[test]fn cooldown_survives_entity_and_dimension_change(){
        let mut e=Engine::default();let mut p=Profile::new();
        p.ranks.insert("power.fury".into(),1);
        e.activate_costed(123,"overworld",10,Power::Fury,100,&mut p,25.0).unwrap();
        e.forget("overworld",10);
        assert!(e.activate_costed(123,"nether",20,Power::Fury,101,&mut p,25.0).is_err());
        let mut other=Profile::new();
        other.ranks.insert("power.fury".into(),1);
        assert!(e.activate_costed(456,"overworld",10,Power::Fury,101,&mut other,25.0).is_ok());
        assert!(e.activate_costed(123,"nether",20,Power::Fury,340,&mut p,25.0).is_ok());
    }
    #[test]fn power_requires_node_stamina_and_cooldown(){
        let mut e=Engine::default();let mut p=Profile::new();
        assert!(e.activate_costed(123,"overworld",10,Power::Fury,100,&mut p,Power::Fury.cost()).is_err());
        p.ranks.insert("power.fury".into(),1);
        e.activate_costed(123,"overworld",10,Power::Fury,100,&mut p,Power::Fury.cost()).unwrap();
        assert_eq!(e.current("overworld",10,180),Some(Power::Fury));
        assert!(e.activate_costed(123,"overworld",10,Power::Fury,180,&mut p,Power::Fury.cost()).is_err());
        assert_eq!(e.current("overworld",10,181),None);
    }
}

#[cfg(test)] mod cost_node_tests {
    use super::*;
    #[test]fn server_supplied_skill_cost_changes_real_stamina_spend(){
        let mut e=Engine::default();let mut p=Profile::new();
        p.ranks.insert("power.fury".into(),1);
        e.activate_costed(123,"overworld",77,Power::Fury,1,&mut p,10.0).unwrap();
        assert!((p.stamina.current-90.0).abs()<0.0001);
    }
}
