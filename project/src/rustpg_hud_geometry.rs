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
    XP_BAR_HEIGHT * VITAL_BAR_RATIO * (if scale.is_finite(){scale.clamp(0.42,1.25)}else{1.0})
}
pub fn compute(w:f32,h:f32,requested:f32,offset:f32)->Option<HudBounds>{
    if !w.is_finite() || !h.is_finite() || w<260.0 || h<150.0 {return None;}
    let requested=if requested.is_finite(){requested.clamp(0.70,1.25)}else{1.0};
    let offset=if offset.is_finite(){offset.clamp(0.0,56.0)}else{0.0};
    // 90 px includes the RPG XP strip, recovery, text and status icon.
    // On small screens reserve at least 132 px on the right for combat.
    let compact=if w<430.0{((w-150.0)/222.0).max(0.42)}else{1.25};
    let scale=requested.min(compact)
        .min((w-22.0)/222.0).min((h-96.0)/90.0).max(0.42);
    let (bw,bh)=(222.0*scale,90.0*scale);
    let x=12.0;
    let y=(29.0+offset).min((h-bh-38.0).max(5.0));
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

/// True for overlap with a margin; edges separated by margin do not collide.
pub fn overlaps(a:[f32;4],b:[f32;4],margin:f32)->bool{
    a[0]<b[0]+b[2]+margin && a[0]+a[2]+margin>b[0] &&
    a[1]<b[1]+b[3]+margin && a[1]+a[3]+margin>b[1]
}
impl HudBounds{pub fn rect(self)->[f32;4]{[self.x,self.y,self.w,self.h]}}
#[cfg(test)]mod invalid_config_tests{
    use super::*;
    #[test]fn bad_settings_fall_back_to_finite_safe_geometry(){
        assert_eq!(compute(320.0,180.0,f32::NAN,f32::NAN),compute(320.0,180.0,1.0,0.0));
        assert!(hp_stamina_height(f32::NAN).is_finite());
        for w in [260.0,320.0,400.0] {
            let b=compute(w,150.0,1.25,56.0).unwrap();
            assert!(w-b.x-b.w>=132.0);
            assert!(b.y+b.h<=150.0-38.0);
        }
    }
}
