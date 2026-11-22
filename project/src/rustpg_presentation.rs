// Presentation Engine 2.3 facade. Backend adapters can depend on this module
// instead of importing every visual subsystem directly.

pub use super::rustpg_bestiary::{BestiaryState, SpeciesRecord};
pub use super::rustpg_damage_visuals::{DamageKind, DamageVisual};
pub use super::rustpg_evil_att::{MutationDef, MUTATIONS};
pub use super::rustpg_hud::HudModel;
pub use super::rustpg_target_hud::TargetSnapshot;
pub use super::rustpg_ui_animation::{AnimationClock, Transition};
pub use super::rustpg_ui_core::{Navigation, Palette, Screen, DARK_FANTASY};
pub use super::rustpg_ui_layout::{BestiaryLayout, HudLayout, Rect};
pub use super::rustpg_ui_widgets::Widget;
