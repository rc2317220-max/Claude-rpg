//! Claude RPG independent skill-effect kernel.
//!
//! Game adapters supply *server-confirmed* events; the kernel never edits a
//! client profile or awards a drop by looking at the HUD. All effects are
//! bounded and deterministic for a given game event, and every stat uses its
//! own key. Extensions can call the same functions from an API hook, Part,
//! injected listener or version-pinned native mixin.
use crate::crpg_attributes::{self,Unit};
use crate::crpg_profile::Stats;

/// A stable channel for interoperating with other Claude RPG modules/mods.
/// The channel is a semantic destination, NOT a claim that a game hook already
/// exists or that a skill has been integrated. See `AttrDef.applied`.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Channel {
    Progression,
    Combat,
    Defense,
    Survival,
    Resource,
    Fishing,
    Farming,
    Enchanting,
    Crafting,
    Equipment,
    Exploration,
    Economy,
    Luck,
}
pub fn channel(key:&str)->Option<Channel>{
    let attr=crpg_attributes::get(key)?;
    let result=if key.starts_with("weapon.") {Channel::Combat}
        else if key.starts_with("defense.") {Channel::Defense}
        else if key.starts_with("luck.") {Channel::Luck}
        else if key.starts_with("fishing.") {Channel::Fishing}
        else if key.starts_with("farming.") {Channel::Farming}
        else if key.starts_with("mining.") || key.starts_with("woodcutting.")
            || key.starts_with("excavation.") {Channel::Resource}
        else if key.starts_with("enchanting.") {Channel::Enchanting}
        else if key.starts_with("crafting.") || key.starts_with("cooking.")
            || key.starts_with("smithing.") || key.starts_with("alchemy.") {Channel::Crafting}
        else if key.starts_with("survival.") {Channel::Survival}
        else if key.starts_with("exploration.") {Channel::Exploration}
        else if key.starts_with("trading.") {Channel::Economy}
        else if key.starts_with("taming.") || key.starts_with("husbandry.")
            || key.starts_with("hunting.") || key.starts_with("looting.") {Channel::Equipment}
        else if key.starts_with("general.") {Channel::Progression}
        else {match attr.category {
            "combat"=>Channel::Combat,"defense"=>Channel::Defense,
            "profession"=>Channel::Resource,_=>Channel::Progression,
        }};
    Some(result)
}
pub const BUS_ABI:u16=1;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Signal {
    PlayerLevelUp, ExperienceOrbPickup, FoodLevelChange,
    FishingRodWear, PlayerEquipmentSync, IncomingHit, OutgoingHit,
    GatherCompleted, CatchCompleted, ItemCrafted, ItemEnchanted,
}
#[derive(Clone,Debug)]
pub struct EffectView<'a>{
    pub stats:&'a Stats,
    pub is_server:bool,
}
impl EffectView<'_>{
    /// A zero default means no effect. Never trust the client to set ranks.
    pub fn value(&self,key:&str)->f64{
        if !self.is_server || crpg_attributes::get(key).is_none(){return 0.0;}
        let v=self.stats.get(key);
        if !v.is_finite(){return 0.0;}
        let cap=crpg_attributes::get(key).and_then(|a|a.cap).unwrap_or(1_000_000.0);
        v.clamp(0.0,cap)
    }
}
pub fn fmt_value(value:f64,unit:Unit)->String {
    let value=if value.is_finite(){value.max(0.0)}else{0.0};
    match unit{
        Unit::Percent|Unit::Chance|Unit::Reduction =>
            format!("{:.3}%",value*100.0),
        Unit::Count=>format!("{:.3} unidades",value),
        Unit::Flat=>format!("{:.3} pontos",value),
    }
}
/// Sane detailed values for every authored node without hiding capped
/// progression behind the formula's unbounded symbolic increment.
pub fn node_detail(node:&crate::crpg_nodes::NodeDef,rank:u64,stats:&Stats)->String{
    let Some(def)=crpg_attributes::get(&node.attribute)else{
        return node.description.clone();
    };
    let current=node.formula.value(rank);
    let next=node.formula.value(rank.saturating_add(1));
    let global=stats.get(&node.attribute);
    let delta=if let Some(cap)=def.cap {
        next.min((cap-global+current).max(current))-current
    }else{next-current};
    format!(
        "{}\\nAtributo: {} [{}]\\nEste nó: {} | Próximo: {} | Incremento: {}\\nTotal efetivo: {}\\nIntegração: {}",
        node.description,def.name,def.id,
        fmt_value(current,def.unit),fmt_value(next,def.unit),
        fmt_value(delta.max(0.0),def.unit),fmt_value(global,def.unit),
        if def.applied{"conectado ao gameplay"}else{"aguarda adaptador runtime"}
    )
}
/// Bounded vanilla XP orb reward (after Mending is still the engine's
/// responsibility). A negative or empty amount never becomes positive.
pub fn orb_value(native:i32,percent:f64)->i32{
    if native<=0{return native;}
    let bonus=if percent.is_finite(){percent.clamp(0.0,100.0)}else{0.0};
    ((native as f64)*(1.0+bonus)).round().clamp(1.0,i32::MAX as f64) as i32
}
/// Stable one-integer roll; never uses the visual client state and does not
/// alter vanilla's world RNG sequence. Seed comes from server identity+tick.
fn roll(seed:u64,index:u64)->f64{
    let mut x=seed.wrapping_add(index.wrapping_mul(0x9E3779B97F4A7C15));
    x^=x>>30;x=x.wrapping_mul(0xBF58476D1CE4E5B9);
    x^=x>>27;x=x.wrapping_mul(0x94D049BB133111EB);
    x^=x>>31;
    (x>>11) as f64/(1u64<<53) as f64
}
/// Only suppresses a fraction of actual *losses*. Eating and positive
/// hunger changes pass untouched, and creative bypass is handled by adapter.
pub fn food_after(from:i32,to:i32,reduction:f64,seed:u64)->i32{
    let from=from.clamp(0,20);let to=to.clamp(0,20);
    if to>=from{return to;}
    let r=if reduction.is_finite(){reduction.clamp(0.0,0.90)}else{0.0};
    let mut lost=0;
    for i in 0..(from-to){
        if roll(seed,i as u64)>=r {lost+=1;}
    }
    (from-lost).clamp(to,from)
}
pub fn rod_wear(native:i32,efficiency:f64,seed:u64)->i32{
    if native<=0{return native.max(0);}
    let p=if efficiency.is_finite(){efficiency.clamp(0.0,1.0)}else{0.0};
    (0..native.min(1000)).filter(|&i|roll(seed,i as u64)>=p).count() as i32
}
/// Fractional general-points accrual; an attribute of 0.02 really gives
/// 0.02 extra point on EACH next level, retaining 1/1000 fractions in saves.
pub fn point_credit(extra_per_level:f64,milli:u16)->(u64,u16){
    let v=if extra_per_level.is_finite(){extra_per_level.clamp(0.0,100.0)}else{0.0};
    let total=(v*1000.0).round() as u64+(milli%1000) as u64;
    (total/1000,(total%1000) as u16)
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn all_nodes_and_attributes_have_a_single_recognized_channel(){
        for a in crpg_attributes::all(){
            assert!(channel(&a.id).is_some(),"unrouted attribute {}",a.id);
        }
        for n in crate::crpg_nodes::all(){
            if let Some(a)=crpg_attributes::get(&n.attribute){
                assert!(channel(&a.id).is_some(), "node {}",n.id);
                assert!(node_detail(n,0,&Stats::default()).contains("Próximo:"));
            }else{
                assert!(n.attribute.starts_with("power.")||n.attribute=="enchanting.transcendence");
            }
        }
    }
    #[test]fn independent_maths_are_bounded_and_client_cannot_grant_stats(){
        let mut s=Stats::default();s.add("general.orb_xp",2.0);
        assert_eq!(EffectView{stats:&s,is_server:false}.value("general.orb_xp"),0.0);
        assert_eq!(EffectView{stats:&s,is_server:true}.value("general.orb_xp"),2.0);
        assert_eq!(orb_value(10,0.5),15);assert_eq!(orb_value(-2,2.0),-2);
        assert_eq!(orb_value(i32::MAX,1000.0),i32::MAX);
        assert_eq!(food_after(15,17,0.8,1),17);
        assert_eq!(food_after(15,11,1.0,1),11); // reduction limited to 90%
        assert_eq!(rod_wear(2,1.0,1),0);
        assert_eq!(rod_wear(1,0.0,1),1);
        assert_eq!(point_credit(0.02,980),(1,0));
        assert_eq!(point_credit(f64::NAN,200),(0,200));
    }
}
