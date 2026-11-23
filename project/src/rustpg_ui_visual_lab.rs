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
fn layer_svg(layer:&Layer,w:f32,h:f32,title:&str)->String {
    let mut out=String::new();
    let _=writeln!(out,r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#);
    // Neutral pseudo-world: enough contrast variation to expose over-opaque panels.
    let _=writeln!(out,r##"<defs><linearGradient id="world" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#35483d"/><stop offset=".48" stop-color="#182c36"/><stop offset="1" stop-color="#4a382b"/></linearGradient></defs>"##);
    let _=writeln!(out,r#"<rect width="100%" height="100%" fill="url(#world)"/><g opacity=".18">"#);
    for x in (0..w as i32).step_by(32) { let _=writeln!(out,r##"<rect x="{x}" y="0" width="16" height="{h}" fill="#9bb99a"/>"##); }
    let _=writeln!(out,"</g>");
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
                    let _=writeln!(out,r##"<text x="{:.2}" y="{:.2}" font-family="monospace" font-size="8" fill="#000" fill-opacity="{:.3}">{}</text>"##,x+1.0,y+8.0+1.0,a*0.72,esc(text));
                }
                let _=writeln!(out,r#"<text x="{x:.2}" y="{:.2}" font-family="monospace" font-size="8" fill="{rgb}" fill-opacity="1">{}</text>"#,y+8.0,esc(text));
            }
            Prim::Sprite{key,dst,tint,..}=>{
                let [x,y,w,h]=*dst;
                let color=tint.unwrap_or(0x403FC9D5);
                let (rgb,a)=rgba(color);
                // Sprite placeholder keeps exact runtime destination bounds. Entity previews
                // are validated separately through the Horizon entity visual lab.
                let _=writeln!(out,r##"<rect x="{x:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{rgb}" fill-opacity="{:.3}" stroke="#69D5D0" stroke-opacity=".28" stroke-width=".5"/>"##,a.max(0.12));
                if w>=28.0 && h>=10.0 {
                    let label=key.rsplit('/').next().unwrap_or(key);
                    let _=writeln!(out,r##"<text x="{:.2}" y="{:.2}" font-family="monospace" font-size="5" fill="#91A0A7" opacity=".6">{}</text>"##,x+2.0,y+6.0,esc(label));
                }
            }
        }
    }
    let _=writeln!(out,r##"<text x="5" y="{:.1}" font-family="monospace" font-size="6" fill="#fff" opacity=".3">{}</text></svg>"##,h-4.0,esc(title));
    out
}



fn hud_scene_svg(w:f32,h:f32,variant:&str)->String {
    use crate::rustpg_ui_theme::{AETHER,GOLD,MOON,MUTED,WHITE};
    let mut out=String::new();
    let _=writeln!(out,r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#);
    let _=writeln!(out,r##"<defs><linearGradient id="world2" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#3D5143"/><stop offset=".5" stop-color="#1A2D38"/><stop offset="1" stop-color="#5B4030"/></linearGradient></defs><rect width="100%" height="100%" fill="url(#world2)"/>"##);
    for x in (0..w as i32).step_by(36) {
        let _=writeln!(out,r##"<rect x="{x}" y="0" width="18" height="{h}" fill="#AFC19C" opacity=".08"/>"##);
    }
    let scale=if w<450.0 {0.86}else{1.0};
    let bx=12.0; let by=29.0; let bar_w=104.0*scale; let bar_h=8.75*scale;
    let hp=if variant=="low_hp" {0.18}else{0.72};
    let stamina=if variant=="exhausted" {0.06}else{0.46};

    let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#CAEEFF">LV 47 · B · 17 PT</text>"##,bx+6.0*scale,by+9.0*scale);
    if variant=="exhausted" {
        let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#FFAA55">EXAUSTO</text>"##,bx+142.0*scale,by+9.0*scale);
    }
    for (i,(label,ratio,fill)) in [("HP",hp,"#E96975"),("SP",stamina,"#72B9F4")].into_iter().enumerate() {
        let y=by+(14.0+i as f32*19.0)*scale;
        let x=bx+25.0*scale;
        let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#E7F5FF">{label}</text>"##,bx+6.0*scale,y+7.0);
        let _=writeln!(out,r##"<rect x="{x:.1}" y="{:.1}" width="{bar_w:.1}" height="{bar_h:.1}" rx="1.8" fill="#0B1720" stroke="#547184" stroke-width=".7"/>"##,y);
        let _=writeln!(out,r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="1.2" fill="{fill}"/>"##,x+4.0*scale,y+2.7*scale,(bar_w-8.0*scale)*ratio,3.4*scale);
        let value=if label=="HP" {if variant=="low_hp"{"36/200"}else{"144/200"}} else {if variant=="exhausted"{"9/155"}else{"71/155"}};
        let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#EDF6FF">{value}</text>"##,x+bar_w+4.0*scale,y+7.0);
    }
    let hy=by+55.0*scale;
    let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#F7D58E">FOME</text>"##,bx+8.0*scale,hy+7.0);
    let _=writeln!(out,r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#392E28"/><rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="#F4BD68"/>"##,bx+41.0*scale,hy+2.0*scale,54.0*scale,4.0*scale,bx+41.0*scale,hy+2.0*scale,43.0*scale,4.0*scale);
    let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#B9D7E4">ARM 38</text>"##,bx+105.0*scale,hy+7.0);

    // XP at the vanilla hotbar position.
    let xp_w=(w-18.0).min(216.0); let xp_h=7.0*scale; let xp_x=(w-xp_w)*0.5; let xp_y=(h-24.0-xp_h).max(8.0);
    let _=writeln!(out,r##"<rect x="{xp_x:.1}" y="{xp_y:.1}" width="{xp_w:.1}" height="{xp_h:.1}" rx="1.5" fill="#0C1C20" stroke="#7F8D86" stroke-width=".6"/><rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="1" fill="#D7B25E"/>"##,xp_x+7.0,xp_y+2.0,(xp_w-14.0)*0.73,(xp_h-4.0).max(1.0));
    if h>=185.0 {
        let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#E9D990">LV 47 · 73%</text>"##,xp_x+xp_w*0.5-23.0,xp_y-3.0);
    }

    // Target / boss / damage telemetry. These use the same runtime dimensions and hierarchy.
    let boss=variant=="boss";
    let target=matches!(variant,"target"|"boss"|"damage"|"critical");
    let mut telemetry_y=7.0;
    if target {
        let cw=if boss {(w*0.46).clamp(190.0,330.0).min(w-14.0)} else {(w*0.24).clamp(142.0,190.0).min(w-12.0)};
        let ch=if boss{52.0}else{45.0};
        let x=if boss{((w-cw)*0.5).max(6.0)}else{(w-cw-7.0).max(5.0)};
        let y=7.0;
        let border=if boss {0xFFEFAF92}else{AETHER};
        let (bc,ba)=rgba(border);
        let _=writeln!(out,r##"<rect x="{:.1}" y="{y:.1}" width="{:.1}" height="{ch:.1}" fill="#0A0D13" fill-opacity=".72"/><rect x="{x:.1}" y="{:.1}" width="2" height="{:.1}" fill="{bc}" fill-opacity="{ba:.2}"/>"##,x+2.0,cw-2.0,y+3.0,ch-6.0);
        let _=writeln!(out,r##"<rect x="{:.1}" y="{:.1}" width="26" height="{:.1}" rx="2" fill="#16212B" stroke="#5E7D88" stroke-width=".6"/><text x="{:.1}" y="{:.1}" font-family="monospace" font-size="5" fill="#91A0A7">MOB 3D</text>"##,x+5.0,y+4.0,ch-9.0,x+7.0,y+18.0);
        let sx=x+39.0; let sw=(cw-46.0).max(42.0);
        let name=if boss{"COLOSSO RÚNICO"}else{"CÃO DE MANA"};
        let sub=if boss{"RANK S · FASE 2 · NV 88"}else{"RARO · NV 42"};
        let _=writeln!(out,r##"<text x="{sx:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#F4F6F5">{name}</text><text x="{sx:.1}" y="{:.1}" font-family="monospace" font-size="7" fill="#91A0A7">{sub}</text>"##,y+11.0,y+22.0);
        let hp=if boss{0.43}else{0.61}; let hp_y=y+28.0; let hp_h=if boss{7.0}else{6.0};
        let fill=if boss{"#53C7AA"}else{"#69D5D0"};
        let _=writeln!(out,r##"<rect x="{sx:.1}" y="{hp_y:.1}" width="{sw:.1}" height="{hp_h:.1}" fill="#070A0E"/><rect x="{sx:.1}" y="{hp_y:.1}" width="{:.1}" height="{hp_h:.1}" fill="{fill}"/>"##,sw*hp);
        if boss {
            for i in 1..5 { let cut=sx+sw*i as f32/5.0; let _=writeln!(out,r##"<rect x="{cut:.1}" y="{hp_y:.1}" width="1" height="{hp_h:.1}" fill="#0A0D13"/>"##); }
        }
        let hp_text=if boss{"430/1000 · 43%"}else{"91/150 · 61%"};
        let _=writeln!(out,r##"<text x="{sx:.1}" y="{:.1}" font-family="monospace" font-size="7" fill="#D7E3E7">{hp_text}</text>"##,y+44.0);
        telemetry_y=y+ch+2.0;
    }
    if matches!(variant,"damage"|"critical") {
        let tw=(w*0.22).clamp(112.0,176.0);
        let tx=if target {(w-(w*0.24).clamp(142.0,190.0).min(w-12.0)-7.0).max(5.0)+39.0}else{(w-tw-7.0).max(5.0)};
        let _=writeln!(out,r##"<text x="{tx:.1}" y="{:.1}" font-family="monospace" font-size="8" fill="#E6BD66">684.7 DMG · 12x COMBO</text><text x="{tx:.1}" y="{:.1}" font-family="monospace" font-size="7" fill="#F8F6F0">FÍSICO  54.3</text><text x="{tx:.1}" y="{:.1}" font-family="monospace" font-size="7" fill="#FFD45D">EXPLOSÃO  88.1</text>"##,telemetry_y+8.0,telemetry_y+19.0,telemetry_y+30.0);
    }
    if variant=="critical" {
        let cx=w*0.54; let cy=h*0.42;
        let _=writeln!(out,r##"<text x="{:.1}" y="{:.1}" font-family="monospace" font-size="12" font-weight="bold" fill="#FFDE51" stroke="#101820" stroke-width="1">★ -684.7</text>"##,cx,cy);
        for i in 0..6 { let a=i as f32*std::f32::consts::TAU/6.0; let px=cx+18.0*a.cos(); let py=cy-4.0+9.0*a.sin(); let _=writeln!(out,r##"<rect x="{px:.1}" y="{py:.1}" width="2" height="2" fill="#FFDE51"/>"##); }
    }
    let (gold,_)=rgba(GOLD); let (moon,_)=rgba(MOON); let (muted,_)=rgba(MUTED); let (white,_)=rgba(WHITE);
    let _=writeln!(out,r##"<text x="5" y="{:.1}" font-family="monospace" font-size="6" fill="{white}" opacity=".35">HUD · {variant} · {w}x{h}</text><!-- palette {gold} {moon} {muted} --></svg>"##,h-4.0);
    out
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
    ["VISÃO GERAL","PERSONAGEM","ATRIBUTOS","PROFISSÕES","EQUIPAMENTOS","ACESSÓRIOS","ENCANTAMENTOS","HABILIDADES","BESTIÁRIO","CONQUISTAS","HISTÓRICO"]
        .into_iter().map(str::to_string).collect()
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
    m.page=8;
    m.bestiary=(0..9).map(|i|SpeciesRow{
        name:format!("rustpg:criatura_{i}"),
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
    m.page=7;
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
    m.page=4;
    m.worn=vec![
        GearRow{id:1,slot:"head".into(),title:"Elmo do Horizonte".into(),ability:"Olhar Rúnico".into(),stat:"DEF".into(),bonus:14.5,rarity:3,power_status:"ATIVAR".into(),power_ready:true},
        GearRow{id:2,slot:"chest".into(),title:"Couraça da Fenda".into(),ability:"Barreira".into(),stat:"HP".into(),bonus:32.0,rarity:4,power_status:"CD 12s".into(),power_ready:false},
        GearRow{id:3,slot:"weapon".into(),title:"Lâmina do Eclipse".into(),ability:"Ruptura".into(),stat:"DMG".into(),bonus:27.0,rarity:5,power_status:"ATIVAR".into(),power_ready:true},
    ];
    m
}
fn sample(kind:&str,area:Rect)->MenuModel {
    match kind {
        "overview"=>base(area,"overview","VISÃO GERAL"),
        "status"=>base(area,"status","PERSONAGEM"),
        "attributes"=>base(area,"attributes","ATRIBUTOS"),
        "bestiary"=>bestiary(area),
        "trees"=>tree(area),
        "equipment"=>equipment(area),
        _=>base(area,"overview","VISÃO GERAL"),
    }
}

#[test]
fn visual_lab_exports_svg() {
    let Ok(root)=env::var("RUSTPG_UI_LAB_OUT") else { return; };
    let root=PathBuf::from(root);
    fs::create_dir_all(&root).unwrap();
    for (label,w,h) in [("phone",380.0,240.0),("wide",640.0,360.0)] {
        for kind in ["overview","status","attributes","bestiary","trees","equipment"] {
            let full=Rect{x:4.0,y:4.0,w:w-8.0,h:h-8.0};
            let main=crate::rustpg_inspector::main_area(full);
            let m=sample(kind,main);
            let mut layer=crate::rustpg_ui_window::draw_menu(&m);
            if let Some(side)=crate::rustpg_inspector::inspector_area(full) {
                let info=crate::rustpg_inspector::inspect(&m);
                let present=crate::rustpg_inspector::presentation(side,&info,0,50);
                layer.prims.extend(present.layer.prims);
                if let Some([x0,y0,x1,y1])=present.preview_rect {
                    layer.rect(Rect{x:x0,y:y0,w:x1-x0,h:y1-y0},0xB0091118);
                    layer.border(Rect{x:x0,y:y0,w:x1-x0,h:y1-y0},0x8057D9E8);
                    layer.text("ENTIDADE 3D",x0+3.0,y0+3.0,0xC0A7DDE8);
                }
            }
            let path=root.join(format!("{kind}-{label}.svg"));
            fs::write(&path,layer_svg(&layer,w,h,&format!("{kind} · {label}"))).unwrap();
        }
    }
    for (label,w,h) in [("phone",380.0,240.0),("wide",640.0,360.0)] {
        for variant in ["normal","low_hp","exhausted","target","boss","damage","critical"] {
            let path=root.join(format!("hud-{variant}-{label}.svg"));
            fs::write(&path,hud_scene_svg(w,h,variant)).unwrap();
        }
    }
    assert_eq!(fs::read_dir(Path::new(&root)).unwrap().count(),26);
}
