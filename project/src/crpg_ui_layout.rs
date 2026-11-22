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
    let left_w = (area.w * 0.32).clamp(120.0, 260.0);
    let list = Rect {
        x: area.x,
        y: area.y,
        w: left_w,
        h: area.h,
    };
    let details = Rect {
        x: area.x + left_w + 6.0,
        y: area.y,
        w: area.w - left_w - 6.0,
        h: area.h,
    };
    let viewport = Rect {
        x: details.x + details.w * 0.42,
        y: details.y + 24.0,
        w: details.w * 0.55,
        h: details.h * 0.58,
    };
    BestiaryLayout {
        list,
        details,
        viewport,
    }
}

pub fn hud(width: f32, height: f32) -> HudLayout {
    HudLayout {
        left: Rect {
            x: 8.0,
            y: (height - 82.0).max(4.0),
            w: (width * 0.34).max(150.0),
            h: 76.0,
        },
        xp: Rect {
            x: 8.0,
            y: 8.0,
            w: (width * 0.28).max(150.0),
            h: 16.0,
        },
        target: Rect {
            x: (width - 220.0).max(8.0),
            y: 8.0,
            w: 212.0,
            h: 58.0,
        },
    }
}
