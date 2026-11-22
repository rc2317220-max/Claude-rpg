// HUD view model. Horizon-specific drawing remains in crpg_main until the adapter is migrated.

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudModel {
    pub vanilla_hp: (f64, f64),
    pub vitality: (f64, f64),
    pub stamina: (f64, f64),
    pub absorption: f64,
    pub xp: (u64, u64),
    pub level: u32,
    pub rank: String,
    pub exhausted: bool,
    pub critical: bool,
    pub target_visible: bool,
}
impl HudModel {
    pub fn hp_fraction(&self) -> f32 {
        if self.vanilla_hp.1 <= 0.0 {
            0.0
        } else {
            (self.vanilla_hp.0 / self.vanilla_hp.1).clamp(0.0, 1.0) as f32
        }
    }
}
