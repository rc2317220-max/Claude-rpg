use super::rustpg_ui_layout::Rect;

// Touch and pointer actions normalized for the Presentation Engine.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiAction {
    Tap([f32; 2]),
    Drag { from: [f32; 2], to: [f32; 2] },
    Scroll(f32),
    Pinch(f32),
    Back,
}
pub fn drag_delta(from: [f32; 2], to: [f32; 2]) -> [f32; 2] {
    [to[0] - from[0], to[1] - from[1]]
}


/// Enlarges a visual control into a comfortable touch target without moving
/// the control itself. The returned hit rectangle is clamped to the owning
/// viewport, so neighboring phone UI cannot steal taps outside the menu.
pub fn touch_target(r:Rect,bounds:Rect,min_side:f32)->Rect {
    let min_side=if min_side.is_finite(){min_side.max(1.0)}else{18.0};
    let w=r.w.max(min_side).min(bounds.w.max(0.0));
    let h=r.h.max(min_side).min(bounds.h.max(0.0));
    let cx=r.x+r.w*0.5;let cy=r.y+r.h*0.5;
    let x=(cx-w*0.5).clamp(bounds.x,(bounds.x+bounds.w-w).max(bounds.x));
    let y=(cy-h*0.5).clamp(bounds.y,(bounds.y+bounds.h-h).max(bounds.y));
    Rect{x,y,w,h}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn compact_controls_receive_phone_sized_hitboxes(){
        let b=Rect{x:0.0,y:0.0,w:320.0,h:180.0};
        let r=touch_target(Rect{x:300.0,y:2.0,w:12.0,h:10.0},b,20.0);
        assert!(r.w>=20.0 && r.h>=20.0);
        assert!(r.x>=b.x && r.y>=b.y);
        assert!(r.x+r.w<=b.x+b.w+0.01 && r.y+r.h<=b.y+b.h+0.01);
    }
}
