//! Deterministic decisions for engine-owned attributes. SDK calls stay in lib.rs.
//! Unlike overwriting vanilla bases, named modifiers coexist with other mods.
//!
//! The v4.0.4 legacy migration is one-time and conservative: a foreign base
//! cannot be distinguished from the old RPG base when they happen to coincide.
pub const HEALTH_BASELINE:f64=20.0;
pub const HEALTH_MODIFIER_ID:&str="rustpg:bonus_max_health";
pub const FATIGUE_MODIFIER_ID:&str="rustpg:exhausted_movement";

pub fn bonus_health(desired:f64)->f64 {
    if !desired.is_finite() {0.0}
    else {(desired-HEALTH_BASELINE).clamp(0.0,2028.0)}
}
/// Only migrate the former "set_attribute_base" write if it is exactly the
/// expected RPG value and the effective health is not modified by other effects.
pub fn can_migrate_old_base(base:f64,effective:f64,expected:f64)->bool {
    base.is_finite() && effective.is_finite() && expected.is_finite()
    && base>HEALTH_BASELINE+0.01
    && (base-expected).abs()<0.01 && (effective-base).abs()<0.01
}
pub fn speed_penalty(exhausted:bool)->Option<f64> {
    if exhausted {Some(-0.40)}else{None}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn health_bonus_adds_not_replaces() {
        assert_eq!(bonus_health(20.0),0.0);
        assert_eq!(bonus_health(21.0),1.0);
        assert_eq!(bonus_health(40.0),20.0);
        assert_eq!(bonus_health(f64::NAN),0.0);
    }
    #[test] fn migration_requires_unmodified_old_base() {
        assert!(can_migrate_old_base(30.0,30.0,30.0));
        assert!(!can_migrate_old_base(30.0,36.0,30.0));
        assert!(!can_migrate_old_base(30.0,30.0,35.0));
        assert!(!can_migrate_old_base(20.0,20.0,20.0));
    }
    #[test] fn fatigue_is_named_multiplicative_modifier() {
        assert_eq!(speed_penalty(true),Some(-0.40));
        assert_eq!(speed_penalty(false),None);
    }
}
