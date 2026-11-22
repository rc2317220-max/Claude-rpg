// Bestiary domain state. The renderer must consume this state; yaw alone is not a completed viewport.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpeciesRecord {
    pub kind: String,
    pub first_seen: bool,
    pub kills: u64,
    pub best_damage: f64,
    pub max_level: u32,
    pub tiers: Vec<String>,
    pub mutations: Vec<String>,
    pub drops: Vec<String>,
    pub reward_flags: u32,
}
impl SpeciesRecord {
    pub fn discovery(&self) -> f32 {
        // Only a confirmed player kill identifies a species.
        // A legacy first_seen flag from proximity never grants access.
        if self.kills == 0 { 0.0 } else { (self.kills as f32 / 500.0).min(1.0) }
    }
    pub fn reward_step(&self) -> u32 {
        match self.kills {
            0 => 0,
            1..=9 => 1,
            10..=49 => 2,
            50..=99 => 3,
            100..=499 => 4,
            _ => 5,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BestiaryState {
    pub selected: usize,
    pub rotation: f32,
    pub zoom: f32,
    pub dragging: bool,
}
impl BestiaryState {
    pub fn tick(&mut self) {
        if !self.dragging {
            self.rotation = (self.rotation + 0.6) % 360.0;
        }
    }
    pub fn drag(&mut self, dx: f32) {
        self.rotation = (self.rotation + dx).rem_euclid(360.0);
    }
    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(-0.5, 1.0);
    }
}
