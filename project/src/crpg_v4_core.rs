//! Claude RPG 4 - mod-owned independent domain / extension library.
//! No game SDK dependency: model inputs come from server hooks and decisions
//! are applied only by the trusted server bridge in lib.rs.
use std::collections::HashSet;
use super::crpg_xp_rules;

pub const EXTENSION_PROTOCOL: u16 = 4;
pub const MAX_EVENT_XP: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureClass { Hostile, Passive, Player, Unknown }

/// LivingDrops fires for actual living entities; unknown namespace kinds may
/// be third-party creatures. Unknown species are counted, not excluded.
pub fn classify_living(kind: &str) -> CreatureClass {
    if kind == "minecraft:player" { return CreatureClass::Player; }
    if kind.is_empty() || !kind.contains(':') { return CreatureClass::Unknown; }
    if super::crpg_scaling::scales(kind) { CreatureClass::Hostile }
    else { CreatureClass::Passive }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Award { pub general: u64, pub profession: u64 }
impl Award {
    pub const ZERO: Self = Self { general:0, profession:0 };
    pub fn bounded(self)->Self {
        Self {general:self.general.min(MAX_EVENT_XP),
              profession:self.profession.min(MAX_EVENT_XP)}
    }
    pub fn add(self, other:Self)->Self {
        Self {general:self.general.saturating_add(other.general),
              profession:self.profession.saturating_add(other.profession)}.bounded()
    }
}

#[derive(Clone, Debug)]
pub enum ProgressionEvent<'a> {
    Mine { block_state:&'a str, player_placed:bool },
    Kill { kind:&'a str, natural_health:f64, xp_multiplier:f64 },
}

#[derive(Clone, Copy)]
pub struct ExtensionInfo {
    pub id: &'static str,
    pub abi: u16,
    pub title: &'static str,
}

/// Rule modules are deterministic and cannot change the game world directly.
/// An extension may only return a reward. The server bridge owns persistence.
pub trait RuleModule: Send + Sync {
    fn info(&self)->ExtensionInfo;
    fn reward(&self, event: &ProgressionEvent<'_>)->Award;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RegisterError { BadId, UnsupportedAbi, Duplicate, TooMany }

pub struct ExtensionRegistry {
    modules: Vec<Box<dyn RuleModule>>,
    ids: HashSet<&'static str>,
}
impl ExtensionRegistry {
    pub fn new()->Self {Self{modules:Vec::new(),ids:HashSet::new()}}
    pub fn register(&mut self, extension:Box<dyn RuleModule>)->Result<(),RegisterError> {
        let info=extension.info();
        if info.id.is_empty() || info.id.len()>64 || !info.id.bytes().all(|b|
            b.is_ascii_lowercase() || b.is_ascii_digit() || b==b'_' || b==b'.' || b==b':')
            || !info.id.contains(':') {return Err(RegisterError::BadId);}
        if info.abi!=EXTENSION_PROTOCOL {return Err(RegisterError::UnsupportedAbi);}
        if self.ids.contains(info.id) {return Err(RegisterError::Duplicate);}
        if self.modules.len()>=32 {return Err(RegisterError::TooMany);}
        self.ids.insert(info.id);
        self.modules.push(extension);
        Ok(())
    }
    pub fn calculate(&self, event:&ProgressionEvent<'_>)->Award {
        // Security invariant belongs to the server-owned registry, never to a plugin:
        // malicious/buggy modules cannot grant mining XP for dirt or reused ore.
        if let ProgressionEvent::Mine {block_state,player_placed} = event {
            if *player_placed || crpg_xp_rules::mined(block_state).is_none() {
                return Award::ZERO;
            }
        }
        if let ProgressionEvent::Kill {kind,..} = event {
            if matches!(classify_living(kind),CreatureClass::Player|CreatureClass::Unknown) {
                return Award::ZERO;
            }
        }
        self.modules.iter().fold(Award::ZERO,|sum,m|sum.add(m.reward(event)))
    }
    pub fn count(&self)->usize {self.modules.len()}
}

struct VanillaRules;
impl RuleModule for VanillaRules {
    fn info(&self)->ExtensionInfo {ExtensionInfo {
        id:"claude_rpg:vanilla_progression",abi:EXTENSION_PROTOCOL,
        title:"Progressão Vanilla 4"}}
    fn reward(&self, event:&ProgressionEvent<'_>)->Award {
        match event {
            ProgressionEvent::Mine{block_state,player_placed} => {
                if *player_placed {return Award::ZERO;}
                let Some(x)=crpg_xp_rules::mined(block_state) else{return Award::ZERO;};
                Award{general:x.general,profession:x.profession}
            },
            ProgressionEvent::Kill{kind,natural_health,xp_multiplier} => {
                match classify_living(kind) {
                    CreatureClass::Player | CreatureClass::Unknown => Award::ZERO,
                    CreatureClass::Hostile => {
                        let hp=if natural_health.is_finite(){natural_health.max(1.0)}else{20.0};
                        let mult=if xp_multiplier.is_finite(){xp_multiplier.clamp(0.0,100.0)}else{1.0};
                        Award { general:super::crpg_progression::kill_xp(hp,mult), profession:12 }.bounded()
                    },
                    CreatureClass::Passive => {
                        // Animals, villagers and third-party living creatures count
                        // in the bestiary, but pay limited XP to discourage farming.
                        let hp=if natural_health.is_finite(){natural_health.max(1.0)}else{10.0};
                        let base=super::crpg_progression::kill_xp(hp,1.0);
                        Award{general:(base/5).max(1).min(8),profession:0}
                    }
                }
            }
        }
    }
}

pub fn builtins()->ExtensionRegistry {
    let mut registry=ExtensionRegistry::new();
    registry.register(Box::new(VanillaRules)).expect("valid built-in module");
    registry
}

/// Bounded target HP changes. This is a visualization FALLBACK, not an
/// authoritative damage calculation (healings, simultaneous hits, or shields).
pub fn observed_health_delta(prior:f64,current:f64)->Option<f64> {
    if !prior.is_finite() || !current.is_finite() || prior<0.0 || current<0.0 {return None;}
    let d=prior-current;
    if d>0.05 && d<=1_000_000.0 {Some(d)}else{None}
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Bonus;
    impl RuleModule for Bonus {
        fn info(&self)->ExtensionInfo {ExtensionInfo{id:"example:bonus",abi:EXTENSION_PROTOCOL,title:"Bonus"}}
        fn reward(&self,_:&ProgressionEvent<'_>)->Award {Award{general:3,profession:1}}
    }
    #[test] fn classification_counts_all_living() {
        assert_eq!(classify_living("minecraft:zombie"),CreatureClass::Hostile);
        assert_eq!(classify_living("minecraft:cow"),CreatureClass::Passive);
        assert_eq!(classify_living("minecraft:chicken"),CreatureClass::Passive);
        assert_eq!(classify_living("addon:pet"),CreatureClass::Passive);
        assert_eq!(classify_living("minecraft:player"),CreatureClass::Player);
    }
    #[test] fn blocks_reward_only_real_ore() {
        let r=builtins();
        for b in ["minecraft:dirt","minecraft:poppy","minecraft:grass_block","minecraft:stone","minecraft:diamond_block"] {
            assert_eq!(r.calculate(&ProgressionEvent::Mine{block_state:b,player_placed:false}),Award::ZERO);
        }
        assert!(r.calculate(&ProgressionEvent::Mine{block_state:"minecraft:diamond_ore",player_placed:false}).general>0);
        assert_eq!(r.calculate(&ProgressionEvent::Mine{block_state:"minecraft:diamond_ore",player_placed:true}),Award::ZERO);
    }
    #[test] fn passive_quests_not_blocked() {
        let r=builtins();
        let a=r.calculate(&ProgressionEvent::Kill{kind:"minecraft:cow",natural_health:10.0,xp_multiplier:1.0});
        assert!(a.general>0 && a.general<=8 && a.profession==0);
        assert_eq!(r.calculate(&ProgressionEvent::Kill{kind:"minecraft:player",natural_health:20.0,xp_multiplier:1.0}),Award::ZERO);
    }
    #[test] fn registry_validates_abi_and_duplicates() {
        let mut r=builtins();
        assert_eq!(r.register(Box::new(Bonus)),Ok(()));
        assert_eq!(r.register(Box::new(Bonus)),Err(RegisterError::Duplicate));
        let blocked=r.calculate(&ProgressionEvent::Mine{block_state:"minecraft:dirt",player_placed:false});
        assert_eq!(blocked,Award::ZERO); // even an extension returning 3 cannot bypass the guard
        let legitimate=r.calculate(&ProgressionEvent::Mine{block_state:"minecraft:diamond_ore",player_placed:false});
        assert!(legitimate.general>=3);
        // Registry must block player farming even if a third-party rule returns XP.
        assert_eq!(r.calculate(&ProgressionEvent::Kill{
            kind:"minecraft:player",natural_health:20.0,xp_multiplier:1.0}),Award::ZERO);
    }
    #[test] fn hp_observer_rejects_invalid() {
        assert_eq!(observed_health_delta(20.0,17.0),Some(3.0));
        assert_eq!(observed_health_delta(17.0,20.0),None);
        assert_eq!(observed_health_delta(f64::NAN,15.0),None);
    }
}
