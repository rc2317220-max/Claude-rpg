// Sistema nutricional próprio. O evento vanilla de fome é apenas a entrada;
// os efeitos são mantidos no Profile e consumidos pelo Engine.
use super::rustpg_num::finite;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Nutrition {
    pub satiety: f64,
    pub stamina_restore: f64,
    pub regen_bonus: f64,
    pub resistance_bonus: f64,
    pub effect_ticks: u32,
}
impl Default for Nutrition {
    fn default() -> Self {
        Self {
            satiety: 0.0,
            stamina_restore: 0.0,
            regen_bonus: 0.0,
            resistance_bonus: 0.0,
            effect_ticks: 0,
        }
    }
}
impl Nutrition {
    /// Aplica um evento positivo de alimentação. Drenagem natural nunca concede buff.
    pub fn eat(&mut self, food_delta: i32) {
        if food_delta <= 0 {
            return;
        }
        let n = food_delta as f64;
        self.satiety = finite((self.satiety + n).min(100.0));
        self.stamina_restore = finite((self.stamina_restore + n * 1.5).min(100.0));
        self.regen_bonus = finite((self.regen_bonus + n * 0.01).min(0.25));
        self.resistance_bonus = finite((self.resistance_bonus + n * 0.005).min(0.15));
        self.effect_ticks = self
            .effect_ticks
            .saturating_add((n * 30.0) as u32)
            .min(20 * 60 * 30);
    }
    /// Return only the newly earned restoration; old saved credit is never paid twice.
    pub fn eat_and_take_restore(&mut self, food_delta: i32) -> f64 {
        if food_delta <= 0 {return 0.0;}
        self.stamina_restore=0.0;
        self.eat(food_delta);
        let amount=self.stamina_restore;
        self.stamina_restore=0.0;
        amount
    }
    pub fn tick(&mut self) {
        self.satiety = (self.satiety - 0.005).max(0.0);
        if self.effect_ticks > 0 {
            self.effect_ticks -= 1;
        } else {
            self.regen_bonus *= 0.98;
            self.resistance_bonus *= 0.98;
        }
        self.stamina_restore *= 0.995;
    }
    pub fn active_regen_bonus(&self) -> f64 {
        if self.effect_ticks > 0 {
            self.regen_bonus
        } else {
            0.0
        }
    }
    pub fn active_resistance_bonus(&self) -> f64 {
        if self.effect_ticks > 0 {
            self.resistance_bonus
        } else {
            0.0
        }
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn eating_grants_one_shot_stamina_and_timed_bonuses() {
        let mut n=Nutrition::default();
        assert_eq!(n.eat_and_take_restore(4),6.0);
        assert_eq!(n.stamina_restore,0.0);
        assert_eq!(n.eat_and_take_restore(-3),0.0);
        assert!((n.active_regen_bonus()-0.04).abs()<1e-9);
        assert!((n.active_resistance_bonus()-0.02).abs()<1e-9);
        for _ in 0..120 {n.tick();}
        assert_eq!(n.active_regen_bonus(),0.0);
        assert_eq!(n.active_resistance_bonus(),0.0);
    }
}
