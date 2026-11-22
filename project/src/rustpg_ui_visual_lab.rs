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
    let _=writeln!(out,r#"<defs><linearGradient id="world" x1="0" y1="0" x2="1" y2="1"><stop stop-color="#35483d"/><stop offset=".48" stop-color="#182c36"/><stop offset="1" stop-color="#4a382b"/></linearGradient></defs>"#);
    let _=writeln!(out,r#"<rect width="100%" height="100%" fill="url(#world)"/><g opacity=".18">"#);
    for x in (0..w as i32).step_by(32) { let _=writeln!(out,r#"<rect x="{x}" y="0" width="16" height="{h}" fill="#9bb99a"/>"#); }
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
                    let _=writeln!(out,r#"<text x="{:.2}" y="{:.2}" font-family="monospace" font-size="8" fill="#000" fill-opacity="{:.3}">{}</text>"#,x+1.0,y+8.0+1.0,a*0.72,esc(text));
                }
                let _=writeln!(out,r#"<text x="{x:.2}" y="{:.2}" font-family="monospace" font-size="8" fill="{rgb}" fill-opacity="{a:.3}">{}</text>"#,y+8.0,esc(text));
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
                    let _=writeln!(out,r#"<text x="{:.2}" y="{:.2}" font-family="monospace" font-size="5" fill="#91A0A7" opacity=".6">{}</text>"#,x+2.0,y+6.0,esc(label));
                }
            }
        }
    }
    let _=writeln!(out,r#"<text x="5" y="{:.1}" font-family="monospace" font-size="6" fill="#fff" opacity=".3">{}</text></svg>"#,h-4.0,esc(title));
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
        Vital{label:"HP".into(),ratio:.72,value:"144/200".into()},
        Vital{label:"SP".into(),ratio:.46,value:"71/155".into()},
        Vital{label:"ABS".into(),ratio:.24,value:"24".into()},
    ]
}
fn pages()->Vec<String> {
    ["VISÃO GERAL","PERSONAGEM","ATRIBUTOS","PROFISSÕES","EQUIPAMENTOS","ACESSÓRIOS","ENCANTAMENTOS","HABILIDADES","BESTIÁRIO","CONQUISTAS","HISTÓRICO"]
        .into_iter().map(str::to_string).collect()
}
fn base(area:Rect,kind:&'static str,title:&str)->MenuModel {
    MenuModel{
        area,mouse:[-100.0,-100.0],tick:37,page:0,pages:pages(),kind,title:title.into(),
        level:47,rank:"B".into(),next_rank:"A".into(),rank_progress:.61,xp:.73,points:17,
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
        discovery:(.18+i as f32*.1).min(1.0),
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
    m.tree=Some(TreeModel{name:"ESPADA".into(),nodes,available_points:17,links,pan:[0.0,0.0],zoom:.78,selected:6,index_mode:false,detail_only:false,detail:vec![]});
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
            let area=Rect{x:4.0,y:4.0,w:w-8.0,h:h-8.0};
            let m=sample(kind,area);
            let layer=crate::rustpg_ui_window::draw_menu(&m);
            let path=root.join(format!("{kind}-{label}.svg"));
            fs::write(&path,layer_svg(&layer,w,h,&format!("{kind} · {label}"))).unwrap();
        }
    }
    assert_eq!(fs::read_dir(Path::new(&root)).unwrap().count(),12);
}
