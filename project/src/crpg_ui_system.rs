// Solo Leveling "System" skin: palette, angular geometry and the playing HUD.
//
// The module is pure: it only builds a `Layer` of primitives. The Horizon
// adapter paints each `Prim` with `Host::fill` (rectangles), `Host::quad`
// (chamfered corners, chevrons, diamonds) and `Host::text`. No texture, sprite
// or shader is invented here, and every number comes from the RPG state the
// caller fills in.

pub use super::crpg_ui_layout::Rect;

// ------------------------------------------------------------------ palette

/// Deep panel body (ARGB).
pub const INK: u32 = 0xE6050B14;
pub const INK_SOFT: u32 = 0xC8071220;
pub const PANEL: u32 = 0xE00A1626;
pub const PANEL_HI: u32 = 0xE60E2036;
pub const LINE: u32 = 0x8036C6F0;
pub const CYAN: u32 = 0xFF4FE3FF;
pub const CYAN_DIM: u32 = 0x664FE3FF;
pub const ICE: u32 = 0xFFBFEFFF;
pub const WHITE: u32 = 0xFFF2FBFF;
pub const MUTED: u32 = 0xFF8FA6C0;
pub const GOLD: u32 = 0xFFFFD36E;
pub const RED: u32 = 0xFFFF4D6D;
pub const RED_DARK: u32 = 0xFF4A0F1A;
pub const VIOLET: u32 = 0xFF9B7BFF;
pub const GREEN: u32 = 0xFF57F2B0;
pub const AMBER: u32 = 0xFFFFA94D;
/// The grey trail a bar leaves after losing value.
pub const GHOST: u32 = 0x44C6D4E2;

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
        if r.w<=0.0 || r.h<=0.0 {return;}
        // A imagem nao pode escapar da area recortada de uma arvore.
        if self.clip.is_some_and(|c|r.x<c.x || r.y<c.y ||
            r.x+r.w>c.x+c.w || r.y+r.h>c.y+c.h) {return;}
        self.prims.push(Prim::Sprite {key,src,dst:[r.x,r.y,r.w,r.h],tint:None});
    }

    pub fn quad(&mut self, p: [[f32; 2]; 4], color: u32) {
        self.prims.push(Prim::Quad { p, color });
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
        self.chamfer(r, 3.0, 0xF0091626);
        self.grid(r, 16.0, 0x054FE3FF);
        self.rect(
            Rect {
                x: r.x + 5.0,
                y: r.y + 1.0,
                w: (r.w * 0.24).min(54.0),
                h: 1.0,
            },
            alpha(accent, 0x80),
        );
        self.chamfer_outline(r, 3.0, 1.0, alpha(accent, 0x58));
        self.hline(r.x + r.w - 8.0, r.y, 6.0, alpha(accent, 0xB0));
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

    /// The System window: halo, translucent body, inner sheen, scanlines and
    /// corner brackets. The core of the Solo Leveling look.
    pub fn panel(&mut self, r: Rect, accent: u32) {
        self.glow(r, 6.0, 3.0, accent, 3);
        self.chamfer(r, 6.0, PANEL);
        // Upper sheen: two translucent bands, the "glass" of the window.
        self.rect(
            Rect {
                x: r.x + 6.0,
                y: r.y + 1.0,
                w: r.w - 12.0,
                h: (r.h * 0.34).min(18.0),
            },
            alpha(0xFF7FD8FF, 0x12),
        );
        self.scanlines(
            Rect {
                x: r.x + 6.0,
                y: r.y + 6.0,
                w: r.w - 12.0,
                h: r.h - 12.0,
            },
            4.0,
            0x05FF_FFFF,
        );
        self.grid(
            Rect {
                x: r.x + 3.0,
                y: r.y + 3.0,
                w: r.w - 6.0,
                h: r.h - 6.0,
            },
            16.0,
            0x044FE3FF,
        );
        self.chamfer_outline(r, 6.0, 1.0, alpha(accent, 0xB0));
        self.chamfer_outline(
            Rect {
                x: r.x + 3.0,
                y: r.y + 3.0,
                w: r.w - 6.0,
                h: r.h - 6.0,
            },
            4.0,
            1.0,
            alpha(accent, 0x22),
        );
        self.corner_brackets(r, 9.0, 1.5, accent);
        self.rect(
            Rect {
                x: r.x + 14.0,
                y: r.y - 1.0,
                w: (r.w * 0.26).min(60.0),
                h: 2.0,
            },
            accent,
        );
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
        if r.w <= 2.0 || r.h <= 1.0 {
            return;
        }
        let ratio = unit(ratio);
        let ghost = unit(ghost).max(ratio);
        self.chamfer(r, 2.0, 0xE602060C);
        if ghost > ratio + 0.001 {
            self.rect(
                Rect {
                    x: r.x + 1.0,
                    y: r.y + (r.h * 0.5 - 1.0).max(1.0),
                    w: (r.w - 2.0) * ghost,
                    h: 2.0,
                },
                GHOST,
            );
        }
        let w = (r.w - 2.0) * ratio;
        if w > 0.5 {
            let body = Rect {
                x: r.x + 1.0,
                y: r.y + 1.0,
                w,
                h: r.h - 2.0,
            };
            self.rect(body, darken(fill, 0.28));
            self.rect(
                Rect {
                    h: (body.h * 0.55).max(1.0),
                    ..body
                },
                fill,
            );
            self.rect(
                Rect {
                    x: body.x,
                    y: body.y,
                    w: body.w,
                    h: 1.0,
                },
                lighten(fill, 0.55),
            );
            // The leading edge catches the light.
            self.rect(
                Rect {
                    x: body.x + body.w - 1.0,
                    y: body.y,
                    w: 1.0,
                    h: body.h,
                },
                lighten(fill, 0.75),
            );
        }
        let mut x = r.x + r.w / 5.0;
        for _ in 1..5 {
            self.vline(x, r.y + 2.0, (r.h - 4.0).max(1.0), 0x1800_0000);
            x += r.w / 5.0;
        }
        self.chamfer_outline(r, 2.0, 1.0, alpha(CYAN, 0x50));
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

    /// A tab button: a thin light rule over dark glass, filled when selected.
    pub fn tab(&mut self, r: Rect, text: &str, selected: bool, hover: bool, accent: u32) {
        let body = if selected {
            alpha(0xFF14314A, 0xF0)
        } else if hover {
            alpha(0xFF0E2436, 0xE0)
        } else {
            alpha(0xFF081520, 0xD0)
        };
        self.chamfer(r, 2.0, body);
        if selected {
            self.rect(
                Rect {
                    x: r.x + 1.0,
                    y: r.y + 1.0,
                    w: r.w - 2.0,
                    h: (r.h * 0.5).max(1.0),
                },
                alpha(accent, 0x22),
            );
        }
        let edge = if selected {
            accent
        } else if hover {
            alpha(accent, 0x90)
        } else {
            alpha(accent, 0x38)
        };
        self.chamfer_outline(r, 2.0, 1.0, edge);
        if selected {
            self.rect(
                Rect {
                    x: r.x + 1.0,
                    y: r.y + r.h - 2.0,
                    w: r.w - 2.0,
                    h: 1.0,
                },
                accent,
            );
        }
        let color = if selected {
            WHITE
        } else if hover {
            ICE
        } else {
            alpha(ICE, 0xB0)
        };
        let text = &fit_tracked(text, r.w - 6.0, 0.8);
        let w = tracked_width(text, 0.8);
        self.text_tracked(
            text,
            r.x + (r.w - w) * 0.5,
            r.y + (r.h - 9.0) * 0.5,
            color,
            0.8,
        );
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

pub const PLAYER_H: f32 = 80.0;
pub const TARGET_W: f32 = 156.0;
pub const TARGET_H: f32 = 58.0;

/// The player card sits at the top left: clear of the hotbar, the chat lines
/// and the touch controls at the bottom of a phone screen.
pub fn hud_player_rect(width: f32, _height: f32) -> Rect {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let x = width.min(6.0);
    let available = (width - x - 6.0).max(0.0);
    let w = (width * 0.40).clamp(170.0, 198.0).min(available);
    Rect {
        x,
        y: 6.0,
        w,
        h: PLAYER_H,
    }
}

/// The target card at the top right, pushed below the player card when the
/// screen is too narrow for both on one row.
pub fn hud_target_rect(width: f32) -> Rect {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let w = TARGET_W.min((width - 10.0).max(0.0));
    let player = hud_player_rect(width, 0.0);
    let x = (width - 6.0 - w).max(0.0);
    let y = if x < player.x + player.w + 4.0 {
        player.y + player.h + 4.0
    } else {
        6.0
    };
    Rect {
        x,
        y,
        w,
        h: TARGET_H,
    }
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
pub fn draw_hud(m: &HudModel) -> Layer {
    let mut l = Layer::new();
    let pulse = if m.tick % 40 < 20 { 1.0 } else { 0.55 };
    let accent = if m.exhausted { alpha(RED, 0xE0) } else { CYAN };

    // ---- player card -------------------------------------------------
    let p = hud_player_rect(m.width, m.height);
    l.panel(p, accent);

    // Rank badge: a diamond with the rank letter, the character's "license".
    let cx = p.x + 20.0;
    let cy = p.y + 21.0;
    l.diamond_outline(cx, cy, 12.0, 1.5, alpha(accent, 0xE0), 0xF00A1A2A);
    l.diamond(cx, cy, 9.5, alpha(0xFF0E2C46, 0xE0));
    let rank = if m.rank.is_empty() {
        "E".to_string()
    } else {
        m.rank.clone()
    };
    l.text(
        &rank,
        cx - text_width(&rank) * 0.5,
        cy - 4.0,
        if m.exhausted { RED } else { GOLD },
    );

    let name = fit_tracked(&m.name.to_uppercase(), p.w - 96.0, 0.7);
    l.text_tracked(&name, p.x + 38.0, p.y + 8.0, WHITE, 0.7);
    let lv = format!("NV {}", m.level.max(1));
    l.text(
        &lv,
        p.x + p.w - 8.0 - text_width(&lv),
        p.y + 8.0,
        alpha(CYAN, 0xF0),
    );

    // XP: a thin gold thread under the name, with its own track.
    let xp = Rect {
        x: p.x + 38.0,
        y: p.y + 21.0,
        w: p.w - 46.0,
        h: 3.0,
    };
    l.rect(xp, alpha(0xFF061420, 0xF0));
    l.rect(
        Rect {
            x: xp.x,
            y: xp.y,
            w: xp.w * unit(m.xp),
            h: xp.h,
        },
        alpha(GOLD, 0xD0),
    );
    l.rect(
        Rect {
            x: xp.x,
            y: xp.y,
            w: xp.w,
            h: 1.0,
        },
        alpha(0xFF071826, 0xF0),
    );

    l.hline(p.x + 8.0, p.y + 28.0, p.w - 16.0, alpha(CYAN, 0x38));

    let row_x = p.x + 8.0;
    let row_w = p.w - 16.0;
    let mut y = p.y + 32.0;
    stat_row(
        &mut l,
        Rect {
            x: row_x,
            y,
            w: row_w,
            h: 10.0,
        },
        &m.hp,
        CYAN,
        m.shield,
    );
    y += 13.0;
    stat_row(
        &mut l,
        Rect {
            x: row_x,
            y,
            w: row_w,
            h: 10.0,
        },
        &m.mp,
        VIOLET,
        0.0,
    );
    y += 13.0;
    stat_row(
        &mut l,
        Rect {
            x: row_x,
            y,
            w: row_w,
            h: 10.0,
        },
        &m.stamina,
        GREEN,
        0.0,
    );

    // ---- status chips under the card ---------------------------------
    let mut chip_y = p.y + p.h + 4.0;
    if let Some(air) = m.air {
        let r = Rect {
            x: p.x,
            y: chip_y,
            w: p.w,
            h: 11.0,
        };
        l.rect(r, alpha(0xFF061420, 0xE0));
        l.border(r, alpha(CYAN, 0x40));
        l.text_tracked("AR", r.x + 4.0, r.y + 1.0, alpha(CYAN, 0xD0), 0.4);
        l.bar(
            Rect {
                x: r.x + 22.0,
                y: r.y + 2.0,
                w: r.w - 26.0,
                h: 7.0,
            },
            air,
            air,
            CYAN,
        );
        chip_y += 13.0;
    }
    if m.points > 0 {
        let tag = format!("+{} PTS", m.points);
        let r = Rect {
            x: p.x,
            y: chip_y,
            w: text_width(&tag) + 12.0,
            h: 11.0,
        };
        l.chip(r, &tag, GOLD);
        chip_y += 13.0;
    }
    if m.exhausted {
        let tag = "EXAUSTO";
        let r = Rect {
            x: p.x,
            y: chip_y,
            w: text_width(tag) + 12.0,
            h: 11.0,
        };
        l.chip(r, tag, RED);
    }

    // ---- target card -------------------------------------------------
    if let Some(t) = &m.target {
        let r = hud_target_rect(m.width);
        let hostile = t.ratio > 0.0 || !t.tier.is_empty();
        let taccent = if hostile { alpha(RED, 0xE0) } else { CYAN };
        l.panel(r, taccent);
        // A pulse dot marks a live target, so the card never reads as static.
        l.rect(
            Rect {
                x: r.x + 5.0,
                y: r.y + 5.0,
                w: 3.0,
                h: 3.0,
            },
            alpha(taccent, (0x60 as f32 * pulse) as u8 + 0x40),
        );
        let name = fit_tracked(&t.name.to_uppercase(), r.w - 48.0, 0.6);
        l.text_tracked(&name, r.x + 11.0, r.y + 4.0, WHITE, 0.6);
        // Rank badge.
        let bx = r.x + r.w - 12.0;
        let by = r.y + 10.0;
        l.diamond_outline(bx, by, 7.0, 1.0, alpha(taccent, 0xE0), 0xF00A1A2A);
        let tr = if t.rank.is_empty() {
            "-".to_string()
        } else {
            t.rank.clone()
        };
        l.text(
            &tr,
            bx - text_width(&tr) * 0.5,
            by - 3.5,
            alpha(taccent, 0xF0),
        );
        let sub = format!("NV {}  {}", t.level, t.tier.to_uppercase());
        l.text(
            &fit(&sub, r.w - 16.0),
            r.x + 8.0,
            r.y + 15.0,
            alpha(MUTED, 0xE0),
        );
        let bar = Rect {
            x: r.x + 8.0,
            y: r.y + 26.0,
            w: r.w - 16.0,
            h: 9.0,
        };
        l.bar(
            bar,
            t.ratio,
            t.ghost.max(t.ratio),
            level_color(t.ratio, RED),
        );
        let hp = fit(&t.hp_text, 56.0);
        l.text(
            &hp,
            r.x + r.w - 8.0 - text_width(&hp),
            r.y + 39.0,
            alpha(ICE, 0xE0),
        );
        let mut bxp = r.x + 8.0;
        for b in t.badges.iter().take(2) {
            let label = fit(&b.to_uppercase(), 44.0);
            let w = text_width(&label) + 8.0;
            if bxp + w > r.x + r.w - 60.0 {
                break;
            }
            l.chip(
                Rect {
                    x: bxp,
                    y: r.y + 38.0,
                    w,
                    h: 10.0,
                },
                &label,
                alpha(GOLD, 0xE0),
            );
            bxp += w + 3.0;
        }
    }

    // ---- toasts ------------------------------------------------------
    // On a narrow screen the top centre is already taken by the player and the
    // target cards, so the toasts drop to the right column, under the target.
    let tw = (m.width * 0.34).clamp(150.0, 236.0);
    let narrow = m.width < 560.0;
    let tx = if narrow {
        m.width - 6.0 - tw
    } else {
        (m.width - tw) * 0.5
    };
    let mut ty = if narrow { 68.0 } else { 6.0 };
    for toast in m.toasts.iter().take(3) {
        let fade = (1.0 - toast.age as f32 / 120.0).clamp(0.0, 1.0);
        let r = Rect {
            x: tx,
            y: ty,
            w: tw,
            h: if toast.title.is_empty() { 22.0 } else { 30.0 },
        };
        l.rect(r, alpha(0xFF08141F, (0xF0 as f32 * fade) as u8));
        l.border(r, alpha(CYAN, (0x70 as f32 * fade) as u8));
        l.rect(
            Rect {
                x: r.x,
                y: r.y,
                w: 2.0,
                h: r.h,
            },
            alpha(GOLD, (0xD0 as f32 * fade) as u8),
        );
        let mut text_y = r.y + 4.0;
        if !toast.title.is_empty() {
            l.text_tracked(
                &fit_tracked(&toast.title.to_uppercase(), r.w - 12.0, 0.7),
                r.x + 7.0,
                text_y,
                fade_color(GOLD, fade),
                0.7,
            );
            text_y += 11.0;
        }
        // The body keeps a floor of contrast: a toast that fades into the
        // panel before it is read is worse than no toast.
        l.text(
            &fit(&toast.body, r.w - 12.0),
            r.x + 7.0,
            text_y,
            fade_color(ICE, fade.max(0.55)),
        );
        ty += r.h + 4.0;
    }

    // ---- floating damage numbers -------------------------------------
    for f in m.floats.iter().take(16) {
        let a = unit(f.alpha);
        if a <= 0.02 {
            continue;
        }
        let color = fade_color(f.color, a);
        let text = if f.crit {
            format!("{}!", f.text)
        } else {
            f.text.clone()
        };
        let w = text_width(&text);
        let x = f.x - w * 0.5;
        // A dark plate keeps the number readable over bright terrain.
        l.rect(
            Rect {
                x: x - 2.0,
                y: f.y - 1.0,
                w: w + 4.0,
                h: 11.0,
            },
            alpha(0xFF04101A, (0x90 as f32 * a) as u8),
        );
        l.text(&text, x, f.y, color);
        if f.crit {
            // Critical hits get brackets and a underline spark.
            l.hline(x, f.y + 9.0, w, alpha(GOLD, (0xA0 as f32 * a) as u8));
            l.text("»", x - 6.0, f.y, alpha(GOLD, (0xC0 as f32 * a) as u8));
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
        assert!(l.len() > 40);
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
