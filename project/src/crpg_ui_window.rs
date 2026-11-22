// The System window: tab bar, page composition, the skill-tree graph and the
// bestiary cards. Drawing and hit-testing share one geometry, so what the
// player sees is exactly what a touch selects.
//
// The composition follows the reference interface: a dark glass plate with thin
// light rules, a boxed centred title, rectangular tab buttons, and inner cards
// with their own borders and a faint vertical grid.

pub use super::crpg_ui_layout::Rect;
use super::crpg_ui_system::{
    alpha, fit, fit_tracked, fit_word, text_width, tracked_width, unit, Layer, AMBER, CYAN, GOLD,
    GREEN, ICE, MUTED, RED, VIOLET, WHITE,
};

// ------------------------------------------------------------------ model

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttrRow {
    pub name: String,
    pub value: String,
    pub ratio: f32,
    pub hint: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeNode {
    pub id: String,
    pub name: String,
    pub rank: u64,
    pub max: String,
    pub cost: u64,
    pub col: usize,
    pub row: usize,
    /// 0 locked, 1 available, 2 owned, 3 maxed.
    pub state: u8,
    /// Another rank can be bought right now (same rules the server checks).
    /// Independent of `state`: an owned node stays buyable until its max rank.
    pub buyable: bool,
    pub description:String,
    pub requirement:String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeModel {
    pub name: String,
    pub nodes: Vec<TreeNode>,
    /// (parent index, child index)
    pub links: Vec<(usize, usize)>,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub selected: usize,
    /// false = graph map, true = paginated, fully readable node index.
    pub index_mode: bool,
    /// Compact GUI uses a FULL detail page instead of two unreadable panes.
    pub detail_only: bool,
    pub detail: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpeciesRow {
    pub name: String,
    pub kills: u64,
    pub level: u32,
    pub best: f64,
    pub tiers: String,
    pub drops: String,
    pub discovery: f32,
}

/// One live bar of the character sheet (HP, vigor, absorption).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vital {
    pub label: String,
    pub ratio: f32,
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GearRow {
    pub id: u64,
    pub slot: String,
    pub title: String,
    pub ability: String,
    pub stat: String,
    pub bonus: f64,
    pub rarity: u8,
    /// Estado do poder ativo calculado a partir do perfil salvo.
    pub power_status:String,
    pub power_ready:bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuModel {
    pub area: Rect,
    pub mouse: [f32; 2],
    pub tick: u32,
    pub page: usize,
    pub pages: Vec<String>,
    pub kind: &'static str,
    pub title: String,
    pub level: u32,
    pub rank: String,
    /// Next hunter rank, empty when the profile already sits at the top.
    pub next_rank: String,
    /// Progress inside the current rank band, 0..=1.
    pub rank_progress: f32,
    /// Experience towards the next level, 0..=1 (real progression data).
    pub xp: f32,
    pub points: u64,
    pub vitals: Vec<Vital>,
    pub lines: Vec<String>,
    pub attrs: Vec<AttrRow>,
    pub tree: Option<TreeModel>,
    pub bestiary: Vec<SpeciesRow>,
    pub bestiary_sel: usize,
    pub bag: Vec<GearRow>,
    pub worn: Vec<GearRow>,
    pub toast: Option<String>,
    pub buy_label: String,
    /// Page offset of the data pages (scroll / arrows), clamped when drawn.
    pub scroll: usize,
    pub detail_scroll: usize,
    /// The main system now shares the viewport with a dedicated hover inspector.
    pub external_inspector: bool,
    pub live_target: Option<super::crpg_target_hud::TargetSnapshot>,
}

// ------------------------------------------------------------------ geometry

pub const TAB_ROWS: usize = 3;
pub const TAB_COLS: usize = 5;
pub const TAB_H: f32 = 15.0;
pub const TAB_GAP: f32 = 3.0;

pub fn window(area: Rect) -> Rect {
    Rect {
        x: area.x + 2.0,
        y: area.y + 2.0,
        w: area.w - 4.0,
        h: area.h - 4.0,
    }
}

pub fn header(win: Rect) -> Rect {
    Rect {
        x: win.x + 8.0,
        y: win.y + 4.0,
        w: win.w - 16.0,
        h: 10.0,
    }
}

/// The centred title plate.
pub fn title_rect(win: Rect) -> Rect {
    let w = (win.w * 0.42).clamp(150.0, 210.0);
    Rect {
        x: win.x + (win.w - w) * 0.5,
        y: win.y + 17.0,
        w,
        h: 20.0,
    }
}

/// Tabs are shown five at a time, with arrows to move between groups: a phone
/// screen at 380x240 cannot afford three rows of tabs.
pub const TAB_SLOTS: usize = 5;
const TAB_ARROW: f32 = 13.0;

pub fn tab_group(page: usize) -> usize {
    page / TAB_SLOTS
}

pub fn tab_groups(n: usize) -> usize {
    n.div_ceil(TAB_SLOTS).max(1)
}

/// The tab strip: `[<] [tab] [tab] [tab] [tab] [tab] [>]`.
pub fn tab_rects(win: Rect, n: usize, page: usize) -> Vec<Rect> {
    let mut v = Vec::with_capacity(n.min(TAB_SLOTS));
    if n == 0 {
        return v;
    }
    let group = tab_group(page.min(n - 1));
    let first = group * TAB_SLOTS;
    let count = (n - first).min(TAB_SLOTS);
    let inner_x = win.x + 8.0 + TAB_ARROW + 3.0;
    let inner_w = win.w - 16.0 - 2.0 * (TAB_ARROW + 3.0);
    let w = (inner_w - TAB_GAP * (count as f32 - 1.0)) / count as f32;
    for i in 0..count {
        v.push(Rect {
            x: inner_x + i as f32 * (w + TAB_GAP),
            y: win.y + 41.0,
            w,
            h: TAB_H,
        });
    }
    v
}

/// Arrow rectangles of the tab strip: index 0 is the previous group.
pub fn tab_arrow(win: Rect, index: usize) -> Rect {
    Rect {
        x: if index == 0 {
            win.x + 8.0
        } else {
            win.x + win.w - 8.0 - TAB_ARROW
        },
        y: win.y + 41.0,
        w: TAB_ARROW,
        h: TAB_H,
    }
}

/// The body: below the tab strip, above the toast strip and the footer.
pub fn content(win: Rect, _tabs: usize) -> Rect {
    let top = win.y + 41.0 + TAB_H + 5.0;
    let bottom = win.y + win.h - 32.0;
    Rect {
        x: win.x + 8.0,
        y: top,
        w: win.w - 16.0,
        h: (bottom - top).max(40.0),
    }
}

/// Left column (list) and right column (details) of a two-pane page.
pub fn split(c: Rect) -> (Rect, Rect) {
    let left = (c.w * 0.40).clamp(112.0, 196.0);
    (
        Rect {
            x: c.x,
            y: c.y,
            w: left - 4.0,
            h: c.h,
        },
        Rect {
            x: c.x + left,
            y: c.y,
            w: c.w - left,
            h: c.h,
        },
    )
}

pub const NODE_W: f32 = 84.0;
pub const NODE_H: f32 = 24.0;
/// Horizontal and vertical distance between node origins, before zoom.
pub const COL_STEP: f32 = 100.0;
pub const ROW_STEP: f32 = 32.0;
/// Margin around the whole graph, before zoom.
pub const GRAPH_PAD: f32 = 10.0;
pub const ZOOM_MIN: f32 = 0.01;
pub const ZOOM_MAX: f32 = 1.6;
/// Below this zoom the node plates are too small for a readable name, so
/// they collapse to the state glyph and the rank (Host::text has no scale).
pub const ZOOM_COMPACT: f32 = 0.75;

pub fn clamp_zoom(z: f32) -> f32 {
    if z.is_finite() && z > 0.0 {
        z.clamp(ZOOM_MIN, ZOOM_MAX)
    } else {
        1.0
    }
}

/// Lays out a tree of any size. `parents[i]` lists the in-tree prerequisites
/// of node `i`. Every node gets a cell: column = prerequisite depth, row = its
/// lane (a chain keeps one row, siblings branch into new rows). Root groups
/// are packed into side-by-side bands so a 140-node tree reads as a wide
/// grid that pans in both axes instead of a single column 4,000 px tall.
/// Deterministic and cycle-safe. Returns (col, row) per node.
pub fn layout_tree(parents: &[Vec<usize>]) -> Vec<(usize, usize)> {
    let n = parents.len();
    if n == 0 {
        return Vec::new();
    }
    // Depth by bounded relaxation: at most n passes, so a cycle cannot hang.
    let mut depth = vec![0usize; n];
    for _ in 0..n {
        let mut changed = false;
        for i in 0..n {
            let d = parents[i]
                .iter()
                .filter(|&&j| j < n && j != i)
                .map(|&j| depth[j].saturating_add(1))
                .max()
                .unwrap_or(0)
                .min(n);
            if d != depth[i] {
                depth[i] = d;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Primary parent = the deepest prerequisite; it owns the child's lane.
    let mut children = vec![Vec::new(); n];
    let mut has_parent = vec![false; n];
    for i in 0..n {
        if let Some(&p) = parents[i]
            .iter()
            .filter(|&&j| j < n && j != i && depth[j] < depth[i])
            .max_by_key(|&&j| depth[j])
        {
            children[p].push(i);
            has_parent[i] = true;
        }
    }
    // Lanes: depth-first, first child continues the parent's row.
    let mut row = vec![usize::MAX; n];
    let mut groups: Vec<(usize, usize)> = Vec::new(); // (first row, rows)
    let mut next = 0usize;
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for root in 0..n {
        if has_parent[root] || row[root] != usize::MAX {
            continue;
        }
        let start = next;
        stack.push((root, next));
        next += 1;
        while let Some((node, r)) = stack.pop() {
            if row[node] != usize::MAX {
                continue;
            }
            row[node] = r;
            for (k, &ch) in children[node].iter().enumerate().rev() {
                let cr = if k == 0 {
                    r
                } else {
                    next += 1;
                    next - 1
                };
                stack.push((ch, cr));
            }
        }
        groups.push((start, next - start));
    }
    // Anything left (only possible with malformed data) gets its own lane.
    for i in 0..n {
        if row[i] == usize::MAX {
            row[i] = next;
            groups.push((next, 1));
            next += 1;
        }
    }
    // Pick a band count whose overall shape is close to a 16:9 viewport.
    let cols = depth.iter().max().copied().unwrap_or(0) + 1;
    let rows = next.max(1);
    let band_w = (cols as f32 + 0.5) * COL_STEP;
    let mut best = (f32::MAX, 1usize);
    for k in 1..=rows.min(12) {
        let h = rows.div_ceil(k) as f32 * ROW_STEP;
        let score = ((k as f32 * band_w) / h - 1.78).abs();
        if score < best.0 {
            best = (score, k);
        }
    }
    let target = rows.div_ceil(best.1);
    // Whole root groups go to a band; a band closes once it reaches target.
    let mut band_of_row = vec![(0usize, 0usize); rows];
    let (mut band, mut used) = (0usize, 0usize);
    for (start, len) in groups {
        if used > 0 && used + len > target {
            band += 1;
            used = 0;
        }
        for k in 0..len {
            if let Some(slot) = band_of_row.get_mut(start + k) {
                *slot = (band, used + k);
            }
        }
        used += len;
    }
    // One empty column between bands keeps the bands visually apart.
    (0..n)
        .map(|i| {
            let (b, r) = band_of_row.get(row[i]).copied().unwrap_or((0, row[i]));
            (b * (cols + 1) + depth[i], r)
        })
        .collect()
}

/// Top-left of a node in graph space (before zoom), relative to the graph.
pub fn node_base(col: usize, row: usize) -> [f32; 2] {
    [
        GRAPH_PAD + col as f32 * COL_STEP,
        GRAPH_PAD + row as f32 * ROW_STEP,
    ]
}

/// Size of the whole graph in graph space (before zoom).
pub fn graph_size(t: &TreeModel) -> [f32; 2] {
    let maxc = t.nodes.iter().map(|n| n.col).max().unwrap_or(0) as f32;
    let maxr = t.nodes.iter().map(|n| n.row).max().unwrap_or(0) as f32;
    [
        GRAPH_PAD * 2.0 + maxc * COL_STEP + NODE_W,
        GRAPH_PAD * 2.0 + maxr * ROW_STEP + NODE_H,
    ]
}

/// Clamps a pan (offset of the graph origin from the viewport's top-left).
/// An axis that fits is centred; an axis that overflows can travel exactly
/// from the first to the last node, so every node is reachable.
pub fn clamp_pan(g: Rect, t: &TreeModel, pan: [f32; 2]) -> [f32; 2] {
    let z = clamp_zoom(t.zoom);
    let size = graph_size(t);
    let axis = |p: f32, content: f32, view: f32| {
        let p = if p.is_finite() { p } else { 0.0 };
        if content <= view {
            (view - content) * 0.5
        } else {
            p.clamp(view - content, 0.0)
        }
    };
    [
        axis(pan[0], size[0] * z, g.w),
        axis(pan[1], size[1] * z, g.h),
    ]
}

/// The effective pan used by drawing and hit-testing alike.
pub fn view_pan(g: Rect, t: &TreeModel) -> [f32; 2] {
    clamp_pan(g, t, t.pan)
}

/// New pan after a zoom change, keeping the viewport centre fixed.
pub fn zoom_pan(g: Rect, t: &TreeModel, new_zoom: f32) -> [f32; 2] {
    let old = view_pan(g, t);
    let z0 = clamp_zoom(t.zoom);
    let z1 = clamp_zoom(new_zoom);
    let cx = g.w * 0.5;
    let cy = g.h * 0.5;
    let pan = [cx - (cx - old[0]) / z0 * z1, cy - (cy - old[1]) / z0 * z1];
    let mut t2 = t.clone();
    t2.zoom = z1;
    clamp_pan(g, &t2, pan)
}

/// Pan that brings node `i` fully into view (minimal movement).
pub fn reveal_pan(g: Rect, t: &TreeModel, i: usize) -> [f32; 2] {
    let pan = view_pan(g, t);
    let Some(n) = t.nodes.get(i) else {
        return pan;
    };
    let z = clamp_zoom(t.zoom);
    let b = node_base(n.col, n.row);
    let (x0, y0) = (b[0] * z + pan[0], b[1] * z + pan[1]);
    let (x1, y1) = (x0 + NODE_W * z, y0 + NODE_H * z);
    let m = 6.0;
    let mut p = pan;
    if x0 < m {
        p[0] += m - x0;
    } else if x1 > g.w - m {
        p[0] -= x1 - (g.w - m);
    }
    if y0 < m {
        p[1] += m - y0;
    } else if y1 > g.h - m {
        p[1] -= y1 - (g.h - m);
    }
    clamp_pan(g, t, p)
}

pub fn node_rect(g: Rect, n: &TreeNode, pan: [f32; 2], zoom: f32) -> Rect {
    let z = clamp_zoom(zoom);
    let b = node_base(n.col, n.row);
    Rect {
        x: g.x + pan[0] + b[0] * z,
        y: g.y + pan[1] + b[1] * z,
        w: NODE_W * z,
        h: NODE_H * z,
    }
}

/// Part of `r` inside `clip`, or None. Touch areas of nodes cut by the
/// viewport are exactly the visible part, never the hidden overflow.
pub fn intersect(r: Rect, clip: Rect) -> Option<Rect> {
    let x0 = r.x.max(clip.x);
    let y0 = r.y.max(clip.y);
    let x1 = (r.x + r.w).min(clip.x + clip.w);
    let y1 = (r.y + r.h).min(clip.y + clip.h);
    (x1 - x0 >= 1.0 && y1 - y0 >= 1.0).then_some(Rect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

/// A dedicated information pane. On a wide screen it sits at the right;
/// on the smallest phone GUI it becomes a taller panel below the graph.
pub fn tree_detail_rect(m: &MenuModel) -> Rect {
    let c=content(window(m.area),m.pages.len());
    if c.w>=500.0 && c.h>=170.0 {
        let split=(c.w*0.67).max(265.0);
        Rect {x:c.x+split+5.0,y:c.y+32.0,w:c.w-split-5.0,h:c.h-32.0}
    } else {
        let dh=(c.h*0.46).clamp(62.0,150.0);
        Rect {x:c.x,y:c.y+c.h-dh,w:c.w,h:dh}
    }
}
/// Viewport for nodes: never overlaps the information pane or buttons.
pub fn tree_graph_rect(m: &MenuModel) -> Rect {
    let c=content(window(m.area),m.pages.len());
    let d=tree_detail_rect(m);
    if d.x>c.x+8.0 {
        Rect {x:c.x,y:c.y+32.0,w:(d.x-c.x-5.0).max(80.0),h:(c.h-32.0).max(50.0)}
    } else {
        Rect {x:c.x,y:c.y+32.0,w:c.w,h:(d.y-c.y-34.0).max(24.0)}
    }
}

/// Zoom that fits the complete graph. Compact glyphs keep even huge trees
/// legible as a constellation; the index remains available for full names.
pub fn fit_zoom(g: Rect, t: &TreeModel) -> f32 {
    let size = graph_size(t);
    if size[0] <= 0.0 || size[1] <= 0.0 {
        return 1.0;
    }
    clamp_zoom((g.w / size[0]).min(g.h / size[1]) * 0.96)
}

/// Pan that centres the selected node, not merely its nearest visible edge.
pub fn center_pan(g: Rect, t: &TreeModel, i: usize) -> [f32; 2] {
    let Some(n) = t.nodes.get(i) else {
        return view_pan(g, t);
    };
    let z = clamp_zoom(t.zoom);
    let b = node_base(n.col, n.row);
    clamp_pan(
        g,
        t,
        [
            g.w * 0.5 - (b[0] + NODE_W * 0.5) * z,
            g.h * 0.5 - (b[1] + NODE_H * 0.5) * z,
        ],
    )
}

pub fn tree_index_rows(g: Rect) -> usize {
    (((g.h - 12.0) / 17.0).floor() as usize).max(1)
}

pub fn tree_index_pages(g: Rect, t: &TreeModel) -> usize {
    t.nodes.len().div_ceil(tree_index_rows(g)).max(1)
}

/// Touch pager at the bottom-right of the index (0 previous, 1 next).
pub fn tree_index_arrow(g: Rect, index: usize) -> Rect {
    Rect {
        x: g.x + g.w - 5.0 - (2 - index) as f32 * 18.0,
        y: g.y + g.h - 13.0,
        w: 16.0,
        h: 12.0,
    }
}

/// Visible rows of the index, using the same page calculation for draw/hits.
pub fn tree_index_layout(m: &MenuModel) -> Vec<(Rect, usize)> {
    let Some(t) = &m.tree else {
        return Vec::new();
    };
    let g = tree_graph_rect(m);
    let rows = tree_index_rows(g);
    let page = m.scroll.min(tree_index_pages(g, t) - 1);
    let start = page * rows;
    (start..(start + rows).min(t.nodes.len()))
        .enumerate()
        .map(|(row, i)| {
            (
                Rect {
                    x: g.x + 3.0,
                    y: g.y + 3.0 + row as f32 * 17.0,
                    w: g.w - 6.0,
                    h: 15.0,
                },
                i,
            )
        })
        .collect()
}

/// Small overview map in the lower-right corner of the graph viewport.
pub fn minimap_rect(g: Rect) -> Rect {
    Rect {
        x: g.x + g.w - 65.0,
        y: g.y + g.h - 39.0,
        w: 61.0,
        h: 35.0,
    }
}

// ------------------------------------------------------------------ drawing

pub fn draw_menu(m: &MenuModel) -> Layer {
    let mut l = Layer::new();
    let win = window(m.area);
    backdrop(&mut l, m.area, m.tick);
    plate(&mut l, win, m);
    chrome(&mut l, win, m);
    tabs(&mut l, win, m);
    let c = content(win, m.pages.len());
    match m.kind {
        "trees" => constellation_page(&mut l, c, m),
        "bestiary" => bestiary_page(&mut l, c, m),
        "status" => status_page(&mut l, c, m),
        "attributes" => attributes_page(&mut l, c, m),
        "overview" => overview_page(&mut l, c, m),
        "daily" | "quests" => quest_page(&mut l, c, m),
        "inventory" => inventory_page(&mut l, c, m),
        "equipment" | "accessories" => equipment_page(&mut l, c, m),
        _ => data_page(&mut l, c, m),
    }
    footer(&mut l, win, m);
    l
}

fn backdrop(l: &mut Layer, a: Rect, tick: u32) {
    l.rect(a, 0xF203060C);
    // A vertical glow behind the window and a slow scanning band.
    for i in 0..10 {
        let t = i as f32 / 10.0;
        let h = a.h / 10.0;
        l.rect(
            Rect {
                x: a.x,
                y: a.y + i as f32 * h,
                w: a.w,
                h,
            },
            alpha(0xFF0B2A44, (18.0 * (1.0 - t) + 3.0) as u8),
        );
    }
    let scan = a.y + (tick % 240) as f32 / 240.0 * a.h;
    l.rect(
        Rect {
            x: a.x,
            y: scan,
            w: a.w,
            h: 10.0,
        },
        alpha(CYAN, 0x0A),
    );
    l.corner_brackets(a, 16.0, 1.0, alpha(CYAN, 0x70));
}

/// The window plate itself: dark glass, thin light border, inner rule and grid.
fn plate(l: &mut Layer, win: Rect, m: &MenuModel) {
    l.glow(win, 4.0, 4.0, CYAN, 3);
    l.chamfer(win, 5.0, 0xF407111D);
    l.grid(win, 18.0, 0x044FE3FF);
    l.chamfer_outline(win, 5.0, 1.0, alpha(ICE, 0x88));
    l.chamfer_outline(
        Rect {
            x: win.x + 3.0,
            y: win.y + 3.0,
            w: win.w - 6.0,
            h: win.h - 6.0,
        },
        3.0,
        1.0,
        alpha(CYAN, 0x2A),
    );
    l.corner_brackets(win, 12.0, 1.0, alpha(CYAN, 0xC0));
    // Accent rules on the outer edges, as in the reference.
    l.rect(
        Rect {
            x: win.x + win.w * 0.5 - 30.0,
            y: win.y - 2.0,
            w: 60.0,
            h: 2.0,
        },
        alpha(CYAN, 0xC0),
    );
    let _ = m;
}

fn chrome(l: &mut Layer, win: Rect, m: &MenuModel) {
    let h = header(win);
    l.text_tracked("SISTEMA", h.x, h.y + 1.0, alpha(CYAN, 0xE0), 1.4);
    l.text("// CLAUDE RPG", h.x + 76.0, h.y + 1.0, alpha(MUTED, 0x70));
    let right = format!("NV {}  ·  RANK {}", m.level.max(1), m.rank);
    l.text(
        &right,
        h.x + h.w - text_width(&right),
        h.y + 1.0,
        alpha(GOLD, 0xC0),
    );
    let plate = title_rect(win);
    // The plate names the open page; the brand already sits on the top left, so
    // repeating "THE SYSTEM" there would waste the only strong focal point.
    let title = m
        .pages
        .get(m.page)
        .cloned()
        .unwrap_or_else(|| m.title.clone());
    l.title_plate(
        plate,
        &fit_word(&title.to_uppercase(), plate.w - 16.0),
        CYAN,
    );
}

/// Tabs must stay readable when the window is only 380 px wide: the long page
/// names are abbreviated instead of cut with an ellipsis.
pub fn short_label(page: &str) -> String {
    let up = page.to_uppercase();
    match up.as_str() {
        "VISÃO GERAL" => "VISÃO",
        "PERSONAGEM" => "HERÓI",
        "ATRIBUTOS" => "ATRIB.",
        "PROFISSÕES" => "PROFISS.",
        "EQUIPAMENTOS" => "EQUIP.",
        "ACESSÓRIOS" => "ACESS.",
        "ENCANTAMENTOS" => "ENCANT.",
        "HABILIDADES" => "HABIL.",
        "BESTIÁRIO" => "BESTIÁR.",
        "CONQUISTAS" => "CONQUIS.",
        "HISTÓRICO" => "HISTÓR.",
        "EVIL ATT." => "EVIL",
        _ => up.as_str(),
    }
    .to_string()
}

fn tabs(l: &mut Layer, win: Rect, m: &MenuModel) {
    let n = m.pages.len();
    let group = tab_group(m.page.min(n.saturating_sub(1)));
    let first = group * TAB_SLOTS;
    for (i, r) in tab_rects(win, n, m.page).into_iter().enumerate() {
        let page = first + i;
        let hover = r.contains(m.mouse);
        let label = fit(&short_label(&m.pages[page]), r.w - 8.0);
        l.tab(r, &label, page == m.page, hover, CYAN);
    }
    // Group arrows, dimmed when there is no group in that direction.
    let prev = tab_arrow(win, 0);
    let next = tab_arrow(win, 1);
    let more = tab_groups(n) > 1;
    let pc = if more && group > 0 {
        CYAN
    } else {
        alpha(MUTED, 0x40)
    };
    let nc = if more && group + 1 < tab_groups(n) {
        CYAN
    } else {
        alpha(MUTED, 0x40)
    };
    l.rect(prev, alpha(0xFF081520, 0xC0));
    l.border(prev, alpha(pc, 0x60));
    l.chevron_left(prev.x + 4.0, prev.y + 4.0, 5.0, 7.0, pc);
    l.rect(next, alpha(0xFF081520, 0xC0));
    l.border(next, alpha(nc, 0x60));
    l.chevron(next.x + 5.0, next.y + 4.0, 5.0, 7.0, nc);
}

fn footer(l: &mut Layer, win: Rect, m: &MenuModel) {
    l.text_tracked(
        if matches!(
            m.kind,
            "trees"
                | "bestiary"
                | "status"
                | "attributes"
                | "overview"
                | "daily"
                | "quests"
                | "equipment"
                | "accessories"
        ) || data_pages(content(win, m.pages.len()), &m.lines) > 1
        {
            hint_for(m.kind)
        } else {
            "SOMENTE LEITURA · DADOS DO SERVIDOR"
        },
        win.x + 8.0,
        win.y + win.h - 15.0,
        alpha(MUTED, 0x88),
        0.5,
    );
    footer_right(l, win, m);
}

/// What the player can do on this page, in one short line.
fn hint_for(kind: &str) -> &'static str {
    match kind {
        "trees" => "MAPA: ARRASTE/ROLE · ÍNDICE: ROLE PARA PAGINAR",
        "bestiary" => "TOQUE NUMA ESPÉCIE",
        "attributes" => "PONTOS VÃO PARA A ÁRVORE",
        "daily" | "quests" => "OBJETIVOS ATUALIZADOS PELO SERVIDOR",
        "overview" => "TOQUE NUMA ABA",
        _ => "ROLE OU USE AS SETAS PARA PAGINAR",
    }
}

fn footer_right(l: &mut Layer, win: Rect, m: &MenuModel) {
    let y = win.y + win.h - 15.0;
    l.hline(win.x + 8.0, y - 3.0, win.w - 16.0, alpha(CYAN, 0x30));
    if m.points > 0 {
        let tag = format!("{} PONTOS", m.points);
        let r = Rect {
            x: win.x + win.w - text_width(&tag) - 16.0,
            y: y - 2.0,
            w: text_width(&tag) + 10.0,
            h: 12.0,
        };
        l.chip(r, &tag, GOLD);
    } else {
        l.text(
            "SYSTEM",
            win.x + win.w - text_width("SYSTEM") - 10.0,
            y,
            alpha(CYAN, 0x60),
        );
    }
    if let Some(t) = &m.toast {
        let r = Rect {
            x: win.x + 8.0,
            y: win.y + win.h - 30.0,
            w: win.w - 16.0,
            h: 12.0,
        };
        l.rect(r, alpha(0xFF06202E, 0xF0));
        l.border(r, alpha(GREEN, 0x60));
        l.rect(
            Rect {
                x: r.x,
                y: r.y,
                w: 2.0,
                h: r.h,
            },
            GREEN,
        );
        l.text(
            &fit(t, r.w - 10.0),
            r.x + 6.0,
            r.y + 2.0,
            alpha(GREEN, 0xF0),
        );
    }
}

/// A section label with a rule that runs to the edge of its column.
fn section(l: &mut Layer, r: Rect, title: &str, accent: u32) -> f32 {
    l.text_tracked(&title.to_uppercase(), r.x, r.y, alpha(accent, 0xE0), 1.0);
    let w = tracked_width(&title.to_uppercase(), 1.0) + 6.0;
    l.hline(r.x, r.y + 11.0, w.min(r.w), alpha(accent, 0x90));
    l.hline(r.x, r.y + 11.0, r.w, alpha(accent, 0x22));
    r.y + 16.0
}

// ---- status ---------------------------------------------------------

fn status_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    let left_w = (c.w * 0.30).clamp(126.0, 176.0);
    let left = Rect {
        x: c.x,
        y: c.y,
        w: left_w - 4.0,
        h: c.h,
    };
    let right = Rect {
        x: c.x + left_w,
        y: c.y,
        w: c.w - left_w,
        h: c.h,
    };
    // Identity card: level as the hero number, rank in a diamond, points.
    l.card(left, CYAN);
    l.text_tracked(
        "NÍVEL",
        left.x + 10.0,
        left.y + 8.0,
        alpha(MUTED, 0xB0),
        0.8,
    );
    let level = m.level.max(1).to_string();
    l.text(&level, left.x + 10.0, left.y + 20.0, WHITE);
    l.hline(
        left.x + 10.0,
        left.y + 36.0,
        left.w - 20.0,
        alpha(CYAN, 0x40),
    );
    let dcx = left.x + 24.0;
    let dcy = left.y + 52.0;
    l.diamond_outline(dcx, dcy, 13.0, 1.5, alpha(GOLD, 0xD0), 0xF00A1A2A);
    l.diamond(dcx, dcy, 10.0, alpha(0xFF14283C, 0xE0));
    let rank = if m.rank.is_empty() { "E" } else { &m.rank };
    l.text(rank, dcx - text_width(rank) * 0.5, dcy - 4.0, GOLD);
    l.text_tracked("RANK", dcx + 20.0, dcy - 14.0, alpha(MUTED, 0xB0), 0.8);
    l.text_tracked("PONTOS", dcx + 20.0, dcy + 0.0, alpha(MUTED, 0xB0), 0.8);
    let pv = m.points.to_string();
    l.text(
        &pv,
        left.x + left.w - 10.0 - text_width(&pv),
        dcy + 0.0,
        GOLD,
    );
    // The right column already lists every attribute, so this card shows what
    // it cannot: the live bars of the character.
    l.text_tracked(
        "VITAIS",
        left.x + 10.0,
        left.y + 70.0,
        alpha(MUTED, 0xA0),
        0.8,
    );
    l.hline(
        left.x + 10.0,
        left.y + 82.0,
        left.w - 20.0,
        alpha(CYAN, 0x28),
    );
    let mut y = left.y + 86.0;
    for v in m.vitals.iter().take(3) {
        if y + 17.0 > left.y + left.h - 2.0 {
            break;
        }
        vital_row(
            l,
            Rect {
                x: left.x + 2.0,
                y,
                w: left.w - 4.0,
                h: 20.0,
            },
            y,
            v,
        );
        y += 18.0;
    }

    // Right: every attribute in two columns, then the engine log.
    l.card(right, CYAN);
    let mut y = section(
        l,
        Rect {
            x: right.x + 10.0,
            y: right.y + 8.0,
            w: right.w - 20.0,
            h: 12.0,
        },
        "Atributos",
        CYAN,
    );
    let col_w = (right.w - 30.0) * 0.5;
    let rows = m.attrs.len().div_ceil(2).min(6);
    for i in 0..rows {
        for col in 0..2 {
            let Some(a) = m.attrs.get(i + col * rows) else {
                continue;
            };
            let x = right.x + 10.0 + col as f32 * (col_w + 10.0);
            l.text(
                &fit_word(&a.name.to_uppercase(), col_w - 34.0),
                x,
                y,
                alpha(ICE, 0xD0),
            );
            let v = a.value.clone();
            l.text(&v, x + col_w - 4.0 - text_width(&v), y, CYAN);
            l.bar(
                Rect {
                    x,
                    y: y + 11.0,
                    w: col_w - 4.0,
                    h: 5.0,
                },
                a.ratio,
                1.0,
                alpha(CYAN, 0xD0),
            );
        }
        y += 24.0;
    }
    // Log block at the bottom of the right column.
    let log = Rect {
        x: right.x + 8.0,
        y: (right.y + right.h - 74.0).max(y + 4.0),
        w: right.w - 16.0,
        h: 66.0,
    };
    if log.y + log.h <= right.y + right.h {
        l.rect(log, alpha(0xFF07141F, 0xD0));
        l.border(log, alpha(CYAN, 0x28));
        l.text_tracked("REGISTRO", log.x + 6.0, log.y + 4.0, alpha(CYAN, 0x90), 0.7);
        let mut ly = log.y + 16.0;
        for line in m.lines.iter().take(4) {
            if ly > log.y + log.h - 10.0 {
                break;
            }
            l.diamond(log.x + 8.0, ly + 3.5, 1.8, alpha(CYAN, 0x80));
            l.text(&fit(line, log.w - 22.0), log.x + 14.0, ly, alpha(ICE, 0xC0));
            ly += 11.0;
        }
    }
}

/// The landing page: the identity plate and the summary lines as data cards.
fn overview_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    let head = Rect {
        x: c.x,
        y: c.y,
        w: c.w,
        h: 52.0,
    };
    l.card(head, CYAN);
    let dcx = head.x + 30.0;
    let dcy = head.y + 26.0;
    l.diamond_outline(dcx, dcy, 18.0, 1.5, alpha(GOLD, 0xD0), 0xF00A1A2A);
    l.diamond(dcx, dcy, 14.0, alpha(0xFF14283C, 0xE0));
    let rank = if m.rank.is_empty() { "E" } else { &m.rank };
    l.text(rank, dcx - text_width(rank) * 0.5, dcy - 4.0, GOLD);
    l.text_tracked(
        "CAÇADOR",
        head.x + 58.0,
        head.y + 8.0,
        alpha(CYAN, 0xC0),
        1.4,
    );
    l.text(
        &format!("NÍVEL {}", m.level.max(1)),
        head.x + 58.0,
        head.y + 20.0,
        WHITE,
    );
    let pts = format!("{} PONTOS LIVRES", m.points);
    l.text(
        &pts,
        head.x + head.w - text_width(&pts) - 10.0,
        head.y + 20.0,
        alpha(GOLD, 0xD0),
    );
    // Experience towards the next level: a real, always-moving thread.
    let xp = Rect {
        x: head.x + 58.0,
        y: head.y + 34.0,
        w: head.w - 68.0,
        h: 6.0,
    };
    l.bar(xp, unit(m.xp), unit(m.xp), GOLD);
    let xp_txt = format!("{:.0}%", unit(m.xp) * 100.0);
    l.text(
        &xp_txt,
        head.x + head.w - text_width(&xp_txt) - 10.0,
        head.y + 42.0,
        alpha(GOLD, 0xC0),
    );
    l.text_tracked(
        "EXPERIÊNCIA",
        head.x + 58.0,
        head.y + 42.0,
        alpha(MUTED, 0x90),
        0.6,
    );

    let ladder_y = head.y + head.h + 5.0;
    let has_ladder = ladder_y + 30.0 <= c.y + c.h;
    if has_ladder {
        let box_r = Rect {
            x: c.x,
            y: ladder_y,
            w: c.w,
            h: 30.0,
        };
        l.rect(box_r, alpha(0xFF0A1725, 0xE0));
        l.border(box_r, alpha(CYAN, 0x30));
        l.text_tracked(
            "PATENTE",
            box_r.x + 8.0,
            box_r.y + 3.0,
            alpha(MUTED, 0x90),
            0.7,
        );
        if !m.next_rank.is_empty() {
            let tag = format!("{:.0}% → {}", unit(m.rank_progress) * 100.0, m.next_rank);
            l.text(
                &tag,
                box_r.x + box_r.w - text_width(&tag) - 8.0,
                box_r.y + 3.0,
                alpha(GOLD, 0xC0),
            );
        }
        let ranks = ["E", "D", "C", "B", "A", "S"];
        // The ladder runs between the label and the progress tag, never
        // under either of them.
        let tag_w = if m.next_rank.is_empty() {
            0.0
        } else {
            text_width(&format!("100% → {}", m.next_rank)) + 12.0
        };
        let step = ((box_r.w - 54.0 - tag_w - 8.0) / ranks.len() as f32).max(12.0);
        for (i, r) in ranks.iter().enumerate() {
            let cx = box_r.x + 54.0 + step * (i as f32 + 0.5);
            let on = *r == m.rank.as_str();
            l.diamond_outline(
                cx,
                box_r.y + 14.0,
                6.5,
                if on { 1.5 } else { 1.0 },
                if on { GOLD } else { alpha(MUTED, 0x60) },
                0x00000000,
            );
            if on {
                l.diamond(cx, box_r.y + 14.0, 4.5, alpha(GOLD, 0xC0));
            }
            l.text(
                *r,
                cx - text_width(r) * 0.5,
                box_r.y + 10.0,
                if on { 0xFF06121E } else { alpha(MUTED, 0xB0) },
            );
            if i + 1 < ranks.len() {
                l.hline(
                    cx + 8.0,
                    box_r.y + 14.0,
                    (step - 16.0).max(2.0),
                    alpha(CYAN, 0x30),
                );
            }
        }
        // How far the hunter already walked inside the current band.
        let track = Rect {
            x: box_r.x + 8.0,
            y: box_r.y + box_r.h - 5.0,
            w: box_r.w - 16.0,
            h: 2.0,
        };
        l.rect(track, alpha(0xFF061420, 0xF0));
        l.rect(
            Rect {
                w: track.w * unit(m.rank_progress),
                ..track
            },
            alpha(GOLD, 0xC0),
        );
    }

    let top = head.y + head.h + if has_ladder { 41.0 } else { 6.0 };
    let bottom = c.y + c.h;
    let space = bottom - top;
    if space < 18.0 {
        return;
    }
    // Live vitals: the same three bars the HUD carries, with real numbers.
    let rows = m.vitals.len().min(3);
    let row_h = 14.0;
    let need = rows as f32 * row_h + 6.0;
    let vit = Rect {
        x: c.x,
        y: top,
        w: c.w,
        h: need.min(space),
    };
    l.rect(vit, alpha(0xFF0A1725, 0xE0));
    l.border(vit, alpha(CYAN, 0x30));
    let mut y = vit.y + 3.0;
    for v in m.vitals.iter().take(3) {
        if y + 12.0 > vit.y + vit.h {
            break;
        }
        vital_row(l, vit, y, v);
        y += row_h;
    }
    // Rank ladder progress, drawn only when the card still has room.
    let rest = bottom - (vit.y + vit.h) - 5.0;
    if rest >= 16.0 && !m.next_rank.is_empty() {
        let nr = Rect {
            x: c.x,
            y: vit.y + vit.h + 5.0,
            w: c.w,
            h: rest.min(24.0),
        };
        l.rect(nr, alpha(0xFF0A1725, 0xE0));
        l.border(nr, alpha(GOLD, 0x30));
        l.text_tracked(
            "PRÓXIMO RANK",
            nr.x + 8.0,
            nr.y + 3.0,
            alpha(MUTED, 0xA0),
            0.6,
        );
        let tag = m.next_rank.clone();
        l.text(&tag, nr.x + nr.w - text_width(&tag) - 8.0, nr.y + 3.0, GOLD);
        l.bar(
            Rect {
                x: nr.x + 8.0,
                y: nr.y + 15.0,
                w: nr.w - 16.0,
                h: 6.0,
            },
            unit(m.rank_progress),
            unit(m.rank_progress),
            GOLD,
        );
    }
}

/// One line of the vitals card: tracked label, thin bar, right-aligned value.
fn vital_row(l: &mut Layer, card: Rect, y: f32, v: &Vital) {
    let color = vital_color(&v.label);
    let label = v.label.to_uppercase();
    l.text_tracked(&label, card.x + 8.0, y + 2.0, alpha(ICE, 0xD0), 0.8);
    let val_w = text_width(&v.value);
    let bar_x = card.x + 8.0 + tracked_width(&label, 0.8) + 8.0;
    let bar_w = (card.x + card.w - 8.0 - val_w - 8.0 - bar_x).max(20.0);
    l.bar(
        Rect {
            x: bar_x,
            y: y + 2.0,
            w: bar_w,
            h: 7.0,
        },
        unit(v.ratio),
        unit(v.ratio),
        color,
    );
    l.text(
        &v.value,
        card.x + card.w - 8.0 - val_w,
        y + 1.0,
        alpha(color, 0xF0),
    );
}

fn vital_color(label: &str) -> u32 {
    match label.to_uppercase().as_str() {
        "HP" | "VIDA" => CYAN,
        "SP" | "VIGOR" | "STA" => GREEN,
        "ABS" | "ESCUDO" => AMBER,
        "MP" | "MANA" => VIOLET,
        _ => ICE,
    }
}

fn attributes_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    l.card(c, CYAN);
    let y = section(
        l,
        Rect {
            x: c.x + 10.0,
            y: c.y + 8.0,
            w: c.w - 20.0,
            h: 12.0,
        },
        "Atributos",
        CYAN,
    );
    // Ten attributes have to fit one phone screen, so the list becomes a
    // two-column grid of compact cells instead of tall text rows.
    let banner_y = c.y + c.h - 18.0;
    let cols = if c.w >= 300.0 { 2 } else { 1 };
    let gap = 6.0;
    let cw = (c.w - 16.0 - gap * (cols as f32 - 1.0)) / cols as f32;
    let rows = m.attrs.len().div_ceil(cols).max(1);
    let avail = (banner_y - 4.0 - y).max(12.0);
    let rh = (avail / rows as f32).clamp(13.0, 26.0);
    for (i, a) in m.attrs.iter().enumerate() {
        let (col, row) = (i / rows, i % rows);
        let cell = Rect {
            x: c.x + 8.0 + col as f32 * (cw + gap),
            y: y + row as f32 * rh,
            w: cw,
            h: rh - 2.0,
        };
        if cell.y + cell.h > banner_y - 2.0 {
            break;
        }
        l.rect(cell, alpha(0xFF0C1C2C, 0xB0));
        l.border(cell, alpha(CYAN, 0x28));
        l.rect(
            Rect {
                x: cell.x,
                y: cell.y,
                w: 2.0,
                h: cell.h,
            },
            alpha(CYAN, 0x60),
        );
        let v = a.value.clone();
        let vw = text_width(&v);
        let name = fit_word(&a.name.to_uppercase(), cell.w - vw - 22.0);
        l.text(&name, cell.x + 7.0, cell.y + 3.0, alpha(ICE, 0xE0));
        l.text(&v, cell.x + cell.w - vw - 6.0, cell.y + 3.0, CYAN);
        l.bar(
            Rect {
                x: cell.x + 6.0,
                y: cell.y + cell.h - 6.0,
                w: cell.w - 12.0,
                h: 4.0,
            },
            a.ratio,
            1.0,
            alpha(CYAN, 0xC0),
        );
    }
    // Points banner: what is left to spend, and the respec control.
    let tag = format!("{} PTS", m.points);
    let r = Rect {
        x: c.x + 8.0,
        y: c.y + c.h - 18.0,
        w: c.w - 16.0,
        h: 14.0,
    };
    l.rect(r, alpha(0xFF0A2030, 0xE0));
    l.border(r, alpha(GOLD, 0x50));
    l.text_tracked(&tag, r.x + 8.0, r.y + 3.0, alpha(GOLD, 0xE0), 0.6);
    let hint = "Gaste na Árvore";
    let hint_x = r.x + 8.0 + tracked_width(&tag, 0.6) + 10.0;
    let rr = respec_rect(c);
    if hint_x + text_width(hint) < rr.x - 6.0 {
        l.text(hint, hint_x, r.y + 3.0, alpha(MUTED, 0x90));
    }
    l.rect(rr, alpha(0xFF2A1220, 0xF0));
    l.border(rr, alpha(RED, 0x70));
    let label = "REDISTRIBUIR";
    l.text_tracked(
        label,
        rr.x + (rr.w - tracked_width(label, 0.5)) * 0.5,
        rr.y + (rr.h - 9.0) * 0.5,
        alpha(RED, 0xE0),
        0.5,
    );
}

/// Greedy word wrap in GUI pixels, so a long note is read instead of cut.
fn wrap(text: &str, max_w: f32) -> Vec<String> {
    let mut out:Vec<String>=Vec::new();
    let mut line=String::new();
    let width=max_w.max(20.0);
    for word in text.split_whitespace(){
        let candidate=if line.is_empty() {word.to_owned()} else {format!("{line} {word}")};
        if text_width(&candidate)<=width {line=candidate;continue;}
        if !line.is_empty(){out.push(std::mem::take(&mut line));}
        // Se uma palavra não couber, dividir por caracteres (sem reticências).
        for ch in word.chars(){
            let mut next=line.clone();next.push(ch);
            if !line.is_empty() && text_width(&next)>width {
                out.push(std::mem::take(&mut line));
            }
            line.push(ch);
        }
    }
    if !line.is_empty(){out.push(line);}
    out
}

// ---- data pages ------------------------------------------------------
//
// Pages whose backend sends text lines (character, professions, enchanting,
// skills, evil, achievements, history) are composed as data plates: each
// line is parsed into a label and its value, numbers like `a/b` become a
// gauge, and the plates are paginated with one shared layout for drawing and
// touch. Nothing is invented: a plate shows exactly the line it came from.

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: String,
    /// Ratio parsed from the first `a/b` in the value, if any.
    pub ratio: Option<f32>,
}

pub fn parse_entry(line: &str) -> Entry {
    let line = line.trim();
    let (key, value) = match line.split_once(": ") {
        Some((k, v)) if !k.is_empty() && text_width(k) <= 150.0 => (k.trim(), v.trim()),
        _ => match line.split_once(" · ") {
            Some((k, v)) if text_width(k) <= 150.0 => (k.trim(), v.trim()),
            _ => ("", line),
        },
    };
    Entry {
        key: key.to_string(),
        value: value.to_string(),
        ratio: first_ratio(value),
    }
}

/// First `a/b` with finite numbers and b > 0, as a 0..=1 ratio.
pub fn first_ratio(s: &str) -> Option<f32> {
    for tok in s.split(|c: char| c.is_whitespace() || c == '(' || c == ')') {
        if let Some((a, b)) = tok.split_once('/') {
            let a: f64 = a
                .trim_start_matches(|c: char| !c.is_ascii_digit())
                .parse()
                .ok()?;
            let b: f64 = b
                .trim_end_matches(|c: char| !c.is_ascii_digit())
                .parse()
                .ok()?;
            if a.is_finite() && b.is_finite() && b > 0.0 {
                return Some((a / b).clamp(0.0, 1.0) as f32);
            }
        }
    }
    None
}

fn page_accent(kind: &str) -> u32 {
    match kind {
        "profs" => GREEN,
        "enchant" => VIOLET,
        "skills" => AMBER,
        "evil" => VIOLET,
        "achievements" => GOLD,
        _ => CYAN,
    }
}

/// Empty-state copy per page: honest about where the data comes from.
fn empty_copy(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "history" => ("SEM REGISTROS", "Os eventos de progressão aparecem aqui."),
        "achievements" => ("SEM CONQUISTAS", "Derrote criaturas para registrar marcos."),
        "skills" => ("SEM BÔNUS DE ARMA", "Compre nós de arma na aba Árvores."),
        "profs" => ("SEM PROFISSÕES", "Aguardando dados de proficiência."),
        _ => ("SEM DADOS", "Aguardando dados do servidor."),
    }
}

pub const PLATE_GAP: f32 = 4.0;

fn plate_height(e: &Entry, w: f32) -> f32 {
    let lines = wrap(&e.value, w - 14.0).len().clamp(1, 3) as f32;
    let key = if e.key.is_empty() { 0.0 } else { 10.0 };
    let bar = if e.ratio.is_some() { 5.0 } else { 0.0 };
    6.0 + key + lines * 10.0 + bar
}

/// Body area of a data page (below the section title, above the pager).
pub fn data_body(c: Rect) -> Rect {
    Rect {
        x: c.x + 8.0,
        y: c.y + 24.0,
        w: c.w - 16.0,
        h: c.h - 28.0,
    }
}

/// Plates of every page: (page, rect, line index). Two columns when the
/// body is wide enough and the entries are label/value pairs.
pub fn data_layout(c: Rect, lines: &[String]) -> Vec<(usize, Rect, usize)> {
    let body = data_body(c);
    let entries: Vec<Entry> = lines.iter().map(|l| parse_entry(l)).collect();
    let paired = entries.iter().filter(|e| !e.key.is_empty()).count() * 2 > entries.len();
    let cols = if body.w >= 300.0 && paired { 2 } else { 1 };
    let col_w = (body.w - PLATE_GAP * (cols as f32 - 1.0)) / cols as f32;
    let mut out = Vec::new();
    let (mut page, mut y, mut i) = (0usize, body.y, 0usize);
    while i < entries.len() {
        // A row holds `cols` plates of the same height (the tallest one).
        let take = cols.min(entries.len() - i);
        let h = (i..i + take)
            .map(|k| plate_height(&entries[k], col_w))
            .fold(0.0, f32::max);
        if y + h > body.y + body.h && y > body.y {
            page += 1;
            y = body.y;
            continue;
        }
        for k in 0..take {
            let r = Rect {
                x: body.x + k as f32 * (col_w + PLATE_GAP),
                y,
                w: col_w,
                h: h.min(body.h),
            };
            out.push((page, r, i + k));
        }
        y += h + PLATE_GAP;
        i += take;
    }
    out
}

pub fn data_pages(c: Rect, lines: &[String]) -> usize {
    data_layout(c, lines).last().map_or(1, |p| p.0 + 1)
}

/// Pager arrows of a data page: index 0 previous, 1 next.
pub fn data_arrow(c: Rect, index: usize) -> Rect {
    let w = 16.0;
    Rect {
        x: c.x + c.w - 8.0 - (2 - index) as f32 * (w + 2.0),
        y: c.y + 5.0,
        w,
        h: 14.0,
    }
}

fn data_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    let accent = page_accent(m.kind);
    l.card(c, accent);
    section(
        l,
        Rect {
            x: c.x + 10.0,
            y: c.y + 8.0,
            w: c.w - 70.0,
            h: 12.0,
        },
        &m.title,
        accent,
    );
    if m.lines.is_empty() {
        let (title, body) = empty_copy(m.kind);
        let cx = c.x + c.w * 0.5;
        let cy = c.y + c.h * 0.5;
        l.diamond_outline(cx, cy - 12.0, 9.0, 1.0, alpha(accent, 0x90), 0xF0081420);
        l.diamond(cx, cy - 12.0, 3.0, alpha(accent, 0x70));
        let t = fit_tracked(title, c.w - 20.0, 0.8);
        l.text_tracked(&t, cx - tracked_width(&t, 0.8) * 0.5, cy + 2.0, WHITE, 0.8);
        let b = fit(body, c.w - 20.0);
        l.text(&b, cx - text_width(&b) * 0.5, cy + 14.0, alpha(MUTED, 0xD0));
        return;
    }
    let layout = data_layout(c, &m.lines);
    let pages = layout.last().map_or(1, |p| p.0 + 1);
    let page = m.scroll.min(pages - 1);
    if pages > 1 {
        let label = format!("{}/{}", page + 1, pages);
        let prev = data_arrow(c, 0);
        let next = data_arrow(c, 1);
        l.text(
            &label,
            prev.x - text_width(&label) - 4.0,
            prev.y + 3.0,
            alpha(MUTED, 0xD0),
        );
        for (r, on) in [(prev, page > 0), (next, page + 1 < pages)] {
            l.rect(r, alpha(0xFF0A1826, 0xE0));
            l.border(r, alpha(accent, if on { 0x90 } else { 0x30 }));
        }
        let col = |on: bool| alpha(accent, if on { 0xE0 } else { 0x50 });
        l.chevron_left(prev.x + 6.0, prev.y + 3.5, 4.0, 7.0, col(page > 0));
        l.chevron(next.x + 7.0, next.y + 3.5, 4.0, 7.0, col(page + 1 < pages));
    }
    let timeline = m.kind == "history";
    for (_, r, i) in layout.iter().filter(|p| p.0 == page) {
        let Some(line) = m.lines.get(*i) else {
            continue;
        };
        let e = parse_entry(line);
        l.rect(*r, alpha(0xFF0A1826, 0xC0));
        l.border(*r, alpha(accent, 0x30));
        l.rect(
            Rect {
                x: r.x,
                y: r.y,
                w: 2.0,
                h: r.h,
            },
            alpha(accent, 0x90),
        );
        // Technical corner tick on the top-right of every plate.
        l.rect(
            Rect {
                x: r.x + r.w - 7.0,
                y: r.y,
                w: 7.0,
                h: 1.0,
            },
            alpha(accent, 0x90),
        );
        let mut y = r.y + 3.0;
        let tx = r.x + 12.0;
        if timeline {
            l.text(
                &format!("#{}", i + 1),
                r.x + r.w - 26.0,
                y,
                alpha(MUTED, 0xA0),
            );
        }
        l.diamond(r.x + 7.0, r.y + 7.0, 2.0, alpha(accent, 0xC0));
        if !e.key.is_empty() {
            let k = fit_tracked(&e.key.to_uppercase(), r.w - 18.0, 0.5);
            l.text_tracked(&k, tx, y, alpha(accent, 0xD0), 0.5);
            y += 10.0;
        }
        let parts = wrap(&e.value, r.w - 14.0);
        let n = parts.len().clamp(1, 3);
        for (k, part) in parts.iter().take(n).enumerate() {
            // The last line is cut only when the value needs a fourth one.
            let part = if k == 2 && parts.len() > 3 {
                fit(&parts[2..].join(" "), r.w - 14.0)
            } else {
                part.clone()
            };
            l.text(&part, tx, y, alpha(WHITE, if k == 0 { 0xF0 } else { 0xC0 }));
            y += 10.0;
        }
        if let Some(ratio) = e.ratio {
            let track = Rect {
                x: tx,
                y: y + 1.0,
                w: r.w - 18.0,
                h: 2.0,
            };
            l.rect(track, alpha(MUTED, 0x40));
            l.rect(
                Rect {
                    w: track.w * ratio,
                    ..track
                },
                alpha(accent, 0xE0),
            );
        }
    }
}

/// A grid of real equipment slots. The engine does not expose inventory slots
// Inventário de relíquias com a geometria de hit-test compartilhada.
// Os itens são virtuais, mas os cliques enviam ações reais ao servidor.
fn rarity_color(r: u8) -> u32 {
    match r { 0 => MUTED, 1 => GREEN, 2 => CYAN, 3 => VIOLET, 4 => GOLD, _ => RED }
}
fn rarity_frame(r:u8)->&'static str {
    match r {
        0=>"claude_rpg:gui/icons/rarities/rarity_common_frame",
        1=>"claude_rpg:gui/icons/rarities/rarity_uncommon_frame",
        2=>"claude_rpg:gui/icons/rarities/rarity_rare_frame",
        3=>"claude_rpg:gui/icons/rarities/rarity_epic_frame",
        4=>"claude_rpg:gui/icons/rarities/rarity_legendary_frame",
        5=>"claude_rpg:gui/icons/rarities/rarity_mythic_frame",
        _=>"claude_rpg:gui/icons/rarities/rarity_divine_frame",
    }
}
fn slot_frame(slot:&str,occupied:bool)->&'static str {
    match (slot,occupied) {
        ("weapon",false)=>"claude_rpg:gui/ui/slots/ui_slot_weapon_empty",
        ("weapon",true)=>"claude_rpg:gui/ui/slots/ui_slot_weapon_occupied",
        ("ring",false)=>"claude_rpg:gui/ui/slots/ui_slot_ring_empty",
        ("ring",true)=>"claude_rpg:gui/ui/slots/ui_slot_ring_occupied",
        ("neck",false)=>"claude_rpg:gui/ui/slots/ui_slot_amulet_empty",
        ("neck",true)=>"claude_rpg:gui/ui/slots/ui_slot_amulet_occupied",
        ("charm",false)=>"claude_rpg:gui/ui/slots/ui_slot_relic_empty",
        ("charm",true)=>"claude_rpg:gui/ui/slots/ui_slot_relic_occupied",
        (_,false)=>"claude_rpg:gui/ui/slots/ui_slot_armor_empty",
        (_,true)=>"claude_rpg:gui/ui/slots/ui_slot_armor_occupied",
    }
}

pub fn gear_rows_per_page(c: Rect) -> usize {
    ((c.h - 34.0).max(24.0) / 27.0).floor().max(1.0) as usize
}
pub fn inventory_row_rect(c: Rect, i: usize) -> Rect {
    Rect {x:c.x+8.0, y:c.y+25.0+i as f32*27.0, w:c.w-16.0,h:24.0}
}
/// Hit regions are the same rectangles used to paint action buttons.
pub fn inventory_equip_rect(row: Rect) -> Rect {
    Rect {x:row.x+row.w-93.0,y:row.y+4.0,w:48.0,h:16.0}
}
pub fn inventory_salvage_rect(row: Rect) -> Rect {
    Rect {x:row.x+row.w-41.0,y:row.y+4.0,w:34.0,h:16.0}
}

pub fn inventory_next_rect(c: Rect, prev: bool) -> Rect {
    Rect {x: if prev {c.x+c.w-58.0} else {c.x+c.w-28.0}, y:c.y+5.0, w:22.0,h:16.0}
}
/// Botões pequenos com acções distintas: nunca equipar/desequipar ao tentar ativar.
pub fn equipped_activate_rect(row:Rect)->Rect {
    Rect{x:row.x+row.w-72.0,y:row.y+5.0,w:65.0,h:13.0}
}
pub fn equipped_remove_rect(row:Rect)->Rect {
    Rect{x:row.x+row.w-72.0,y:row.y+21.0,w:65.0,h:12.0}
}
pub fn equipped_rects(c: Rect, count: usize) -> Vec<Rect> {
    // Historical geometry retained for compatibility tests and callers.
    let cols = if c.w >= 270.0 { 2 } else { 1 };
    let gap = 5.0;
    let cw=(c.w-16.0-gap*(cols as f32-1.0))/cols as f32;
    (0..count).map(|i|Rect {x:c.x+8.0+(i%cols) as f32*(cw+gap),
        y:c.y+8.0+(i/cols) as f32*42.0,w:cw,h:37.0}).collect()
}
/// Paginated equipment cards. Short Android screens never make later slots
/// impossible to reach. Both rendering and hit-testing call this function.
pub fn equipped_page_size(c:Rect)->usize {
    let cols=if c.w>=270.0 {2}else{1};
    let rows=((c.h-28.0)/42.0).floor().max(1.0) as usize;
    cols*rows
}
pub fn equipped_page_count(c:Rect,count:usize)->usize {
    count.div_ceil(equipped_page_size(c)).max(1)
}
pub fn equipped_visible_rects(c:Rect,count:usize,scroll:usize)->Vec<(usize,Rect)> {
    let cols=if c.w>=270.0 {2}else{1};
    let cw=(c.w-16.0-5.0*(cols as f32-1.0))/cols as f32;
    let cap=equipped_page_size(c);
    let start=scroll.min(equipped_page_count(c,count)-1)*cap;
    (start..(start+cap).min(count)).enumerate().map(|(i,idx)| {
        (idx,Rect{x:c.x+8.0+(i%cols) as f32*(cw+5.0),
            y:c.y+25.0+(i/cols) as f32*42.0,w:cw,h:37.0})
    }).filter(|(_,r)|r.y+r.h<=c.y+c.h+1.0).collect()
}
pub fn equipped_page_arrow(c:Rect,next:bool)->Rect {
    Rect{x:if next {c.x+c.w-25.0}else{c.x+c.w-53.0},
        y:c.y+3.0,w:22.0,h:17.0}
}

/// Mouse/focus details use original untruncated text. The panel is drawn LAST
/// so it is never hidden behind list rows; no fixed number of characters.
fn relic_wrap(text:&str,max_width:f32)->Vec<String> {
    let mut lines=Vec::new();let mut line=String::new();
    for word in text.split_whitespace() {
        let candidate=if line.is_empty(){word.to_string()}else{format!("{line} {word}")};
        if !line.is_empty() && text_width(&candidate)>max_width {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }else{line=candidate;}
    }
    if !line.is_empty(){lines.push(line);}
    lines
}
fn relic_detail(l:&mut Layer,c:Rect,g:&GearRow) {
    let max_width=(c.w-32.0).max(90.0);
    let mut lines=relic_wrap(&g.title,max_width);
    lines.extend(relic_wrap(&g.ability,max_width));
    lines.push(format!("PODER: {}",g.power_status));
    lines.extend(relic_wrap(&format!("{} +{:.2}",g.stat,g.bonus),max_width));
    let max_lines=((c.h-14.0)/10.0).max(1.0) as usize;
    let shown=lines.len().min(max_lines);
    let h=15.0+10.0*shown as f32;
    let panel=Rect{x:c.x+6.0,y:(c.y+c.h-h-5.0).max(c.y+4.0),
        w:c.w-12.0,h};
    l.rect(panel,alpha(0xFF071320,0xFA));
    l.border(panel,rarity_color(g.rarity));
    for (i,txt) in lines.into_iter().take(shown).enumerate() {
        l.text(txt,panel.x+6.0,panel.y+6.0+i as f32*10.0,
            if i==0 {rarity_color(g.rarity)}else{ICE});
    }
}

fn inventory_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    l.card(c,CYAN);
    let rows=gear_rows_per_page(c);
    let pages=m.bag.len().div_ceil(rows).max(1);
    let page=m.scroll.min(pages-1);
    let head=format!("BOLSA {}/64   ·   {}/{}",m.bag.len(),page+1,pages);
    l.text_tracked(&head,c.x+10.0,c.y+9.0,ICE,0.55);
    for (r,txt) in [(inventory_next_rect(c,true),"‹"),(inventory_next_rect(c,false),"›")] {
        l.rect(r,alpha(0xFF0C2235,0xE0));l.border(r,alpha(CYAN,0x88));
        l.text_middle(txt,r,CYAN);
    }
    if m.bag.is_empty() { l.text_middle("SEM RELÍQUIAS · DERROTE MONSTROS",c,MUTED);return; }
    for (i,g) in m.bag.iter().skip(page*rows).take(rows).enumerate() {
        let r=inventory_row_rect(c,i);
        if r.y+r.h>c.y+c.h {break;}
        let ac=rarity_color(g.rarity);
        l.rect(r,alpha(0xFF091A29,0xEC));l.border(r,alpha(ac,0x70));
        l.rect(Rect{x:r.x,y:r.y,w:2.0,h:r.h},ac);
        l.sprite(rarity_frame(g.rarity),[0.0,0.0,256.0,256.0],Rect{
            x:r.x+3.0,y:r.y+2.0,w:20.0,h:20.0,
        });
        let name=fit_word(&format!("#{} {}",g.id,g.title),r.w-124.0);
        l.text(&name,r.x+27.0,r.y+4.0,ac);
        let bonus=fit(&format!("{} +{:.1}",g.stat,g.bonus),r.w-126.0);
        l.text(&bonus,r.x+27.0,r.y+14.0,alpha(MUTED,0xCC));
        let equip=inventory_equip_rect(r);
        l.rect(equip,alpha(0xFF103B4D,0xD8));l.border(equip,alpha(CYAN,0xA0));
        l.text_middle("USAR",equip,WHITE);
        let recycle=inventory_salvage_rect(r);
        l.rect(recycle,alpha(0xFF4A2418,0xD8));l.border(recycle,alpha(GOLD,0xA0));
        l.text_middle("REC",recycle,GOLD);
    }
    if let Some(g)=m.bag.iter().skip(page*rows).take(rows).enumerate()
        .find(|(i,_)|inventory_row_rect(c,*i).contains(m.mouse))
        .map(|(_,g)|g) {
        relic_detail(l,c,g);
    }
}

fn equipment_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    l.card(c,CYAN);
    let acc=m.kind=="accessories";
    let slots: &[(&str,&str)] = if acc {
        &[("ring","ANEL"),("neck","COLAR"),("charm","TALISMÃ")]
    } else { &[("head","ELMO"),("chest","PEITORAL"),("legs","CALÇAS"),
        ("feet","BOTAS"),("weapon","ARMA"),("ring","ANEL"),("neck","COLAR"),("charm","TALISMÃ")] };
    let pages=equipped_page_count(c,slots.len());
    let page=m.scroll.min(pages-1);
    l.text(&format!("EQUIPAMENTO · {}/{}",page+1,pages),c.x+9.0,c.y+8.0,MUTED);
    if pages>1 {
        for (next,title) in [(false,"‹"),(true,"›")] {
            let ar=equipped_page_arrow(c,next);
            l.rect(ar,0xFF122B3D);l.border(ar,alpha(CYAN,0xB0));
            l.text_middle(title,ar,CYAN);
        }
    }
    let rects=equipped_visible_rects(c,slots.len(),page);
    for (i,r) in &rects {
        let (slot,label)=slots[*i];
        let r=*r;
        let gear=m.worn.iter().find(|g| g.slot==slot);
        let ac=gear.map_or(MUTED, |g|rarity_color(g.rarity));
        l.rect(r,alpha(0xFF0C1C2C,0xED));l.border(r,alpha(ac,0x90));
        l.sprite(slot_frame(slot,gear.is_some()),[0.0,0.0,256.0,256.0],Rect{
            x:r.x+4.0,y:r.y+6.0,w:24.0,h:24.0,
        });
        l.text_tracked(label,r.x+34.0,r.y+4.0,ac,0.4);
        if let Some(g)=gear {
            l.text(&fit_word(&g.title,r.w-112.0),r.x+34.0,r.y+15.0,ICE);
            l.text(&fit(&format!("{} +{:.2}",g.stat,g.bonus),r.w-112.0),
                r.x+34.0,r.y+26.0,alpha(MUTED,0xBC));
            let power=equipped_activate_rect(r);
            l.rect(power,if g.power_ready {0xFF126174}else{0xFF24333B});
            l.border(power,if g.power_ready {CYAN}else{MUTED});
            l.text_middle(&fit_word(&g.power_status,power.w-3.0),power,
                if g.power_ready {ICE}else{MUTED});
            let remove=equipped_remove_rect(r);
            l.rect(remove,0xFF422B28);l.border(remove,alpha(AMBER,0xAA));
            l.text_middle("REMOVER",remove,AMBER);
        } else {
            l.text("VAZIO",r.x+6.0,r.y+19.0,alpha(MUTED,0x80));
        }
    }
    if let Some(g)=rects.iter().find_map(|(i,r)| {
        let (slot,_)=slots[*i];
        if r.contains(m.mouse) {m.worn.iter().find(|g|g.slot==slot)}else{None}
    }) {relic_detail(l,c,g);}
}



fn quest_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    // Live objectives use the same clipped/paginated list as other data pages.
    // Values come only from the Profile synchronized by the server.
    data_page(l, c, m);
}

fn species_name(name: &str) -> String {
    name.trim_start_matches("minecraft:").replace('_', " ")
}

/// Rows of the bestiary list that fit in `left`.
fn bestiary_rows(left: Rect) -> usize {
    (((left.h - 26.0) / 16.0).floor() as usize).max(1)
}

fn bestiary_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    let (left, right) = split(c);
    l.card(left, CYAN);
    let mut y = section(
        l,
        Rect {
            x: left.x + 8.0,
            y: left.y + 6.0,
            w: left.w - 16.0,
            h: 12.0,
        },
        "Espécies",
        CYAN,
    );
    let rows = bestiary_rows(left);
    let start = m.bestiary_sel.saturating_sub(rows.saturating_sub(1));
    for (i, s) in m.bestiary.iter().enumerate().skip(start).take(rows) {
        let r = Rect {
            x: left.x + 6.0,
            y: y - 2.0,
            w: left.w - 12.0,
            h: 15.0,
        };
        let sel = i == m.bestiary_sel;
        if sel {
            l.rect(r, alpha(CYAN, 0x22));
            l.border(r, alpha(CYAN, 0x70));
            l.rect(
                Rect {
                    x: r.x,
                    y: r.y,
                    w: 2.0,
                    h: r.h,
                },
                CYAN,
            );
        } else if r.contains(m.mouse) {
            l.rect(r, alpha(CYAN, 0x10));
        }
        l.diamond(
            r.x + 7.0,
            r.y + 7.0,
            2.4,
            if sel { CYAN } else { alpha(MUTED, 0x70) },
        );
        let k = format!("×{}", s.kills);
        // The name yields room to the kill counter instead of running into it.
        let name_w = r.w - 14.0 - text_width(&k) - 10.0;
        l.text(
            &fit_word(&species_name(&s.name).to_uppercase(), name_w),
            r.x + 14.0,
            r.y + 3.0,
            if sel { WHITE } else { alpha(ICE, 0xD0) },
        );
        l.text(
            &k,
            r.x + r.w - text_width(&k) - 4.0,
            r.y + 3.0,
            alpha(GOLD, 0xC0),
        );
        y += 16.0;
    }
    if m.bestiary.is_empty() {
        l.text(
            "Nenhuma espécie registrada.",
            left.x + 10.0,
            y,
            alpha(MUTED, 0xC0),
        );
    }

    // Detail card.
    l.card(right, CYAN);
    let Some(s) = m.bestiary.get(m.bestiary_sel) else {
        l.text(
            "Selecione uma espécie.",
            right.x + 12.0,
            right.y + 12.0,
            alpha(MUTED, 0xC0),
        );
        return;
    };
    // The universal 3-D entity viewer now lives in the right inspector;
    // do not leave an empty fake model slot in the bestiary main page.
    if m.external_inspector {
        l.text("PASSE O MOUSE OU TOQUE",right.x+8.0,right.y+13.0,CYAN);
        l.text("EM UMA ESPÉCIE",right.x+8.0,right.y+27.0,WHITE);
        l.text("As informações detalhadas",right.x+8.0,right.y+49.0,MUTED);
        l.text("aparecem no Inspector.",right.x+8.0,right.y+61.0,MUTED);
        return;
    }
    // A framed "portrait" slot: the creature's data plate.
    let side = (right.w * 0.34)
        .clamp(78.0, 118.0)
        .min(right.h - 70.0)
        .max(70.0);
    let portrait = Rect {
        x: right.x + 10.0,
        y: right.y + 10.0,
        w: side,
        h: side,
    };
    l.rect(portrait, alpha(0xFF06131F, 0xE0));
    l.grid(portrait, 10.0, 0x084FE3FF);
    l.border(portrait, alpha(CYAN, 0x40));
    // Radar-like discovery ring, drawn from the real discovery ratio.
    let pcx = portrait.x + portrait.w * 0.5;
    let pcy = portrait.y + portrait.h * 0.5;
    let rad = (portrait.w.min(portrait.h) * 0.5 - 12.0).max(12.0);
    l.diamond_outline(pcx, pcy, rad, 1.0, alpha(CYAN, 0x50), 0x00000000);
    l.diamond_outline(pcx, pcy, rad * 0.62, 1.0, alpha(CYAN, 0x30), 0x00000000);
    l.diamond(pcx, pcy, rad * unit(s.discovery), alpha(CYAN, 0x60));
    l.text(
        &format!("{:.0}%", s.discovery * 100.0),
        pcx - text_width(&format!("{:.0}%", s.discovery * 100.0)) * 0.5,
        pcy - 4.0,
        WHITE,
    );
    l.text_tracked(
        "REGISTRO",
        portrait.x + 6.0,
        portrait.y + portrait.h - 12.0,
        alpha(MUTED, 0x90),
        0.6,
    );

    let dx = portrait.x + portrait.w + 12.0;
    let dw = right.x + right.w - 10.0 - dx;
    let name = fit_tracked(&species_name(&s.name).to_uppercase(), dw, 0.7);
    l.text_tracked(&name, dx, right.y + 10.0, WHITE, 0.7);
    let sub = format!("NV {}   {}", s.level, s.tiers.to_uppercase());
    l.text(&fit(&sub, dw), dx, right.y + 22.0, alpha(MUTED, 0xD0));
    l.hline(dx, right.y + 34.0, dw, alpha(CYAN, 0x40));
    let mut y = right.y + 40.0;
    for (k, v) in [
        ("ABATES", format!("{}", s.kills)),
        ("MAIOR DANO", format!("{:.1}", s.best)),
    ] {
        if y > right.y + right.h - 14.0 {
            break;
        }
        l.text_tracked(k, dx, y, alpha(MUTED, 0x90), 0.6);
        l.text(&fit(&v, dw), dx, y + 10.0, WHITE);
        y += 26.0;
    }
    // Drops: a full-width block under the portrait, so long lists fit.
    let drop = Rect {
        x: right.x + 10.0,
        y: portrait.y + portrait.h + 8.0,
        w: right.w - 20.0,
        h: (right.y + right.h - 12.0 - (portrait.y + portrait.h + 8.0)).clamp(26.0, 64.0),
    };
    // Honest note where a 3D model would live: the pinned revision exposes no
    // entity-model rendering hook, so this stays a data plate. Drawn only
    // when it fits above the drops block.
    if y + 20.0 <= drop.y {
        l.text_tracked("MODELO 3D", dx, y, alpha(MUTED, 0x80), 0.6);
        l.text(
            &fit("indisponível nesta revisão", dw),
            dx,
            y + 10.0,
            alpha(MUTED, 0x70),
        );
    }
    l.rect(drop, alpha(0xFF08151F, 0xE0));
    l.border(drop, alpha(CYAN, 0x30));
    l.text_tracked("DROPS", drop.x + 6.0, drop.y + 4.0, alpha(MUTED, 0x90), 0.6);
    // Wrapped over the lines the block can hold; the last one is cut only if
    // the list is longer than the whole block.
    let drops = if s.drops.trim().is_empty() {
        "sem drops registrados".to_string()
    } else {
        s.drops.clone()
    };
    let rows = (((drop.h - 14.0) / 11.0).floor() as usize).max(1);
    let parts = wrap(&drops, drop.w - 12.0);
    for (k, part) in parts.iter().take(rows).enumerate() {
        let text = if k + 1 == rows && parts.len() > rows {
            fit(&parts[k..].join(" "), drop.w - 12.0)
        } else {
            part.clone()
        };
        l.text(
            &text,
            drop.x + 6.0,
            drop.y + 14.0 + k as f32 * 11.0,
            alpha(ICE, if k == 0 { 0xD0 } else { 0xA0 }),
        );
    }
}

// ---- skill tree -----------------------------------------------------

fn tree_header(_m: &MenuModel, c: Rect) -> Rect {
    Rect {
        x: c.x,
        y: c.y,
        w: c.w,
        h: 30.0,
    }
}

/// Toolbar controls: 0 map, 1 index, 2 fit, 3 centre, 4 previous, 5 next.
pub fn tree_control(head: Rect, index: usize) -> Rect {
    let (x, w) = match index {
        0 => (head.x + 4.0, 42.0),
        1 => (head.x + 48.0, 50.0),
        2 => (head.x + 100.0, 52.0),
        3 => (head.x + 154.0, 52.0),
        4 => (head.x + head.w - 32.0, 13.0),
        _ => (head.x + head.w - 17.0, 13.0),
    };
    Rect {
        x,
        y: head.y + 16.0,
        w,
        h: 12.0,
    }
}

fn tree_index_page(l: &mut Layer, g: Rect, m: &MenuModel, t: &TreeModel) {
    l.rect(g, alpha(0xFF050F1A, 0xF0));
    l.grid(g, 20.0, 0x044FE3FF);
    l.border(g, alpha(CYAN, 0x38));
    for (r, i) in tree_index_layout(m) {
        let Some(n) = t.nodes.get(i) else {
            continue;
        };
        let selected = i == t.selected;
        let (edge, state) = match n.state {
            3 => (GOLD, "MÁXIMO"),
            2 => (CYAN, "ATIVO"),
            1 => (GREEN, "LIVRE"),
            _ => (alpha(MUTED, 0x70), "BLOQ."),
        };
        l.chamfer(
            r,
            2.0,
            if selected {
                alpha(0xFF12344C, 0xF0)
            } else {
                alpha(0xFF091724, 0xE0)
            },
        );
        l.chamfer_outline(r, 2.0, 1.0, alpha(edge, if selected { 0xE0 } else { 0x50 }));
        l.rect(
            Rect {
                x: r.x,
                y: r.y + 2.0,
                w: 2.0,
                h: r.h - 4.0,
            },
            edge,
        );
        l.diamond(r.x + 9.0, r.y + 7.0, if selected { 3.0 } else { 2.2 }, edge);
        let rank = format!("{}/{}", n.rank, n.max);
        let state_w = text_width(state) + 8.0;
        let rank_w = text_width(&rank) + 6.0;
        let number = format!("{:03}", i + 1);
        let name_x = r.x + 38.0;
        let name_w = r.w - 38.0 - state_w - rank_w - 10.0;
        l.text(&number, r.x + 16.0, r.y + 3.0, alpha(MUTED, 0x80));
        l.text(
            &fit_word(&n.name.to_uppercase(), name_w),
            name_x,
            r.y + 3.0,
            if n.state == 0 {
                alpha(ICE, 0x80)
            } else {
                WHITE
            },
        );
        l.text(state, r.x + r.w - state_w - rank_w, r.y + 3.0, edge);
        l.text(
            &rank,
            r.x + r.w - rank_w + 2.0,
            r.y + 3.0,
            alpha(edge, 0xE0),
        );
    }
    let pages = tree_index_pages(g, t);
    let page = m.scroll.min(pages - 1) + 1;
    let label = format!("PÁGINA {page}/{pages}");
    l.rect(
        Rect {
            x: g.x + 3.0,
            y: g.y + g.h - 11.0,
            w: text_width(&label) + 8.0,
            h: 10.0,
        },
        alpha(0xFF050F1A, 0xF0),
    );
    l.text(&label, g.x + 7.0, g.y + g.h - 10.0, alpha(MUTED, 0xC0));
    let current = m.scroll.min(pages - 1);
    for (i, enabled) in [(0, current > 0), (1, current + 1 < pages)] {
        let r = tree_index_arrow(g, i);
        let color = alpha(CYAN, if enabled { 0xE0 } else { 0x35 });
        l.chamfer(r, 2.0, alpha(0xFF0A1B2A, 0xF0));
        l.chamfer_outline(r, 2.0, 1.0, color);
        if i == 0 {
            l.chevron_left(r.x + 6.0, r.y + 3.0, 4.0, 6.0, color);
        } else {
            l.chevron(r.x + 6.0, r.y + 3.0, 4.0, 6.0, color);
        }
    }
}

fn tree_minimap(l: &mut Layer, g: Rect, t: &TreeModel, pan: [f32; 2]) {
    if g.w < 210.0 || g.h < 70.0 {
        return;
    }
    let r = minimap_rect(g);
    l.rect(r, alpha(0xFF030B13, 0xF2));
    l.border(r, alpha(CYAN, 0x55));
    l.text_tracked("MAPA", r.x + 4.0, r.y + 2.0, alpha(CYAN, 0x80), 0.4);
    let inner = Rect {
        x: r.x + 4.0,
        y: r.y + 12.0,
        w: r.w - 8.0,
        h: r.h - 15.0,
    };
    let size = graph_size(t);
    let scale = (inner.w / size[0]).min(inner.h / size[1]);
    let ox = inner.x + (inner.w - size[0] * scale) * 0.5;
    let oy = inner.y + (inner.h - size[1] * scale) * 0.5;
    for n in &t.nodes {
        let b = node_base(n.col, n.row);
        let color = match n.state {
            3 => GOLD,
            2 => CYAN,
            1 => ICE,
            _ => alpha(MUTED, 0x55),
        };
        l.rect(
            Rect {
                x: ox + b[0] * scale,
                y: oy + b[1] * scale,
                w: 1.5,
                h: 1.5,
            },
            color,
        );
    }
    let z = clamp_zoom(t.zoom);
    let view = Rect {
        x: ox - pan[0] * scale / z,
        y: oy - pan[1] * scale / z,
        w: g.w * scale / z,
        h: g.h * scale / z,
    };
    if let Some(v) = intersect(view, inner) {
        l.border(v, alpha(WHITE, 0xD0));
    }
}


// -------------------------------------------------- v4 Modular Skill Codex
// Node symbols are genuine atlas assets, not stretched rectangles. Every
// ability is a selectable row; no zoom/drag is required at any resolution.
fn talent_symbol(id: &str) -> &'static str {
    if id.starts_with("bridge.") {"claude_rpg:gui/icons/skills/icon_experience"}
    else if id.contains("health") || id.contains("vitality") || id.contains("regen") {
        "claude_rpg:gui/icons/attributes/icon_vitality"
    } else if id.contains("crit") {"claude_rpg:gui/icons/attributes/icon_critical"}
    else if id.contains("luck") || id.contains("rarity") {"claude_rpg:gui/icons/attributes/icon_luck"}
    else if id.contains("armor") || id.contains("defense") {"claude_rpg:gui/icons/attributes/icon_defense"}
    else if id.contains("resist") {"claude_rpg:gui/icons/attributes/icon_resistance"}
    else if id.contains("precision") || id.contains("accuracy") {"claude_rpg:gui/icons/attributes/icon_precision"}
    else if id.contains("penetrat") {"claude_rpg:gui/icons/attributes/icon_penetration"}
    else if id.contains("speed") || id.contains("agility") {"claude_rpg:gui/icons/attributes/icon_agility"}
    else if id.contains("mana") || id.contains("intelligence") {"claude_rpg:gui/icons/attributes/icon_intelligence"}
    else if id.contains("mining") {"claude_rpg:gui/icons/professions/icon_mining"}
    else if id.contains("fishing") {"claude_rpg:gui/icons/professions/icon_fishing"}
    else if id.contains("farming") {"claude_rpg:gui/icons/professions/icon_farming"}
    else if id.contains("exploration") {"claude_rpg:gui/icons/professions/icon_exploration"}
    else {"claude_rpg:gui/icons/attributes/icon_strength"}
}
// -------------------------------------------------- v4.0.2 Connected Skill Graph
// The graph is primary. Separate branches use actual prerequisites and linked
// nodes, not a flat list; large trees are split into readable segments.
// The selector shows all categories at once, as in a data-driven skill UI.
fn constellation_tree_arrow(c:Rect,i:usize)->Rect {
    Rect {x:c.x+c.w-139.0+i as f32*20.0,y:c.y+4.0,w:17.0,h:14.0}
}
fn constellation_catalog_rect(c:Rect)->Rect {
    Rect { x:c.x+c.w-96.0, y:c.y+3.0, w:92.0, h:15.0 }
}

pub fn constellation_compact(m:&MenuModel)->bool {
    if m.external_inspector{return false;}
    let c=content(window(m.area),m.pages.len());
    c.w<505.0 || c.h<175.0
}
pub fn constellation_detail_toggle(m:&MenuModel)->Rect {
    let c=content(window(m.area),m.pages.len());
    Rect {x:c.x+c.w-83.0,y:c.y+20.0,w:80.0,h:15.0}
}
/// The detail inspector never replaces the graph: narrow screens stack both.
pub fn constellation_regions(m:&MenuModel)->(Rect,Rect) {
    let c=content(window(m.area),m.pages.len());
    let top=c.y+37.0;
    let available=(c.h-37.0).max(1.0);
    if m.external_inspector {
        return (Rect{x:c.x,y:top,w:c.w,h:available},
                Rect{x:c.x+c.w+8.0,y:top,w:0.0,h:0.0});
    }
    if c.w>=505.0 && available>=175.0 {
        let w=(c.w*0.60).clamp(225.0,c.w-180.0);
        (Rect{x:c.x,y:top,w:w-4.0,h:available},
         Rect{x:c.x+w+3.0,y:top,w:c.w-w-3.0,h:available})
    } else {
        let detail_h=(available*0.42).max(35.0).min((available-26.0).max(1.0));
        let graph_h=(available-detail_h-4.0).max(1.0);
        (Rect{x:c.x,y:top,w:c.w,h:graph_h},
         Rect{x:c.x,y:top+graph_h+4.0,w:c.w,h:detail_h})
    }
}

pub fn constellation_detail_rect(m:&MenuModel)->Rect {constellation_regions(m).1}
fn constellation_rows(m:&MenuModel)->usize {
    let (g,_) = constellation_regions(m);
    if g.h>=305.0 {2}else{1}
}
fn constellation_segments(m:&MenuModel)->Vec<super::crpg_skillgraph::Segment> {
    let (g,_) = constellation_regions(m);
    let cap=if g.h<125.0 {4} else if g.h<195.0 {5}else{7};
    match m.tree.as_ref() {
        Some(t)=>super::crpg_skillgraph::orbital_groups(t.nodes.len(),&t.links,cap),
        None=>Vec::new(),
    }
}
fn constellation_category_rows(m:&MenuModel)->usize {
    let (g,_) =constellation_regions(m);
    (((g.h-24.0)/25.0).floor() as usize).max(1)
}
fn constellation_category_cols(m:&MenuModel)->usize {
    let (g,_) =constellation_regions(m);
    if g.w>=320.0 {2} else {1}
}
pub fn constellation_pages(m:&MenuModel)->usize {
    if m.tree.as_ref().is_some_and(|t|t.index_mode) {
        let size=constellation_category_rows(m)*constellation_category_cols(m);
        return super::crpg_nodes::trees().len().div_ceil(size.max(1)).max(1);
    }
    // The skill map is continuous; it pans and zooms like the reference GUI.
    1
}

pub fn constellation_page_arrow(m:&MenuModel,right:bool)->Rect {
    let (g,_) =constellation_regions(m);
    Rect{x:if right{g.x+g.w-22.0}else{g.x+4.0},
         y:g.y+2.0,w:18.0,h:16.0}
}
fn constellation_groups_on_page(m:&MenuModel)->Vec<(usize,super::crpg_skillgraph::Segment)> {
    let per=constellation_rows(m);
    let seg=constellation_segments(m);
    let start=m.scroll.min(constellation_pages(m).saturating_sub(1))*per;
    seg.into_iter().skip(start).take(per).enumerate().collect()
}
fn constellation_branch_rect(m:&MenuModel,row:usize)->Rect {
    let (g,_) =constellation_regions(m);
    let rows=constellation_rows(m) as f32;
    let h=(g.h-23.0)/rows;
    Rect{x:g.x+3.0,y:g.y+21.0+row as f32*h,
         w:(g.w-6.0).max(20.0),h:(h-2.0).max(24.0)}
}
fn constellation_segment_rects(m:&MenuModel,row:usize,
    s:&super::crpg_skillgraph::Segment)->Vec<(Rect,usize)> {
    let r=constellation_branch_rect(m,row);
    let count=s.members.len().max(1);
    // Seven legible nodes: one central keystone and six around a ring. No zoom.
    let side=if r.h<115.0 {19.0}else{25.0};
    let rx=((r.w-side-30.0)*0.40).clamp(16.0,140.0);
    let ry=((r.h-side-20.0)*0.43).clamp(10.0,68.0);
    let cx=r.x+r.w*0.5;
    let cy=r.y+r.h*0.52;
    s.members.iter().enumerate().map(|(slot,&id)| {
        let (x,y)=if r.h<95.0 {
            let gap=r.w/(count+1) as f32;
            (r.x+gap*(slot+1) as f32,cy)
        } else if slot==0 {(cx,cy)} else {
            let angle=-std::f32::consts::FRAC_PI_2+
                std::f32::consts::TAU*(slot-1) as f32/(count-1) as f32;
            (cx+angle.cos()*rx,cy+angle.sin()*ry)
        };
        (Rect{x:x-side*0.5,y:y-side*0.5,w:side,h:side},id)
    }).collect()
}
/// Circles share the *same* geometry for rendering, hover and pointer hits.
fn constellation_node_rect(g:Rect,t:&TreeModel,n:&TreeNode,i:usize,pan:[f32;2])->Rect {
    let raw=node_rect(g,n,pan,t.zoom);
    let is_root=!t.links.iter().any(|&(_,child)|child==i);
    let base=if is_root{30.0}else if n.state==3{26.0}else{23.0};
    let side=(base*clamp_zoom(t.zoom)).max(5.0);
    Rect{x:raw.x+raw.w*0.5-side*0.5,
         y:raw.y+raw.h*0.5-side*0.5,w:side,h:side}
}
pub fn constellation_node_rects(m:&MenuModel)->Vec<(Rect,usize)> {
    let Some(t)=m.tree.as_ref()else{return Vec::new();};
    if t.index_mode{return Vec::new();}
    let (g,_) =constellation_regions(m);
    let pan=view_pan(g,t);
    t.nodes.iter().enumerate().filter_map(|(i,n)|{
        intersect(constellation_node_rect(g,t,n,i,pan),g).map(|r|(r,i))
    }).collect()
}

pub fn constellation_categories(m:&MenuModel)->Vec<(Rect,usize)> {
    if !m.tree.as_ref().is_some_and(|t|t.index_mode){return Vec::new();}
    let all=super::crpg_nodes::trees();
    let (g,_) =constellation_regions(m);
    let cols=constellation_category_cols(m);
    let rows=constellation_category_rows(m);
    let count=cols*rows;
    let start=m.scroll.min(constellation_pages(m)-1)*count;
    let w=(g.w-7.0-(cols as f32-1.0)*4.0)/cols as f32;
    (start..all.len().min(start+count)).map(|i|{
        let j=i-start;
        (Rect {x:g.x+3.0+(j%cols) as f32*(w+4.0),
             y:g.y+21.0+(j/cols) as f32*25.0,w,h:22.0},i)
    }).collect()
}
pub fn constellation_buy_rect(m:&MenuModel)->Rect {
    let d=constellation_detail_rect(m);
    Rect{x:d.x+6.0,y:d.y+d.h-24.0,w:(d.w-12.0).max(10.0),h:18.0}
}
fn constellation_label(raw:&str)->String {
    let short=raw.split(" — ").next().unwrap_or(raw);
    short.to_string()
}
fn constellation_category_label(id:&str)->String {
    let suffix=id.strip_prefix("weapon.").map(|s|format!("Armas · {s}"))
        .or_else(||id.strip_prefix("luck.").map(|s|format!("Sorte · {s}")))
        .unwrap_or_else(||id.to_string());
    suffix.replace('_'," ")
}
fn constellation_draw_detail(l:&mut Layer,m:&MenuModel,hovered:Option<usize>) {
    let Some(t)=m.tree.as_ref()else{return;};
    let d=constellation_detail_rect(m);
    l.rect(d,0xF30A1727);
    l.border(d,alpha(CYAN,0x88));
    l.rect(Rect{x:d.x,y:d.y,w:2.0,h:d.h},alpha(CYAN,0xE0));
    let target=hovered.unwrap_or(t.selected);
    let Some(n)=t.nodes.get(target)else{return;};
    let panel_w=(d.w-16.0).max(24.0);
    let mut lines:Vec<(String,u32)>=Vec::new();
    let hint=if hovered.is_some_and(|i|i!=t.selected) {
        "PRÉVIA · TOQUE PARA SELECIONAR"
    } else {"HABILIDADE SELECIONADA"};
    lines.push((hint.to_owned(),CYAN));
    for s in wrap(&n.name,panel_w){lines.push((s,WHITE));}
    let state=match n.state {3=>"CONCLUÍDA",2=>"ADQUIRIDA",1=>"DISPONÍVEL",_=>"BLOQUEADA"};
    lines.push((format!("{state} · {}/{} · {} PT",n.rank,n.max,n.cost),GOLD));
    lines.push(("EFEITO".into(),CYAN));
    for s in wrap(&n.description,panel_w){lines.push((s,ICE));}
    lines.push(("REQUISITOS".into(),CYAN));
    for s in wrap(&n.requirement,panel_w){lines.push((s,if n.buyable{GREEN}else{AMBER}));}
    let nvis=(((d.h-32.0)/11.0).floor() as usize).max(1);
    let offset=if hovered.is_some_and(|i|i!=t.selected){0}else{
        m.detail_scroll.min(lines.len().saturating_sub(nvis))};
    for (idx,(s,color)) in lines.iter().skip(offset).take(nvis).enumerate(){
        l.text(s,d.x+8.0,d.y+5.0+idx as f32*11.0,*color);
    }
    if lines.len()>nvis {
        l.text(&format!("{}-{} / {} · ROLE",offset+1,(offset+nvis).min(lines.len()),lines.len()),
            d.x+8.0,d.y+d.h-33.0,MUTED);
    }
    let b=constellation_buy_rect(m);
    let can=t.nodes.get(t.selected).is_some_and(|n|n.buyable);
    l.rect(b,if can{0xFF12627A}else{0xFF1A2837});
    l.border(b,if can{GREEN}else{alpha(MUTED,0x99)});
    let selected_def=t.nodes.get(t.selected).and_then(|n|super::crpg_nodes::get(&n.id));
    let label=if selected_def.is_some_and(|d|!super::crpg_nodes::runtime_supported(d)) {
        "EFEITO NÃO IMPLEMENTADO"
    }else if can{"ADQUIRIR NÍVEL"}else{"REQUISITOS PENDENTES"};
    l.text(&fit_word(label,b.w-8.0),b.x+4.0,b.y+5.0,
        if can{WHITE}else{MUTED});
}
fn constellation_fit_rect(m:&MenuModel)->Rect {
    let (g,_) =constellation_regions(m);
    Rect{x:g.x+g.w*0.5-21.0,y:g.y+3.0,w:42.0,h:14.0}
}

/// Liang-Barsky clipping keeps diagonal links and orbit outlines *inside* the
/// map instead of painting over the permanently visible description panel.
fn constellation_wire(l:&mut Layer,g:Rect,p:[f32;2],q:[f32;2],width:f32,color:u32) {
    let (dx,dy)=(q[0]-p[0],q[1]-p[1]);
    if !dx.is_finite()||!dy.is_finite()||dx*dx+dy*dy<0.001{return;}
    let mut lo=0.0_f32;
    let mut hi=1.0_f32;
    for (a,b) in [
        (-dx,p[0]-g.x),(dx,g.x+g.w-p[0]),
        (-dy,p[1]-g.y),(dy,g.y+g.h-p[1]),
    ] {
        if a.abs()<0.0001 {if b<0.0{return;}}
        else {
            let k=b/a;
            if a<0.0{lo=lo.max(k);}else{hi=hi.min(k);}
        }
    }
    if lo>hi{return;}
    let a=[p[0]+lo*dx,p[1]+lo*dy];
    let b=[p[0]+hi*dx,p[1]+hi*dy];
    let norm=(dx*dx+dy*dy).sqrt().max(0.01);
    let t=width.max(0.25)*0.5;
    let nx=-dy/norm*t;let ny=dx/norm*t;
    l.quad([[a[0]+nx,a[1]+ny],[b[0]+nx,b[1]+ny],
            [b[0]-nx,b[1]-ny],[a[0]-nx,a[1]-ny]],color);
}

/// Continuous node-and-link map inspired by Remnants' rendering mechanics:
/// connections behind nodes, state-dependent rings, glow, viewport zoom/pan
/// and persistent inspector. It uses the Claude RPG's own icons and palette.
fn constellation_page(l:&mut Layer,c:Rect,m:&MenuModel) {
    let Some(t)=m.tree.as_ref()else{
        l.text("Carregando habilidades...",c.x+8.0,c.y+8.0,CYAN);return;
    };
    l.text(&fit_word(&t.name.to_uppercase(),(c.w-157.0).max(35.0)),
        c.x+5.0,c.y+4.0,WHITE);
    for i in 0..2 {
        let r=constellation_tree_arrow(c,i);
        l.border(r,alpha(CYAN,0xA0));
        if i==0{l.chevron_left(r.x+3.0,r.y+3.0,5.0,7.0,CYAN)}
        else{l.chevron(r.x+4.0,r.y+3.0,5.0,7.0,CYAN)}
    }
    let cat=constellation_catalog_rect(c);
    l.rect(cat,if t.index_mode{0xFF14546B}else{0xFF112B3E});
    l.border(cat,if t.index_mode{GREEN}else{alpha(CYAN,0xA0)});
    l.text(&fit_word(if t.index_mode{"VOLTAR AO MAPA"}else{"ESCOLAS"},cat.w-10.0),
        cat.x+5.0,cat.y+4.0,WHITE);
    let owned=t.nodes.iter().filter(|n|n.state>=2).count();
    l.text(&fit_word(&format!("NÓS {owned}/{} · {} PONTOS",
        t.nodes.len(),m.points),(c.w-10.0).max(35.0)),c.x+5.0,c.y+23.0,GOLD);
    let (g,d)=constellation_regions(m);
    l.rect(g,0xF0091525);
    l.border(g,alpha(CYAN,0x83));
    if t.index_mode {
        let pages=constellation_pages(m);
        let page=m.scroll.min(pages-1);
        l.text_center(&format!("ESCOLAS {}/{}",page+1,pages),
            Rect{x:g.x+26.0,y:g.y,w:g.w-52.0,h:19.0},g.y+6.0,MUTED);
        for (right,enabled) in [(false,page>0),(true,page+1<pages)] {
            let ar=constellation_page_arrow(m,right);
            l.border(ar,if enabled{CYAN}else{alpha(MUTED,0x60)});
            if right{l.chevron(ar.x+6.0,ar.y+5.0,5.0,7.0,CYAN)}
            else{l.chevron_left(ar.x+6.0,ar.y+5.0,5.0,7.0,CYAN)}
        }
        let names=super::crpg_nodes::trees();
        for (r,i) in constellation_categories(m) {
            let Some(id)=names.get(i)else{continue;};
            let active=t.nodes.first().and_then(|n|super::crpg_nodes::get(&n.id))
                .is_some_and(|n|n.tree==*id);
            l.chamfer(r,5.0,if active{0xFF1C5261}else{0xFF102638});
            l.frame(r,5.0,1.0,if active{GREEN}else{alpha(CYAN,0x55)});
            l.sprite(talent_symbol(id),[0.0,0.0,128.0,128.0],
                Rect{x:r.x+4.0,y:r.y+3.0,w:15.0,h:15.0});
            l.text(&fit_word(&constellation_category_label(id),r.w-26.0),
                r.x+24.0,r.y+7.0,if active{GREEN}else{WHITE});
        }
    } else {
        // A stationary celestial background helps players read panning.
        for i in 0..80u32 {
            let xx=g.x+4.0+((i.wrapping_mul(89)+37)%997) as f32/997.0*(g.w-8.0).max(1.0);
            let yy=g.y+3.0+((i.wrapping_mul(233)+71)%991) as f32/991.0*(g.h-6.0).max(1.0);
            let pulse=if (m.tick/18+i)%5==0 {0x55}else{0x25};
            l.rect(Rect{x:xx,y:yy,w:1.0,h:1.0},alpha(ICE,pulse));
        }
        let zoom=clamp_zoom(t.zoom);
        let pan=view_pan(g,t);
        for &radius in &[0.22_f32,0.38,0.53] {
            let cx=g.x+g.w*0.5;
            let cy=g.y+g.h*0.5;
            let r=g.w.min(g.h)*radius;
            for j in 0..36 {
                let a=std::f32::consts::TAU*(j as f32)/36.0;
                let b=std::f32::consts::TAU*((j+1) as f32)/36.0;
                constellation_wire(l,g,[cx+a.cos()*r,cy+a.sin()*r],
                    [cx+b.cos()*r,cy+b.sin()*r],0.55,alpha(CYAN,0x18));
            }
        }
        for &(a,b) in &t.links {
            let (Some(pa),Some(pb))=(t.nodes.get(a),t.nodes.get(b))else{continue;};
            let ra=constellation_node_rect(g,t,pa,a,pan);
            let rb=constellation_node_rect(g,t,pb,b,pan);
            let p=[ra.x+ra.w*0.5,ra.y+ra.h*0.5];
            let q=[rb.x+rb.w*0.5,rb.y+rb.h*0.5];
            let color=if pa.rank>0 && pb.rank>0 {GOLD}
                else if pa.rank>0 && pb.state==1 {alpha(GREEN,0xD0)}
                else if pb.state==1 {alpha(CYAN,0xA0)}
                else {alpha(MUTED,0x65)};
            if pa.rank>0 && pb.rank>0 {
                constellation_wire(l,g,p,q,(3.2*zoom).max(1.2),alpha(GOLD,0x29));
            }
            constellation_wire(l,g,p,q,(1.55*zoom).max(0.55),color);
        }
        let hovered=constellation_node_rects(m).into_iter()
            .find(|(r,_)|r.contains(m.mouse)).map(|(_,i)|i);
        for (i,n) in t.nodes.iter().enumerate() {
            let r=constellation_node_rect(g,t,n,i,pan);
            if intersect(r,g).is_none(){continue;}
            let hot=hovered==Some(i);
            let selected=i==t.selected;
            let root=!t.links.iter().any(|&(_,child)|child==i);
            let edge=match n.state{3=>GOLD,2=>GREEN,1=>CYAN,_=>alpha(MUTED,0xA0)};
            if selected||hot {l.glow(r,8.0,3.0,edge,2);}
            let halo=Rect{x:r.x-3.0*zoom,y:r.y-3.0*zoom,
                w:r.w+6.0*zoom,h:r.h+6.0*zoom};
            if root||selected {l.frame(halo,halo.w*0.35,0.7,alpha(edge,0x75));}
            l.chamfer(r,r.w*0.34,0xF0081927);
            l.frame(r,r.w*0.34,if selected{1.8}else{1.1},edge);
            if r.w>=13.0 {
                let inset=(r.w*0.21).max(2.0);
                l.sprite(talent_symbol(&n.id),[0.0,0.0,128.0,128.0],
                    Rect{x:r.x+inset,y:r.y+inset,w:r.w-2.0*inset,h:r.h-2.0*inset});
            } else {
                l.diamond(r.x+r.w*0.5,r.y+r.h*0.5,r.w*0.23,edge);
            }
            if n.buyable && r.w>=11.0 {
                l.diamond(r.x+r.w-1.0,r.y+1.0,2.1,GREEN);
            }
        }
        l.text_center(&format!("ZOOM {:.0}%",zoom*100.0),
            Rect{x:g.x+30.0,y:g.y,w:g.w-60.0,h:17.0},g.y+5.0,MUTED);
        let fit=constellation_fit_rect(m);
        l.border(fit,alpha(CYAN,0xA0));
        l.text_center("AJUSTAR",fit,fit.y+4.0,CYAN);
        for (right,label) in [(false,"−"),(true,"+")] {
            let ar=constellation_page_arrow(m,right);
            l.border(ar,alpha(CYAN,0xA0));
            l.text_center(label,ar,ar.y+3.0,WHITE);
        }
    }
    // No switch to another screen: the selected node's description stays visible.
    if d.w>1.0 && d.h>1.0 {constellation_draw_detail(l,m,None);}
}

// -------------------------------------------------- v3.4 Skill Codex
// A talent catalog, NOT a graph. Every talent keeps a readable physical size;
// even an arbitrary number of nodes is reachable by pagination. It never
// needs pan/zoom, and a dedicated panel owns complete description + requirements.
pub fn codex_regions(m:&MenuModel)->(Rect,Rect) {
    let c=content(window(m.area),m.pages.len());
    let y=c.y+29.0;
    let h=(c.h-29.0).max(35.0);
    if c.w>=455.0 && h>=115.0 {
        let lw=(c.w*0.46).clamp(170.0,c.w-180.0);
        (Rect{x:c.x,y,w:lw-5.0,h},Rect{x:c.x+lw+3.0,y,w:c.w-lw-3.0,h})
    } else {
        let lh=(h*0.48).max(35.0).min(h-28.0);
        (Rect{x:c.x,y,w:c.w,h:lh},Rect{x:c.x,y:y+lh+3.0,w:c.w,h:(h-lh-3.0).max(25.0)})
    }
}
pub fn codex_rows_per_page(m:&MenuModel)->usize {
    let (list,_) =codex_regions(m);
    (((list.h-20.0)/24.0).floor() as usize).max(1)
}
pub fn codex_pages(m:&MenuModel)->usize {
    m.tree.as_ref().map_or(1,|t|t.nodes.len().div_ceil(codex_rows_per_page(m)).max(1))
}
pub fn codex_rows(m:&MenuModel)->Vec<(Rect,usize)> {
    let Some(tree)=&m.tree else{return Vec::new();};
    let (list,_) =codex_regions(m);
    let rows=codex_rows_per_page(m);
    let page=m.scroll.min(codex_pages(m).saturating_sub(1));
    (page*rows..(page+1)*rows).take_while(|i|*i<tree.nodes.len())
        .enumerate().map(|(j,i)|(
            Rect{x:list.x+5.0,y:list.y+4.0+j as f32*24.0,w:list.w-10.0,h:22.0},i))
        .collect()
}
pub fn codex_page_arrow(m:&MenuModel,right:bool)->Rect {
    let (list,_)=codex_regions(m);
    Rect{x:if right {list.x+list.w-24.0}else{list.x+5.0},
      y:list.y+list.h-15.0,w:19.0,h:13.0}
}
pub fn codex_detail_rect(m:&MenuModel)->Rect {codex_regions(m).1}
pub fn codex_buy_rect(m:&MenuModel)->Rect {
    let (_,detail)=codex_regions(m);
    Rect{x:detail.x+6.0,y:detail.y+detail.h-23.0,w:(detail.w-12.0).max(15.0),h:18.0}
}
fn tree_codex_page(l:&mut Layer,c:Rect,m:&MenuModel) {
    let Some(t)=m.tree.as_ref() else { l.text("Carregando catálogo...",c.x+8.0,c.y+8.0,CYAN);return;};
    let title=fit_word(&format!("{}  /  CATÁLOGO DE TALENTOS",t.name.to_uppercase()),(c.w-64.0).max(80.0));
    l.text(&title,c.x+7.0,c.y+3.0,CYAN);
    // Category switcher. Different trees stay separate and never collapse
    // into one arbitrary 78x3 grid.
    let back=tree_arrow(tree_header(m,c),0);
    let next=tree_arrow(tree_header(m,c),1);
    l.border(back,alpha(CYAN,0x90));l.border(next,alpha(CYAN,0x90));
    l.chevron_left(back.x+4.0,back.y+3.0,6.0,7.0,CYAN);
    l.chevron(next.x+5.0,next.y+3.0,6.0,7.0,CYAN);
    let owned=t.nodes.iter().filter(|n|n.state>=2).count();
    let done=format!("{} / {} adquiridas · {} PT",owned,t.nodes.len(),m.points);
    l.text(&fit_word(&done,(c.w-12.0).max(60.0)),c.x+7.0,c.y+18.0,GOLD);
    let (list,detail)=codex_regions(m);
    l.rect(list,alpha(0xFF071624,0xF6));l.border(list,alpha(CYAN,0x60));
    l.rect(detail,alpha(0xFF091A2B,0xFA));
    l.sprite("claude_rpg:gui/ui/tooltips/ui_panel_tooltip",[0.0,0.0,512.0,256.0],detail);
    l.border(detail,alpha(CYAN,0x90));
    let rows=codex_rows(m);
    let hovered=rows.iter().find(|(r,_)|r.contains(m.mouse)).map(|(_,i)|*i);
    for (r,i) in rows {
        let Some(n)=t.nodes.get(i) else{continue;};
        let selected=i==t.selected;
        let preview=Some(i)==hovered;
        let accent=match n.state {3=>GOLD,2=>CYAN,1=>GREEN,_=>MUTED};
        l.rect(r,if selected{0xFF163B4D}else if preview{0xFF122C41}else{0xDD0B2233});
        l.border(r,alpha(accent,if selected||preview{0xFF}else{0x60}));
        l.rect(Rect{x:r.x,y:r.y,w:2.0,h:r.h},accent);
        // Real assets represent the unlocked/owned/locked states.
        let asset=match n.state {
            3=>"claude_rpg:gui/ui/panels/ui_tree_node_maxed",
            2=>"claude_rpg:gui/ui/panels/ui_tree_node_owned",
            1=>"claude_rpg:gui/ui/panels/ui_tree_node_available",
            _=>"claude_rpg:gui/ui/panels/ui_tree_node_locked",
        };
        // The node-state frame is displayed at icon size. Previously the
        // 256x128 sprite was distorted across the entire button.
        l.sprite(asset,[0.0,0.0,256.0,128.0],Rect {
            x:r.x+4.0,y:r.y+2.0,w:25.0,h:18.0,
        });
        l.sprite(talent_symbol(&n.id),[0.0,0.0,128.0,128.0],Rect {
            x:r.x+8.0,y:r.y+4.0,w:15.0,h:14.0,
        });
        let badge=match n.state {3=>"MAX",2=>"ATIVO",1=>"LIVRE",_=>"TRAVA"};
        let rank=format!("{}/{}",n.rank,n.max);
        let name_w=(r.w-text_width(&rank)-text_width(badge)-64.0).max(20.0);
        l.text(&fit_word(&n.name,name_w),r.x+36.0,r.y+6.0,if n.state==0{MUTED}else{WHITE});
        let status_x=r.x+r.w-text_width(&badge)-text_width(&rank)-13.0;
        l.text(badge,status_x,r.y+6.0,accent);
        l.text(&rank,r.x+r.w-text_width(&rank)-5.0,r.y+6.0,GOLD);
    }
    let current=m.scroll.min(codex_pages(m)-1);
    let pages=codex_pages(m);
    let label=format!("PÁGINA {}/{}",current+1,pages);
    let page_y=list.y+list.h-13.0;
    l.text(&label,list.x+(list.w-text_width(&label))*0.5,page_y,MUTED);
    for (right,active) in [(false,current>0),(true,current+1<pages)] {
        let r=codex_page_arrow(m,right);
        l.border(r,if active{CYAN}else{MUTED});
        if right{l.chevron(r.x+6.0,r.y+3.0,5.0,6.0,if active{CYAN}else{MUTED});}
        else{l.chevron_left(r.x+6.0,r.y+3.0,5.0,6.0,if active{CYAN}else{MUTED});}
    }
    // Hover previews; clicking a row selects the node for purchase.
    let chosen=hovered.unwrap_or(t.selected);
    if let Some(n)=t.nodes.get(chosen) {
        if detail.w>125.0 && detail.h>80.0 {
            l.sprite(talent_symbol(&n.id),[0.0,0.0,128.0,128.0],Rect {
                x:detail.x+detail.w-28.0,y:detail.y+3.0,w:22.0,h:22.0,
            });
        }
        let inner=(detail.w-16.0).max(24.0);
        let mut lines=Vec::<(String,u32)>::new();
        lines.push(("DETALHES DA HABILIDADE".into(),CYAN));
        for text in wrap(&n.name,inner){lines.push((text,WHITE));}
        lines.push((format!("NÍVEL {}/{}    •    CUSTO: {} PT",n.rank,n.max,n.cost),GOLD));
        lines.push(("EFEITO".into(),CYAN));
        for text in wrap(&n.description,inner){lines.push((text,ICE));}
        lines.push(("REQUISITOS".into(),CYAN));
        for text in wrap(&n.requirement,inner){lines.push((text,if n.buyable{GREEN}else{AMBER}));}
        let visible=(((detail.h-32.0)/11.0).floor() as usize).max(1);
        // Hover previews and selected talents use the same scroll state;
        // long descriptions remain accessible rather than being truncated.
        let start=m.detail_scroll.min(lines.len().saturating_sub(visible));
        for (i,(text,color)) in lines.iter().skip(start).take(visible).enumerate(){
            let yy=detail.y+7.0+i as f32*11.0;
            if yy+10.0<detail.y+detail.h-19.0 {
                l.text(text,detail.x+8.0,yy,*color);
            }
        }
        if lines.len()>visible {
            let counter=format!("TEXTO {}/{} · ROLE",start+1,lines.len());
            l.text(&counter,detail.x+8.0,detail.y+detail.h-32.0,MUTED);
        }
    }
    let buy=codex_buy_rect(m);
    let can_buy=t.nodes.get(t.selected).is_some_and(|n|n.buyable);
    l.rect(buy,if can_buy{0xFF14566A}else{0xFF23303C});
    l.border(buy,if can_buy{GREEN}else{MUTED});
    let text=if can_buy{"APRENDER HABILIDADE"}else{"REQUISITOS PENDENTES"};
    l.text(&fit_word(text,buy.w-10.0),buy.x+5.0,buy.y+5.0,if can_buy{WHITE}else{MUTED});
}

fn tree_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    let Some(t) = &m.tree else {
        l.card(c, CYAN);
        l.text("Carregando árvore...", c.x + 12.0, c.y + 12.0, MUTED);
        return;
    };
    let head = tree_header(m, c);
    l.card(
        Rect {
            x: head.x,
            y: head.y,
            w: head.w,
            h: head.h,
        },
        CYAN,
    );
    l.text_tracked(
        &fit_tracked(
            &t.name.to_uppercase(),
            head.w
                - 48.0
                - text_width(&format!("{}/{} · 100%", t.nodes.len(), t.nodes.len()))
                - 12.0,
            0.8,
        ),
        head.x + 8.0,
        head.y + 3.0,
        CYAN,
        0.8,
    );
    // Tree paging arrows.
    let prev = tree_arrow(head, 0);
    let next = tree_arrow(head, 1);
    l.chevron_left(prev.x + 5.0, prev.y + 4.0, 5.0, 7.0, alpha(CYAN, 0xC0));
    l.chevron(next.x + 6.0, next.y + 4.0, 5.0, 7.0, alpha(CYAN, 0xC0));
    let owned = t.nodes.iter().filter(|n| n.state >= 2).count();
    let zoom = format!(
        "{}/{} NÓS · Z{:.0}%",
        owned,
        t.nodes.len(),
        clamp_zoom(t.zoom) * 100.0
    );
    l.text(
        &zoom,
        prev.x - text_width(&zoom) - 8.0,
        head.y + 3.0,
        alpha(MUTED, 0xC0),
    );
    // Two complementary ways to explore the same complete catalog.
    l.tab(tree_control(head, 0), "MAPA", !t.index_mode, false, CYAN);
    l.tab(tree_control(head, 1), "ÍNDICE", t.index_mode, false, CYAN);
    if !t.index_mode {
        l.tab(tree_control(head, 2), "GERAL", false, false, CYAN);
        l.tab(tree_control(head, 3), "CENTRO", false, false, CYAN);
    }
    let before = tree_control(head, 4);
    let after = tree_control(head, 5);
    l.chevron_left(before.x + 4.0, before.y + 3.0, 4.0, 6.0, CYAN);
    l.chevron(after.x + 5.0, after.y + 3.0, 4.0, 6.0, CYAN);
    let selected = format!("NÓ {}/{}", t.selected.saturating_add(1), t.nodes.len());
    l.text(
        &selected,
        before.x - text_width(&selected) - 6.0,
        before.y + 2.0,
        alpha(ICE, 0xC0),
    );

    let g = tree_graph_rect(m);
    if t.index_mode {
        tree_index_page(l, g, m, t);
    } else {
        l.rect(g, alpha(0xFF050F1A, 0xE6));
        l.grid(g, 18.0, 0x064FE3FF);
        l.border(g, alpha(CYAN, 0x2A));
        // Everything inside the graph is clipped to the viewport: a node that runs
        // past the edge is cut, never drawn over the header or the detail strip.
        l.set_clip(Some(g));

        let pan = view_pan(g, t);
        // Connections first, so nodes sit on top.
        for (a, b) in &t.links {
            let (Some(pa), Some(pb)) = (t.nodes.get(*a), t.nodes.get(*b)) else {
                continue;
            };
            let ra = node_rect(g, pa, pan, t.zoom);
            let rb = node_rect(g, pb, pan, t.zoom);
            // Run the wire from edge to edge, never under the node plates.
            let (x0, y0, x1, y1) = if rb.x >= ra.x + ra.w * 0.5 {
                (ra.x + ra.w, ra.y + ra.h * 0.5, rb.x, rb.y + rb.h * 0.5)
            } else {
                (ra.x, ra.y + ra.h * 0.5, rb.x + rb.w, rb.y + rb.h * 0.5)
            };
            let color = match (pa.state, pb.state) {
                (_, 3) | (_, 2) => alpha(CYAN, 0xB0),
                (_, 1) => alpha(ICE, 0x60),
                _ => alpha(MUTED, 0x40),
            };
            // Elbow connector: horizontal, vertical, horizontal.
            let mx = (x0 + x1) * 0.5;
            l.rect(
                Rect {
                    x: x0,
                    y: y0 - 1.0,
                    w: (mx - x0).max(0.0),
                    h: 2.0,
                },
                color,
            );
            l.rect(
                Rect {
                    x: mx - 1.0,
                    y: y0.min(y1),
                    w: 2.0,
                    h: (y1 - y0).abs().max(2.0),
                },
                color,
            );
            l.rect(
                Rect {
                    x: mx,
                    y: y1 - 1.0,
                    w: (x1 - mx).max(0.0),
                    h: 2.0,
                },
                color,
            );
        }

        for (i, n) in t.nodes.iter().enumerate() {
            let r = node_rect(g, n, pan, t.zoom);
            if intersect(r, g).is_none() {
                continue;
            }
            let (fill, edge, text) = match n.state {
                3 => (alpha(0xFF123A4E, 0xF0), GOLD, WHITE),
                2 => (alpha(0xFF0E2E44, 0xF0), CYAN, WHITE),
                1 => (alpha(0xFF0A2030, 0xF0), alpha(ICE, 0xC0), ICE),
                _ => (
                    alpha(0xFF081420, 0xE0),
                    alpha(MUTED, 0x50),
                    alpha(MUTED, 0xB0),
                ),
            };
            l.chamfer(r, 3.0, fill);
            let texture=match n.state {
                3=>"claude_rpg:gui/ui/panels/ui_tree_node_maxed",
                2=>"claude_rpg:gui/ui/panels/ui_tree_node_owned",
                1=>"claude_rpg:gui/ui/panels/ui_tree_node_available",
                _=>"claude_rpg:gui/ui/panels/ui_tree_node_locked",
            };
            l.sprite(texture,[0.0,0.0,256.0,128.0],r);
            l.chamfer_outline(r, 3.0, 1.0, edge);
            // A coloured spine on the left edge encodes the state at a glance.
            l.rect(
                Rect {
                    x: r.x,
                    y: r.y,
                    w: 2.0,
                    h: r.h,
                },
                edge,
            );
            let max_n: u64 = n.max.parse().unwrap_or(0);
            let ratio = if max_n > 0 {
                (n.rank as f32 / max_n as f32).clamp(0.0, 1.0)
            } else if n.rank > 0 {
                1.0
            } else {
                0.0
            };
            if clamp_zoom(t.zoom) < 0.55 {
                // Overview glyph: the whole tree remains visible without tiny text.
                l.diamond(
                    r.x + r.w * 0.5,
                    r.y + r.h * 0.5,
                    (r.w.min(r.h) * 0.22).clamp(0.6, 2.4),
                    edge,
                );
            } else if clamp_zoom(t.zoom) < ZOOM_COMPACT {
                // Compact plate: a rank diamond and the rank fraction only.
                let cy = r.y + r.h * 0.5;
                l.diamond(r.x + 8.0, cy, 3.0, edge);
                let full = format!("{}/{}", n.rank, n.max);
                let rank = if text_width(&full) <= r.w - 16.0 {
                    full
                } else {
                    fit(&n.rank.to_string(), r.w - 16.0)
                };
                l.text(&rank, r.x + 14.0, cy - 4.0, text);
            } else {
                // Name on the first line; rank and its gauge on the second one.
                let rank = format!("{}/{}", n.rank, n.max);
                let rw = text_width(&rank);
                // Catalog names are "Base — Tier"; a lane repeats the base, so
                // only its first node shows it and the rest show their tier.
                let label = match n.name.split_once(" — ") {
                    Some((_, tier)) if t.links.iter().any(|(_, b)| *b == i) => tier.to_string(),
                    Some((base, _)) => base.to_string(),
                    None => n.name.clone(),
                };
                let name = fit_word(&label.to_uppercase(), r.w - 10.0);
                l.text(&name, r.x + 6.0, r.y + 3.0, text);
                let rank_color = if n.state == 3 {
                    GOLD
                } else {
                    alpha(edge, 0xF0)
                };
                l.text(&rank, r.x + r.w - rw - 4.0, r.y + r.h - 11.0, rank_color);
                let gauge_w = (r.w - rw - 16.0).max(8.0);
                if (1..=6).contains(&max_n) {
                    l.pips(
                        r.x + 7.0,
                        r.y + r.h - 7.0,
                        max_n as u32,
                        n.rank.min(max_n) as u32,
                        edge,
                        5.0,
                    );
                } else {
                    // Large or unbounded ranks: a thin track with a fill.
                    let track = Rect {
                        x: r.x + 6.0,
                        y: r.y + r.h - 8.0,
                        w: gauge_w,
                        h: 2.0,
                    };
                    l.rect(track, alpha(MUTED, 0x40));
                    l.rect(
                        Rect {
                            w: track.w * ratio,
                            ..track
                        },
                        edge,
                    );
                }
            }
            if i == t.selected {
                // Selection reticle: four corner ticks outside the node.
                l.corner_brackets(
                    Rect {
                        x: r.x - 3.0,
                        y: r.y - 3.0,
                        w: r.w + 6.0,
                        h: r.h + 6.0,
                    },
                    5.0,
                    1.5,
                    WHITE,
                );
            }
        }
        l.set_clip(None);
        // Scroll indicators: where the viewport sits inside the whole graph.
        {
            let z = clamp_zoom(t.zoom);
            let size = graph_size(t);
            let (cw, ch) = (size[0] * z, size[1] * z);
            if cw > g.w {
                let w = (g.w * g.w / cw).max(12.0);
                let x = g.x + (-pan[0] / (cw - g.w)).clamp(0.0, 1.0) * (g.w - w);
                l.rect(
                    Rect {
                        x,
                        y: g.y + g.h - 3.0,
                        w,
                        h: 2.0,
                    },
                    alpha(CYAN, 0xA0),
                );
            }
            if ch > g.h {
                let h = (g.h * g.h / ch).max(12.0);
                let y = g.y + (-pan[1] / (ch - g.h)).clamp(0.0, 1.0) * (g.h - h);
                l.rect(
                    Rect {
                        x: g.x + g.w - 3.0,
                        y,
                        w: 2.0,
                        h,
                    },
                    alpha(CYAN, 0xA0),
                );
            }
        }
        // Soft inner shading on the four edges: a node cut by the viewport then
        // reads as "scroll for more" instead of as a drawing bug.
        for i in 0..3 {
            let a = 0x46 - i * 0x14;
            let d = i as f32;
            l.rect(
                Rect {
                    x: g.x,
                    y: g.y + d,
                    w: g.w,
                    h: 1.0,
                },
                alpha(0xFF040C15, a as u8),
            );
            l.rect(
                Rect {
                    x: g.x,
                    y: g.y + g.h - 1.0 - d,
                    w: g.w,
                    h: 1.0,
                },
                alpha(0xFF040C15, a as u8),
            );
            l.rect(
                Rect {
                    x: g.x + d,
                    y: g.y,
                    w: 1.0,
                    h: g.h,
                },
                alpha(0xFF040C15, a as u8),
            );
            l.rect(
                Rect {
                    x: g.x + g.w - 1.0 - d,
                    y: g.y,
                    w: 1.0,
                    h: g.h,
                },
                alpha(0xFF040C15, a as u8),
            );
        }

        if g.w>=90.0 && g.h>=43.0 {tree_minimap(l, g, t, pan);}
        let visible = t
            .nodes
            .iter()
            .filter(|n| intersect(node_rect(g, n, pan, t.zoom), g).is_some())
            .count();
        let visible_label = format!("{visible}/{} VISÍVEIS", t.nodes.len());
        let label_rect = Rect {
            x: g.x + 4.0,
            y: g.y + g.h - 13.0,
            w: text_width(&visible_label) + 8.0,
            h: 11.0,
        };
        l.rect(label_rect, alpha(0xFF030B13, 0xF0));
        l.text(
            &visible_label,
            label_rect.x + 4.0,
            label_rect.y + 1.0,
            alpha(CYAN, 0xC0),
        );
    }

    // Full detail panel; mouse-over previews a node without purchasing it.
    // Selected node remains the authoritative buy target.
    let detail=tree_detail_rect(m);
    l.rect(detail,alpha(0xFF071320,0xFC));
    l.border(detail,alpha(CYAN,0x85));
    l.rect(Rect{x:detail.x,y:detail.y,w:3.0,h:detail.h},CYAN);
    let hovered=if t.index_mode {
        tree_index_layout(m).into_iter().find(|(r,_)|r.contains(m.mouse)).map(|(_,i)|i)
    } else {
        let pan=view_pan(g,t);
        t.nodes.iter().enumerate().find(|(_,n)|
            intersect(node_rect(g,n,pan,t.zoom),g).is_some_and(|r|r.contains(m.mouse)))
            .map(|(i,_)|i)
    };
    let details_for=hovered.unwrap_or(t.selected);
    let target=t.nodes.get(details_for);
    if let Some(n)=target {
        let pad=10.0;
        let inner_w=(detail.w-2.0*pad).max(32.0);
        let compact=detail.h<130.0;
        let mut line_y;
        if compact {
            // At phone GUI sizes, show the *entire* name in the scrollable text,
            // instead of consuming the panel with a title and rank badges.
            l.text(&fit_word("DETALHES · ROLE PARA LER",inner_w),detail.x+pad,detail.y+4.0,CYAN);
            line_y=detail.y+17.0;
        } else {
            l.text(&fit_word("DETALHES DA HABILIDADE",inner_w),detail.x+pad,detail.y+7.0,CYAN);
            line_y=detail.y+22.0;
            for line in wrap(&n.name,inner_w) {
                l.text(&line,detail.x+pad,line_y,WHITE);
                line_y+=11.0;
            }
            let state=match n.state {3=>"MAXIMIZADO",2=>"ADQUIRIDO",1=>"DISPONÍVEL",_=>"BLOQUEADO"};
            l.text(state,detail.x+pad,line_y+2.0,if n.state==0 {RED}else{GREEN});
            line_y+=17.0;
            l.text(&format!("RANK {}/{}   ·   {} PT",n.rank,n.max,n.cost),
                detail.x+pad,line_y,GOLD);
            line_y+=17.0;
            l.rect(Rect{x:detail.x+pad,y:line_y,w:inner_w,h:1.0},alpha(CYAN,0x70));
            line_y+=9.0;
        }
        // Real multiline wrapping; no ellipsis or truncation of the effect.
        let mut desc=if compact {
            let mut v=wrap(&n.name,inner_w);
            v.push(format!("RANK {}/{} · {} PT",n.rank,n.max,n.cost));
            v
        } else {Vec::new()};
        desc.push("EFEITO".to_string());
        desc.extend(wrap(if n.description.trim().is_empty(){"Descrição não fornecida pelo catálogo."}
            else {&n.description},inner_w));
        desc.push("".into());
        desc.push("REQUISITOS".into());
        desc.extend(wrap(&n.requirement,inner_w));
        let visible_h=(detail.y+detail.h-38.0-line_y).max(11.0);
        let max_lines=(visible_h/11.0).floor().max(1.0) as usize;
        let start=if hovered.is_some() {0} else {
            m.detail_scroll.min(desc.len().saturating_sub(max_lines))
        };
        for (i,txt) in desc.iter().skip(start).take(max_lines).enumerate(){
            let yy=line_y+i as f32*11.0;
            if yy+10.0>detail.y+detail.h-36.0 {break;}
            let color=if txt=="EFEITO"||txt=="REQUISITOS" {CYAN}else{ICE};
            l.text(txt,detail.x+pad,yy,color);
        }
        if desc.len()>max_lines {
            let hint=format!("LINHAS {}-{} / {}  ·  ROLE",start+1,(start+max_lines).min(desc.len()),desc.len());
            l.text(&hint,detail.x+pad,detail.y+detail.h-35.0,MUTED);
        }
    }
    let buy=buy_rect(c);
    let enabled=t.nodes.get(t.selected).is_some_and(|n|n.buyable);
    l.rect(buy,if enabled {0xFF134B61}else{0xFF17222D});
    l.border(buy,if enabled {CYAN}else{MUTED});
    let label=if enabled {"ADQUIRIR"} else {"INDISPONÍVEL"};
    l.text(&fit_word(label,buy.w-8.0),buy.x+6.0,buy.y+5.0,
        if enabled {WHITE}else{MUTED});

}

/// The respec control of the attribute page: bottom right of the body.
pub fn respec_rect(c: Rect) -> Rect {
    Rect {
        x: c.x + c.w - 8.0 - 88.0,
        y: c.y + c.h - 18.0,
        w: 88.0,
        h: 14.0,
    }
}

pub fn buy_rect(c: Rect) -> Rect {
    // Same geometry as tree_detail_rect without needing the whole model.
    let right_side=c.w>=500.0 && c.h>=170.0;
    let dx=if right_side {(c.w*0.67).max(265.0)+5.0}else{0.0};
    Rect {x:c.x+dx+6.0,y:c.y+c.h-25.0,
        w:(c.w-dx-12.0).max(30.0),h:19.0}
}

/// Arrow buttons of the tree header: index 0 is previous, 1 is next.
pub fn tree_arrow(head: Rect, index: usize) -> Rect {
    let w = 14.0;
    Rect {
        x: head.x + head.w - (2 - index) as f32 * (w + 2.0) - 4.0,
        y: head.y + 2.0,
        w,
        h: 12.0,
    }
}

// ------------------------------------------------------------------ hit test

pub fn menu_hits(m: &MenuModel) -> Vec<(Rect, String)> {
    let win = window(m.area);
    let mut out = Vec::new();
    let n = m.pages.len();
    let group = tab_group(m.page.min(n.saturating_sub(1)));
    for (i, r) in tab_rects(win, n, m.page).into_iter().enumerate() {
        out.push((r, format!("page:{}", group * TAB_SLOTS + i)));
    }
    if tab_groups(n) > 1 {
        out.push((tab_arrow(win, 0), "tgroup:-1".to_string()));
        out.push((tab_arrow(win, 1), "tgroup:1".to_string()));
    }
    let c = content(win, m.pages.len());
    if !matches!(
        m.kind,
        "trees"
            | "bestiary"
            | "status"
            | "attributes"
            | "overview"
            | "equipment"
            | "accessories"
            | "inventory"
    ) && data_pages(c, &m.lines) > 1
    {
        out.push((data_arrow(c, 0).inset(-4.0), "rpage:-1".to_string()));
        out.push((data_arrow(c, 1).inset(-4.0), "rpage:1".to_string()));
    }
    match m.kind {
        "inventory" => {
            let rows=gear_rows_per_page(c);
            let pages=m.bag.len().div_ceil(rows).max(1);
            if pages > 1 {
                out.push((inventory_next_rect(c,true),"rpage:-1".into()));
                out.push((inventory_next_rect(c,false),"rpage:1".into()));
            }
            for (i,g) in m.bag.iter().skip(m.scroll.min(pages-1)*rows).take(rows).enumerate() {
                let row=inventory_row_rect(c,i);
                if row.y+row.h>c.y+c.h {break;}
                out.push((inventory_equip_rect(row),format!("equip:{}",g.id)));
                out.push((inventory_salvage_rect(row),format!("salvage:{}",g.id)));
            }
        }
        "equipment" | "accessories" => {
            let acc=m.kind=="accessories";
            let slots: &[&str] = if acc { &["ring","neck","charm"] } else {
                &["head","chest","legs","feet","weapon","ring","neck","charm"]
            };
            if equipped_page_count(c,slots.len())>1 {
                out.push((equipped_page_arrow(c,false),"rpage:-1".into()));
                out.push((equipped_page_arrow(c,true),"rpage:1".into()));
            }
            for (idx,r) in equipped_visible_rects(c,slots.len(),m.scroll) {
                let slot=slots[idx];
                if m.worn.iter().any(|g|g.slot==slot) {
                    // Distinct clickable controls, avoiding accidental unequip.
                    out.push((equipped_activate_rect(r),format!("activate:{slot}")));
                    out.push((equipped_remove_rect(r),format!("unequip:{slot}")));
                }
            }
        }
        "trees" => {
            out.push((constellation_tree_arrow(c,0),"tree:-1".into()));
            out.push((constellation_tree_arrow(c,1),"tree:1".into()));
            out.push((constellation_catalog_rect(c),"catalog".into()));
            if m.tree.as_ref().is_some_and(|t|t.index_mode) {
                for (r,i) in constellation_categories(m) {
                    out.push((r,format!("treecat:{i}")));
                }
            } else {
                for (r,i) in constellation_node_rects(m) {
                    out.push((r,format!("node:{i}")));
                }
                out.push((constellation_page_arrow(m,false),"zoom:-1".into()));
                out.push((constellation_page_arrow(m,true),"zoom:1".into()));
                out.push((constellation_fit_rect(m),"fit".into()));
            }
            if !m.external_inspector && !m.tree.as_ref().is_some_and(|t|t.index_mode) {
                out.push((constellation_buy_rect(m),"buy".into()));
            }
            if m.tree.as_ref().is_some_and(|t|t.index_mode) && constellation_pages(m)>1 {
                out.push((constellation_page_arrow(m,false),"rpage:-1".into()));
                out.push((constellation_page_arrow(m,true),"rpage:1".into()));
            }
        }
        "bestiary" => {
            let (left, _) = split(c);
            let rows = bestiary_rows(left);
            let start = m.bestiary_sel.saturating_sub(rows.saturating_sub(1));
            let y = left.y + 22.0;
            for i in 0..m.bestiary.len().saturating_sub(start).min(rows) {
                out.push((
                    Rect {
                        x: left.x + 6.0,
                        y: y + i as f32 * 16.0 - 2.0,
                        w: left.w - 12.0,
                        h: 15.0,
                    },
                    format!("species:{}", start + i),
                ));
            }
        }
        "attributes" => {
            out.push((respec_rect(c), "respec".to_string()));
        }
        _ => {}
    }
    out
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 960.0,
            h: 480.0,
        }
    }

    fn model(kind: &'static str) -> MenuModel {
        MenuModel {
            area: area(),
            pages: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            kind,
            title: "STATUS".into(),
            level: 12,
            rank: "B".into(),
            xp: 0.4,
            points: 2,
            ..Default::default()
        }
    }

    #[test]
    fn tabs_never_overlap_and_stay_inside_the_window() {
        for n in 1..=16usize {
            let win = window(area());
            for page in 0..n {
                let rects = tab_rects(win, n, page);
                assert!(rects.len() <= TAB_SLOTS);
                assert!(!rects.is_empty());
                for (i, a) in rects.iter().enumerate() {
                    assert!(a.x >= win.x && a.x + a.w <= win.x + win.w + 0.01);
                    assert!(a.y >= win.y && a.y + a.h <= win.y + win.h);
                    // The strip must not run under the group arrows.
                    assert!(a.x >= tab_arrow(win, 0).x + tab_arrow(win, 0).w);
                    assert!(a.x + a.w <= tab_arrow(win, 1).x + 0.01);
                    for b in rects.iter().skip(i + 1) {
                        let overlap = a.x < b.x + b.w
                            && b.x < a.x + a.w
                            && a.y < b.y + b.h
                            && b.y < a.y + a.h;
                        assert!(!overlap, "abas {i} e {n} sobrepostas");
                    }
                }
            }
        }
    }

    #[test]
    fn content_sits_below_every_tab_row() {
        let win = window(area());
        for n in [3usize, 5, 10, 15, 16] {
            let c = content(win, n);
            let last = tab_rects(win, n, 0)
                .into_iter()
                .map(|r| r.y + r.h)
                .fold(0.0, f32::max);
            assert!(c.y > last, "conteúdo sobre as abas ({n} abas)");
            assert!(c.y + c.h < win.y + win.h, "conteúdo passa do rodapé");
        }
    }

    #[test]
    fn compact_skill_tree_always_keeps_graph_and_inspector_visible() {
        let mut m=model("trees");m.area=phone();
        let mut tree=big_tree(30);tree.detail_only=false;
        m.tree=Some(tree);
        for detail_only in [false,true] {
            m.tree.as_mut().unwrap().detail_only=detail_only;
            let (graph,detail)=constellation_regions(&m);
            assert!(graph.w>150.0 && graph.h>=25.0);
            assert!(detail.w>150.0 && detail.h>=25.0);
            assert!(graph.y+graph.h<=detail.y);
            assert!(menu_hits(&m).iter().any(|(_,id)|id=="buy"));
            assert!(!menu_hits(&m).iter().any(|(_,id)|id=="details"));
        }
    }

    #[test]
    fn every_page_draws_without_panicking() {
        for kind in [
            "overview",
            "status",
            "attributes",
            "trees",
            "bestiary",
            "quests",
            "daily",
            "rows",
        ] {
            let mut m = model(kind);
            m.attrs = vec![AttrRow {
                name: "Força".into(),
                value: "10".into(),
                ratio: 0.5,
                hint: "+1".into(),
            }];
            m.lines = vec!["linha".into()];
            m.bestiary = vec![SpeciesRow {
                name: "zombie".into(),
                kills: 3,
                level: 4,
                best: 12.5,
                tiers: "Normal".into(),
                drops: "osso".into(),
                discovery: 0.5,
            }];
            m.tree = Some(TreeModel {
                name: "Vitality".into(),
                nodes: vec![TreeNode {
                    id: "a".into(),
                    name: "A".into(),
                    rank: 1,
                    max: "3".into(),
                    cost: 1,
                    col: 0,
                    row: 0,
                    state: 2,
                    buyable: false,
                    description:"Teste: habilidade de combate".into(),
                    requirement:"Nenhum".into(),
                }],
                links: vec![],
                pan: [0.0, 0.0],
                zoom: 1.0,
                selected: 0,
                index_mode: false,
                detail_only: false,
                detail: vec!["d".into()],
            });
            let layer = draw_menu(&m);
            assert!(!layer.prims.is_empty(), "{kind} não desenhou nada");
        }
    }

    #[test]
    fn tree_nodes_are_centred_in_the_viewport() {
        let mut m = model("trees");
        m.tree = Some(TreeModel {
            name: "V".into(),
            nodes: (0..4)
                .map(|col| TreeNode {
                    id: col.to_string(),
                    name: "N".into(),
                    rank: 0,
                    max: "3".into(),
                    cost: 1,
                    col,
                    row: col % 2,
                    state: 1,
                    buyable: false,
                    description:"Teste: habilidade de combate".into(),
                    requirement:"Nenhum".into(),
                })
                .collect(),
            links: vec![],
            pan: [0.0, 0.0],
            zoom: 1.0,
            selected: 0,
            index_mode: false,
            detail_only: false,
            detail: vec![],
        });
        let g = tree_graph_rect(&m);
        let t = m.tree.as_ref().unwrap();
        let pan = view_pan(g, t);
        let xs: Vec<f32> = t
            .nodes
            .iter()
            .map(|n| node_rect(g, n, pan, t.zoom).x)
            .collect();
        let left = xs.iter().cloned().fold(f32::MAX, f32::min);
        let right = xs.iter().map(|x| x + NODE_W).fold(f32::MIN, f32::max);
        let mid = (left + right) * 0.5;
        assert!(
            (mid - (g.x + g.w * 0.5)).abs() < 1.0,
            "grafo fora do centro: {mid} vs {}",
            g.x + g.w * 0.5
        );
    }

    #[test]
    fn hits_reproduce_the_drawn_rects() {
        let mut m=model("trees");
        m.tree=Some(big_tree(141));
        let (rect,index)=constellation_node_rects(&m)[0];
        let (hit,action)=menu_hits(&m).into_iter()
            .find(|(_,a)|*a==format!("node:{index}")).expect("nó sem toque");
        assert_eq!(hit,rect);
        assert_eq!(action,format!("node:{index}"));
        assert!(menu_hits(&m).iter().any(|(_,a)|a=="buy"));
    }

    #[test]
    fn bestiary_hits_line_up_with_the_list_rows() {
        let mut m = model("bestiary");
        m.bestiary = (0..8)
            .map(|i| SpeciesRow {
                name: format!("m{i}"),
                kills: i as u64,
                ..Default::default()
            })
            .collect();
        m.bestiary_sel = 0;
        let hits: Vec<_> = menu_hits(&m)
            .into_iter()
            .filter(|(_, a)| a.starts_with("species:"))
            .collect();
        assert!(!hits.is_empty());
        for (rect, action) in hits {
            let i: usize = action.split(':').nth(1).unwrap().parse().unwrap();
            assert!(i < m.bestiary.len());
            assert!(rect.h > 10.0 && rect.w > 40.0);
        }
    }

    #[test]
    fn the_tab_strip_shows_one_group_of_five_at_a_time() {
        let win = window(Rect {
            x: 0.0,
            y: 0.0,
            w: 380.0,
            h: 240.0,
        });
        // 15 pages: three groups, five tabs each, arrows on both sides.
        assert_eq!(tab_groups(15), 3);
        assert_eq!(tab_rects(win, 15, 0).len(), 5);
        assert_eq!(tab_rects(win, 15, 7).len(), 5);
        assert_eq!(tab_rects(win, 15, 14).len(), 5);
        assert_eq!(tab_group(7), 1);
        // The last group may be partial.
        assert_eq!(tab_rects(win, 13, 12).len(), 3);
        // A group change lands on the first page of the next group.
        assert_eq!((tab_group(4) + 1) * TAB_SLOTS, 5);
    }

    #[test]
    fn the_body_gets_most_of_a_phone_screen() {
        let win = window(Rect {
            x: 0.0,
            y: 0.0,
            w: 380.0,
            h: 240.0,
        });
        let c = content(win, 15);
        assert!(c.h >= 140.0, "corpo com apenas {} px de altura", c.h);
    }

    pub fn big_tree_pub(n: usize) -> TreeModel {
        big_tree(n)
    }

    fn big_tree(n: usize) -> TreeModel {
        // Chains of three: 0 <- 1 <- 2, 3 <- 4 <- 5, ...
        let parents: Vec<Vec<usize>> = (0..n)
            .map(|i| if i % 3 == 0 { vec![] } else { vec![i - 1] })
            .collect();
        let cells = layout_tree(&parents);
        TreeModel {
            name: "Mining".into(),
            nodes: cells
                .iter()
                .enumerate()
                .map(|(i, &(col, row))| TreeNode {
                    id: i.to_string(),
                    name: format!("Nó {i}"),
                    rank: 0,
                    max: "100".into(),
                    cost: 1,
                    col,
                    row,
                    state: 1,
                    buyable: false,
                    description:"Teste: habilidade de combate".into(),
                    requirement:"Nenhum".into(),
                })
                .collect(),
            links: (0..n).filter(|i| i % 3 != 0).map(|i| (i - 1, i)).collect(),
            pan: [0.0, 0.0],
            zoom: 1.0,
            selected: 0,
            index_mode: false,
            detail_only: false,
            detail: vec![],
        }
    }

    #[test]
    fn layout_gives_every_node_a_unique_cell() {
        let t = big_tree(141);
        let mut cells: Vec<_> = t.nodes.iter().map(|n| (n.col, n.row)).collect();
        cells.sort();
        cells.dedup();
        assert_eq!(cells.len(), 141, "dois nós no mesmo lugar");
        // A chain stays on one row, one column per prerequisite step.
        assert_eq!(t.nodes[0].row, t.nodes[2].row);
        assert_eq!(t.nodes[1].col, t.nodes[0].col + 1);
    }

    #[test]
    fn layout_survives_cycles_and_bad_indices() {
        let cells = layout_tree(&[vec![1], vec![0], vec![99], vec![]]);
        assert_eq!(cells.len(), 4);
        assert!(layout_tree(&[]).is_empty());
    }

    #[test]
    fn graph_has_one_continuous_page_and_hit_geometry_matches_render() {
        for size in [(380.0,240.0),(960.0,480.0)] {
            let mut m=model("trees");
            m.area=Rect{x:0.0,y:0.0,w:size.0,h:size.1};
            m.tree=Some(big_tree(141));
            assert_eq!(constellation_pages(&m),1);
            assert!(!constellation_node_rects(&m).is_empty());
            for (rect,i) in constellation_node_rects(&m) {
                assert!(menu_hits(&m).iter().any(|(hit,a)|
                    *hit==rect && *a==format!("node:{i}")));
            }
            assert!(menu_hits(&m).iter().any(|(_,a)|a=="fit"));
            assert!(menu_hits(&m).iter().any(|(_,a)|a=="zoom:1"));
        }
    }

    #[test]
    fn pan_is_bounded_by_the_real_graph_size() {
        let m = model("trees");
        let t = big_tree(141);
        let g = tree_graph_rect(&m);
        let far = clamp_pan(g, &t, [-1.0e9, -1.0e9]);
        let size = graph_size(&t);
        assert!((far[0] - (g.w - size[0])).abs() < 0.01);
        assert!((far[1] - (g.h - size[1])).abs() < 0.01);
        assert_eq!(clamp_pan(g, &t, [1.0e9, 1.0e9]), [0.0, 0.0]);
        let nan = clamp_pan(g, &t, [f32::NAN, f32::INFINITY]);
        assert!(nan[0].is_finite() && nan[1].is_finite());
    }

    #[test]
    fn zoom_keeps_the_viewport_centre_and_stays_in_range() {
        let m = model("trees");
        let mut t = big_tree(141);
        let g = tree_graph_rect(&m);
        t.pan = [-200.0, -150.0];
        let p0 = view_pan(g, &t);
        let centre = [(g.w * 0.5 - p0[0]) / t.zoom, (g.h * 0.5 - p0[1]) / t.zoom];
        let p1 = zoom_pan(g, &t, 1.3);
        let back = [(g.w * 0.5 - p1[0]) / 1.3, (g.h * 0.5 - p1[1]) / 1.3];
        assert!((centre[0] - back[0]).abs() < 0.5 && (centre[1] - back[1]).abs() < 0.5);
        assert_eq!(clamp_zoom(10.0), ZOOM_MAX);
        assert_eq!(clamp_zoom(f32::NAN), 1.0);
    }

    #[test]
    fn tree_hits_never_leave_the_graph_viewport() {
        let mut m=model("trees");
        m.area=phone();
        m.tree=Some(big_tree(141));
        let (g,_) =constellation_regions(&m);
        for page in 0..constellation_pages(&m) {
            m.scroll=page;
            for (r,a) in menu_hits(&m) {
                if a.starts_with("node:") {
                    assert!(r.x>=g.x && r.y>=g.y);
                    assert!(r.x+r.w<=g.x+g.w+0.01);
                    assert!(r.y+r.h<=g.y+g.h+0.01);
                }
            }
        }
    }

    fn phone() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: 380.0,
            h: 240.0,
        }
    }

    #[test]
    fn no_text_leaves_the_window_on_any_page_at_380x240() {
        use super::super::crpg_ui_system::Prim;
        let long = "Experiência geral — Transcendência suprema de mineração: nível 999 · XP 123456/999999 · pontos 18446744073709551615 · ação àéíóúãõç";
        let kinds = [
            "overview",
            "char",
            "status",
            "attributes",
            "trees",
            "profs",
            "equipment",
            "accessories",
            "enchant",
            "skills",
            "bestiary",
            "evil",
            "quests",
            "achievements",
            "history",
        ];
        for kind in kinds {
            let mut m = model(kind);
            m.area = phone();
            m.pages = kinds.iter().map(|k| k.to_string()).collect();
            m.page = kinds.iter().position(|k| *k == kind).unwrap_or(0);
            m.title = "Encantamentos de transcendência".into();
            m.lines = (0..40).map(|i| format!("{long} {i}")).collect();
            m.next_rank = "S".into();
            m.vitals = vec![Vital {
                label: "HP".into(),
                ratio: f32::NAN,
                value: "99999999/99999999".into(),
            }];
            m.bestiary = vec![SpeciesRow {
                name: "minecraft:wither_skeleton_supremo_de_teste".into(),
                drops: long.into(),
                tiers: "Boss, Elite".into(),
                ..Default::default()
            }];
            let mut t = big_tree_pub(60);
            t.nodes[0].name = long.into();
            m.tree = Some(t);
            m.toast = Some(long.into());
            let win = window(m.area);
            for p in draw_menu(&m).prims {
                if let Prim::Text { text, x, .. } = p {
                    let right = x + text_width(&text);
                    assert!(
                        right <= win.x + win.w + 0.5 && x >= win.x - 0.5,
                        "{kind}: texto fora da janela: {text:?} termina em {right}"
                    );
                }
            }
        }
    }

    #[test]
    fn fit_tracked_respects_the_width_with_accents() {
        for track in [0.0f32, 0.6, 1.0] {
            let s = fit_tracked("CONFIGURAÇÃO DE MINERAÇÃO ÀÉÍÓÚ", 80.0, track);
            assert!(tracked_width(&s, track) <= 80.0, "{s} com tracking {track}");
            assert!(s.ends_with('…'));
        }
        assert_eq!(fit_tracked("OK", 80.0, 1.0), "OK");
        assert_eq!(tracked_width("", 1.0), 0.0);
    }

    #[test]
    fn data_pages_paginate_and_pager_hits_match_the_drawing() {
        let mut m = model("enchant");
        m.area = phone();
        m.lines = (0..30)
            .map(|i| format!("Linha {i}: valor {i}/30"))
            .collect();
        let c = content(window(m.area), m.pages.len());
        let layout = data_layout(c, &m.lines);
        // Every line is placed exactly once, inside the body.
        let mut seen: Vec<usize> = layout.iter().map(|p| p.2).collect();
        seen.sort();
        assert_eq!(seen, (0..30).collect::<Vec<_>>());
        let body = data_body(c);
        for (_, r, _) in &layout {
            assert!(r.y >= body.y && r.y + r.h <= body.y + body.h + 0.01);
        }
        assert!(data_pages(c, &m.lines) > 1);
        let hits = menu_hits(&m);
        let next = hits.iter().find(|(_, a)| a == "rpage:1").expect("sem seta");
        assert!(next
            .0
            .contains([data_arrow(c, 1).x + 2.0, data_arrow(c, 1).y + 2.0]));
        assert_eq!(parse_entry("Abates totais: 189").key, "Abates totais");
        assert_eq!(first_ratio("XP 25/100"), Some(0.25));
        assert_eq!(first_ratio("0/0"), None);
    }

    #[test]
    fn fit_view_shows_all_141_nodes_on_a_phone() {
        let mut m = model("trees");
        m.area = phone();
        let mut t = big_tree(141);
        let g = tree_graph_rect(&m);
        t.zoom = fit_zoom(g, &t);
        let pan = view_pan(g, &t);
        for (i, n) in t.nodes.iter().enumerate() {
            let r = node_rect(g, n, pan, t.zoom);
            assert!(
                r.x >= g.x - 0.01
                    && r.y >= g.y - 0.01
                    && r.x + r.w <= g.x + g.w + 0.01
                    && r.y + r.h <= g.y + g.h + 0.01,
                "nó {i} fora da visão ajustada: {r:?} em {g:?}"
            );
        }
    }

    #[test]
    fn category_browser_lists_every_category_exactly_once() {
        let mut m=model("trees");
        m.area=phone();
        let mut t=big_tree(141);
        t.index_mode=true;
        m.tree=Some(t);
        let total=super::super::crpg_nodes::trees().len();
        let mut seen=Vec::new();
        for page in 0..constellation_pages(&m) {
            m.scroll=page;
            for (r,i) in constellation_categories(&m) {
                assert!(menu_hits(&m).iter().any(|(hit,a)|
                    *hit==r && *a==format!("treecat:{i}")));
                seen.push(i);
            }
            assert!(constellation_node_rects(&m).is_empty());
        }
        seen.sort_unstable();
        assert_eq!(seen,(0..total).collect::<Vec<_>>());
    }

    #[test]
    fn constellation_has_separate_category_navigation_and_page_hits() {
        let mut m=model("trees");
        m.area=phone();
        m.tree=Some(big_tree(141));
        let hits=menu_hits(&m);
        for action in ["tree:-1","tree:1","catalog","rpage:1","node:0","buy"] {
            assert!(hits.iter().any(|(_,a)|a==action),"controle ausente: {action}");
        }
        let a=constellation_tree_arrow(content(window(m.area),m.pages.len()),1);
        let cat=constellation_catalog_rect(content(window(m.area),m.pages.len()));
        assert!(a.x+a.w<cat.x,"botões de árvore e biblioteca sobrepostos");
    }

    #[test]
    fn minimap_stays_inside_the_graph_viewport() {
        let mut m = model("trees");
        m.area = phone();
        let g = tree_graph_rect(&m);
        let r = minimap_rect(g);
        assert!(r.x >= g.x && r.y >= g.y);
        assert!(r.x + r.w <= g.x + g.w && r.y + r.h <= g.y + g.h);
    }

    #[test]
    fn all_36_real_skill_trees_expose_every_node_without_zoom() {
        use super::super::crpg_nodes as catalog;
        let names=catalog::trees();
        assert_eq!(names.len(),36);
        let mut total=0;
        for name in names {
            let defs=catalog::in_tree(&name);
            total+=defs.len();
            let index:std::collections::HashMap<&str,usize>=defs.iter()
                .enumerate().map(|(i,n)|(n.id.as_str(),i)).collect();
            let mut links=Vec::new();
            for (i,n) in defs.iter().enumerate() {
                for (p,_) in &n.prereqs {
                    if let Some(&j)=index.get(p.as_str()) {links.push((j,i));}
                }
            }
            let mut m=model("trees");
            m.area=phone();
            m.tree=Some(TreeModel {
                name:name.clone(),
                nodes:defs.iter().map(|n|TreeNode {
                    id:n.id.clone(),name:n.name.clone(),rank:0,max:n.max_label(),
                    cost:1,col:0,row:0,state:1,buyable:false,
                    description:n.description.clone(),requirement:"Nenhum".into(),
                }).collect(),
                links,pan:[0.0,0.0],zoom:1.0,selected:0,
                index_mode:false,detail_only:false,detail:vec![],
            });
            let mut seen=Vec::new();
            for page in 0..constellation_pages(&m) {
                m.scroll=page;
                seen.extend(constellation_node_rects(&m).iter().map(|(_,i)|*i));
            }
            seen.sort_unstable();
            assert_eq!(seen,(0..defs.len()).collect::<Vec<_>>(),"árvore {name}");
        }
        assert!(total>=1300);
    }

    #[test]
    fn hud_survives_nan_infinite_and_empty_server_data() {
        use super::super::crpg_ui_system::{draw_hud, FloatModel, HudModel, Prim, TargetModel};
        let mut m = HudModel {
            width: 380.0,
            height: 240.0,
            ..Default::default()
        };
        m.hp.ratio = f32::NAN;
        m.hp.ghost = f32::INFINITY;
        m.stamina.ratio = f32::NEG_INFINITY;
        m.shield = f32::NAN;
        m.air = Some(f32::NAN);
        m.xp = f32::NAN;
        m.target = Some(TargetModel {
            ratio: f32::NAN,
            ghost: f32::NAN,
            ..Default::default()
        });
        m.floats = vec![FloatModel {
            x: f32::NAN,
            y: f32::INFINITY,
            alpha: f32::NAN,
            ..Default::default()
        }];
        let l = draw_hud(&m);
        for p in l.prims {
            if let Prim::Rect { x, y, w, h, .. } = p {
                assert!(
                    x.is_finite() && y.is_finite() && w.is_finite() && h.is_finite(),
                    "retângulo NaN no HUD"
                );
            }
        }
    }
}

#[cfg(test)]
mod full_mobile_equipment_tests {
    use super::*;
    #[test] fn tiny_screen_keeps_all_equipment_accessible() {
        let c=Rect{x:2.0,y:9.0,w:235.0,h:100.0};
        let pages=equipped_page_count(c,8);
        let mut indices=Vec::new();
        for page in 0..pages {
            for (i,r) in equipped_visible_rects(c,8,page) {
                assert!(r.x>=c.x && r.x+r.w<=c.x+c.w+0.01);
                assert!(r.y>=c.y && r.y+r.h<=c.y+c.h+0.01);
                indices.push(i);
            }
        }
        assert_eq!(indices,(0..8).collect::<Vec<_>>());
    }
    #[test] fn buttons_do_not_overlap() {
        for r in equipped_rects(Rect{x:0.0,y:0.0,w:500.0,h:300.0},8) {
            let a=equipped_activate_rect(r);let b=equipped_remove_rect(r);
            assert!(a.y+a.h<=b.y);
            assert!(a.x>=r.x && a.x+a.w<=r.x+r.w);
        }
    }
}
