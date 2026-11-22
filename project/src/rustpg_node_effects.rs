//! RustPG independent skill-effect kernel.
//!
//! Game adapters supply *server-confirmed* events; the kernel never edits a
//! client profile or awards a drop by looking at the HUD. All effects are
//! bounded and deterministic for a given game event, and every stat uses its
//! own key. Extensions can call the same functions from an API hook, Part,
//! stable public hook or Part.
use crate::rustpg_attributes::{self,Unit};
use crate::rustpg_profile::Stats;

/// A stable channel for interoperating with other RustPG modules/mods.
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
    let attr=rustpg_attributes::get(key)?;
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
        if !self.is_server || rustpg_attributes::get(key).is_none(){return 0.0;}
        let v=self.stats.get(key);
        if !v.is_finite(){return 0.0;}
        let cap=rustpg_attributes::get(key).and_then(|a|a.cap).unwrap_or(1_000_000.0);
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
pub fn node_detail(node:&crate::rustpg_nodes::NodeDef,rank:u64,stats:&Stats)->String{
    let Some(def)=rustpg_attributes::get(&node.attribute)else{
        return node.description.clone();
    };
    let current=node.formula.value(rank);
    let next=node.formula.value(rank.saturating_add(1));
    let global=stats.get(&node.attribute);
    let delta=if let Some(cap)=def.cap {
        next.min((cap-global+current).max(current))-current
    }else{next-current};
    let maximum=node.max_rank.map(|max|fmt_value(node.formula.value(max),def.unit))
        .unwrap_or_else(||"Sem teto próprio".to_string());
    format!(
        "{}\nAtributo: {} [{}]\nAtual: {} | Próximo: {} | Incremento: {}\nNo máximo: {}\nTotal efetivo: {}\nIntegração: {}",
        node.description,def.name,def.id,
        fmt_value(current,def.unit),fmt_value(next,def.unit),
        fmt_value(delta.max(0.0),def.unit),maximum,fmt_value(global,def.unit),
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
/// All XP routes use the same bounded arithmetic. A percent is an
/// independent node's fractional contribution (0.01 = +1%), not a whole
/// category-wide perk. Saturation prevents save corruption on huge ranks.
pub fn scaled_xp(native:u64,percent:f64)->u64{
    if native==0{return 0;}
    let bonus=if percent.is_finite(){percent.max(0.0).min(10_000.0)}else{0.0};
    ((native as f64)*(1.0+bonus)).round().min(u64::MAX as f64) as u64
}
/// Explicit, authoritative whitelist of resource-specific mining XP keys.
/// Silk Touch, placed ores and anti-loop checks are performed by the caller
/// before this function is invoked, never by the UI.
pub fn mined_ore_key(state:&str)->Option<&'static str>{
    let id=state.split('[').next().unwrap_or(state);
    let id=id.strip_prefix("minecraft:")?;
    let name=id.strip_prefix("deepslate_").unwrap_or(id);
    match name {
        "coal_ore"=>Some("coal"),"copper_ore"=>Some("copper"),
        "iron_ore"=>Some("iron"),"gold_ore"|"nether_gold_ore"=>Some("gold"),
        "redstone_ore"=>Some("redstone"),"lapis_ore"=>Some("lapis"),
        "diamond_ore"=>Some("diamond"),"emerald_ore"=>Some("emerald"),
        "nether_quartz_ore"=>Some("quartz"),"ancient_debris"=>Some("netherite"),
        "amethyst_cluster"=>Some("amethyst"),
        _=>None,
    }
}
pub fn mature_crop_key(state:&str)->Option<&'static str>{
    let reward=crate::rustpg_gathering_rules::resource(state)?;
    if reward.profession!="farming"{return None;}
    match state.split('[').next().unwrap_or(state) {
        "minecraft:wheat"=>Some("wheat"),"minecraft:carrots"=>Some("carrots"),
        "minecraft:potatoes"=>Some("potatoes"),
        "minecraft:beetroots"=>Some("beetroots"),
        "minecraft:nether_wart"=>Some("nether_wart"),
        "minecraft:sweet_berry_bush"=>Some("sweet_berry_bush"),
        "minecraft:cocoa"=>Some("cocoa"),
        _=>None,
    }
}
pub fn resource_xp(stats:&Stats,career:&str,state:&str,amount:u64)->u64{
    let resource=match career {
        "mining"=>mined_ore_key(state),
        "farming"=>mature_crop_key(state),
        _=>None,
    };
    let extra=resource.map(|name|stats.get(&format!("{career}.{}.{}.xp",
        if career=="mining" {"ore"}else{"crop"},name))).unwrap_or(0.0);
    scaled_xp(amount,extra)
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
/// The final damage modifier runs AFTER vanilla armor/shields and BEFORE HP.
/// A melee weapon is passed only after a confirmed server attack_entity record;
/// never infer it from the player's current held item on an arrow or DoT.
/// All stat keys are independent. Mob tiers/health come from live snapshots.
pub fn damage_after_armor(
    amount:f32,
    armor_reduction:f32,
    melee:Option<(&Stats,&str)>,
    defender:Option<&Stats>,
    damage_type:&str,
    victim_kind:&str,
    victim_tier:Option<crate::rustpg_scaling::Tier>,
    attacker_tier:Option<crate::rustpg_scaling::Tier>,
    victim_health_fraction:Option<f64>,
    critical_attack:bool,
    seed:u64,
)->f32 {
    if !amount.is_finite() || amount<=0.0 {return amount;}
    // Void, commands and starvation should never become a dodge/block reward.
    if matches!(damage_type.rsplit(':').next().unwrap_or(""),
        "out_of_world"|"generic_kill"|"starve") {return amount;}
    let mut value=amount as f64;
    if let Some((s,weapon))=melee {
        let attr=|name:&str|->f64 {
            let key=format!("weapon.{weapon}.{name}");
            let n=s.get(&key);
            if n.is_finite(){n.max(0.0).min(20.0)}else{0.0}
        };
        // Armor penetration restores only the portion the engine explicitly
        // classified as Armor reduction. It never bypasses shields, absorption,
        // resistance effects or our own typed-defense layer.
        let armor=if armor_reduction.is_finite(){f64::from(armor_reduction.max(0.0))}else{0.0};
        value+=armor*attr("armor_pen").clamp(0.0,1.0);
        value=(value+attr("damage_flat").min(2048.0))*
            (1.0+attr("damage_pct").min(5.0));
        if let Some(tier)=victim_tier {
            use crate::rustpg_scaling::Tier;
            if tier>=Tier::MiniBoss {value*=1.0+attr("vs_boss").min(3.0);}
            else if tier>=Tier::Elite {value*=1.0+attr("vs_elite").min(3.0);}
        }
        if crate::rustpg_combat::is_flying(victim_kind) {
            value*=1.0+attr("vs_flying").min(3.0);
        }
        if victim_health_fraction.is_some_and(|hp|hp.is_finite()&&hp<=0.5) {
            value*=1.0+attr("execute").min(3.0);
        }
        if roll(seed,0)<attr("crit_chance").min(1.0) {
            value*=1.5+attr("crit_mult").min(3.0);
        }
    }
    if let Some(s)=defender {
        // Incoming mitigation is *post-armor*; never re-run the vanilla armor
        // formula or divide by max health (native max HP is already modified).
        let stat=|key:&str|->f64 {
            let n=s.get(key);
            if n.is_finite(){n.max(0.0).min(0.85)}else{0.0}
        };
        let key=crate::rustpg_combat::reduction_attr(damage_type);
        value*=1.0-stat(key);
        let tier=attacker_tier;
        if let Some(tier)=tier {
            use crate::rustpg_scaling::Tier;
            if tier>=Tier::MiniBoss {value*=1.0-stat("defense.res_boss");}
            else if tier>=Tier::Elite {value*=1.0-stat("defense.res_elite");}
        }
        if critical_attack {
            value*=1.0-stat("defense.res_crit");
        }
        if roll(seed,1)<stat("defense.block").min(0.75) {
            value*=0.20; // successful RPG block leaves a bounded scratch
        }
    }
    if value.is_finite(){value.max(0.0).min(1.0e9) as f32}else{amount}
}
/// A successful weapon hit can schedule server-side bleed ticks. The returned
/// tuple is (damage per second, number of one-second ticks). Bleed is generated
/// only from a server-confirmed weapon snapshot; delayed generic bleed damage
/// therefore cannot recursively roll another bleed.
pub fn bleed_plan(stats:&Stats,weapon:&str,seed:u64)->Option<(f32,u16)> {
    let key=|name:&str|->f64{
        let n=stats.get(&format!("weapon.{weapon}.{name}"));
        if n.is_finite(){n.max(0.0)}else{0.0}
    };
    let chance=key("bleed_chance").clamp(0.0,1.0);
    if chance<=0.0 || roll(seed,7)>=chance{return None;}
    let damage=(0.5+key("bleed_damage").min(200.0)).clamp(0.5,200.5) as f32;
    let seconds=(2.0+key("bleed_duration").min(38.0)).ceil().clamp(1.0,40.0) as u16;
    Some((damage,seconds))
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn all_nodes_and_attributes_have_a_single_recognized_channel(){
        for a in rustpg_attributes::all(){
            assert!(channel(&a.id).is_some(),"unrouted attribute {}",a.id);
        }
        for n in crate::rustpg_nodes::all(){
            if let Some(a)=rustpg_attributes::get(&n.attribute){
                assert!(channel(&a.id).is_some(), "node {}",n.id);
                let detail=node_detail(n,0,&Stats::default()); assert!(detail.contains("Próximo:")); assert!(detail.contains("No máximo:"));
            }else{
                assert!(n.attribute.starts_with("power.")||n.attribute=="enchanting.transcendence");
            }
        }
    }
    #[test]fn ore_and_mature_crop_xp_never_credit_replanted_or_nonresource_blocks(){
        let mut stats=Stats::default();
        stats.add("mining.ore.diamond.xp",0.5);
        stats.add("farming.crop.wheat.xp",0.25);
        assert_eq!(resource_xp(&stats,"mining",
            "minecraft:deepslate_diamond_ore",20),30);
        assert_eq!(resource_xp(&stats,"mining",
            "minecraft:coal_ore",20),20);
        assert_eq!(resource_xp(&stats,"mining",
            "minecraft:dirt",20),20);
        assert_eq!(resource_xp(&stats,"farming",
            "minecraft:wheat[age=7]",20),25);
        assert_eq!(resource_xp(&stats,"farming",
            "minecraft:wheat[age=3]",20),20);
        assert_eq!(mature_crop_key("minecraft:wheat[age=3]"),None);
        assert_eq!(mined_ore_key("minecraft:diamond_block"),None);
        assert_eq!(mined_ore_key("minecraft:amethyst_cluster"),Some("amethyst"));
        assert_eq!(mined_ore_key("modded:diamond_ore"),None);
    }
    #[test]fn independent_weapon_and_damage_type_stats_reach_real_hits(){
        use crate::rustpg_scaling::Tier;
        let mut a=Stats::default();
        a.add("weapon.sword.damage_flat",2.0);
        a.add("weapon.sword.damage_pct",0.25);
        a.add("weapon.sword.vs_boss",0.50);
        let mut d=Stats::default();
        d.add("defense.res_fire",0.50);
        let ordinary=damage_after_armor(10.0,0.0,Some((&a,"sword")),None,
            "minecraft:mob_attack","minecraft:zombie",None,None,None,false,10);
        assert!((ordinary-15.0).abs()<0.01);
        let boss=damage_after_armor(10.0,0.0,Some((&a,"sword")),None,
            "minecraft:mob_attack","minecraft:zombie",Some(Tier::Boss),None,None,false,10);
        assert!((boss-22.5).abs()<0.01);
        let axe=damage_after_armor(10.0,0.0,Some((&a,"axe")),None,
            "minecraft:mob_attack","minecraft:zombie",None,None,None,false,10);
        assert_eq!(axe,10.0);
        let fire=damage_after_armor(10.0,0.0,None,Some(&d),
            "minecraft:in_fire","minecraft:player",None,None,None,false,10);
        assert!((fire-5.0).abs()<0.01);
        let physical=damage_after_armor(10.0,0.0,None,Some(&d),
            "minecraft:mob_attack","minecraft:player",None,None,None,false,10);
        assert_eq!(physical,10.0);
        d.add("defense.res_crit",0.40);
        let crit=damage_after_armor(10.0,0.0,None,Some(&d),
            "minecraft:mob_attack","minecraft:player",None,None,None,true,10);
        assert!((crit-6.0).abs()<0.01);
        let void=damage_after_armor(10.0,0.0,None,Some(&d),
            "minecraft:out_of_world","minecraft:player",None,None,None,false,10);
        assert_eq!(void,10.0);
        let mut pen=Stats::default();
        pen.add("weapon.sword.armor_pen",0.5);
        let pierced=damage_after_armor(6.0,4.0,Some((&pen,"sword")),None,
            "minecraft:mob_attack","minecraft:zombie",None,None,None,false,11);
        assert!((pierced-8.0).abs()<0.01);
        let mut bleed=Stats::default();
        bleed.add("weapon.sword.bleed_chance",1.0);
        bleed.add("weapon.sword.bleed_damage",1.5);
        bleed.add("weapon.sword.bleed_duration",2.0);
        assert_eq!(bleed_plan(&bleed,"sword",1),Some((2.0,4)));
    }
    #[test]fn independent_maths_are_bounded_and_client_cannot_grant_stats(){
        let mut s=Stats::default();s.add("general.orb_xp",2.0);
        assert_eq!(EffectView{stats:&s,is_server:false}.value("general.orb_xp"),0.0);
        assert_eq!(EffectView{stats:&s,is_server:true}.value("general.orb_xp"),2.0);
        assert_eq!(orb_value(10,0.5),15);assert_eq!(orb_value(-2,2.0),-2);
        assert_eq!(orb_value(i32::MAX,1000.0),i32::MAX);
        assert_eq!(scaled_xp(100,0.25),125);
        assert_eq!(scaled_xp(0,100.0),0);
        assert_eq!(scaled_xp(100,f64::NAN),100);
        assert_eq!(food_after(15,17,0.8,1),17);
        let reduced=food_after(15,11,1.0,1);
        assert!((11..=15).contains(&reduced)); // reduction capped to 90%
        assert_eq!(food_after(15,11,0.0,1),11);
        assert_eq!(rod_wear(2,1.0,1),0);
        assert_eq!(rod_wear(1,0.0,1),1);
        assert_eq!(point_credit(0.02,980),(1,0));
        assert_eq!(point_credit(f64::NAN,200),(0,200));
    }
}
