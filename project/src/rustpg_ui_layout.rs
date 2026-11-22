// Resolution-independent layout primitives for touch-first screens.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub fn contains(self, p: [f32; 2]) -> bool {
        p[0] >= self.x && p[1] >= self.y && p[0] < self.x + self.w && p[1] < self.y + self.h
    }
    pub fn inset(self, n: f32) -> Self {
        Self {
            x: self.x + n,
            y: self.y + n,
            w: (self.w - 2.0 * n).max(0.0),
            h: (self.h - 2.0 * n).max(0.0),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BestiaryLayout {
    pub list: Rect,
    pub details: Rect,
    pub viewport: Rect,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HudLayout {
    pub left: Rect,
    pub xp: Rect,
    pub target: Rect,
}

pub fn bestiary(area: Rect) -> BestiaryLayout {
    let portrait_stack=area.h>area.w*1.16 || area.w<270.0;
    if portrait_stack {
        let list_h=(area.h*0.34).clamp(86.0,138.0).min((area.h-82.0).max(1.0));
        let list=Rect{x:area.x,y:area.y,w:area.w,h:list_h};
        let details=Rect{x:area.x,y:area.y+list_h+5.0,w:area.w,
            h:(area.h-list_h-5.0).max(1.0)};
        let side=(details.w*0.34).clamp(62.0,104.0).min((details.h-16.0).max(1.0));
        let viewport=Rect{x:details.x+7.0,y:details.y+7.0,w:side,h:side};
        BestiaryLayout{list,details,viewport}
    } else {
        let left_w=(area.w*0.29).clamp(108.0,188.0).min((area.w-128.0).max(1.0));
        let list=Rect{x:area.x,y:area.y,w:left_w,h:area.h};
        let details=Rect{x:area.x+left_w+5.0,y:area.y,
            w:(area.w-left_w-5.0).max(1.0),h:area.h};
        let side=(details.w*0.32).clamp(72.0,116.0).min((details.h-16.0).max(1.0));
        let viewport=Rect{x:details.x+7.0,y:details.y+7.0,w:side,h:side};
        BestiaryLayout{list,details,viewport}
    }
}

pub fn hud(width: f32, height: f32) -> HudLayout {
    let width=if width.is_finite(){width.max(0.0)}else{0.0};
    let height=if height.is_finite(){height.max(0.0)}else{0.0};
    let left_w=(width*0.34).clamp(148.0,206.0).min((width-12.0).max(0.0));
    let left_h=58.0_f32.min((height-8.0).max(0.0));
    let target_w=(width*0.30).clamp(142.0,208.0).min((width-12.0).max(0.0));
    HudLayout {
        left:Rect{x:6.0,y:(height-left_h-6.0).max(2.0),w:left_w,h:left_h},
        xp:Rect{x:8.0,y:(height-5.0).max(0.0),w:(width-16.0).max(0.0),h:2.0},
        target:Rect{x:((width-target_w)*0.5).max(4.0),y:6.0,w:target_w,h:48.0},
    }
}


#[cfg(test)]
mod responsive_tests {
    use super::*;
    #[test] fn bestiary_stacks_in_portrait(){
        let a=Rect{x:0.0,y:0.0,w:220.0,h:420.0};
        let l=bestiary(a);
        assert!(l.details.y>l.list.y+l.list.h);
        assert!(l.viewport.x>=l.details.x && l.viewport.y>=l.details.y);
    }
    #[test] fn bestiary_keeps_two_columns_in_landscape(){
        let a=Rect{x:0.0,y:0.0,w:620.0,h:300.0};
        let l=bestiary(a);
        assert!(l.details.x>l.list.x+l.list.w);
    }
}
