// Stamina do RPG Engine. A unidade temporal é o tick do servidor (20/s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stamina {
    pub current: f64,
    pub max: f64,
    pub regen_per_tick: f64,
    pub regen_delay_ticks: u32,
    pub delay_left: u32,
    pub zero_ticks: u32,
    /// Latch: exhaustion lasts until >=50% of the current stamina cap.
    pub exhausted: bool,
}

impl Stamina {
    pub fn new(max: f64, regen_per_tick: f64, regen_delay_ticks: u32) -> Self {
        let max = max.max(1.0).min(1.0e9);
        Self {
            current: max,
            max,
            regen_per_tick: regen_per_tick.max(0.0),
            regen_delay_ticks,
            delay_left: 0,
            zero_ticks: 0,
            exhausted: false,
        }
    }
    pub fn clamp(&mut self) {
        self.max = self.max.max(1.0).min(1.0e9);
        self.current = self.current.clamp(0.0, self.max);
        self.regen_per_tick = self.regen_per_tick.max(0.0).min(self.max);
        if self.current <= 1e-9 {self.exhausted=true;}
        if self.current >= self.max * super::crpg_v4_fatigue::RECOVERY_FRACTION {
            self.exhausted=false;
        }
    }
    /// Consome stamina. Caso não haja energia suficiente, esgota e aciona fadiga.
    pub fn consume(&mut self, cost: f64) -> bool {
        if !cost.is_finite() || cost<0.0 {return false;}
        if cost==0.0 {return true;}
        if self.exhausted {return false;}
        // An action that uses the remaining stamina triggers exhaustion.
        if self.current + 1e-9 < cost {
            self.current=0.0;
            self.delay_left=self.regen_delay_ticks;
            self.zero_ticks=1;
            self.exhausted=true;
            return false;
        }
        self.current=(self.current-cost).max(0.0);
        self.delay_left=self.regen_delay_ticks;
        self.clamp();
        true
    }
    pub fn tick(&mut self) {
        self.clamp();
        if self.current<=1e-9 {self.zero_ticks=self.zero_ticks.saturating_add(1);}
        else {self.zero_ticks=0;}
        if self.delay_left>0 {self.delay_left-=1;return;}
        self.current=(self.current+self.regen_per_tick).min(self.max);
        self.clamp();
    }
    /// Do not lift exhaustion in mid-air: the character must recover at least
    /// half their stamina AND regain ground contact. Jumping cannot bypass fatigue.
    pub fn tick_while_grounded(&mut self, on_ground:bool) {
        let previous=self.exhausted;
        self.tick();
        if previous && !on_ground {self.exhausted=true;}
    }
    pub fn zero_penalty(&self) -> f64 {
        if self.exhausted {super::crpg_v4_fatigue::EXHAUSTED_WALK_FACTOR as f64}
        else {1.0}
    }
    /// Progress towards the threshold needed to leave exhaustion.
    pub fn recovery_progress(&self)->f64 {
        (self.current/(self.max*super::crpg_v4_fatigue::RECOVERY_FRACTION).max(1.0))
            .clamp(0.0,1.0)
    }
    pub fn fraction(&self) -> f64 {
        (self.current / self.max).clamp(0.0, 1.0)
    }
}

impl Default for Stamina {
    fn default() -> Self {
        Self::new(100.0, 2.5, 20)
    }
}

#[cfg(test)]
mod exhaustion_tests {
    use super::*;
    #[test] fn stays_exhausted_until_half_capacity() {
        let mut s=Stamina::new(100.0,1.0,0);
        assert!(!s.exhausted);
        assert!(!s.consume(110.0));
        assert!(s.exhausted);
        for _ in 0..49 {s.tick();}
        assert!(s.exhausted);
        assert!((s.current-49.0).abs()<0.001);
        s.tick();
        assert_eq!(s.current,50.0);
        assert!(!s.exhausted);
    }
    #[test] fn exhaustion_blocks_further_action_drain() {
        let mut s=Stamina::new(10.0,1.0,0);
        assert!(s.consume(10.0));
        assert!(s.exhausted);
        s.tick();
        assert!(!s.consume(1.0));
        assert_eq!(s.current,1.0);
    }
    #[test] fn jumping_cannot_release_exhaustion() {
        let mut s=Stamina::new(100.0,5.0,0);
        s.consume(100.0);
        for _ in 0..12 {s.tick_while_grounded(false);}
        assert_eq!(s.current,60.0);
        assert!(s.exhausted);
        s.tick_while_grounded(true);
        assert!(!s.exhausted);
    }
    #[test] fn increasing_max_preserves_exhaustion_until_new_half() {
        let mut s=Stamina::new(100.0,0.0,0);
        s.consume(100.0);s.current=50.0;s.max=200.0;s.clamp();
        assert!(s.exhausted);
        s.current=100.0;s.clamp();assert!(!s.exhausted);
    }
}
