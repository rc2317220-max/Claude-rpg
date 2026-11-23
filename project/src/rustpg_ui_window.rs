// RustPG system window. Drawing and hit-testing share the same responsive
// geometry, so every visible touch target is the target the input layer uses.
// Visual language is owned by rustpg_ui_theme rather than by a reference mod.

pub use super::rustpg_ui_layout::Rect;
use super::rustpg_ui_system::{
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
    pub available_points: u64,
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
    /// Registry identifier; always used for engine lookups.
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub live_entity: Option<u32>,
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
    pub live_target: Option<super::rustpg_target_hud::TargetSnapshot>,
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
pub const ZOOM_MIN: f32 = 0.50; // Keep even overview nodes large enough to tap
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
pub fn zoom_pan_at(g: Rect, t: &TreeModel, new_zoom: f32, anchor:[f32;2]) -> [f32; 2] {
    let old = view_pan(g, t);
    let z0 = clamp_zoom(t.zoom);
    let z1 = clamp_zoom(new_zoom);
    let cx = anchor[0].clamp(0.0,g.w);
    let cy = anchor[1].clamp(0.0,g.h);
    let pan = [cx - (cx - old[0]) / z0 * z1, cy - (cy - old[1]) / z0 * z1];
    let mut t2 = t.clone();
    t2.zoom = z1;
    clamp_pan(g, &t2, pan)
}
pub fn zoom_pan(g: Rect, t: &TreeModel, new_zoom: f32) -> [f32; 2] {
    zoom_pan_at(g,t,new_zoom,[g.w*0.5,g.h*0.5])
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
        "inventory" => inventory_page(&mut l, c, m),
        "equipment" | "accessories" => equipment_page(&mut l, c, m),
        _ => data_page(&mut l, c, m),
    }
    footer(&mut l, win, m);
    l
}

fn backdrop(l:&mut Layer,a:Rect,_tick:u32) {
    // Preserve the world as context. The overlay only lowers contrast enough
    // for text; no full-screen brackets, scanlines or decorative grid.
    l.rect(a,0x5C030508);
}

/// Compact translucent plate: content gets priority over decoration.
fn plate(l:&mut Layer,win:Rect,_m:&MenuModel) {
    l.chamfer(win,5.0,super::rustpg_ui_theme::INK);
    l.chamfer_outline(win,5.0,1.0,alpha(super::rustpg_ui_theme::LINE,0x78));
    l.rect(Rect{x:win.x+8.0,y:win.y,w:(win.w*0.20).clamp(28.0,72.0),h:1.5},
        alpha(CYAN,0xC0));
}

fn chrome(l:&mut Layer,win:Rect,m:&MenuModel) {
    let h=header(win);
    l.text_tracked("RUSTPG",h.x,h.y+1.0,alpha(CYAN,0xD8),0.9);
    l.sprite("rustpg:gui/branding/rustpg_logo",[0.0,0.0,384.0,110.0],
        Rect{x:h.x+50.0,y:h.y-4.0,w:56.0,h:16.0});
    let right=format!("NV {} · {}",m.level.max(1),if m.rank.is_empty(){"E"}else{m.rank.as_str()});
    l.text(&right,h.x+h.w-text_width(&right),h.y+1.0,alpha(GOLD,0xC0));

    let plate=title_rect(win);
    let title=m.pages.get(m.page).cloned().unwrap_or_else(||m.title.clone());
    l.title_plate(plate,&fit_word(&title.to_uppercase(),plate.w-16.0),CYAN);
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
        "trees" => "TOQUE EM UM NÓ · ROLE PARA TROCAR DE RAMO",
        "bestiary" => "TOQUE NUMA ESPÉCIE",
        "attributes" => "PONTOS VÃO PARA A ÁRVORE",
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
            "RUSTPG",
            win.x + win.w - text_width("RUSTPG") - 10.0,
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
const MENU_XP_HEIGHT: f32 = 6.0;

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
        h: MENU_XP_HEIGHT,
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
            h: MENU_XP_HEIGHT * super::rustpg_hud_geometry::VITAL_BAR_RATIO,
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
        0=>"rustpg:gui/icons/rarities/rarity_common_frame",
        1=>"rustpg:gui/icons/rarities/rarity_uncommon_frame",
        2=>"rustpg:gui/icons/rarities/rarity_rare_frame",
        3=>"rustpg:gui/icons/rarities/rarity_epic_frame",
        4=>"rustpg:gui/icons/rarities/rarity_legendary_frame",
        5=>"rustpg:gui/icons/rarities/rarity_mythic_frame",
        _=>"rustpg:gui/icons/rarities/rarity_divine_frame",
    }
}
fn slot_frame(slot:&str,occupied:bool)->&'static str {
    match (slot,occupied) {
        ("weapon",false)=>"rustpg:gui/ui/slots/ui_slot_weapon_empty",
        ("weapon",true)=>"rustpg:gui/ui/slots/ui_slot_weapon_occupied",
        ("ring",false)=>"rustpg:gui/ui/slots/ui_slot_ring_empty",
        ("ring",true)=>"rustpg:gui/ui/slots/ui_slot_ring_occupied",
        ("neck",false)=>"rustpg:gui/ui/slots/ui_slot_amulet_empty",
        ("neck",true)=>"rustpg:gui/ui/slots/ui_slot_amulet_occupied",
        ("charm",false)=>"rustpg:gui/ui/slots/ui_slot_relic_empty",
        ("charm",true)=>"rustpg:gui/ui/slots/ui_slot_relic_occupied",
        (_,false)=>"rustpg:gui/ui/slots/ui_slot_armor_empty",
        (_,true)=>"rustpg:gui/ui/slots/ui_slot_armor_occupied",
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
    Rect {x:row.x+row.w-93.0,y:row.y+2.0,w:48.0,h:20.0}
}
pub fn inventory_salvage_rect(row: Rect) -> Rect {
    Rect {x:row.x+row.w-41.0,y:row.y+2.0,w:34.0,h:20.0}
}

pub fn inventory_next_rect(c: Rect, prev: bool) -> Rect {
    Rect {x: if prev {c.x+c.w-58.0} else {c.x+c.w-28.0}, y:c.y+5.0, w:22.0,h:16.0}
}
/// Botões pequenos com acções distintas: nunca equipar/desequipar ao tentar ativar.
pub fn equipped_activate_rect(row:Rect)->Rect {
    Rect{x:row.x+row.w-72.0,y:row.y+4.0,w:65.0,h:15.0}
}
pub fn equipped_remove_rect(row:Rect)->Rect {
    Rect{x:row.x+row.w-72.0,y:row.y+21.0,w:65.0,h:15.0}
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
    for (prev,txt,active) in [(true,"‹",page>0),(false,"›",page+1<pages)] {
        let r=inventory_next_rect(c,prev);
        l.rect(r,alpha(0xFF0C2235,0xC0));
        l.border(r,alpha(if active{CYAN}else{MUTED},if active{0x88}else{0x30}));
        l.text_middle(txt,r,if active{CYAN}else{alpha(MUTED,0x40)});
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
    // The inspector already displays the full description on touch/hover.
    // Avoid an opaque duplicate tooltip covering the real inventory buttons.
    if !m.external_inspector {
        if let Some(g)=m.bag.iter().skip(page*rows).take(rows).enumerate()
            .find(|(i,_)|inventory_row_rect(c,*i).contains(m.mouse))
            .map(|(_,g)|g) {
            relic_detail(l,c,g);
        }
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
        for (next,title,active) in [(false,"‹",page>0),(true,"›",page+1<pages)] {
            let ar=equipped_page_arrow(c,next);
            l.rect(ar,0xFF122B3D);
            l.border(ar,alpha(if active{CYAN}else{MUTED},if active{0xB0}else{0x30}));
            l.text_middle(title,ar,if active{CYAN}else{alpha(MUTED,0x40)});
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
    if !m.external_inspector {
        if let Some(g)=rects.iter().find_map(|(i,r)| {
            let (slot,_)=slots[*i];
            if r.contains(m.mouse) {m.worn.iter().find(|g|g.slot==slot)}else{None}
        }) {relic_detail(l,c,g);}
    }
}



/// Rows of the bestiary list that fit in `left`.
fn bestiary_rows(left: Rect) -> usize {
    (((left.h - 44.0) / 16.0).floor() as usize).max(1)
}

fn bestiary_page_buttons(left: Rect, selected: usize, count: usize) -> Vec<(Rect, usize, bool)> {
    let rows = bestiary_rows(left);
    let start = selected / rows * rows;
    [false, true].into_iter().filter_map(|next| {
        let target = if next {start.checked_add(rows)?} else {start.checked_sub(rows)?};
        if target >= count {return None;}
        let x = if next {left.x + left.w - 25.0} else {left.x + 5.0};
        Some((Rect{x, y:left.y + left.h - 18.0, w:20.0, h:16.0}, target, next))
    }).collect()
}

pub fn bestiary_portrait(m: &MenuModel) -> Rect {
    let c=content(window(m.area),m.pages.len());
    super::rustpg_ui_layout::bestiary(c).viewport
}

fn bestiary_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    let layout=super::rustpg_ui_layout::bestiary(c);
    let left=layout.list;
    let right=layout.details;

    l.card(left,CYAN);
    let total_kills:u64=m.bestiary.iter().map(|s|s.kills).sum();
    let head=section(l,Rect{x:left.x+8.0,y:left.y+6.0,w:left.w-16.0,h:12.0},
        "Bestiário",CYAN);
    l.text(&fit(&format!("{} ESPÉCIES · {} ABATES",m.bestiary.len(),total_kills),
        left.w-16.0),left.x+8.0,head,alpha(MUTED,0xC0));

    let rows=bestiary_rows(left);
    let start=(m.bestiary_sel/rows)*rows;
    let mut y=head+14.0;
    for (i,species) in m.bestiary.iter().enumerate().skip(start).take(rows) {
        if y+15.0>left.y+left.h-20.0 {break;}
        let row=Rect{x:left.x+6.0,y,w:left.w-12.0,h:15.0};
        let selected=i==m.bestiary_sel;
        let hover=row.contains(m.mouse);
        let state=if selected {super::rustpg_ui_theme::UiState::Selected}
            else if hover {super::rustpg_ui_theme::UiState::Hover}
            else {super::rustpg_ui_theme::UiState::Normal};
        l.chamfer(row,2.0,super::rustpg_ui_theme::state_fill(state));
        if selected||hover {
            l.chamfer_outline(row,2.0,1.0,alpha(super::rustpg_ui_theme::state_edge(state),
                if selected{0xA8}else{0x58}));
        }
        let discovery=unit(species.discovery);
        let dot=if discovery>=0.85{GOLD}else if discovery>=0.45{GREEN}else{CYAN};
        l.diamond(row.x+6.0,row.y+7.5,2.1,alpha(dot,if selected{0xFF}else{0xA0}));
        let kills=format!("×{}",species.kills);
        let name_w=(row.w-17.0-text_width(&kills)-5.0).max(20.0);
        l.text(&fit_word(&species.display_name,name_w),row.x+12.0,row.y+3.0,
            if selected{WHITE}else{alpha(ICE,0xD0)});
        l.text(&kills,row.x+row.w-text_width(&kills)-4.0,row.y+3.0,
            if selected{GOLD}else{alpha(MUTED,0xB0)});
        y+=16.0;
    }
    if m.bestiary.is_empty() {
        l.text("Nenhuma espécie descoberta.",left.x+8.0,y+4.0,MUTED);
        l.text("O primeiro abate confirmado cria o registro.",left.x+8.0,y+16.0,alpha(MUTED,0xA0));
    }
    if m.bestiary.len()>rows {
        let foot=Rect{x:left.x,y:left.y+left.h-18.0,w:left.w,h:16.0};
        l.text_center(&format!("{} / {}",start/rows+1,m.bestiary.len().div_ceil(rows)),
            foot,foot.y+4.0,MUTED);
        for (r,_,next) in bestiary_page_buttons(left,m.bestiary_sel,m.bestiary.len()) {
            l.state_button(r,if next{"›"}else{"‹"},super::rustpg_ui_theme::UiState::Normal,CYAN);
        }
    }

    l.card(right,CYAN);
    let Some(species)=m.bestiary.get(m.bestiary_sel) else {
        l.text_middle("SELECIONE UMA ESPÉCIE",right,MUTED);
        return;
    };
    let discovery=unit(species.discovery);
    let portrait=bestiary_portrait(m);
    l.chamfer(portrait,4.0,alpha(0xFF080C12,0xB8));
    l.chamfer_outline(portrait,4.0,1.0,alpha(CYAN,0x48));
    l.corner_brackets(portrait,7.0,1.0,alpha(CYAN,0x58));
    let reg=format!("{:.0}% REG.",discovery*100.0);
    l.chip(Rect{x:portrait.x+4.0,y:portrait.y+portrait.h-13.0,
        w:(text_width(&reg)+9.0).min(portrait.w-8.0),h:10.0},&reg,
        if discovery>=0.8{GOLD}else{CYAN});

    let info_x=portrait.x+portrait.w+8.0;
    let info_w=(right.x+right.w-8.0-info_x).max(38.0);
    l.text_tracked(&fit_tracked(&species.display_name.to_uppercase(),info_w,0.55),
        info_x,right.y+8.0,WHITE,0.55);
    l.text(&fit(&species.name,info_w),info_x,right.y+20.0,alpha(MUTED,0xB8));

    let tier=if discovery>=0.25 && !species.tiers.trim().is_empty() {
        species.tiers.to_uppercase()
    } else {"???".into()};
    let level=if species.level>0 {format!("NV {}",species.level)}else{"NV ?".into()};
    let mut sy=right.y+35.0;
    for (label,value,color) in [
        ("CLASSIFICAÇÃO",tier,CYAN),
        ("OBSERVADO",level,ICE),
        ("ABATES",species.kills.to_string(),GOLD),
        ("MAIOR DANO",if discovery>=0.35{format!("{:.1}",species.best)}else{"—".into()},ICE),
    ] {
        if sy+18.0>portrait.y+portrait.h {break;}
        l.text_tracked(label,info_x,sy,alpha(MUTED,0x9C),0.35);
        l.text(&fit(&value,info_w),info_x,sy+9.0,color);
        sy+=20.0;
    }

    let box_y=(portrait.y+portrait.h+6.0).max(sy+3.0);
    let desc=Rect{x:right.x+7.0,y:box_y,w:right.w-14.0,
        h:(right.y+right.h-box_y-7.0).max(24.0)};
    let body=l.description_box(desc,"Registro de campo",
        Some(if discovery>=0.65{"DADOS CONSOLIDADOS"}else{"OBSERVAÇÃO EM PROGRESSO"}),CYAN);
    let narrative=if discovery>=0.35 {
        species.description.trim().to_string()
    } else {
        "Informações insuficientes. Continue enfrentando esta espécie para consolidar o registro.".into()
    };
    let drops=if discovery>=0.65 && !species.drops.trim().is_empty() {
        species.drops.clone()
    } else {"Não descobertos".into()};
    let field=if discovery>=0.55{"Habitat: observado durante encontros; biome ainda não é persistido pela API atual."}
        else{"Habitat: não observado"};
    let lines=[
        narrative,
        field.into(),
        format!("Drops: {drops}"),
    ];
    let mut by=body.y;
    for (idx,line) in lines.iter().enumerate() {
        if by+9.0>body.y+body.h {break;}
        let color=if idx==0{ICE}else{alpha(MUTED,0xD0)};
        for part in wrap(line,body.w) {
            if by+9.0>body.y+body.h {break;}
            l.text(&part,body.x,by,color);
            by+=11.0;
        }
        if idx==0 {by+=3.0;}
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
    if id.starts_with("bridge.") {"rustpg:gui/icons/skills/icon_experience"}
    else if id.contains("health") || id.contains("vitality") || id.contains("regen") {
        "rustpg:gui/icons/attributes/icon_vitality"
    } else if id.contains("crit") {"rustpg:gui/icons/attributes/icon_critical"}
    else if id.contains("luck") || id.contains("rarity") {"rustpg:gui/icons/attributes/icon_luck"}
    else if id.contains("armor") || id.contains("defense") {"rustpg:gui/icons/attributes/icon_defense"}
    else if id.contains("resist") {"rustpg:gui/icons/attributes/icon_resistance"}
    else if id.contains("precision") || id.contains("accuracy") {"rustpg:gui/icons/attributes/icon_precision"}
    else if id.contains("penetrat") {"rustpg:gui/icons/attributes/icon_penetration"}
    else if id.contains("speed") || id.contains("agility") {"rustpg:gui/icons/attributes/icon_agility"}
    else if id.contains("mana") || id.contains("intelligence") {"rustpg:gui/icons/attributes/icon_intelligence"}
    else if id.contains("mining") {"rustpg:gui/icons/professions/icon_mining"}
    else if id.contains("fishing") {"rustpg:gui/icons/professions/icon_fishing"}
    else if id.contains("farming") {"rustpg:gui/icons/professions/icon_farming"}
    else if id.contains("exploration") {"rustpg:gui/icons/professions/icon_exploration"}
    else {"rustpg:gui/icons/attributes/icon_strength"}
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
        let detail_h=(available*0.57).max(59.0).min((available-27.0).max(1.0));
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
fn constellation_segments(m:&MenuModel)->Vec<super::rustpg_skillgraph::Segment> {
    let (g,_) = constellation_regions(m);
    let cap=if g.h<125.0 {4} else if g.h<195.0 {5}else{7};
    match m.tree.as_ref() {
        Some(t)=>super::rustpg_skillgraph::orbital_groups(t.nodes.len(),&t.links,cap),
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
        return super::rustpg_nodes::trees().len().div_ceil(size.max(1)).max(1);
    }
    let per=constellation_rows(m).max(1);
    constellation_segments(m).len().div_ceil(per).max(1)
}
pub fn constellation_page_arrow(m:&MenuModel,right:bool)->Rect {
    let (g,_) = constellation_regions(m);
    Rect{x:if right{g.x+g.w-22.0}else{g.x+4.0},y:g.y+2.0,w:18.0,h:16.0}
}
fn constellation_groups_on_page(m:&MenuModel)->Vec<(usize,super::rustpg_skillgraph::Segment)> {
    let per=constellation_rows(m);
    let seg=constellation_segments(m);
    let start=m.scroll.min(constellation_pages(m).saturating_sub(1))*per;
    seg.into_iter().skip(start).take(per).enumerate().collect()
}
fn constellation_branch_rect(m:&MenuModel,row:usize)->Rect {
    let (g,_) =constellation_regions(m);
    let rows=constellation_rows(m) as f32;
    let h=(g.h-23.0)/rows;
    Rect{x:g.x+3.0,y:g.y+21.0+row as f32*h,w:(g.w-6.0).max(20.0),h:(h-2.0).max(24.0)}
}
fn constellation_segment_rects(m:&MenuModel,row:usize,
    segment:&super::rustpg_skillgraph::Segment)->Vec<(Rect,usize)> {
    let r=constellation_branch_rect(m,row);
    let count=segment.members.len().max(1);
    let side=if r.h<80.0 {18.0}else{22.0};
    let usable_w=(r.w-26.0).max(side);
    let columns=if count<=4 {count}else{(count+1)/2}.max(2);
    let rows=count.div_ceil(columns).max(1);
    let gap_x=if columns<=1 {0.0}else{((usable_w-side)/(columns-1) as f32).min(58.0)};
    let width=side+gap_x*(columns.saturating_sub(1)) as f32;
    let x0=r.x+(r.w-width)*0.5;
    let row_gap=if rows>1 {(r.h-side-24.0).max(8.0)/(rows-1) as f32}else{0.0};
    let y0=r.y+18.0+(r.h-18.0-(side+row_gap*(rows.saturating_sub(1)) as f32))*0.5;
    segment.members.iter().enumerate().map(|(slot,&id)| {
        let rr=slot/columns; let cc=slot%columns;
        let offset=if rr%2==1 && columns>1 {gap_x*0.5}else{0.0};
        let x=(x0+cc as f32*gap_x+offset).min(r.x+r.w-side-5.0);
        let y=y0+rr as f32*row_gap;
        (Rect{x,y,w:side,h:side},id)
    }).collect()
}
pub fn constellation_node_rects(m:&MenuModel)->Vec<(Rect,usize)> {
    if m.tree.as_ref().is_none_or(|t|t.index_mode){return Vec::new();}
    let mut out=Vec::new();
    for (row,segment) in constellation_groups_on_page(m) {
        out.extend(constellation_segment_rects(m,row,&segment));
    }
    out
}
pub fn constellation_categories(m:&MenuModel)->Vec<(Rect,usize)> {
    if !m.tree.as_ref().is_some_and(|t|t.index_mode){return Vec::new();}
    let all=super::rustpg_nodes::trees();
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
    raw.split(" — ").next().unwrap_or(raw).to_string()
}
pub fn constellation_category_label(id: &str) -> String {
    use super::rustpg_progression::proficiency_name;
    let name = |key: &str| match key {
        "general" => "Geral", "defense" => "Defesa", "inventory" => "Inventário",
        "powers" => "Poderes", "chests" => "Baús", "bosses" => "Chefes", "mobs" => "Criaturas",
        other => proficiency_name(other),
    }.replace('_', " ");
    if let Some(key) = id.strip_prefix("weapon.") {return format!("Armas · {}",name(key));}
    if let Some(key) = id.strip_prefix("luck.") {return format!("Sorte · {}",name(key));}
    name(id)
}
fn constellation_draw_detail(l:&mut Layer,m:&MenuModel,hovered:Option<usize>) {
    let Some(t)=m.tree.as_ref()else{return;};
    let d=constellation_detail_rect(m);
    if d.w<=1.0 || d.h<=1.0{return;}
    let target=hovered.unwrap_or(t.selected);
    let Some(n)=t.nodes.get(target)else{return;};
    let state=match n.state {3=>"MAXIMIZADA",2=>"ADQUIRIDA",1=>"DISPONÍVEL",_=>"BLOQUEADA"};
    let shell=Rect{x:d.x,y:d.y,w:d.w,h:(d.h-27.0).max(30.0)};
    let subtitle=format!("{state} · {}/{} · {} PT",n.rank,n.max,n.cost);
    let body=l.description_box(shell,&n.name,Some(&subtitle),
        match n.state{3=>GOLD,2=>GREEN,1=>CYAN,_=>MUTED});

    let mut rows:Vec<(String,u32)>=Vec::new();
    for raw in n.description.lines() {
        let color=if raw.contains("No máximo:"){GOLD}
            else if raw.contains("Atual:")||raw.contains("Próximo:"){ICE}
            else if raw.contains("Integração:"){MUTED}
            else{WHITE};
        for line in wrap(raw,body.w){rows.push((line,color));}
    }
    if !n.requirement.trim().is_empty() {
        rows.push(("REQUISITOS".into(),CYAN));
        for line in wrap(&n.requirement,body.w) {
            rows.push((line,if n.buyable{GREEN}else{AMBER}));
        }
    }
    let visible=((body.h/11.0).floor() as usize).max(1);
    let offset=if hovered.is_some_and(|i|i!=t.selected){0}
        else{m.detail_scroll.min(rows.len().saturating_sub(visible))};
    for (i,(line,color)) in rows.iter().skip(offset).take(visible).enumerate() {
        l.text(line,body.x,body.y+i as f32*11.0,*color);
    }
    let b=constellation_buy_rect(m);
    let can=t.nodes.get(t.selected).is_some_and(|node|node.buyable);
    let button_state=if can{super::rustpg_ui_theme::UiState::Available}
        else{super::rustpg_ui_theme::UiState::Disabled};
    let label=if can{"ADQUIRIR NÍVEL"}else if n.state==3{"NÍVEL MÁXIMO"}else{"REQUISITOS PENDENTES"};
    l.state_button(b,label,button_state,if can{GREEN}else{CYAN});
}
fn constellation_wire(l:&mut Layer,g:Rect,p:[f32;2],q:[f32;2],width:f32,color:u32) {
    let (dx,dy)=(q[0]-p[0],q[1]-p[1]);
    if !dx.is_finite()||!dy.is_finite()||dx*dx+dy*dy<0.001{return;}
    let mut lo=0.0_f32;let mut hi=1.0_f32;
    for (a,b) in [(-dx,p[0]-g.x),(dx,g.x+g.w-p[0]),(-dy,p[1]-g.y),(dy,g.y+g.h-p[1])] {
        if a.abs()<0.0001 {if b<0.0{return;}}
        else {let k=b/a;if a<0.0{lo=lo.max(k);}else{hi=hi.min(k);}}
    }
    if lo>hi{return;}
    let a=[p[0]+lo*dx,p[1]+lo*dy];
    let b=[p[0]+hi*dx,p[1]+hi*dy];
    let norm=(dx*dx+dy*dy).sqrt().max(0.01);
    let t=width.max(0.25)*0.5;
    let nx=-dy/norm*t;let ny=dx/norm*t;
    l.quad([[a[0]+nx,a[1]+ny],[b[0]+nx,b[1]+ny],[b[0]-nx,b[1]-ny],[a[0]-nx,a[1]-ny]],color);
    // Direction marker near the child node. It stays subtle enough that a
    // dense branch remains readable on a phone.
    let ux=dx/norm;let uy=dy/norm;
    let px=-uy;let py=ux;
    let tip=[b[0]-ux*1.5,b[1]-uy*1.5];
    let base=[tip[0]-ux*4.0,tip[1]-uy*4.0];
    l.quad([tip,[base[0]+px*2.0,base[1]+py*2.0],
        [base[0]-px*2.0,base[1]-py*2.0],[base[0]-px*2.0,base[1]-py*2.0]],color);
}
fn constellation_page(l:&mut Layer,c:Rect,m:&MenuModel) {
    let Some(t)=m.tree.as_ref()else{
        l.text("Carregando habilidades...",c.x+8.0,c.y+8.0,CYAN);return;
    };
    l.text_tracked(&fit_tracked(&t.name.to_uppercase(),(c.w-157.0).max(35.0),0.45),
        c.x+5.0,c.y+4.0,WHITE,0.45);
    for i in 0..2 {
        let r=constellation_tree_arrow(c,i);
        l.state_button(r,if i==0{"‹"}else{"›"},super::rustpg_ui_theme::UiState::Normal,CYAN);
    }
    let cat=constellation_catalog_rect(c);
    l.state_button(cat,if t.index_mode{"VOLTAR"}else{"ESCOLAS"},
        if t.index_mode{super::rustpg_ui_theme::UiState::Selected}else{super::rustpg_ui_theme::UiState::Normal},CYAN);

    let owned=t.nodes.iter().filter(|n|n.state>=2).count();
    l.text(&fit_word(&format!("{owned}/{} NÓS · {} PT",t.nodes.len(),t.available_points),
        (c.w-10.0).max(35.0)),c.x+5.0,c.y+23.0,GOLD);

    let (g,d)=constellation_regions(m);
    l.chamfer(g,3.0,super::rustpg_ui_theme::SURFACE_LOW);
    l.chamfer_outline(g,3.0,1.0,alpha(super::rustpg_ui_theme::LINE,0x58));
    let pages=constellation_pages(m);
    let page=m.scroll.min(pages.saturating_sub(1));
    l.text_center(&format!("{} {}/{}",if t.index_mode{"ESCOLAS"}else{"RAMOS"},page+1,pages),
        Rect{x:g.x+26.0,y:g.y,w:g.w-52.0,h:19.0},g.y+6.0,MUTED);
    for (right,enabled) in [(false,page>0),(true,page+1<pages)] {
        let ar=constellation_page_arrow(m,right);
        l.state_button(ar,if right{"›"}else{"‹"},
            if enabled{super::rustpg_ui_theme::UiState::Normal}else{super::rustpg_ui_theme::UiState::Disabled},CYAN);
    }

    l.set_clip(Some(g));
    if t.index_mode {
        let names=super::rustpg_nodes::trees();
        for (r,i) in constellation_categories(m) {
            let Some(id)=names.get(i)else{continue;};
            let active=t.nodes.first().and_then(|n|super::rustpg_nodes::get(&n.id))
                .is_some_and(|n|n.tree==*id);
            let state=if active{super::rustpg_ui_theme::UiState::Selected}
                else{super::rustpg_ui_theme::UiState::Normal};
            l.chamfer(r,3.0,super::rustpg_ui_theme::state_fill(state));
            l.chamfer_outline(r,3.0,1.0,alpha(super::rustpg_ui_theme::state_edge(state),0x88));
            l.sprite(talent_symbol(id),[0.0,0.0,128.0,128.0],
                Rect{x:r.x+4.0,y:r.y+3.0,w:15.0,h:15.0});
            l.text(&fit_word(&constellation_category_label(id),r.w-26.0),
                r.x+24.0,r.y+7.0,if active{CYAN}else{WHITE});
        }
    } else {
        let visible=constellation_groups_on_page(m);
        let hovered=constellation_node_rects(m).into_iter()
            .find(|(r,_)|r.contains(m.mouse)).map(|(_,i)|i);
        for (row,segment) in &visible {
            let branch=constellation_branch_rect(m,*row);
            l.chamfer(branch,2.0,alpha(super::rustpg_ui_theme::SURFACE,0x78));
            l.rect(Rect{x:branch.x+5.0,y:branch.y,w:(branch.w*0.18).clamp(18.0,56.0),h:1.0},
                alpha(CYAN,0x52));
            let branch_name=segment.members.first().and_then(|i|t.nodes.get(*i))
                .map(|n|n.name.split(" — ").next().unwrap_or(&n.name)).unwrap_or("Habilidade");
            l.text_tracked(&fit_tracked(&branch_name.to_uppercase(),branch.w-10.0,0.3),
                branch.x+5.0,branch.y+4.0,alpha(ICE,0xB8),0.3);
            let rects=constellation_segment_rects(m,*row,segment);
            for &(a,b) in &t.links {
                let ra=rects.iter().find(|(_,i)|*i==a).map(|(r,_)|*r);
                let rb=rects.iter().find(|(_,i)|*i==b).map(|(r,_)|*r);
                if let (Some(ra),Some(rb))=(ra,rb) {
                    let pa=[ra.x+ra.w*0.5,ra.y+ra.h*0.5];
                    let pb=[rb.x+rb.w*0.5,rb.y+rb.h*0.5];
                    let color=if t.nodes.get(a).is_some_and(|n|n.rank>0)
                        &&t.nodes.get(b).is_some_and(|n|n.rank>0){GOLD}
                        else if t.nodes.get(b).is_some_and(|n|n.state==1){CYAN}
                        else{alpha(MUTED,0x4A)};
                    constellation_wire(l,g,pa,pb,1.0,color);
                }
            }
            for (r,i) in rects {
                let Some(n)=t.nodes.get(i)else{continue;};
                let hot=hovered==Some(i);let selected=i==t.selected;
                let state=match n.state {
                    3=>super::rustpg_ui_theme::UiState::Maxed,
                    2=>super::rustpg_ui_theme::UiState::Owned,
                    1=>super::rustpg_ui_theme::UiState::Available,
                    _=>super::rustpg_ui_theme::UiState::Locked,
                };
                let edge=super::rustpg_ui_theme::state_edge(if selected{
                    super::rustpg_ui_theme::UiState::Selected}else{state});
                let active=selected||matches!(state,super::rustpg_ui_theme::UiState::Available);
                let motion=if active{super::rustpg_ui_animation::action_pulse(m.tick,40)}else{1.0};
                if selected {l.glow(r,3.0,1.0,alpha(edge,(0x70 as f32*motion) as u8+0x30),1);}
                l.chamfer(r,3.0,super::rustpg_ui_theme::state_fill(state));
                l.chamfer_outline(r,3.0,if selected{1.5}else{1.0},
                    alpha(edge,if hot{0xFF}else{(0x88 as f32*motion) as u8+0x30}));
                let inset=(r.w*0.18).max(2.0);
                l.sprite(talent_symbol(&n.id),[0.0,0.0,128.0,128.0],
                    Rect{x:r.x+inset,y:r.y+inset,w:r.w-2.0*inset,h:r.h-2.0*inset});
                if n.buyable {l.diamond(r.x+r.w-1.0,r.y+1.0,2.2,GREEN);}
                if n.rank>0 {
                    let tag=n.rank.to_string();
                    l.text(&tag,r.x+2.0,r.y+r.h-7.0,if n.state==3{GOLD}else{ICE});
                }
            }
        }
    }
    l.set_clip(None);
    if d.w>1.0&&d.h>1.0{constellation_draw_detail(l,m,None);}
}
pub fn respec_rect(c: Rect) -> Rect {
    Rect {x:c.x+c.w-8.0-88.0,y:c.y+c.h-18.0,w:88.0,h:14.0}
}
pub fn buy_rect(c: Rect) -> Rect {
    let right_side=c.w>=500.0 && c.h>=170.0;
    let dx=if right_side {(c.w*0.67).max(265.0)+5.0}else{0.0};
    Rect {x:c.x+dx+6.0,y:c.y+c.h-25.0,w:(c.w-dx-12.0).max(30.0),h:19.0}
}
pub fn tree_arrow(head: Rect, index: usize) -> Rect {
    let w=14.0;
    Rect{x:head.x+head.w-(2-index) as f32*(w+2.0)-4.0,y:head.y+2.0,w,h:12.0}
}
pub fn page_count(m: &MenuModel) -> usize {
    let c = content(window(m.area), m.pages.len());
    match m.kind {
        "trees" => constellation_pages(m),
        "inventory" => m.bag.len().div_ceil(gear_rows_per_page(c)).max(1),
        "equipment" => equipped_page_count(c, 8),
        "accessories" => equipped_page_count(c, 3),
        "overview" | "status" | "attributes" | "bestiary" => 1,
        _ => data_pages(c, &m.lines),
    }
}

pub fn menu_hits(m: &MenuModel) -> Vec<(Rect, String)> {
    let win=window(m.area); let mut out=Vec::new(); let n=m.pages.len();
    let group=tab_group(m.page.min(n.saturating_sub(1)));
    // Disabled arrows were still tappable and wrapped to the opposite group.
    // Put active arrows before tabs: expanded phone hit targets must not be
    // stolen by the adjacent first/last page slot.
    let groups=tab_groups(n);
    if group>0 {
        out.push((super::rustpg_ui_input::touch_target(tab_arrow(win,0),win,18.0),"tgroup:-1".into()));
    }
    if group+1<groups {
        out.push((super::rustpg_ui_input::touch_target(tab_arrow(win,1),win,18.0),"tgroup:1".into()));
    }
    for (i,r) in tab_rects(win,n,m.page).into_iter().enumerate(){
        out.push((super::rustpg_ui_input::touch_target(r,win,20.0),
            format!("page:{}",group*TAB_SLOTS+i)));
    }
    let c=content(win,m.pages.len());
    if !matches!(m.kind,"trees"|"bestiary"|"status"|"attributes"|"overview"|"equipment"|"accessories"|"inventory")
        && data_pages(c,&m.lines)>1 {
        out.push((data_arrow(c,0).inset(-4.0),"rpage:-1".into())); out.push((data_arrow(c,1).inset(-4.0),"rpage:1".into()));
    }
    match m.kind {
        "inventory"=>{
            let rows=gear_rows_per_page(c); let pages=m.bag.len().div_ceil(rows).max(1);
            if m.scroll>0 {out.push((inventory_next_rect(c,true),"rpage:-1".into()));}
            if m.scroll+1<pages {out.push((inventory_next_rect(c,false),"rpage:1".into()));}
            for (i,g) in m.bag.iter().skip(m.scroll.min(pages-1)*rows).take(rows).enumerate() {
                let row=inventory_row_rect(c,i); if row.y+row.h>c.y+c.h {break;}
                out.push((inventory_equip_rect(row),format!("equip:{}",g.id))); out.push((inventory_salvage_rect(row),format!("salvage:{}",g.id)));
            }
        }
        "equipment"|"accessories"=>{
            let acc=m.kind=="accessories"; let slots:&[&str]=if acc{&["ring","neck","charm"]}else{&["head","chest","legs","feet","weapon","ring","neck","charm"]};
            if m.scroll>0 {out.push((equipped_page_arrow(c,false),"rpage:-1".into()));}
            if m.scroll+1<equipped_page_count(c,slots.len()) {
                out.push((equipped_page_arrow(c,true),"rpage:1".into()));
            }
            for (idx,r) in equipped_visible_rects(c,slots.len(),m.scroll) {
                let slot=slots[idx];
                if let Some(g)=m.worn.iter().find(|g|g.slot==slot) {
                    if g.power_ready {
                        out.push((equipped_activate_rect(r),format!("activate:{slot}")));
                    }
                    out.push((equipped_remove_rect(r),format!("unequip:{slot}")));
                }
            }
        }
        "trees"=>{
            out.push((super::rustpg_ui_input::touch_target(constellation_tree_arrow(c,0),c,20.0),"tree:-1".into()));out.push((super::rustpg_ui_input::touch_target(constellation_tree_arrow(c,1),c,20.0),"tree:1".into()));
            out.push((super::rustpg_ui_input::touch_target(constellation_catalog_rect(c),c,20.0),"catalog".into()));
            if m.tree.as_ref().is_some_and(|t|t.index_mode) {
                for (r,i) in constellation_categories(m){out.push((r,format!("treecat:{i}")));}
            }else{
                for (r,i) in constellation_node_rects(m){out.push((r,format!("node:{i}")));}
                if constellation_pages(m)>1 {out.push((super::rustpg_ui_input::touch_target(constellation_page_arrow(m,false),c,20.0),"rpage:-1".into()));out.push((super::rustpg_ui_input::touch_target(constellation_page_arrow(m,true),c,20.0),"rpage:1".into()));}
            }
            if !m.external_inspector&&!m.tree.as_ref().is_some_and(|t|t.index_mode){out.push((constellation_buy_rect(m),"buy".into()));}
            if m.tree.as_ref().is_some_and(|t|t.index_mode)&&constellation_pages(m)>1 {
                out.push((constellation_page_arrow(m,false),"rpage:-1".into()));out.push((constellation_page_arrow(m,true),"rpage:1".into()));
            }
        }
        "bestiary"=>{
            let (left,_)=split(c);let rows=bestiary_rows(left);let start=(m.bestiary_sel / rows) * rows;let y=left.y+22.0;
            for (r, target, _) in bestiary_page_buttons(left, m.bestiary_sel, m.bestiary.len()) {
                out.push((r,format!("species:{target}")));
            }
            for i in 0..m.bestiary.len().saturating_sub(start).min(rows) {
                out.push((Rect{x:left.x+6.0,y:y+i as f32*16.0-2.0,w:left.w-12.0,h:15.0},format!("species:{}",start+i)));
            }
        }
        "attributes"=>out.push((respec_rect(c),"respec".into())),
        _=>{}
    }
    out
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    fn area()->Rect {Rect{x:0.0,y:0.0,w:420.0,h:260.0}}
    fn model(kind:&'static str)->MenuModel {
        MenuModel{area:area(),pages:vec!["Resumo".into(),"Habilidades".into(),"Bestiário".into()],
            kind,title:kind.to_uppercase(),level:1,rank:"E".into(),..Default::default()}
    }

    #[test]
    fn disabled_tab_arrows_are_not_clickable_and_all_pages_reachable(){
        let mut m=model("overview");
        m.pages=(0..9).map(|i|format!("Aba {i}")).collect();
        m.page=0;
        let first=menu_hits(&m);
        assert!(!first.iter().any(|(_,a)|a=="tgroup:-1"));
        assert!(first.iter().any(|(_,a)|a=="tgroup:1"));
        assert!(first.iter().any(|(_,a)|a=="page:4"));
        m.page=7;
        let last=menu_hits(&m);
        assert!(last.iter().any(|(_,a)|a=="tgroup:-1"));
        assert!(!last.iter().any(|(_,a)|a=="tgroup:1"));
        assert!(last.iter().any(|(_,a)|a=="page:8"));
        assert!(!last.iter().any(|(_,a)|a=="page:9"));
    }

    #[test]
    fn paged_skill_tree_has_no_camera_controls() {
        let mut m=model("trees");
        m.tree=Some(TreeModel{name:"Geral".into(),available_points:0,nodes:Vec::new(),links:Vec::new(),
            pan:[0.0,0.0],zoom:1.0,selected:0,index_mode:false,detail_only:false,detail:Vec::new()});
        let actions=menu_hits(&m).into_iter().map(|(_,a)|a).collect::<Vec<_>>();
        assert!(actions.iter().all(|a|!a.starts_with("zoom:") && a!="fit" && a!="center"));
        assert!(constellation_pages(&m)>=1);
    }

    #[test]
    fn bestiary_touch_pages_reach_last_partial_page_and_return() {
        let area = Rect{x:0.0,y:0.0,w:160.0,h:100.0};
        let rows = bestiary_rows(area);
        let count = rows * 2 + 1;
        let first = bestiary_page_buttons(area,0,count);
        assert_eq!(first.len(),1);
        assert_eq!(first[0].1,rows);
        let middle = bestiary_page_buttons(area,rows,count);
        assert_eq!(middle.len(),2);
        let last = bestiary_page_buttons(area,middle[1].1,count);
        assert_eq!(last.len(),1);
        assert_eq!(last[0].1,rows);
    }

    #[test]
    fn bestiary_rows_use_registry_ids_but_display_metadata_can_be_modded() {
        let mut m=model("bestiary");
        m.bestiary=vec![SpeciesRow{name:"thirdmod:abyss_beast".into(),
            display_name:"Abyss Beast · thirdmod".into(),
            description:"Entidade observada pelo RustPG.".into(),live_entity:Some(42),
            kills:3,level:7,best:18.0,tiers:"Elite".into(),drops:"fragmento".into(),discovery:0.2}];
        assert!(menu_hits(&m).iter().any(|(_,a)|a=="species:0"));
        assert_eq!(m.bestiary[0].name,"thirdmod:abyss_beast");
    }
}
