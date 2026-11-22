//! V4 exhaustion rules independent of Starling (server authoritative bridge).
//! A "200% slower" break means time = 300% of normal, speed = 1/3.
pub const RECOVERY_FRACTION: f64 = 0.50;
pub const EXHAUSTED_WALK_FACTOR: f32 = 0.60; // 40% slower walking/sprinting
pub const EXHAUSTED_MINING_FACTOR: f32 = 1.0 / 3.0; // breaking takes 3x as long
pub const EXHAUSTED_DAMAGE_FACTOR: f32 = 0.75; // 25% less damage
pub const NORMAL_ATTACK_COST: f64 = 2.5;
/// A 1/1.10 cost multiplier gives exactly 10% more sprint time at the same SP cap.
pub const SPRINT_COST_MULTIPLIER:f64=1.0/1.10;

pub fn adjusted_break_speed(original: f32, exhausted: bool) -> f32 {
    if exhausted && original.is_finite() {original.max(0.0)*EXHAUSTED_MINING_FACTOR}
    else {original}
}
pub fn adjusted_damage(original: f32, exhausted: bool) -> f32 {
    if exhausted && original.is_finite() {original.max(0.0)*EXHAUSTED_DAMAGE_FACTOR}
    else {original}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn penalties_are_quantitatively_correct() {
        assert!((adjusted_break_speed(3.0,true)-1.0).abs()<0.00001);
        assert!((adjusted_damage(20.0,true)-15.0).abs()<0.00001);
        assert_eq!(adjusted_damage(20.0,false),20.0);
        assert_eq!(EXHAUSTED_WALK_FACTOR,0.60);
        assert_eq!(RECOVERY_FRACTION,0.50);
        assert!((1.0/SPRINT_COST_MULTIPLIER-1.10).abs()<0.00001);
    }
}
