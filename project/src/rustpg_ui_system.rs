// RustPG interface primitives: compact glass panels, angular geometry and HUD.
//
// The module is pure: it only builds a `Layer` of primitives. The Horizon
// adapter paints each `Prim` with `Host::fill` (rectangles), `Host::quad`
// (chamfered corners, chevrons, diamonds) and `Host::text`. No texture, sprite
// or shader is invented here, and every number comes from the RPG state the
// caller fills in.

pub use super::rustpg_ui_layout::Rect;

// ------------------------------------------------------------------ design system

// Re-export semantic tokens so the older rendering modules can migrate
// incrementally without duplicating palette values.
pub use super::rustpg_ui_theme::*;

// ------------------------------------------------------------------ colours

pub fn alpha(color: u32, a: u8) -> u32 {
    (color & 0x00FF_FFFF) | ((a as u32) << 24)
}

/// Linear mix of two ARGB colours, `t` in 0..=1 toward `b`.
pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let t = unit(t);
    let ch = |s: u32| {
        let x = (a >> s) & 0xFF;
        let y = (b >> s) & 0xFF;
        ((x as f32 + (y as f32 - x as f32) * t).round() as u32) & 0xFF
    };
    (ch(24) << 24) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

pub fn lighten(c: u32, t: f32) -> u32 {
    mix(c, 0xFFFF_FFFF, t)
}

pub fn darken(c: u32, t: f32) -> u32 {
    mix(c, 0xFF00_0000, t)
}

/// Text never blends by alpha (`Host::text` takes an opaque rgb), so a fade is
/// a mix toward the panel's own dark blue.
pub fn fade_color(color: u32, a: f32) -> u32 {
    mix(0xFF06101C, color, unit(a))
}

// ------------------------------------------------------------------ primitives

#[derive(Clone, Debug, PartialEq)]
pub enum Prim {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: u32,
    },
    /// Four corners, clockwise from the top left. Two equal points make a triangle.
    Quad { p: [[f32; 2]; 4], color: u32 },
    Text {
        text: String,
        x: f32,
        y: f32,
        color: u32,
        shadow: bool,
    },
    Sprite {
        key: &'static str,
        src: [f32;4],
        dst: [f32;4],
        tint: Option<u32>,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layer {
    pub prims: Vec<Prim>,
    /// Optional scissor rectangle. Rects are intersected with it; text that
    /// would cross the edge is dropped, because the host font cannot be cropped
    /// and a half-drawn glyph reads as a bug.
    clip: Option<Rect>,
}

impl Layer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.prims.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prims.is_empty()
    }

    /// Restricts every following primitive to `r` (or clears the restriction).
    pub fn set_clip(&mut self, r: Option<Rect>) {
        self.clip = r;
    }

    fn scissor(&self, r: Rect) -> Option<Rect> {
        match self.clip {
            None => Some(r),
            Some(c) => {
                let x0 = r.x.max(c.x);
                let y0 = r.y.max(c.y);
                let x1 = (r.x + r.w).min(c.x + c.w);
                let y1 = (r.y + r.h).min(c.y + c.h);
                if x1 - x0 <= 0.0 || y1 - y0 <= 0.0 {
                    None
                } else {
                    Some(Rect {
                        x: x0,
                        y: y0,
                        w: x1 - x0,
                        h: y1 - y0,
                    })
                }
            }
        }
    }

    /// True when a text run of `w` pixels at `(x, y)` can be drawn whole.
    fn text_fits(&self, x: f32, y: f32, w: f32) -> bool {
        match self.clip {
            None => true,
            Some(c) => {
                x >= c.x - 0.5
                    && y >= c.y - 0.5
                    && x + w <= c.x + c.w + 0.5
                    && y + 9.0 <= c.y + c.h + 0.5
            }
        }
    }

    pub fn rect(&mut self, r: Rect, color: u32) {
        if r.w <= 0.0 || r.h <= 0.0 {
            return;
        }
        let Some(r) = self.scissor(r) else {
            return;
        };
        self.prims.push(Prim::Rect {
            x: r.x,
            y: r.y,
            w: r.w,
            h: r.h,
            color,
        });
    }

    pub fn sprite(&mut self, key:&'static str, src:[f32;4], r:Rect) {
        self.sprite_tinted(key,src,r,None);
    }

    pub fn sprite_tinted(&mut self,key:&'static str,src:[f32;4],r:Rect,tint:Option<u32>) {
        if r.w<=0.0 || r.h<=0.0 {return;}
        // Sprites use the same exact visible rectangle for runtime and visual QA.
        if self.clip.is_some_and(|c|r.x<c.x || r.y<c.y ||
            r.x+r.w>c.x+c.w || r.y+r.h>c.y+c.h) {return;}
        self.prims.push(Prim::Sprite {key,src,dst:[r.x,r.y,r.w,r.h],tint});
    }

    pub fn quad(&mut self, p: [[f32; 2]; 4], color: u32) {
        if !p.iter().flatten().all(|v|v.is_finite()){return;}
        let Some(clip)=self.clip else{
            self.prims.push(Prim::Quad{p,color});return;
        };
        if p.iter().all(|point|clip.contains(*point) ||
            (point[0] <= clip.x+clip.w && point[0]>=clip.x &&
             point[1]<=clip.y+clip.h && point[1]>=clip.y)) {
            self.prims.push(Prim::Quad{p,color});return;
        }
        // Sutherland-Hodgman: clip convex quads (glows, links, chamfers)
        // against the skill viewport. Layer::rect already clips itself,
        // but raw quad primitives previously escaped into the inspector.
        let mut poly=p.to_vec();
        for (axis,limit,keep_greater) in [
            (0,clip.x,true),(0,clip.x+clip.w,false),
            (1,clip.y,true),(1,clip.y+clip.h,false),
        ] {
            let input=std::mem::take(&mut poly);
            if input.is_empty(){return;}
            let inside=|pt:[f32;2]| if keep_greater {
                pt[axis]>=limit
            }else{pt[axis]<=limit};
            let intersect=|a:[f32;2],b:[f32;2]| {
                let delta=b[axis]-a[axis];
                let factor=if delta.abs()<0.000001 {0.0}
                    else{((limit-a[axis])/delta).clamp(0.0,1.0)};
                [a[0]+(b[0]-a[0])*factor,
                 a[1]+(b[1]-a[1])*factor]
            };
            let mut prev=*input.last().expect("non-empty polygon");
            for &current in &input {
                match (inside(prev),inside(current)){
                    (true,true)=>poly.push(current),
                    (true,false)=>poly.push(intersect(prev,current)),
                    (false,true)=>{poly.push(intersect(prev,current));poly.push(current);},
                    (false,false)=>{},
                }
                prev=current;
            }
        }
        if poly.len()<3{return;}
        // Engine renderer accepts a 4-point primitive. Fan triangulation
        // with a repeated vertex draws the clipped convex polygon exactly.
        for i in 1..poly.len()-1 {
            self.prims.push(Prim::Quad{p:[poly[0],poly[i],poly[i+1],poly[i+1]],color});
        }
    }

    pub fn text(&mut self, text: impl Into<String>, x: f32, y: f32, color: u32) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        if !self.text_fits(x, y, text_width(&text)) {
            return;
        }
        self.prims.push(Prim::Text {
            text,
            x,
            y,
            color,
            shadow: true,
        });
    }

    pub fn text_flat(&mut self, text: impl Into<String>, x: f32, y: f32, color: u32) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        if !self.text_fits(x, y, text_width(&text)) {
            return;
        }
        self.prims.push(Prim::Text {
            text,
            x,
            y,
            color,
            shadow: false,
        });
    }

    /// Text centred inside `r` on the horizontal axis.
    pub fn text_center(&mut self, text: impl Into<String>, r: Rect, y: f32, color: u32) {
        let text = text.into();
        let w = text_width(&text);
        self.text(text, r.x + (r.w - w) * 0.5, y, color);
    }

    /// Text centred both ways in `r` (9 px font).
    pub fn text_middle(&mut self, text: impl Into<String>, r: Rect, color: u32) {
        let text = text.into();
        let w = text_width(&text);
        self.text(text, r.x + (r.w - w) * 0.5, r.y + (r.h - 9.0) * 0.5, color);
    }

    // ------------------------------------------------------------ flat chrome

    /// A one-pixel rectangle border. The reference interface is built from thin
    /// light rules over dark glass, so this is the most used primitive.
    pub fn border(&mut self, r: Rect, color: u32) {
        self.rect(
            Rect {
                x: r.x,
                y: r.y,
                w: r.w,
                h: 1.0,
            },
            color,
        );
        self.rect(
            Rect {
                x: r.x,
                y: r.y + r.h - 1.0,
                w: r.w,
                h: 1.0,
            },
            color,
        );
        self.rect(
            Rect {
                x: r.x,
                y: r.y,
                w: 1.0,
                h: r.h,
            },
            color,
        );
        self.rect(
            Rect {
                x: r.x + r.w - 1.0,
                y: r.y,
                w: 1.0,
                h: r.h,
            },
            color,
        );
    }

    /// An inner card: dark glass, angular corners and a restrained technical
    /// grid. The content leads; the frame does not compete with it.
    pub fn card(&mut self, r: Rect, accent: u32) {
        // Cards are quiet surfaces. Hierarchy comes from spacing and one
        // material edge instead of a grid/glow stack behind every paragraph.
        self.chamfer(r, CUT_SM, SURFACE);
        self.chamfer_outline(r, CUT_SM, 1.0, alpha(LINE, 0x58));
        self.rect(Rect {
            x: r.x + 5.0, y: r.y + 1.0,
            w: (r.w * 0.22).clamp(18.0, 52.0), h: 1.0,
        }, alpha(accent, 0xB8));
        self.rect(Rect {
            x: r.x + CUT_SM, y: r.y + r.h - 1.0,
            w: (r.w - CUT_SM * 2.0).max(0.0), h: 1.0,
        }, alpha(0xFF000000, 0x34));
    }

    /// Vertical hairlines, the technical grid of the System.
    pub fn grid(&mut self, r: Rect, step: f32, color: u32) {
        let step = step.max(4.0);
        let mut x = r.x + step;
        while x < r.x + r.w - 1.0 {
            self.rect(
                Rect {
                    x,
                    y: r.y + 1.0,
                    w: 1.0,
                    h: r.h - 2.0,
                },
                color,
            );
            x += step;
        }
    }

    /// Text with extra tracking (the wide letter spacing of the System).
    pub fn text_tracked(
        &mut self,
        text: impl Into<String>,
        x: f32,
        y: f32,
        color: u32,
        track: f32,
    ) {
        let s = text.into();
        let mut pen = x;
        for c in s.chars() {
            self.text(c.to_string(), pen, y, color);
            pen += track_cell(c) + track;
        }
    }

    /// A small square button with a plus, as on the attribute rows.
    pub fn plus(&mut self, r: Rect, color: u32) {
        self.rect(r, alpha(0xFF0C2033, 0xF0));
        self.border(r, alpha(color, 0xB0));
        let cx = r.x + r.w * 0.5;
        let cy = r.y + r.h * 0.5;
        self.rect(
            Rect {
                x: cx - 3.0,
                y: cy - 0.5,
                w: 6.0,
                h: 1.5,
            },
            color,
        );
        self.rect(
            Rect {
                x: cx - 0.5,
                y: cy - 3.0,
                w: 1.5,
                h: 6.0,
            },
            color,
        );
    }

    /// A chamfered, centred, tracked title: the `STATUS` plate of the System.
    pub fn title_plate(&mut self, r: Rect, text: &str, accent: u32) {
        self.glow(r, 3.0, 2.0, accent, 2);
        self.chamfer(r, 3.0, 0xF0091A2C);
        self.chamfer_outline(r, 3.0, 1.0, alpha(accent, 0xA0));
        self.hline(r.x + 8.0, r.y + r.h - 2.0, r.w - 16.0, alpha(accent, 0x30));
        self.diamond(r.x + 8.0, r.y + r.h * 0.5, 1.8, alpha(accent, 0xC0));
        self.diamond(r.x + r.w - 8.0, r.y + r.h * 0.5, 1.8, alpha(accent, 0xC0));
        let text = &fit_tracked(text, r.w - 24.0, 1.0);
        let w = tracked_width(text, 1.0);
        self.text_tracked(
            text,
            r.x + (r.w - w) * 0.5,
            r.y + (r.h - 9.0) * 0.5,
            ICE,
            1.0,
        );
    }

    // -------------------------------------------------------------- geometry

    /// A rectangle with its four corners cut at 45 degrees.
    pub fn chamfer(&mut self, r: Rect, cut: f32, color: u32) {
        let c = cut.clamp(0.0, r.w.min(r.h) * 0.5);
        if c <= 0.5 {
            self.rect(r, color);
            return;
        }
        let (x, y, w, h) = (r.x, r.y, r.w, r.h);
        self.rect(
            Rect {
                x,
                y: y + c,
                w,
                h: h - 2.0 * c,
            },
            color,
        );
        self.rect(
            Rect {
                x: x + c,
                y,
                w: w - 2.0 * c,
                h: c,
            },
            color,
        );
        self.rect(
            Rect {
                x: x + c,
                y: y + h - c,
                w: w - 2.0 * c,
                h: c,
            },
            color,
        );
        // Corner triangles (a quad with a repeated point).
        self.quad(
            [[x + c, y], [x, y + c], [x + c, y + c], [x + c, y + c]],
            color,
        );
        self.quad(
            [
                [x + w - c, y],
                [x + w, y + c],
                [x + w - c, y + c],
                [x + w - c, y + c],
            ],
            color,
        );
        self.quad(
            [
                [x, y + h - c],
                [x + c, y + h],
                [x + c, y + h - c],
                [x + c, y + h - c],
            ],
            color,
        );
        self.quad(
            [
                [x + w - c, y + h],
                [x + w, y + h - c],
                [x + w - c, y + h - c],
                [x + w - c, y + h - c],
            ],
            color,
        );
    }

    /// A hollow chamfered frame `t` pixels thick.
    pub fn frame(&mut self, r: Rect, cut: f32, t: f32, color: u32) {
        self.chamfer_outline(r, cut, t, color);
    }

    /// A halo of `layers` chamfers growing outward, each fainter.
    pub fn glow(&mut self, r: Rect, cut: f32, spread: f32, color: u32, layers: u32) {
        let n = layers.max(1);
        for i in (1..=n).rev() {
            let d = spread * i as f32 / n as f32;
            let a = (0x30 / i).max(0x0A) as u8;
            self.chamfer(
                Rect {
                    x: r.x - d,
                    y: r.y - d,
                    w: r.w + 2.0 * d,
                    h: r.h + 2.0 * d,
                },
                cut + d,
                alpha(color, a),
            );
        }
    }

    /// Primary panel geometry shared by HUD and menus.
    pub fn panel(&mut self, r: Rect, accent: u32) {
        // Primary surfaces deliberately avoid the old neon-grid treatment.
        // A soft material plate plus a short accent rail reads cleanly over
        // Minecraft scenery and costs far fewer primitives on Android.
        self.chamfer(r, CUT_MD, PANEL);
        self.rect(Rect {
            x:r.x+CUT_MD, y:r.y+1.0,
            w:(r.w-CUT_MD*2.0).max(0.0),
            h:(r.h*0.22).clamp(2.0,9.0),
        }, alpha(MOON,0x0C));
        self.chamfer_outline(r,CUT_MD,1.0,alpha(LINE,0x78));
        self.rect(Rect {
            x:r.x+CUT_MD+5.0,y:r.y,
            w:(r.w*0.24).clamp(20.0,58.0),h:1.5,
        },alpha(accent,0xD8));
        self.corner_brackets(r,6.0,1.0,alpha(accent,0x52));
    }

    pub fn chamfer_outline(&mut self, r: Rect, cut: f32, t: f32, color: u32) {
        let c = cut.clamp(0.0, r.w.min(r.h) * 0.5);
        let t = t.max(0.5).min(r.w * 0.25).min(r.h * 0.25);
        if c <= 0.5 {
            // A plain rectangle's border: four strips.
            self.rect(
                Rect {
                    x: r.x,
                    y: r.y,
                    w: r.w,
                    h: t,
                },
                color,
            );
            self.rect(
                Rect {
                    x: r.x,
                    y: r.y + r.h - t,
                    w: r.w,
                    h: t,
                },
                color,
            );
            self.rect(
                Rect {
                    x: r.x,
                    y: r.y + t,
                    w: t,
                    h: r.h - 2.0 * t,
                },
                color,
            );
            self.rect(
                Rect {
                    x: r.x + r.w - t,
                    y: r.y + t,
                    w: t,
                    h: r.h - 2.0 * t,
                },
                color,
            );
            return;
        }
        let (x, y, w, h) = (r.x, r.y, r.w, r.h);
        // Straight edges.
        self.rect(
            Rect {
                x: x + c,
                y,
                w: w - 2.0 * c,
                h: t,
            },
            color,
        );
        self.rect(
            Rect {
                x: x + c,
                y: y + h - t,
                w: w - 2.0 * c,
                h: t,
            },
            color,
        );
        self.rect(
            Rect {
                x,
                y: y + c,
                w: t,
                h: h - 2.0 * c,
            },
            color,
        );
        self.rect(
            Rect {
                x: x + w - t,
                y: y + c,
                w: t,
                h: h - 2.0 * c,
            },
            color,
        );
        // The four 45-degree corners, each a thin parallelogram between the
        // outer and the inset edge.
        self.quad(
            [[x, y + c], [x + c, y], [x + c, y + t], [x + t, y + c]],
            color,
        );
        self.quad(
            [
                [x + w - c, y],
                [x + w, y + c],
                [x + w - t, y + c],
                [x + w - c, y + t],
            ],
            color,
        );
        self.quad(
            [
                [x, y + h - c],
                [x + t, y + h - c],
                [x + c, y + h - t],
                [x + c, y + h],
            ],
            color,
        );
        self.quad(
            [
                [x + w - c, y + h],
                [x + w - c, y + h - t],
                [x + w - t, y + h - c],
                [x + w, y + h - c],
            ],
            color,
        );
    }

    pub fn scanlines(&mut self, r: Rect, step: f32, color: u32) {
        let step = step.max(2.0);
        let mut y = r.y;
        while y < r.y + r.h {
            self.rect(
                Rect {
                    x: r.x,
                    y,
                    w: r.w,
                    h: 1.0,
                },
                color,
            );
            y += step;
        }
    }

    pub fn corner_brackets(&mut self, r: Rect, len: f32, t: f32, color: u32) {
        let l = len.min(r.w * 0.4).min(r.h * 0.4);
        for (x, y, sx, sy) in [
            (r.x, r.y, 1.0, 1.0),
            (r.x + r.w - l, r.y, 1.0, 1.0),
            (r.x, r.y + r.h - t, 1.0, 1.0),
            (r.x + r.w - l, r.y + r.h - t, 1.0, 1.0),
        ] {
            self.rect(Rect { x, y, w: l, h: t }, color);
            let _ = (sx, sy);
        }
        for x in [r.x, r.x + r.w - t] {
            self.rect(
                Rect {
                    x,
                    y: r.y,
                    w: t,
                    h: l,
                },
                color,
            );
            self.rect(
                Rect {
                    x,
                    y: r.y + r.h - l,
                    w: t,
                    h: l,
                },
                color,
            );
        }
    }

    pub fn hline(&mut self, x: f32, y: f32, w: f32, color: u32) {
        self.rect(Rect { x, y, w, h: 1.0 }, color);
    }

    pub fn vline(&mut self, x: f32, y: f32, h: f32, color: u32) {
        self.rect(Rect { x, y, w: 1.0, h }, color);
    }

    /// A diamond (the System's node and badge shape).
    pub fn diamond(&mut self, cx: f32, cy: f32, radius: f32, color: u32) {
        let r = radius.max(1.0);
        self.quad(
            [[cx, cy - r], [cx + r, cy], [cx, cy + r], [cx - r, cy]],
            color,
        );
    }

    pub fn diamond_outline(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        t: f32,
        color: u32,
        fill: u32,
    ) {
        self.diamond(cx, cy, radius, color);
        self.diamond(cx, cy, (radius - t).max(1.0), fill);
    }

    /// A right-pointing chevron, `w` wide and `h` tall, at `(x, y)`.
    pub fn chevron(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        let t = (h * 0.35).max(1.5);
        self.quad(
            [
                [x, y],
                [x + w * 0.6, y + h * 0.5],
                [x, y + h],
                [x + t, y + h * 0.5],
            ],
            color,
        );
    }

    /// A left-pointing chevron, the mirror of [`Layer::chevron`].
    pub fn chevron_left(&mut self, x: f32, y: f32, w: f32, h: f32, color: u32) {
        let t = (h * 0.35).max(1.5);
        self.quad(
            [
                [x + w, y],
                [x + w - w * 0.6, y + h * 0.5],
                [x + w, y + h],
                [x + w - t, y + h * 0.5],
            ],
            color,
        );
    }

    // -------------------------------------------------------------- bars

    /// A System bar: dark track, grey ghost of the lost value, gradient fill,
    /// a hot top line and ten notches.
    pub fn bar(&mut self, r: Rect, ratio: f32, ghost: f32, fill: u32) {
        if r.w <= 2.0 || r.h <= 1.0 { return; }
        let ratio=unit(ratio);
        let ghost=unit(ghost).max(ratio);
        self.chamfer(r,1.5,alpha(0xFF05080D,0xDC));
        let inner=Rect{x:r.x+1.0,y:r.y+1.0,w:(r.w-2.0).max(0.0),h:(r.h-2.0).max(1.0)};
        if ghost>ratio+0.001 {
            self.rect(Rect{x:inner.x,y:inner.y+inner.h-1.5,
                w:inner.w*ghost,h:1.5},GHOST);
        }
        let w=inner.w*ratio;
        if w>0.4 {
            let body=Rect{x:inner.x,y:inner.y,w,h:inner.h};
            self.rect(body,darken(fill,0.28));
            self.rect(Rect{x:body.x,y:body.y,w:body.w,
                h:(body.h*0.48).max(1.0)},fill);
            self.rect(Rect{x:body.x,y:body.y,w:body.w,h:1.0},
                alpha(lighten(fill,0.55),0xE0));
            if body.w>2.0 {
                self.rect(Rect{x:body.x+body.w-1.0,y:body.y,
                    w:1.0,h:body.h},lighten(fill,0.68));
            }
        }
        self.chamfer_outline(r,1.5,1.0,alpha(LINE,0x56));
    }

    /// A segmented meter (hunger, air) drawn as small diamonds.
    pub fn pips(&mut self, x: f32, y: f32, count: u32, filled: u32, color: u32, gap: f32) {
        for i in 0..count {
            let cx = x + i as f32 * gap;
            if i < filled {
                self.diamond(cx, y, 2.2, color);
            } else {
                self.diamond(cx, y, 2.2, alpha(MUTED, 0x50));
            }
        }
    }

    /// A small angular tag with text.
    pub fn chip(&mut self, r: Rect, text: &str, color: u32) {
        self.rect(r, alpha(0xFF08161F, 0xE0));
        self.border(r, alpha(color, 0xA0));
        self.rect(
            Rect {
                x: r.x,
                y: r.y,
                w: 2.0,
                h: r.h,
            },
            alpha(color, 0xC0),
        );
        // The label is fitted to the plate, so a wider fallback font in a
        // preview can never spill over the border.
        let label = fit(text, r.w - 8.0);
        self.text_center(&label, r, r.y + (r.h - 9.0) * 0.5, color);
    }

    /// Reusable description surface. The caller draws body rows in the
    /// returned rectangle; its geometry never depends on typewriter progress.
    pub fn description_box(&mut self, r:Rect, title:&str, subtitle:Option<&str>, accent:u32)->Rect {
        self.chamfer(r,CUT_MD,SURFACE_LOW);
        self.chamfer_outline(r,CUT_MD,1.0,alpha(LINE,0x64));
        self.rect(Rect{x:r.x,y:r.y+CUT_MD,w:2.0,h:(r.h-CUT_MD*2.0).max(0.0)},alpha(accent,0xC8));
        let title=fit_tracked(&title.to_uppercase(),(r.w-18.0).max(8.0),0.55);
        self.text_tracked(title,r.x+9.0,r.y+6.0,WHITE,0.55);
        let mut top=r.y+19.0;
        if let Some(sub)=subtitle.filter(|v|!v.trim().is_empty()) {
            self.text(&fit(sub,(r.w-18.0).max(8.0)),r.x+9.0,top,alpha(MUTED,0xC8));
            top+=12.0;
        }
        self.hline(r.x+9.0,top-4.0,(r.w-18.0).max(0.0),alpha(accent,0x32));
        Rect{x:r.x+9.0,y:top+1.0,w:(r.w-18.0).max(1.0),h:(r.y+r.h-top-7.0).max(1.0)}
    }

    pub fn state_button(&mut self,r:Rect,label:&str,state:UiState,accent:u32) {
        let edge=match state {
            UiState::Selected|UiState::Available=>accent,
            _=>state_edge(state),
        };
        self.chamfer(r,CUT_SM,state_fill(state));
        self.chamfer_outline(r,CUT_SM,1.0,alpha(edge,0xA8));
        if matches!(state,UiState::Selected|UiState::Available|UiState::Maxed) {
            self.rect(Rect{x:r.x+2.0,y:r.y+r.h-2.0,w:(r.w-4.0).max(0.0),h:1.0},edge);
        }
        let tc=if matches!(state,UiState::Disabled|UiState::Locked){MUTED}else{WHITE};
        self.text_middle(&fit_word(label,(r.w-6.0).max(4.0)),r,tc);
    }

    /// A tab button: a thin light rule over dark glass, filled when selected.
    pub fn tab(&mut self, r: Rect, text: &str, selected: bool, hover: bool, accent: u32) {
        let state=if selected {UiState::Selected}else if hover {UiState::Hover}else{UiState::Normal};
        self.chamfer(r,CUT_SM,state_fill(state));
        self.chamfer_outline(r,CUT_SM,1.0,alpha(state_edge(state),if selected{0xC8}else{0x62}));
        if selected {
            self.rect(Rect{x:r.x+3.0,y:r.y+r.h-2.0,w:(r.w-6.0).max(0.0),h:1.5},accent);
        }
        let color=if selected{WHITE}else if hover{MOON}else{alpha(MOON,0xB8)};
        let text=&fit_tracked(text,r.w-6.0,0.45);
        let w=tracked_width(text,0.45);
        self.text_tracked(text,r.x+(r.w-w)*0.5,r.y+(r.h-9.0)*0.5,color,0.45);
    }
}

/// Like `fit`, but cuts at the last whole word before the ellipsis
/// ("VELOCIDADE DE…" instead of "VELOCIDADE D…"). Falls back to `fit` when the
/// first word alone doesn't fit. UTF-8 safe: cuts only at char boundaries.
pub fn fit_word(s: &str, max: f32) -> String {
    let hard = fit(s, max);
    if hard == s {
        return hard;
    }
    let kept = hard.strip_suffix('…').unwrap_or(&hard);
    // A cut exactly at a word boundary keeps the whole kept prefix.
    if s[kept.len()..].starts_with(' ') {
        return format!("{}…", kept.trim_end());
    }
    match kept.rfind(' ') {
        Some(i) if i > 0 => format!("{}…", kept[..i].trim_end()),
        _ => hard,
    }
}

// ------------------------------------------------------------------ text metrics

/// Approximate Minecraft advance width of `s` at the default 6 px grid. Used
/// only for centring and truncation; the font does the real shaping.
/// A 0..=1 ratio that is safe to draw: NaN and infinities from partial
/// server data become 0 instead of NaN geometry (or a clamp panic).
pub fn unit(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn char_width(c: char) -> f32 {
    match c {
        // Advance widths of the vanilla bitmap font (glyph plus its one pixel
        // gap). Overestimating a letter pushes every following one to the
        // right, which is what made the tracked labels look uneven.
        ' ' => 4.0,
        'i' | '.' | ',' | ':' | ';' | '|' | '!' => 2.0,
        'l' | '\'' | '`' => 3.0,
        'I' | 't' | '[' | ']' | '(' | ')' => 4.0,
        '"' | '*' | '<' | '>' | 'f' | 'k' => 5.0,
        '@' => 7.0,
        _ => 6.0,
    }
}

pub fn text_width(s: &str) -> f32 {
    // `§x` colour codes take no space.
    let mut w = 0.0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '§' {
            let _ = chars.next();
            continue;
        }
        w += char_width(c);
    }
    w
}

/// Like `fit`, for text drawn with `text_tracked(.., track)`: the result
/// never exceeds `max` once the tracking is added.
pub fn fit_tracked(s: &str, max: f32, track: f32) -> String {
    let track = if track.is_finite() {
        track.max(0.0)
    } else {
        0.0
    };
    if tracked_width(s, track) <= max {
        return s.to_string();
    }
    let ell = char_width('…');
    let mut out = String::new();
    let mut w = 0.0;
    for c in s.chars() {
        let cw = track_cell(c) + track;
        if w + cw + ell > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    let out = out.trim_end().to_string();
    out + "…"
}

/// Cut `s` to at most `max` GUI pixels.
pub fn fit(s: &str, max: f32) -> String {
    if text_width(s) <= max {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0.0;
    for c in s.chars() {
        let cw = char_width(c);
        if w + cw > max - 4.0 {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push('…');
    out
}

// ------------------------------------------------------------------ text metrics

/// Width of `s` once `track` pixels are added between every pair of glyphs.
/// Cell advance used by tracked text. Every glyph gets the same cell, so the
/// letter spacing stays even instead of opening a gap after each narrow letter.
/// Advance of one tracked glyph: the real glyph width, so tracked labels have
/// even optical gaps instead of a fixed 6 px cell that left holes after
/// narrow letters (i, l, t, I).
pub fn track_cell(c: char) -> f32 {
    char_width(c)
}

pub fn tracked_width(s: &str, track: f32) -> f32 {
    let mut w = 0.0;
    let mut n = 0usize;
    for c in s.chars() {
        w += track_cell(c);
        n += 1;
    }
    if n == 0 {
        return 0.0;
    }
    w + track * (n as f32 - 1.0)
}

// ------------------------------------------------------------------ HUD model

#[derive(Clone, Debug, PartialEq)]
pub struct BarModel {
    pub ratio: f32,
    /// Where the grey trail sits (>= ratio).
    pub ghost: f32,
    pub label: String,
    pub value: String,
}

impl Default for BarModel {
    fn default() -> Self {
        Self {
            ratio: 0.0,
            ghost: 0.0,
            label: String::new(),
            value: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TargetModel {
    pub name: String,
    pub level: u32,
    pub tier: String,
    pub rank: String,
    pub ratio: f32,
    pub ghost: f32,
    pub hp_text: String,
    pub badges: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToastModel {
    pub title: String,
    pub body: String,
    pub age: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FloatModel {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub color: u32,
    pub alpha: f32,
    pub crit: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudModel {
    pub width: f32,
    pub height: f32,
    pub tick: u32,
    pub level: u32,
    pub rank: String,
    pub name: String,
    pub xp: f32,
    pub points: u64,
    pub hp: BarModel,
    pub mp: BarModel,
    pub stamina: BarModel,
    /// Absorption as a fraction of the health bar (the real shield).
    pub shield: f32,
    pub food: u32,
    pub air: Option<f32>,
    pub exhausted: bool,
    pub target: Option<TargetModel>,
    pub toasts: Vec<ToastModel>,
    pub floats: Vec<FloatModel>,
}

pub const PLAYER_H:f32=61.0;
pub const TARGET_W:f32=174.0;
pub const TARGET_H:f32=48.0;

/// The player card sits at the top left: clear of the hotbar, the chat lines
/// and the touch controls at the bottom of a phone screen.
pub fn hud_player_rect(width:f32,_height:f32)->Rect {
    let width=if width.is_finite(){width.max(0.0)}else{0.0};
    let x=width.min(6.0);
    let available=(width-x-6.0).max(0.0);
    let w=(width*0.38).clamp(164.0,210.0).min(available);
    Rect{x,y:6.0,w,h:PLAYER_H}
}

/// The target card at the top right, pushed below the player card when the
/// screen is too narrow for both on one row.
pub fn hud_target_rect(width:f32)->Rect {
    let width=if width.is_finite(){width.max(0.0)}else{0.0};
    let w=TARGET_W.min((width-10.0).max(0.0));
    let player=hud_player_rect(width,0.0);
    let x=(width-6.0-w).max(0.0);
    let y=if x<player.x+player.w+5.0 {player.y+player.h+5.0}else{6.0};
    Rect{x,y,w,h:TARGET_H}
}

/// Colour of a bar by how full it is: bright while healthy, amber, then red.
fn level_color(ratio: f32, base: u32) -> u32 {
    if ratio <= 0.22 {
        RED
    } else if ratio <= 0.45 {
        AMBER
    } else {
        base
    }
}

/// One labelled row of the player card: label, bar, value.
fn stat_row(l: &mut Layer, r: Rect, b: &BarModel, color: u32, shield: f32) {
    let color = level_color(b.ratio, color);
    l.text_tracked(
        &fit_tracked(&b.label.to_uppercase(), 24.0, 0.4),
        r.x,
        r.y + 1.0,
        alpha(MUTED, 0xD0),
        0.4,
    );
    let bar = Rect {
        x: r.x + 26.0,
        y: r.y,
        w: (r.w - 26.0 - 48.0).max(20.0),
        h: 8.0,
    };
    l.bar(bar, b.ratio, b.ghost.max(b.ratio), color);
    // Absorption rides on top of the health bar: a pale sleeve past the fill.
    if shield > 0.004 {
        let start = (bar.x + bar.w * b.ratio).min(bar.x + bar.w);
        let end = (bar.x + bar.w * (b.ratio + shield).min(1.0)).min(bar.x + bar.w);
        if end > start {
            l.rect(
                Rect {
                    x: start,
                    y: bar.y + 1.0,
                    w: end - start,
                    h: bar.h - 2.0,
                },
                alpha(ICE, 0x70),
            );
            l.rect(
                Rect {
                    x: start,
                    y: bar.y + 1.0,
                    w: 1.0,
                    h: bar.h - 2.0,
                },
                alpha(WHITE, 0xC0),
            );
        }
    }
    let v = fit(&b.value, 46.0);
    l.text(&v, r.x + r.w - text_width(&v), r.y + 1.0, alpha(ICE, 0xE6));
}

/// Builds the whole playing HUD.
pub fn draw_hud(m:&HudModel)->Layer {
    let mut l=Layer::new();
    let pulse=if m.tick%40<20{1.0}else{0.62};
    let p=hud_player_rect(m.width,m.height);
    let accent=if m.exhausted{RED}else{AETHER};

    // Player HUD: one compact identity rail. HP owns the strongest geometry,
    // stamina is shorter/thinner, MP is tertiary and XP is only a gold thread.
    l.chamfer(p,3.0,alpha(SURFACE,0xE8));
    l.chamfer_outline(p,3.0,1.0,alpha(LINE,0x70));
    l.rect(Rect{x:p.x,y:p.y+3.0,w:2.0,h:p.h-6.0},alpha(accent,0xC8));

    let rank=if m.rank.trim().is_empty(){"E"}else{m.rank.as_str()};
    l.diamond_outline(p.x+13.0,p.y+13.0,7.5,1.0,alpha(GOLD,0xD0),0x00000000);
    l.text(rank,p.x+13.0-text_width(rank)*0.5,p.y+9.0,GOLD);
    let name=fit_tracked(&m.name.to_uppercase(),(p.w-75.0).max(40.0),0.45);
    l.text_tracked(name,p.x+25.0,p.y+5.0,WHITE,0.45);
    let lv=format!("NV {}",m.level.max(1));
    l.text(&lv,p.x+p.w-6.0-text_width(&lv),p.y+5.0,alpha(MOON,0xD0));

    let hp_label=if m.hp.label.is_empty(){"HP"}else{m.hp.label.as_str()};
    l.text_tracked(&hp_label.to_uppercase(),p.x+7.0,p.y+20.0,alpha(MUTED,0xC8),0.35);
    let hp_value=fit(&m.hp.value,(p.w*0.42).max(38.0));
    l.text(&hp_value,p.x+p.w-7.0-text_width(&hp_value),p.y+20.0,MOON);
    let hp=Rect{x:p.x+7.0,y:p.y+30.0,w:p.w-14.0,h:9.0};
    l.bar(hp,m.hp.ratio,m.hp.ghost.max(m.hp.ratio),level_color(m.hp.ratio,HP));
    if m.shield>0.004 {
        let start=hp.x+hp.w*unit(m.hp.ratio);
        let end=hp.x+hp.w*unit(m.hp.ratio+m.shield);
        if end>start {
            l.rect(Rect{x:start.min(hp.x+hp.w),y:hp.y+1.0,
                w:(end.min(hp.x+hp.w)-start.min(hp.x+hp.w)).max(0.0),
                h:hp.h-2.0},alpha(SHIELD,0x88));
        }
    }

    l.text_tracked("STA",p.x+7.0,p.y+43.0,
        alpha(if m.exhausted{RED}else{MUTED},if m.exhausted{0xFF}else{0xC0}),0.3);
    let stamina=Rect{x:p.x+31.0,y:p.y+43.0,
        w:((p.w-39.0)*0.82).max(40.0),h:6.0};
    l.bar(stamina,m.stamina.ratio,m.stamina.ghost.max(m.stamina.ratio),
        level_color(m.stamina.ratio,STAMINA));
    if m.exhausted {
        l.chamfer_outline(stamina,1.5,1.0,alpha(RED,(0x70 as f32*pulse) as u8+0x40));
    }

    if m.mp.ratio>0.0 || !m.mp.value.is_empty() {
        let mana=Rect{x:p.x+31.0,y:p.y+52.0,
            w:((p.w-39.0)*0.62).max(32.0),h:3.0};
        l.text_tracked("MP",p.x+7.0,p.y+50.0,alpha(MUTED,0x9C),0.3);
        l.bar(mana,m.mp.ratio,m.mp.ghost.max(m.mp.ratio),MANA);
    }
    let xp=Rect{x:p.x+4.0,y:p.y+p.h-2.0,w:p.w-8.0,h:1.5};
    l.rect(xp,alpha(0xFF080B10,0xD0));
    l.rect(Rect{w:xp.w*unit(m.xp),..xp},XP);

    let mut chip_y=p.y+p.h+3.0;
    if m.points>0 {
        let tag=format!("+{} PONTOS",m.points);
        let w=(text_width(&tag)+10.0).min(p.w);
        l.chip(Rect{x:p.x,y:chip_y,w,h:11.0},&tag,GOLD);
        chip_y+=13.0;
    }
    if let Some(air)=m.air {
        let r=Rect{x:p.x,y:chip_y,w:(p.w*0.62).max(80.0),h:10.0};
        l.text("AR",r.x+3.0,r.y+1.0,alpha(MUTED,0xC0));
        l.bar(Rect{x:r.x+20.0,y:r.y+2.0,w:r.w-23.0,h:5.0},air,air,AETHER);
    }

    if let Some(t)=&m.target {
        let r=hud_target_rect(m.width);
        let edge=if t.ratio<=0.22{RED}else{AMBER};
        // An open combat rail instead of a floating black card.
        l.chamfer(r,3.0,alpha(SURFACE_LOW,0xC8));
        l.rect(Rect{x:r.x,y:r.y+3.0,w:2.0,h:r.h-6.0},alpha(edge,0xD0));
        let name=fit_tracked(&t.name.to_uppercase(),r.w-48.0,0.4);
        l.text_tracked(name,r.x+7.0,r.y+4.0,WHITE,0.4);
        let badge=if t.rank.is_empty(){"-"}else{t.rank.as_str()};
        l.text(badge,r.x+r.w-8.0-text_width(badge),r.y+4.0,GOLD);
        let sub=format!("NV {} · {}",t.level,t.tier.to_uppercase());
        l.text(&fit(&sub,r.w-14.0),r.x+7.0,r.y+15.0,alpha(MUTED,0xC0));
        let bar=Rect{x:r.x+7.0,y:r.y+27.0,w:r.w-14.0,h:7.0};
        l.bar(bar,t.ratio,t.ghost.max(t.ratio),level_color(t.ratio,RED));
        let hp=fit(&t.hp_text,64.0);
        l.text(&hp,r.x+r.w-7.0-text_width(&hp),r.y+37.0,MOON);
        let mut x=r.x+7.0;
        for b in t.badges.iter().take(2) {
            let label=fit(&b.to_uppercase(),42.0);
            let w=text_width(&label)+7.0;
            if x+w>r.x+r.w-72.0{break;}
            l.text(&label,x,r.y+37.0,alpha(AMBER,0xD0));
            x+=w+4.0;
        }
    }

    let tw=(m.width*0.30).clamp(144.0,220.0);
    let narrow=m.width<560.0;
    let tx=if narrow{m.width-6.0-tw}else{(m.width-tw)*0.5};
    let mut ty=if narrow{hud_target_rect(m.width).y+TARGET_H+4.0}else{6.0};
    for toast in m.toasts.iter().take(3) {
        let fade=(1.0-toast.age as f32/120.0).clamp(0.0,1.0);
        let r=Rect{x:tx,y:ty,w:tw,h:if toast.title.is_empty(){20.0}else{28.0}};
        l.chamfer(r,2.0,alpha(SURFACE,(0xD8 as f32*fade) as u8));
        l.rect(Rect{x:r.x,y:r.y+2.0,w:2.0,h:r.h-4.0},alpha(GOLD,(0xD0 as f32*fade) as u8));
        let mut y=r.y+3.0;
        if !toast.title.is_empty() {
            l.text_tracked(&fit_tracked(&toast.title.to_uppercase(),r.w-12.0,0.4),
                r.x+7.0,y,fade_color(GOLD,fade),0.4);
            y+=10.0;
        }
        l.text(&fit(&toast.body,r.w-12.0),r.x+7.0,y,fade_color(MOON,fade.max(0.55)));
        ty+=r.h+3.0;
    }

    for f in m.floats.iter().take(16) {
        let a=unit(f.alpha); if a<=0.02{continue;}
        let text=if f.crit{format!("{}!",f.text)}else{f.text.clone()};
        let w=text_width(&text); let x=f.x-w*0.5;
        // Native shadow plus a tiny critical spark is enough; no black plaque.
        l.text_flat(&text,x+1.0,f.y+1.0,alpha(0xFF05070A,(0xB0 as f32*a) as u8));
        l.text(&text,x,f.y,fade_color(f.color,a));
        if f.crit {
            l.hline(x,f.y+9.0,w,alpha(GOLD,(0xB0 as f32*a) as u8));
            l.diamond(x-4.0,f.y+4.0,1.8,alpha(GOLD,(0xD0 as f32*a) as u8));
        }
    }
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn chamfer_stays_inside_the_rect() {
        let mut l = Layer::new();
        l.chamfer(r(10.0, 10.0, 100.0, 40.0), 6.0, WHITE);
        for p in &l.prims {
            match p {
                Prim::Rect { x, y, w, h, .. } => {
                    assert!(*x >= 9.999 && *y >= 9.999);
                    assert!(*x + *w <= 110.001 && *y + *h <= 50.001);
                }
                Prim::Quad { p, .. } => {
                    for q in p {
                        assert!(q[0] >= 9.999 && q[0] <= 110.001);
                        assert!(q[1] >= 9.999 && q[1] <= 50.001);
                    }
                }
                Prim::Text { .. } | Prim::Sprite { .. } => {}
            }
        }
    }

    #[test]
    fn bar_fill_never_exceeds_the_track() {
        let mut l = Layer::new();
        let track = r(0.0, 0.0, 120.0, 8.0);
        l.bar(track, 0.5, 0.9, RED);
        let mut filled = 0.0_f32;
        for p in &l.prims {
            if let Prim::Rect { x, y, w, h, color } = p {
                if *color == darken(RED, 0.28) {
                    filled = filled.max(*w);
                    assert!(*x >= track.x + 0.999);
                    assert!(*y >= track.y + 0.999);
                    assert!(*h <= track.h - 1.999);
                }
            }
        }
        assert!((filled - 59.0).abs() < 0.01, "{filled}");
    }

    #[test]
    fn text_metrics_ignore_colour_codes() {
        assert_eq!(text_width("§cHP"), text_width("HP"));
        assert!(text_width("MMMM") > text_width("iiii"));
        assert!(fit("BESTIARIO LONGO DEMAIS", 30.0).ends_with('…'));
    }

    #[test]
    fn hud_layers_do_not_leave_the_screen() {
        let m = HudModel {
            width: 320.0,
            height: 180.0,
            level: 7,
            rank: "D".into(),
            name: "JIN-WOO".into(),
            xp: 0.4,
            points: 3,
            hp: BarModel {
                ratio: 0.6,
                ghost: 0.8,
                label: "HP".into(),
                value: "120/200".into(),
            },
            mp: BarModel {
                ratio: 0.3,
                ghost: 0.3,
                label: "MP".into(),
                value: "30/100".into(),
            },
            stamina: BarModel {
                ratio: 0.2,
                ghost: 0.2,
                label: "STA".into(),
                value: String::new(),
            },
            food: 17,
            air: Some(0.5),
            exhausted: true,
            target: Some(TargetModel {
                name: "GOBLIN".into(),
                level: 3,
                tier: "ELITE".into(),
                rank: "D".into(),
                ratio: 0.5,
                ghost: 0.9,
                hp_text: "50/100".into(),
                badges: vec!["SANGRANDO".into()],
            }),
            toasts: vec![ToastModel {
                title: "NÍVEL 8".into(),
                body: "Novo nível alcançado".into(),
                age: 10,
            }],
            floats: vec![FloatModel {
                text: "-42".into(),
                x: 160.0,
                y: 90.0,
                color: RED,
                alpha: 0.8,
                crit: true,
            }],
            ..Default::default()
        };
        let l = draw_hud(&m);
        assert!(l.len() > 24);
        let p = hud_player_rect(320.0, 180.0);
        assert!(p.x + p.w <= 320.0 && p.y + p.h <= 180.0);
        let t = hud_target_rect(320.0);
        assert!(t.x + t.w <= 320.0);
    }

    #[test]
    fn hud_cards_fit_narrow_phone_widths() {
        for width in [160.0, 240.0, 320.0, 700.0] {
            let player = hud_player_rect(width, 180.0);
            let target = hud_target_rect(width);
            assert!(
                player.x >= 0.0 && player.x + player.w <= width,
                "player at {width}"
            );
            assert!(
                target.x >= 0.0 && target.x + target.w <= width,
                "target at {width}"
            );
        }
    }

    #[test]
    fn exhausted_hud_uses_the_danger_accent() {
        let base = HudModel {
            width: 300.0,
            height: 200.0,
            ..Default::default()
        };
        let calm = draw_hud(&base);
        let tired = draw_hud(&HudModel {
            exhausted: true,
            ..base
        });
        assert_ne!(calm, tired);
    }
}


#[cfg(test)] mod quad_clip_tests {
    use super::*;
    #[test]fn clipped_node_glow_cannot_cover_fixed_description_panel(){
        let mut layer=Layer::new();
        let clip=Rect{x:15.0,y:22.0,w:80.0,h:40.0};
        layer.set_clip(Some(clip));
        layer.quad([[5.0,15.0],[105.0,15.0],[105.0,70.0],[5.0,70.0]],0xFFFFFFFF);
        assert!(!layer.prims.is_empty());
        for primitive in &layer.prims {
            if let Prim::Quad{p,..}=primitive {
                for pt in p {
                    assert!(pt[0]>=clip.x-0.01 && pt[0]<=clip.x+clip.w+0.01);
                    assert!(pt[1]>=clip.y-0.01 && pt[1]<=clip.y+clip.h+0.01);
                }
            }
        }
        let before=layer.len();
        layer.quad([[0.0,0.0],[3.0,0.0],[3.0,3.0],[0.0,3.0]],0xFFFFFFFF);
        assert_eq!(layer.len(),before);
        layer.set_clip(None);
        layer.quad([[0.0,0.0],[3.0,0.0],[3.0,3.0],[0.0,3.0]],0xFFFFFFFF);
        assert_eq!(layer.len(),before+1);
    }
}
