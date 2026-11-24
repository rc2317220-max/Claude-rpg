//! RustPG visual language.
//!
//! Field journal palette. Ink and patinated slate anchor the reading surfaces.
//! Brass marks the selection, vellum the text, and jade an available action.

use super::rustpg_ui_layout::Rect;

pub const INK: u32       = 0xF3101A20;
pub const INK_SOFT: u32  = 0xE3142328;
pub const PANEL: u32     = 0xEE192A2F;
pub const PANEL_HI: u32  = 0xF2263B3D;
pub const SURFACE: u32   = 0xEA17272C;
pub const SURFACE_HI:u32 = 0xF02B4040;
pub const SURFACE_LOW:u32= 0xD1101C21;

pub const LINE: u32      = 0x805C7C79;
pub const AETHER: u32    = 0xFF91D1BB;
pub const AETHER_DIM:u32 = 0x5591D1BB;
pub const MOON: u32      = 0xFFD9DDD1;
pub const WHITE: u32     = 0xFFF6F0DC;
pub const MUTED: u32     = 0xFFA1B2AC;
pub const GOLD: u32      = 0xFFEAC987;
pub const RED: u32       = 0xFFEC8384;
pub const RED_DARK:u32   = 0xFF572831;
pub const VIOLET:u32     = 0xFFB49CD9;
pub const GREEN:u32      = 0xFF8CC697;
pub const AMBER:u32      = 0xFFE8AB69;
pub const GHOST:u32      = 0x3CC8CEC2;

// Compatibility aliases used by the older drawing code.  New code should use
// the semantic names above, but keeping these aliases prevents unrelated
// gameplay modules from becoming part of a visual migration.
pub const CYAN:u32 = AETHER;
pub const CYAN_DIM:u32 = AETHER_DIM;
pub const ICE:u32 = MOON;

// Semantic meter colours.
pub const HP:u32 = 0xFFE5777C;
pub const STAMINA:u32 = 0xFF82C99A;
pub const MANA:u32 = 0xFFA99AD4;
pub const XP:u32 = 0xFFE3BE78;
pub const SHIELD:u32 = 0xFF9BC9D4;

// Geometry tokens. Horizon's text renderer is pixel-oriented, therefore these
// stay integral/half-integral instead of pretending to be CSS points.
pub const SPACE_1:f32 = 3.0;
pub const SPACE_2:f32 = 5.0;
pub const SPACE_3:f32 = 8.0;
pub const SPACE_4:f32 = 12.0;
pub const CUT_SM:f32 = 2.0;
pub const CUT_MD:f32 = 4.0;
pub const TOUCH_MIN:f32 = 18.0;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum UiState { Normal, Hover, Pressed, Selected, Disabled, Locked, Available, Owned, Maxed }

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Responsive {
    pub compact: bool,
    pub portrait: bool,
    pub gap: f32,
    pub touch: f32,
    pub side_pad: f32,
}

pub fn responsive(area:Rect)->Responsive {
    let w=area.w.max(0.0);
    let h=area.h.max(0.0);
    Responsive {
        compact: w<430.0 || h<245.0,
        portrait: h>w,
        gap: if w<360.0 {3.0}else{5.0},
        touch: if w<300.0 {18.0}else{20.0},
        side_pad: if w<360.0 {5.0}else{8.0},
    }
}

pub fn state_edge(state:UiState)->u32 {
    match state {
        UiState::Selected => GOLD,
        UiState::Available => AETHER,
        UiState::Owned => GREEN,
        UiState::Maxed => GOLD,
        UiState::Pressed => MOON,
        UiState::Disabled|UiState::Locked => 0xFF73847F,
        UiState::Hover => 0xFFADDCCB,
        UiState::Normal => 0xFF69817C,
    }
}

pub fn state_fill(state:UiState)->u32 {
    match state {
        UiState::Selected => 0xEF393427,
        UiState::Available => 0xE21D3532,
        UiState::Owned => 0xDC22332A,
        UiState::Maxed => 0xE43A3225,
        UiState::Pressed => 0xEF33423F,
        UiState::Disabled|UiState::Locked => 0xDC122024,
        UiState::Hover => 0xE22A3939,
        UiState::Normal => SURFACE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn phone_density_remains_touchable(){
        let d=responsive(Rect{x:0.0,y:0.0,w:320.0,h:180.0});
        assert!(d.compact);
        assert!(d.touch>=TOUCH_MIN);
    }
    #[test] fn states_are_visually_distinct(){
        assert_ne!(state_edge(UiState::Locked),state_edge(UiState::Available));
        assert_ne!(state_fill(UiState::Selected),state_fill(UiState::Disabled));
    }
}
