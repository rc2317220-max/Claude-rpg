//! Visual regression exporter for RustPG's pure menu Layer.
//! Test-only: never compiled into the .hmod.

use std::{env,fmt::Write as _,fs,path::{Path,PathBuf}};
use crate::rustpg_ui_layout::Rect;
use crate::rustpg_ui_system::{Layer,Prim};
use crate::rustpg_ui_window::{
    AttrRow,GearRow,MenuModel,SpeciesRow,TreeModel,TreeNode,Vital,
};

fn esc(s:&str)->String {
    s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;")
        .replace('"',"&quot;")
}
fn rgba(argb:u32)->(String,f32) {
    let a=((argb>>24)&255) as f32/255.0;
    let r=(argb>>16)&255; let g=(argb>>8)&255; let b=argb&255;
    (format!("#{r:02X}{g:02X}{b:02X}"),a)
}
fn emit_fill(out:&mut String,color:u32,extra:&str) {
    let (rgb,a)=rgba(color);
    let _=write!(out," fill=\"{rgb}\" fill-opacity=\"{a:.3}\"{extra}");
}
fn sprite_asset(key: &str) -> Option<(Vec<u8>, [u32; 2])> {
    let path = if let Some(species) = key.strip_prefix("lab:entity/") {
        PathBuf::from(env::var_os("RUSTPG_ENTITY_PREVIEWS")?)
            .join(species).join("idle-three_quarter.png")
    } else {
        let asset = key.strip_prefix("rustpg:gui/")?;
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/rustpg/textures/gui").join(format!("{asset}.png"))
    };
    let bytes = fs::read(path).ok()?;
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    (w > 0 && h > 0).then_some((bytes, [w, h]))
}

// Embedding once per sprite lets the exported SVG survive the private build
// directory, including nine-sliced sprites used repeatedly by the same scene.
fn png_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let word = u32::from_be_bytes([0, chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)]);
        for (index, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            encoded.push(if index > chunk.len() { '=' } else { ALPHABET[((word >> shift) & 63) as usize] as char });
        }
    }
    encoded
}

fn add_entity_preview(layer: &mut Layer, key: &'static str, rect: Rect) {
    if rect.w <= 0.0 || rect.h <= 0.0 { return; }
    if env::var_os("RUSTPG_ENTITY_PREVIEWS").is_none() {
        let label = crate::rustpg_ui_system::fit("Sem prévia", rect.w);
        layer.text_middle(label, rect, crate::rustpg_ui_theme::MUTED);
        return;
    }
    let (_, [width, height]) = sprite_asset(key)
        .unwrap_or_else(|| panic!("Missing Horizon entity lab capture for {key}"));
    let (width, height) = (width as f32, height as f32);
    let scale = (rect.w / width).min(rect.h / height);
    let (w, h) = (width * scale, height * scale);
    let dst = Rect { x: rect.x + (rect.w - w) * 0.5, y: rect.y + (rect.h - h) * 0.5, w, h };
    layer.sprite(key, [0.0, 0.0, width, height], dst);
}

fn layer_svg(layer:&Layer,w:f32,h:f32,title:&str)->String {
    let mut out=String::new();
    let _=writeln!(out,r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><title>{}</title>"#, esc(title));
    // A consistent high-contrast simulated world tests pale and dark HUD text.
    // Keep scene metadata out of the pixels presented to the reviewer.
    out.push_str(r##"<svg width="100%" height="100%" viewBox="0 0 400 200" preserveAspectRatio="none">
<defs><linearGradient id="sky" x2="0" y2="1"><stop stop-color="#516f85"/><stop offset=".72" stop-color="#b5b5a0"/><stop offset="1" stop-color="#81958b"/></linearGradient></defs>
<rect width="400" height="200" fill="url(#sky)"/>
<path fill="#899ca0" d="M0 98h15v-7h14V75h12v-6h16v-7h16v13h12v9h37v9h33V81h18V70h16V62h24V46h17v10h15v12h12v11h20v12h33v-8h24V68h24V58h20v8h22v-7h20v141H0z"/>
<path fill="#647c79" d="M0 107h34v-9h22V87h15v-5h14v8h26v12h27v9h45v-7h25V89h22v-8h17v-8h16v8h22v5h34v13h22v-9h28V79h23V68h8v132H0z"/>
<path fill="#465d55" d="M0 89h21v5h29v9h29v8h35v5h36v10h43v5h43v-6h45v-6h21v-9h31V94h25V84h23V72h19v128H0z"/>
<path fill="#32483c" d="M0 112h23v8h28v4h24v8h39v6h65v9h45v-4h74v-8h32v-7h30v-13h40v85H0z"/>
<path fill="#405345" d="M0 155h56v3h58v6h75v8h68v-8h58v-8h44v-5h41v49H0z"/>
</svg>"##);
    let mut sprites = std::collections::BTreeMap::new();
    for p in &layer.prims {
        match p {
            Prim::Rect{x,y,w,h,color}=>{
                let _=write!(out,r#"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}""#);
                emit_fill(&mut out,*color,"");
                let _=writeln!(out,"/>");
            }
            Prim::Quad{p,color}=>{
                let pts=p.iter().map(|v|format!("{:.2},{:.2}",v[0],v[1])).collect::<Vec<_>>().join(" ");
                let _=write!(out,r#"<polygon points="{pts}""#);
                emit_fill(&mut out,*color,"");
                let _=writeln!(out,"/>");
            }
            Prim::Text{text,x,y,color,shadow}=>{
                let (rgb,a)=rgba(*color);
                if *shadow {
                    let _=writeln!(out,r##"<text x="{:.2}" y="{:.2}" font-family="sans-serif" font-size="9" fill="#000" fill-opacity="{:.3}">{}</text>"##,x+1.0,y+9.0,a*0.72,esc(text));
                }
                let _=writeln!(out,r#"<text x="{x:.2}" y="{:.2}" font-family="sans-serif" font-size="9" fill="{rgb}">{}</text>"#,y+8.0,esc(text));
            }
            Prim::Sprite{key,src,dst,tint}=>{
                let next_id = sprites.len();
                let id = sprites.entry(*key).or_insert_with(|| {
                    let (bytes, [iw, ih]) = sprite_asset(key)
                        .unwrap_or_else(|| panic!("Missing visual lab sprite {key}"));
                    let encoded = png_base64(&bytes);
                    let _ = writeln!(out, r#"<defs><image id="asset-{next_id}" width="{iw}" height="{ih}" xlink:href="data:image/png;base64,{encoded}"/></defs>"#);
                    next_id
                });
                let [x, y, w, h] = *dst;
                let [sx, sy, sw, sh] = *src;
                let color = tint.unwrap_or(0xFFFF_FFFF);
                let opacity = (color >> 24) as f32 / 255.0;
                let mut filter = String::new();
                if color & 0xFF_FFFF != 0xFF_FFFF {
                    let [_, r, g, b] = color.to_be_bytes();
                    let (r, g, b) = (r as f32/255.0, g as f32/255.0, b as f32/255.0);
                    let filter_id = format!("tint-{}", out.len());
                    let _ = writeln!(out, r#"<defs><filter id="{filter_id}" color-interpolation-filters="sRGB"><feColorMatrix type="matrix" values="{r} 0 0 0 0  0 {g} 0 0 0  0 0 {b} 0 0  0 0 0 1 0"/></filter></defs>"#);
                    filter = format!(r#" filter="url(#{filter_id})""#);
                }
                let _ = writeln!(out, r##"<svg x="{x}" y="{y}" width="{w}" height="{h}" viewBox="{sx} {sy} {sw} {sh}" preserveAspectRatio="none" overflow="hidden"><use xlink:href="#asset-{id}" opacity="{opacity:.3}"{filter}/></svg>"##);
            }
        }
    }
    out.push_str("</svg>");
    out
}

fn bounds_error(layer:&Layer, bounds:Rect, label:&str) -> Option<String> {
    let eps=1.5_f32;
    let inside=|x:f32,y:f32| {
        x>=bounds.x-eps && y>=bounds.y-eps &&
        x<=bounds.x+bounds.w+eps && y<=bounds.y+bounds.h+eps
    };
    for (i,p) in layer.prims.iter().enumerate() {
        let ok=match p {
            Prim::Rect{x,y,w,h,..}=>inside(*x,*y)&&inside(*x+*w,*y+*h),
            Prim::Quad{p,..}=>p.iter().all(|v|inside(v[0],v[1])),
            Prim::Text{x,y,text,..}=>{
                // Host font is 9px high; use the same approximate width as the
                // layout engine so regression checks catch footer/column leaks.
                let tw=crate::rustpg_ui_system::text_width(text);
                inside(*x,*y)&&inside(*x+tw,*y+9.0)
            }
            Prim::Sprite{dst,..}=>inside(dst[0],dst[1])&&inside(dst[0]+dst[2],dst[1]+dst[3]),
        };
        if !ok { return Some(format!("visual primitive {i} escaped {label}: {p:?}")); }
    }
    None
}

fn assert_layer_inside(layer: &Layer, bounds: Rect, label: &str) {
    if let Some(error) = bounds_error(layer, bounds, label) { panic!("{error}"); }
}

fn hud_scene_svg(w: f32, h: f32, variant: &str) -> String {
    use crate::rustpg_bar_vfx::BarState;
    use crate::rustpg_damage_meter::DamageFamily;
    use crate::rustpg_hud::{self, CombatLayout, Meter, PlayerVitals};

    let bounds = crate::rustpg_hud_geometry::compute(w, h, 1.0, 0.0).unwrap();
    let mut layer = Layer::new();
    let low = variant == "low_hp";
    let exhausted = variant == "exhausted";
    let mut hp_fx = BarState::default();
    hp_fx.step(0.72);
    if low { hp_fx.step(0.18); }

    rustpg_hud::player(&mut layer, w, bounds, &PlayerVitals {
        level: 47, rank: "B", points: 17,
        hp: Meter { fraction: hp_fx.display, value: if low { "36/200" } else { "144/200" }.into(), animation: hp_fx },
        stamina: Meter { fraction: if exhausted { 0.06 } else { 0.46 },
            value: if exhausted { "9/155" } else { "71/155" }.into(), animation: BarState::default() },
        food: 0.8, armor: 38.0, air: None, recovery: exhausted.then_some(0.38), relic: None,
    });
    rustpg_hud::experience(&mut layer, w, h, bounds.scale, 47, 0.73, BarState::default());

    if matches!(variant, "target" | "boss" | "damage" | "critical") {
        let boss = variant == "boss";
        let (kind, key, name, metadata, fraction, hp) = if boss {
            ("rustpg:rune_colossus", "lab:entity/rune_colossus", "Colosso Rúnico",
             "S · FASE 2 · NV 88", 0.43, "438/1000")
        } else {
            ("rustpg:mana_hound", "lab:entity/mana_hound", "Cão de Mana",
             "NV 42 · Raro · ARM 38", 0.61, "146/240")
        };
        let palette = crate::rustpg_boss_ui::style(kind, "", fraction, 37);
        let layout = CombatLayout::new(w, bounds, boss);
        rustpg_hud::target(&mut layer, layout, name, metadata, Some((fraction, hp.into())), palette);
        add_entity_preview(&mut layer, key, layout.preview);
        if matches!(variant, "damage" | "critical") {
            let latest = if variant == "critical" { DamageFamily::Critical } else { DamageFamily::Physical };
            rustpg_hud::damage_meter(&mut layer, layout.telemetry(), h, 782.4, 5,
                [(latest, 684.7), (DamageFamily::Fire, 43.2)]);
            rustpg_hud::damage_popup(&mut layer, [w*0.49, h*0.64], 0, 9, 684.7, latest);
        }
    }
    assert_layer_inside(&layer, Rect { x: 0.0, y: 0.0, w, h }, variant);
    layer_svg(&layer, w, h, &format!("HUD · {variant}"))
}

fn attrs()->Vec<AttrRow> {
    ["VITALIDADE","FORÇA","AGILIDADE","DEFESA","RESISTÊNCIA","PRECISÃO","CRÍTICO","PENETRAÇÃO","SORTE","INTELIGÊNCIA"]
        .iter().enumerate().map(|(i,n)|AttrRow{
            name:(*n).into(),value:format!("{}",18+i*3),
            ratio:0.28+i as f32*0.055,hint:"".into()
        }).collect()
}
fn vitals()->Vec<Vital> {
    vec![
        Vital{label:"HP".into(),ratio:0.72,value:"144/200".into()},
        Vital{label:"SP".into(),ratio:0.46,value:"71/155".into()},
        Vital{label:"ABS".into(),ratio:0.24,value:"24".into()},
    ]
}
fn pages()->Vec<String> {
    crate::rustpg_view::PAGES.iter().map(|(_,name)|name.to_string()).collect()
}
fn page_index(kind:&str)->usize {
    crate::rustpg_view::PAGES.iter().position(|(k,_)|*k==kind).unwrap_or(0)
}
fn base(area:Rect,kind:&'static str,title:&str)->MenuModel {
    MenuModel{
        area,mouse:[-100.0,-100.0],tick:37,page:0,pages:pages(),kind,title:title.into(),
        level:47,rank:"B".into(),next_rank:"A".into(),rank_progress:0.61,xp:0.73,points:17,
        vitals:vitals(),attrs:attrs(),
        lines:vec!["Poder total: 18.420".into(),"Sequência máxima: 31".into(),"Fendas concluídas: 12".into(),"Maior golpe: 684.7".into()],
        ..Default::default()
    }
}
fn bestiary(area:Rect)->MenuModel {
    let mut m=base(area,"bestiary","BESTIÁRIO");
    m.page=page_index("bestiary");
    m.bestiary=(0..9).map(|i|SpeciesRow{
        name:format!("rustpg:{}", ["ashen_sentinel","rift_hunter","mana_hound","grave_knight","rift_mantis","crystal_wyrm","eclipse_ogre","storm_gryphon","rune_colossus"][i]),
        display_name:["Sentinela Cinéreo","Caçador da Fenda","Cão de Mana","Cavaleiro Sepulcral","Louva-a-deus da Fenda","Wyrm de Cristal","Ogro do Eclipse","Grifo da Tempestade","Colosso Rúnico"][i].into(),
        description:"Uma criatura marcada pela energia instável das Fendas. A observação revela padrões de combate, resistências e comportamento.".into(),
        live_entity:None,kills:(i as u64+1)*7,level:22+i as u32*4,best:120.0+i as f64*38.5,
        tiers:"Raro · Elite".into(),drops:"Fragmento rúnico · Essência".into(),
        discovery:(0.18+i as f32*0.1).min(1.0),
    }).collect();
    m.bestiary_sel=5;
    m
}
fn tree(area:Rect)->MenuModel {
    let mut m=base(area,"trees","ÁRVORE DE HABILIDADES");
    m.page=page_index("trees");
    let nodes=(0..18).map(|i|TreeNode{
        id:format!("combat.node_{i}"),name:format!("Técnica {}",i+1),rank:if i<6{2}else{0},
        max:"5".into(),cost:1,col:i%6,row:i/6,state:if i<5{2}else if i<9{1}else{0},
        buyable:(5..9).contains(&i),description:"Aprimora uma propriedade específica sem elevar atributos não relacionados.".into(),
        requirement:if i<5{"Concluído".into()}else{"Requer o nó anterior".into()},
    }).collect::<Vec<_>>();
    let links=(1..18).map(|i|(i-1,i)).collect();
    m.tree=Some(TreeModel{name:"ESPADA".into(),nodes,available_points:17,links,pan:[0.0,0.0],zoom:0.78,selected:6,index_mode:false,detail_only:false,detail:vec![]});
    m
}
fn equipment(area:Rect)->MenuModel {
    let mut m=base(area,"equipment","EQUIPAMENTOS");
    m.page=page_index("equipment");
    m.worn=vec![
        GearRow{id:1,slot:"head".into(),title:"Elmo do Horizonte".into(),ability:"Olhar Rúnico".into(),stat:"DEF".into(),bonus:14.5,rarity:3,power_status:"ATIVAR".into(),power_ready:true},
        GearRow{id:2,slot:"chest".into(),title:"Couraça da Fenda".into(),ability:"Barreira".into(),stat:"HP".into(),bonus:32.0,rarity:4,power_status:"CD 12s".into(),power_ready:false},
        GearRow{id:3,slot:"weapon".into(),title:"Lâmina do Eclipse".into(),ability:"Ruptura".into(),stat:"DMG".into(),bonus:27.0,rarity:5,power_status:"ATIVAR".into(),power_ready:true},
    ];
    m
}
fn inventory(area:Rect)->MenuModel {
    let mut m=base(area,"inventory","RELÍQUIAS");
    m.page=page_index("inventory");
    m.bag=vec![
        GearRow{id:11,slot:"weapon".into(),title:"Lâmina do Eclipse".into(),ability:"Ruptura".into(),stat:"DMG".into(),bonus:27.0,rarity:5,power_status:"EQUIPE PARA ATIVAR".into(),power_ready:false},
        GearRow{id:12,slot:"ring".into(),title:"Anel de Mana".into(),ability:"Fluxo".into(),stat:"MANA".into(),bonus:18.0,rarity:3,power_status:"EQUIPE PARA ATIVAR".into(),power_ready:false},
        GearRow{id:13,slot:"chest".into(),title:"Couraça da Fenda".into(),ability:"Barreira".into(),stat:"HP".into(),bonus:32.0,rarity:4,power_status:"EQUIPE PARA ATIVAR".into(),power_ready:false},
    ];
    m
}
fn data_sample(area:Rect,kind:&'static str,title:&str,lines:&[&str])->MenuModel {
    let mut m=base(area,kind,title);
    m.page=page_index(kind);
    m.lines=lines.iter().map(|s|(*s).to_string()).collect();
    m
}
fn sample(kind:&str,area:Rect)->MenuModel {
    match kind {
        "overview"=>{let mut m=base(area,"overview","RESUMO");m.page=page_index("overview");m},
        "status"=>{let mut m=base(area,"status","STATUS");m.page=page_index("status");m},
        "attributes"=>base(area,"attributes","ATRIBUTOS"),
        "bestiary"=>bestiary(area),
        "trees"=>tree(area),
        "equipment"=>equipment(area),
        "inventory"=>inventory(area),
        "profs"=>data_sample(area,"profs","PROFISSÕES",&[
            "Mineração: 42/100 · bônus de rendimento 8%",
            "Combate: 68/100 · dano treinado 12%",
            "Exploração: 35/100 · ressonância 64/100",
            "Agricultura: 21/100 · crescimento 4%",
        ]),
        "enchant"=>data_sample(area,"enchant","ENCANTAMENTOS",&[
            "Afiada: 184/250 · nível acumulado 184",
            "Proteção: 96/150 · mitigação ativa",
            "Fortuna: 33/75 · bônus de drop",
            "Limite: acúmulo validado pelo servidor",
        ]),
        "history"=>data_sample(area,"history","HISTÓRICO",&[
            "Fenda Rank B concluída · nota A",
            "Colosso Rúnico derrotado · +1 registro",
            "Técnica 7 adquirida · rank 2/5",
            "Maior golpe atualizado · 684.7",
        ]),
        _=>base(area,"overview","RESUMO"),
    }
}

#[test]
fn visual_lab_exports_svg() {
    let Ok(root)=env::var("RUSTPG_UI_LAB_OUT") else { return; };
    let root=PathBuf::from(root);
    let mut errors = Vec::new();
    fs::create_dir_all(&root).unwrap();
    for (label,w,h) in [
        ("compact",320.0,180.0),
        ("phone",380.0,240.0),
        ("wide",640.0,360.0),
        ("ultrawide",820.0,360.0),
    ] {
        for kind in ["overview","status","trees","bestiary","profs","equipment","inventory","enchant","history","attributes"] {
            let full=Rect{x:4.0,y:4.0,w:w-8.0,h:h-8.0};
            let main=crate::rustpg_inspector::main_area(full);
            let mut m=sample(kind,main);
            m.external_inspector=crate::rustpg_inspector::inspector_area(full).is_some();
            let mut layer=crate::rustpg_ui_window::chrome_layer(main,&m);
            layer.prims.extend(crate::rustpg_ui_window::draw_menu(&m).prims);
            if kind == "bestiary" {
                let portrait = crate::rustpg_ui_window::bestiary_portrait(&m);
                if portrait.h > 16.0 {
                    let dst = Rect { x: portrait.x + 1.0, y: portrait.y + 1.0,
                        w: portrait.w - 2.0, h: portrait.h - 13.0 };
                    add_entity_preview(&mut layer, "lab:entity/crystal_wyrm", dst);
                }
            }
            if let Some(side)=crate::rustpg_inspector::inspector_area(full) {
                let info=crate::rustpg_inspector::inspect(&m);
                let present=crate::rustpg_inspector::presentation(side,&info,0,50);
                layer.prims.extend(present.layer.prims);
                if let Some([x0,y0,x1,y1]) = present.preview_rect {
                    assert_eq!(present.preview_kind.as_deref(), Some("rustpg:crystal_wyrm"));
                    add_entity_preview(&mut layer, "lab:entity/crystal_wyrm",
                        Rect {x:x0, y:y0, w:x1-x0, h:y1-y0});
                }
            }
            if let Some(error) = bounds_error(&layer, Rect { x:0.0, y:0.0, w, h }, &format!("{kind}-{label}")) {
                errors.push(error);
            }
            let path=root.join(format!("{kind}-{label}.svg"));
            fs::write(&path,layer_svg(&layer,w,h,&format!("{kind} · {label}"))).unwrap();
        }
    }
    for (label,w,h) in [
        ("compact",320.0,180.0),
        ("phone",380.0,240.0),
        ("wide",640.0,360.0),
        ("ultrawide",820.0,360.0),
    ] {
        for variant in ["normal","low_hp","exhausted","target","boss","damage","critical"] {
            let path=root.join(format!("hud-{variant}-{label}.svg"));
            fs::write(&path,hud_scene_svg(w,h,variant)).unwrap();
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    assert_eq!(fs::read_dir(Path::new(&root)).unwrap().count(),68);
}
