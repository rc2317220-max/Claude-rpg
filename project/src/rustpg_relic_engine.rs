//! RustPG v4.0.5: biblioteca original de relíquias, independente do Starling.
//! Nenhum mod externo recebe &mut ServerHost; o servidor recebe decisões limitadas.
//! API versionada permite novas famílias de poderes sem alterar hooks do jogo.
use std::collections::BTreeSet;
use super::rustpg_gear::{Gear, Inventory};

pub const RELIC_ABI: u16 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RelicEffects {
    /// Fração curada de dano REAL aplicado (máximo 16%).
    pub life_steal: f32,
    /// Fração de dano recebido anulada (máximo 24%).
    pub guard: f32,
    /// Dano adicional de ataque (máximo 18%).
    pub attack_bonus: f32,
    /// Stamina adicional recuperada por segundo (máximo 4 SP/s).
    pub stamina_regen: f64,
    /// Mineração mais rápida enquanto não exausto (máximo 30%).
    pub mining_bonus: f32,
    /// Desconto no custo de corrida/ataque (máximo 35%).
    pub stamina_discount: f64,
}
impl RelicEffects {
    pub(crate) fn bounded(mut self)->Self {
        fn safe32(v:f32,max:f32)->f32 {if v.is_finite(){v.clamp(0.0,max)}else{0.0}}
        fn safe64(v:f64,max:f64)->f64 {if v.is_finite(){v.clamp(0.0,max)}else{0.0}}
        self.life_steal=safe32(self.life_steal,0.16);
        self.guard=safe32(self.guard,0.24);
        self.attack_bonus=safe32(self.attack_bonus,0.18);
        self.stamina_regen=safe64(self.stamina_regen,4.0);
        self.mining_bonus=safe32(self.mining_bonus,0.30);
        self.stamina_discount=safe64(self.stamina_discount,0.35);
        self
    }
    pub(crate) fn combine(self,other:Self)->Self {
        Self {
            life_steal:self.life_steal+other.life_steal,
            guard:self.guard+other.guard,
            attack_bonus:self.attack_bonus+other.attack_bonus,
            stamina_regen:self.stamina_regen+other.stamina_regen,
            mining_bonus:self.mining_bonus+other.mining_bonus,
            stamina_discount:self.stamina_discount+other.stamina_discount,
        }.bounded()
    }
    pub fn attack_cost(self,base:f64)->f64 {
        if !base.is_finite() {return base;}
        (base * (1.0-self.stamina_discount)).max(0.0)
    }
    pub fn mining_speed(self,base:f32,exhausted:bool)->f32 {
        if exhausted || !base.is_finite() {base} else {base.max(0.0)*(1.0+self.mining_bonus)}
    }
    pub fn final_hit(self,base:f32,exhausted:bool,defender:Self)->f32 {
        if !base.is_finite() {return base;}
        let fatigue=if exhausted {0.75} else {1.0};
        let amount=base.max(0.0) * fatigue * (1.0+self.attack_bonus) * (1.0-defender.guard);
        if amount.is_finite() {amount.max(0.0)} else {base}
    }
    pub fn heal_from_hit(self,actual_health_damage:f32)->f32 {
        if !actual_health_damage.is_finite() || actual_health_damage<=0.0 {return 0.0;}
        (actual_health_damage*self.life_steal).clamp(0.0,4.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RelicInfo {pub id:&'static str,pub title:&'static str,pub abi:u16}

/// Pure extension point: no world access, IO, mutation or profile ownership.
/// Only EQUIPPED gear is passed to extensions. All magnitudes are capped.
pub trait RelicAbility: Send+Sync {
    fn info(&self)->RelicInfo;
    fn bonus(&self,item:&Gear)->RelicEffects;
}
#[derive(Debug,PartialEq,Eq)]
pub enum RegisterError {BadId,Duplicate,UnsupportedAbi,TooMany}

pub struct RelicRegistry { abilities:Vec<Box<dyn RelicAbility>>,ids:BTreeSet<&'static str> }
impl RelicRegistry {
    pub fn new()->Self {Self{abilities:Vec::new(),ids:BTreeSet::new()}}
    pub fn register(&mut self,ability:Box<dyn RelicAbility>)->Result<(),RegisterError> {
        let info=ability.info();
        if info.abi!=RELIC_ABI {return Err(RegisterError::UnsupportedAbi);}
        if info.id.len()>64 || info.id.is_empty() || !info.id.contains(':') ||
            !info.id.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c,b'_'|b'.'|b':'|b'/')) {
            return Err(RegisterError::BadId);
        }
        if self.ids.contains(info.id) {return Err(RegisterError::Duplicate);}
        if self.abilities.len()>=32 {return Err(RegisterError::TooMany);}
        self.ids.insert(info.id);self.abilities.push(ability);Ok(())
    }
    pub fn snapshot(&self,items:&Inventory)->RelicEffects {
        let mut out=RelicEffects::default();
        // Inventory::sanitize must already have validated wear slots on decode.
        // Fail closed on corrupted equipment even before migration/sanitize.
        for (slot,item) in &items.worn {
            if &item.slot!=slot || !super::rustpg_gear::SLOTS.iter().any(|(s,_)|*s==slot.as_str()) {continue;}
            for ability in &self.abilities {out=out.combine(ability.bonus(item));}
        }
        out
    }
    pub fn count(&self)->usize {self.abilities.len()}
}

#[derive(Clone,Copy)]
struct BuiltinAbility {motif:u8,id:&'static str,title:&'static str}
impl RelicAbility for BuiltinAbility {
    fn info(&self)->RelicInfo {RelicInfo{id:self.id,title:self.title,abi:RELIC_ABI}}
    fn bonus(&self,g:&Gear)->RelicEffects {
        if g.motif!=self.motif {return RelicEffects::default();}
        let strength=(0.80 + g.rarity.min(5) as f32 * 0.16
            + (g.level.min(10_000) as f32).sqrt()*0.008).min(2.30);
        let mut out=RelicEffects::default();
        match self.motif {
            0=>out.guard=0.035*strength,               // Vigia: proteção real
            1=>out.life_steal=0.025*strength,         // Eclipse: roubo de vida
            2=>out.attack_bonus=0.024*strength,       // Abismo: fúria
            3=>out.stamina_regen=0.48*f64::from(strength), // Aurora: recuperação
            4=>out.mining_bonus=0.042*strength,       // Caçador: extração
            5=>out.stamina_discount=0.050*f64::from(strength),// Monarca: eficiência
            _=>{},
        }
        out.bounded()
    }
}
pub const DESCRIPTIONS:[(&str,&str);6]=[
    ("rustpg:watcher","Vigia: reduz dano recebido enquanto equipado"),
    ("rustpg:eclipse","Eclipse: rouba vida do dano realmente causado"),
    ("rustpg:abyss","Abismo: aumenta dano do ataque"),
    ("rustpg:aurora","Aurora: acelera a recuperação de stamina"),
    ("rustpg:hunter","Caçador: acelera a mineração sem exaustão"),
    ("rustpg:monarch","Monarca: reduz gasto de stamina"),
];
pub fn describe(g:&Gear)->&'static str {
    DESCRIPTIONS.get(g.motif as usize).map_or("Sem habilidade",|(_,label)|*label)
}
pub fn builtins()->RelicRegistry {
    let mut registry=RelicRegistry::new();
    for (i,(id,title)) in DESCRIPTIONS.iter().enumerate() {
        registry.register(Box::new(BuiltinAbility{motif:i as u8,id:*id,title:*title}))
            .expect("IDs estáticos validados");
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    fn gear(motif:u8,slot:&str)->Gear {
        Gear{uid:1,slot:slot.into(),rarity:4,level:36,motif}
    }
    #[test] fn six_abilities_and_unworn_items_do_nothing() {
        let registry=builtins();assert_eq!(registry.count(),6);
        let mut inv=Inventory::default();
        inv.add(gear(1,"ring")).unwrap();
        assert_eq!(registry.snapshot(&inv),RelicEffects::default());
        inv.equip(1).unwrap();
        assert!(registry.snapshot(&inv).life_steal>0.0);
    }
    #[test] fn each_motif_has_real_distinct_effect() {
        let r=builtins();
        for motif in 0..6 {
            let mut inv=Inventory::default();
            inv.worn.insert("ring".into(),gear(motif,"ring"));
            let s=r.snapshot(&inv);
            let count=[s.guard>0.0,s.life_steal>0.0,s.attack_bonus>0.0,
                s.stamina_regen>0.0,s.mining_bonus>0.0,s.stamina_discount>0.0]
                .into_iter().filter(|yes|*yes).count();
            assert_eq!(count,1,"motif {motif}");
        }
    }
    #[test] fn healing_uses_actual_damage_and_never_revives() {
        let mut s=RelicEffects::default();s.life_steal=0.16;
        assert_eq!(s.heal_from_hit(1000.0),4.0);
        assert_eq!(s.heal_from_hit(0.0),0.0);
        assert_eq!(s.heal_from_hit(f32::NAN),0.0);
    }
    #[test] fn multipliers_and_caps_respect_exhaustion() {
        let mut attacker=RelicEffects::default();attacker.attack_bonus=0.18;
        let mut defender=RelicEffects::default();defender.guard=0.24;
        assert!((attacker.final_hit(100.0,true,defender)-67.26).abs()<0.02);
        assert!((attacker.mining_speed(10.0,false)-10.0).abs()<0.01);
        attacker.stamina_discount=0.35;
        assert!((attacker.attack_cost(10.0)-6.5).abs()<0.001);
    }
    #[test] fn malicious_extension_values_are_clamped() {
        let effect=RelicEffects{life_steal:f32::NAN,guard:f32::INFINITY,
            attack_bonus:999.0,stamina_regen:f64::INFINITY,
            mining_bonus:500.0,stamina_discount:f64::NAN}.bounded();
        assert_eq!(effect.life_steal,0.0);
        assert_eq!(effect.guard,0.0);
        assert_eq!(effect.attack_bonus,0.18);
        assert_eq!(effect.mining_bonus,0.30);
        assert_eq!(effect.stamina_regen,0.0);
        assert_eq!(effect.stamina_discount,0.0);
    }
    #[test] fn invalid_equipment_cannot_activate_abilities() {
        let r=builtins();let mut inv=Inventory::default();
        inv.worn.insert("ring".into(),gear(1,"head"));
        assert_eq!(r.snapshot(&inv),RelicEffects::default());
    }
}
