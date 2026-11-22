// Vitalidade própria do engine. O cálculo é autoritativo e testável; a ponte
// para substituir a vida vanilla depende de um hook de escrita de saúde.
use super::crpg_num::finite;

/// Converts the game's 20-point health scale to the RPG's effective max-HP scale.
pub fn from_vanilla_amount(amount: f64, max_health: f64) -> f64 {
    finite(amount.max(0.0) * max_health.max(20.0) / 20.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vitality {
    pub current: f64,
    pub max: f64,
    pub regen_per_tick: f64,
}
impl Vitality {
    pub fn new(max: f64, regen_per_tick: f64) -> Self {
        let max = max.max(1.0).min(1.0e9);
        Self {
            current: max,
            max,
            regen_per_tick: regen_per_tick.max(0.0),
        }
    }
    pub fn damage(&mut self, amount: f64) -> f64 {
        let amount = finite(amount.max(0.0));
        let before = self.current;
        self.current = (self.current - amount).max(0.0);
        before - self.current
    }
    pub fn heal(&mut self, amount: f64) -> f64 {
        let amount = finite(amount.max(0.0));
        let before = self.current;
        self.current = (self.current + amount).min(self.max);
        self.current - before
    }
    pub fn tick(&mut self) {
        let _ = self.heal(self.regen_per_tick);
    }
    pub fn fraction(&self) -> f64 {
        (self.current / self.max.max(1.0)).clamp(0.0, 1.0)
    }
}
impl Default for Vitality {
    fn default() -> Self {
        Self::new(20.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_damage_and_healing_scale_to_effective_rpg_health() {
        assert_eq!(from_vanilla_amount(6.0, 30.0), 9.0);
        assert_eq!(from_vanilla_amount(6.0, 10.0), 6.0);
    }

    #[test]
    fn damage_and_healing_update_vitality_with_bounds() {
        let mut vitality = Vitality::new(30.0, 0.0);
        assert_eq!(vitality.damage(12.0), 12.0);
        assert_eq!(vitality.current, 18.0);
        assert_eq!(vitality.heal(20.0), 12.0);
        assert_eq!(vitality.current, 30.0);
    }
}
