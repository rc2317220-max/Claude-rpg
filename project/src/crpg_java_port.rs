//! Reimplementation of the *algorithms* identified by javap in the four supplied JARs.
//! This is original Rust, not a JVM or Forge compatibility layer.
//! The SDK adapter is in lib.rs. Hooks which cannot be guaranteed on the pinned
//! Horizon build are intentionally NOT advertised as live node effects.
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation { AddValue, AddMultipliedBase, AddMultipliedTotal }

#[derive(Clone, Debug, PartialEq)]
pub struct NamedModifier {
    pub source: String,
    pub amount: f64,
    pub operation: Operation,
}

/// Mirrors the ordering in Puffish's `DynamicModificationImpl.applyTo`:
/// additive -> multiplied base -> multiplied total, without modifying vanilla
/// attribute bases and without touching modifiers owned by other mods.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub base: f64,
    pub min: f64,
    pub max: f64,
    modifiers: BTreeMap<String, NamedModifier>,
}
impl Attribute {
    pub fn new(base: f64, min: f64, max: f64) -> Self {
        Self { base, min, max, modifiers: BTreeMap::new() }
    }
    pub fn replace(&mut self, m: NamedModifier) -> bool {
        if m.source.is_empty() || !m.amount.is_finite() { return false; }
        self.modifiers.insert(m.source.clone(), m);
        true
    }
    pub fn remove(&mut self, source: &str) -> bool {
        self.modifiers.remove(source).is_some()
    }
    pub fn effective(&self) -> f64 {
        let valid = |n:f64| if n.is_finite(){n.clamp(-1e12,1e12)}else{0.0};
        let base = valid(self.base);
        let mut current=base;
        for m in self.modifiers.values() {
            if m.operation==Operation::AddValue { current += valid(m.amount); }
        }
        current = valid(current);
        for m in self.modifiers.values() {
            if m.operation==Operation::AddMultipliedBase { current += base*valid(m.amount); }
        }
        for m in self.modifiers.values() {
            if m.operation==Operation::AddMultipliedTotal {
                current *= 1.0+valid(m.amount).clamp(-1.0,1e6);
                current=valid(current);
            }
        }
        let min=if self.min.is_finite(){self.min}else{f64::MIN};
        let max=if self.max.is_finite(){self.max.max(min)}else{f64::MAX};
        valid(current).clamp(min,max)
    }
    pub fn modifiers(&self) -> impl Iterator<Item=&NamedModifier> { self.modifiers.values() }
}

/// Positive/negative contribution is evaluated *per operation* as in Puffish.
/// The game adapter must read the modifiers actually attached to each entity.
#[derive(Clone, Copy)]
pub enum Sign { Plus, Minus }
pub fn dynamic_apply(base: f64, sources: &[(&Attribute, Sign)]) -> f64 {
    if !base.is_finite() {return 0.0;}
    let mut val=base;
    for &(a, sign) in sources {
        let polarity=match sign {Sign::Plus=>1.0,Sign::Minus=>-1.0};
        for m in a.modifiers.values().filter(|m|m.operation==Operation::AddValue) {
            val+=polarity*m.amount;
        }
    }
    let additive_result=val;
    for &(a, sign) in sources {
        let polarity=match sign {Sign::Plus=>1.0,Sign::Minus=>-1.0};
        for m in a.modifiers.values().filter(|m|m.operation==Operation::AddMultipliedBase) {
            val+=additive_result*polarity*m.amount;
        }
    }
    for &(a, sign) in sources {
        let polarity=match sign {Sign::Plus=>1.0,Sign::Minus=>-1.0};
        for m in a.modifiers.values().filter(|m|m.operation==Operation::AddMultipliedTotal) {
            val*=1.0+polarity*m.amount;
        }
    }
    if val.is_finite() {val.clamp(-1e12,1e12)}else{0.0}
}

/// From More Attributes's probability/strength transform; avoids adding raw
/// probabilities directly. Strengths are additive; final chance is bounded.
/// E.g., two independent 50% bonuses produce 75%, not an impossible 100%.
pub fn strength_from_probability(p:f64)->f64 {
    if !p.is_finite(){return 0.0;}
    if p>=1.0{return f64::INFINITY;}
    if p<=-1.0{return f64::NEG_INFINITY;}
    -(-p).ln_1p()
}
pub fn probability_from_strength(strength:f64)->f64 {
    if strength==f64::INFINITY {return 1.0;}
    if strength==f64::NEG_INFINITY {return -1.0;}
    if !strength.is_finite(){return 0.0;}
    -(-strength).exp_m1()
}
pub fn combine_independent_chances(chances: &[f64])->f64 {
    let mut total=0.0;
    for p in chances {
        total+=strength_from_probability((*p).clamp(0.0,1.0));
    }
    probability_from_strength(total).clamp(0.0,1.0)
}

/// More Attributes: `AttributeCalculationHelper.getFinalHitProbability`.
/// The result is a *probability only*; the combat hook must draw its own
/// server-authoritative RNG value and only decide on the accepted attack once.
pub fn hit_probability(hit_rate:f64, evasion:f64, final_hit:f64, final_evasion:f64)->f64 {
    let a=if hit_rate.is_finite(){hit_rate.max(0.0)}else{0.0};
    let b=if evasion.is_finite(){evasion.clamp(0.0,1.0)}else{0.0};
    let c=if final_hit.is_finite(){final_hit}else{0.0};
    let d=if final_evasion.is_finite(){final_evasion}else{0.0};
    (a*(1.0-b)+c-d).clamp(0.0,1.0)
}

/// More Attributes: `AttributeCalculationHelper.applyPenetration`.
/// Note: the source method applies this to a *damage-reduction strength*,
/// not unconditionally to raw damage dealt; callers must preserve that stage.
pub fn penetrated_reduction(reduction_strength:f64, flat:f64, percent:f64)->f64 {
    if !reduction_strength.is_finite()||reduction_strength<=0.0{return 0.0;}
    let flat=if flat.is_finite(){flat}else{0.0};
    let percent=if percent.is_finite(){percent.clamp(0.0,1.0)}else{0.0};
    (reduction_strength*(1.0-percent)-flat).max(0.0)
}

/// Additional Attributes uses stochastic rounding to preserve fractional bonuses.
/// `roll` is supplied by the authoritative server (0 <= roll < 1).
/// Returns a nonnegative count; does not create/copy an ItemStack itself.
pub fn stochastic_count(expected:f64,roll:f64)->u32 {
    if !expected.is_finite() || expected<=0.0 {return 0;}
    let expected=expected.min(1_000_000.0);
    let whole=expected.floor();
    let r=if roll.is_finite(){roll.clamp(0.0,1.0)}else{1.0};
    (whole as u32).saturating_add(u32::from(r<expected.fract()))
}

pub fn scaled_break_speed(original:f32, mining_bonus:f64, tool_bonus:f64) -> f32 {
    if !original.is_finite()||original<=0.0 {return original;}
    let a=if mining_bonus.is_finite(){mining_bonus.clamp(0.0,4.0)}else{0.0};
    let b=if tool_bonus.is_finite(){tool_bonus.clamp(0.0,4.0)}else{0.0};
    (f64::from(original)*(1.0+a)*(1.0+b)).min(f32::MAX as f64) as f32
}

/// A verified, actual-health-damage-based life-steal award. Friendly fire,
/// self-hits, absorbed-only hits and generic periodic damage must be excluded
/// by the server adapter BEFORE calling this function.
pub fn life_steal(actual_hp_damage:f32, bonus:f64) -> f32 {
    if !actual_hp_damage.is_finite()||actual_hp_damage<=0.0||!bonus.is_finite(){return 0.0;}
    (f64::from(actual_hp_damage)*bonus.clamp(0.0,1.0)).min(2048.0) as f32
}

/// Fractional regeneration values are per SECOND; game tick is 20 Hz.
pub fn regen_for_elapsed_ticks(per_second:f64,elapsed_ticks:u32)->f32 {
    if !per_second.is_finite() || per_second<=0.0 || elapsed_ticks==0{return 0.0;}
    (per_second.clamp(0.0,64.0)*f64::from(elapsed_ticks.min(20))/20.0) as f32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackClass { Standard64, Standard16, Unique }
impl StackClass {
    pub fn from_original(max:u32)->Self {match max {64=>Self::Standard64,16=>Self::Standard16,_=>Self::Unique}}
    pub fn initial(self,original:u32)->u32 {match self {Self::Standard64=>16,Self::Standard16=>4,Self::Unique=>original}}
}
/// An *engine-integration contract* for a player-aware inventory implementation.
/// The native game must invoke this for pickup, shift-click, drag, crafting,
/// containers, stack splits, hoppers, item entities, and remote transfers.
/// Merely displaying this capacity does NOT change the engine's inventory rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PersonalStacks {pub rank_64:u32,pub rank_16:u32}
impl PersonalStacks {
    pub fn capacity(self,original:u32)->u32 {
        match StackClass::from_original(original) {
            StackClass::Standard64=>(16u32).saturating_add(4*self.rank_64.min(12)).min(64),
            StackClass::Standard16=>(4u32).saturating_add(self.rank_16.min(12)).min(16),
            StackClass::Unique=>original,
        }
    }
}
/// Transfer quantity is bounded by the item's ORIGINAL registered maximum
/// and by the per-player upgraded maximum. The original (64 or 16) MUST be
/// captured BEFORE the global ÷4 component rewrite, because the current
/// component size (16 or 4) would block all upgrades above the baseline.
/// This must be invoked only from a native transactional inventory hook that
/// also overrides the destination ItemStack/component cap and its codecs.
pub fn transfer_amount(existing:u32,incoming:u32,original_max:u32,player_max:u32)->u32 {
    let cap=original_max.min(player_max).max(1);
    incoming.min(cap.saturating_sub(existing))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn m(id:&str,operation:Operation,amount:f64)->NamedModifier{
        NamedModifier{source:id.into(),operation,amount}
    }
    #[test]fn modifier_order_and_ownership(){
        let mut a=Attribute::new(10.0,0.0,100.0);
        a.replace(m("mod:flat",Operation::AddValue,2.0));
        a.replace(m("mod:base",Operation::AddMultipliedBase,0.5));
        a.replace(m("mod:total",Operation::AddMultipliedTotal,0.1));
        assert!((a.effective()-18.7).abs()<0.001);
        a.replace(m("mod:total",Operation::AddMultipliedTotal,0.2));
        assert!((a.effective()-20.4).abs()<0.001);
        assert_eq!(a.modifiers().count(),3);
        assert!(a.remove("mod:flat"));
    }
    #[test]fn dynamic_negative_and_positive(){
        let mut a=Attribute::new(0.0,-1e3,1e3);
        a.replace(m("a",Operation::AddValue,2.0));
        a.replace(m("b",Operation::AddMultipliedTotal,0.5));
        assert!((dynamic_apply(10.0,&[(&a,Sign::Plus)])-18.0).abs()<0.001);
        assert!((dynamic_apply(10.0,&[(&a,Sign::Minus)])-4.0).abs()<0.001);
    }
    #[test]fn probability_does_not_exceed_one(){
        let p=combine_independent_chances(&[0.5,0.5]);
        assert!((p-0.75).abs()<1e-10);
        assert!(combine_independent_chances(&[0.9,0.9,0.9])<1.0);
        assert_eq!(combine_independent_chances(&[1.0]),1.0);
    }
    #[test]fn hit_and_penetration_match_jvm_formulas(){
        assert!((hit_probability(0.8,0.25,0.05,0.1)-0.55).abs()<1e-10);
        assert!((penetrated_reduction(20.0,2.0,0.25)-13.0).abs()<1e-10);
        assert_eq!(penetrated_reduction(1.0,5.0,0.0),0.0);
    }
    #[test]fn harvest_expected_count_preserved(){
        assert_eq!(stochastic_count(2.25,0.1),3);
        assert_eq!(stochastic_count(2.25,0.9),2);
    }
    #[test]fn stacks_only_go_up_by_ranks(){
        let zero=PersonalStacks{rank_64:0,rank_16:0};
        assert_eq!(zero.capacity(64),16);
        assert_eq!(zero.capacity(16),4);
        assert_eq!(zero.capacity(1),1);
        let trained=PersonalStacks{rank_64:12,rank_16:12};
        assert_eq!(trained.capacity(64),64);
        assert_eq!(trained.capacity(16),16);
        assert_eq!(transfer_amount(15,8,64,16),1);
        assert_eq!(transfer_amount(16,8,64,16),0);
        assert_eq!(transfer_amount(16,50,64,64),48);
        assert_eq!(transfer_amount(4,10,16,16),10);
    }
    #[test]fn math_is_finite_with_corrupt_input(){
        assert_eq!(life_steal(f32::INFINITY,1.0),0.0);
        assert_eq!(scaled_break_speed(f32::NAN,1.0,2.0).is_nan(),true);
        assert_eq!(stochastic_count(f64::NAN,0.2),0);
    }
}
