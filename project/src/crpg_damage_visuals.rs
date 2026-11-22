// World-space combat floaters. Rendering is performed by the RenderHooks adapter.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageKind {
    Physical,
    Critical,
    Unarmed,
    Elemental,
    Bleed,
    Periodic,
    Heal,
    Absorption,
    Blocked,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DamageVisual {
    pub entity: u32,
    pub origin: [f64; 3],
    pub amount: f64,
    pub kind: DamageKind,
    pub age: u32,
    pub lifetime: u32,
}
impl DamageVisual {
    pub fn position(&self) -> [f64; 3] {
        let t = self.age as f64;
        [
            self.origin[0],
            self.origin[1] + 0.35 + t * 0.035,
            self.origin[2],
        ]
    }
    pub fn alpha(&self) -> f32 {
        if self.lifetime == 0 {
            0.0
        } else {
            (1.0 - self.age as f32 / self.lifetime as f32).clamp(0.0, 1.0)
        }
    }
    pub fn alive(&self) -> bool {
        self.age < self.lifetime
    }
}
pub fn color(kind: DamageKind) -> u32 {
    match kind {
        DamageKind::Critical => 0xFFFFD34E,
        DamageKind::Heal => 0xFF67E88A,
        DamageKind::Blocked => 0xFF7FD8FF,
        DamageKind::Elemental => 0xFFFF8C54,
        DamageKind::Bleed | DamageKind::Periodic => 0xFFE34B5F,
        _ => 0xFFFFFFFF,
    }
}
