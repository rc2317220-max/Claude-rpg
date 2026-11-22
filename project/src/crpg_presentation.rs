// Presentation Engine 2.3 facade. Backend adapters can depend on this module
// instead of importing every visual subsystem directly.

pub use super::crpg_bestiary::{BestiaryState, SpeciesRecord};
pub use super::crpg_damage_visuals::{DamageKind, DamageVisual};
pub use super::crpg_evil_att::{MutationDef, MUTATIONS};
pub use super::crpg_hud::HudModel;
pub use super::crpg_target_hud::TargetSnapshot;
pub use super::crpg_ui_animation::{AnimationClock, Transition};
pub use super::crpg_ui_core::{Navigation, Palette, Screen, DARK_FANTASY};
pub use super::crpg_ui_layout::{BestiaryLayout, HudLayout, Rect};
pub use super::crpg_ui_widgets::Widget;
