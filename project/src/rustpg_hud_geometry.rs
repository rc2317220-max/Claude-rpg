//! Responsive HUD geometry in GUI pixels; independent from Starling renderer.
//! Reserve the vanilla hotbar region, do not stack an additional power HUD.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct HudBounds{pub x:f32,pub y:f32,pub w:f32,pub h:f32,pub scale:f32}
/// Vanilla RPG XP bar remains 7 GUI pixels tall. HP and stamina are 125%
/// of that height at HUD scale 1.0 for a compact touch-screen layout.
/// No changes are made to the XP bar or its atlas.
pub const XP_BAR_HEIGHT: f32 = 7.0;
pub const VITAL_BAR_RATIO: f32 = 1.25;
pub const VITAL_BAR_WIDTH: f32 = 104.0;
pub fn hp_stamina_height(scale: f32) -> f32 {
    XP_BAR_HEIGHT * VITAL_BAR_RATIO * scale.clamp(0.42, 1.25)
}
pub fn compute(w:f32,h:f32,requested:f32,offset:f32)->Option<HudBounds>{
    if !w.is_finite() || !h.is_finite() || w<260.0 || h<150.0 {return None;}
    let scale=requested.clamp(0.70,1.25)
        .min((w-22.0)/222.0).min((h-96.0)/71.0).max(0.42);
    let (bw,bh)=(222.0*scale,71.0*scale);
    // Left side, below the launcher status buttons. Never over the hotbar.
    let x=12.0;
    let y=(29.0+offset.clamp(0.0,56.0)).min((h-bh-38.0).max(5.0));
    Some(HudBounds{x,y,w:bw,h:bh,scale})
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn hp_and_stamina_never_look_like_thick_panels(){
        let height=hp_stamina_height(1.0);
        for scale in [0.42, 0.7, 1.0, 1.25] {
            assert!((hp_stamina_height(scale) / (XP_BAR_HEIGHT * scale) - 1.25).abs() < 0.001);
        }
        assert!(VITAL_BAR_WIDTH < 216.0); // substantially shorter than XP
        assert!(hp_stamina_height(0.70) < height);
    }
    #[test]fn fit_phone_tablet_desktop(){
        for (w,h) in [(260.0,150.0),(320.0,180.0),(400.0,240.0),
            (800.0,480.0),(1920.0,1080.0)]{
            let b=compute(w,h,1.0,0.0).unwrap();
            assert_eq!(b.x,12.0);
            assert!(b.x+b.w<=w);
            assert!(b.y>=4.0 && b.y+b.h<=h-7.0);
        }
    }
    #[test]fn does_not_open_in_menu_or_invalid_viewport(){
        assert!(compute(180.0,110.0,1.0,0.0).is_none());
        assert!(compute(f32::NAN,200.0,1.0,0.0).is_none());
    }
}
