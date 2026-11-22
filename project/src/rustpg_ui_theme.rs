//! RustPG visual language.
//!
//! Central tokens for every HUD/menu surface.  The identity is deliberately
//! "rift-forged": cold moon-metal, desaturated void glass and restrained
//! arcane accents.  Components choose hierarchy through geometry first and
//! colour second, so the UI stays readable on small Android screens.

use super::rustpg_ui_layout::Rect;

pub const INK: u32       = 0xE30A0D13;
pub const INK_SOFT: u32  = 0xC70E121A;
pub const PANEL: u32     = 0xD9131822;
pub const PANEL_HI: u32  = 0xE21A2230;
pub const SURFACE: u32   = 0xD30F151E;
pub const SURFACE_HI:u32 = 0xDF18212D;
pub const SURFACE_LOW:u32= 0xB90A0F16;

pub const LINE: u32      = 0x725F879A;
pub const AETHER: u32    = 0xFF69D5D0;
pub const AETHER_DIM:u32 = 0x5569D5D0;
pub const MOON: u32      = 0xFFD7E3E7;
pub const WHITE: u32     = 0xFFF4F6F5;
pub const MUTED: u32     = 0xFF91A0A7;
pub const GOLD: u32      = 0xFFE6BD66;
pub const RED: u32       = 0xFFE85D68;
pub const RED_DARK:u32   = 0xFF542229;
pub const VIOLET:u32     = 0xFF9D82D6;
pub const GREEN:u32      = 0xFF69C995;
pub const AMBER:u32      = 0xFFE69A55;
pub const GHOST:u32      = 0x3CBFC8CA;

// Compatibility aliases used by the older drawing code.  New code should use
// the semantic names above, but keeping these aliases prevents unrelated
// gameplay modules from becoming part of a visual migration.
pub const CYAN:u32 = AETHER;
pub const CYAN_DIM:u32 = AETHER_DIM;
pub const ICE:u32 = MOON;

// Semantic meter colours.
pub const HP:u32 = 0xFFE05261;
pub const STAMINA:u32 = 0xFF65C58A;
pub const MANA:u32 = 0xFF8A78CF;
pub const XP:u32 = 0xFFD7B25E;
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
        UiState::Selected|UiState::Available => AETHER,
        UiState::Owned => GREEN,
        UiState::Maxed => GOLD,
        UiState::Pressed => MOON,
        UiState::Disabled|UiState::Locked => 0xFF5F6970,
        UiState::Hover => 0xFF8BE1DC,
        UiState::Normal => 0xFF647A84,
    }
}

pub fn state_fill(state:UiState)->u32 {
    match state {
        UiState::Selected => 0xDF1B3035,
        UiState::Available => 0xD719292C,
        UiState::Owned => 0xD7192923,
        UiState::Maxed => 0xD72E2819,
        UiState::Pressed => 0xE0222A31,
        UiState::Disabled|UiState::Locked => 0xB80B0F14,
        UiState::Hover => 0xD819252C,
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
