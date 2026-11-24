//! RustPG v4 contextual inspector. The split and hit geometry share the
//! MenuModel used for drawing and pointer dispatch; no separate fake UI state.
//! No Starling SDK dependency: all text/layout may be unit tested independently.
use super::rustpg_ui_layout::Rect;
use super::rustpg_ui_system::{Layer,text_width,alpha,CYAN,ICE,GOLD,GREEN,MUTED,WHITE,AMBER,LINE,SURFACE};
use super::rustpg_ui_window::{self as window, MenuModel, GearRow};

const GAP:f32=7.0;
const MIN_DOCK_HEIGHT:f32=62.0;
// The inspector is part of the real touch layout, not just a desktop hover.
// In narrow screens it becomes a compact bottom dock. The same geometries are
// used for drawing, scrolling and pointer handling.
fn mobile_dock_height(full:Rect)->f32 {
    // On 380x240 the old 76px minimum consumed almost a third of the gameplay
    // screen. The compact presentation below keeps the same contextual data
    // readable in 62-72px while returning precious height to the main page.
    (full.h*0.27).clamp(MIN_DOCK_HEIGHT,132.0).min((full.h-122.0).max(0.0))
}
pub fn main_area(full:Rect)->Rect {
    if full.w < 455.0 {
        let dock=mobile_dock_height(full);
        // Very short Android GUIs cannot afford a persistent dock. Pages that
        // need details already have an internal detail mode/selection surface.
        if dock<MIN_DOCK_HEIGHT {return full;}
        return Rect{x:full.x,y:full.y,w:full.w,h:(full.h-dock-GAP).max(25.0)};
    }
    // Preserve more map space on Android landscape screens.
    let sidebar=(full.w*0.25).clamp(152.0,225.0);
    Rect{x:full.x,y:full.y,w:full.w-sidebar-GAP,h:full.h}
}
pub fn inspector_area(full:Rect)->Option<Rect>{
    if full.w<455.0 {
        let dock=mobile_dock_height(full);
        return (dock>=MIN_DOCK_HEIGHT).then_some(Rect{
            x:full.x,y:full.y+full.h-dock,w:full.w,h:dock});
    }
    let main=main_area(full);
    Some(Rect{x:main.x+main.w+GAP,y:full.y,w:(full.x+full.w-main.x-main.w-GAP).max(0.0),h:full.h})
}

#[derive(Clone,Debug)]
pub struct Info {
    pub key:String,
    pub title:String,
    pub subtitle:String,
    pub paragraphs:Vec<String>,
    pub entity_kind:Option<String>,
    pub icon:&'static str,
    pub accent:u32,
    pub can_buy:bool,
}
fn info(key:&str,title:&str,subtitle:&str,icon:&'static str,accent:u32,paragraphs:Vec<String>)->Info {
    Info{key:key.into(),title:title.into(),subtitle:subtitle.into(),paragraphs,entity_kind:None,icon,accent,can_buy:false}
}
pub fn valid_kind(s:&str)->bool {
    let Some(id) = mod_api::Identifier::parse_namespaced(s) else {return false;};
    id.path().split('/').all(|part| !matches!(part, "" | "." | ".."))
}
fn page_hint(kind:&str)->(&'static str,&'static str){
    match kind {
        "overview"=>("Resumo do personagem","Nível, vitalidade, stamina, pontos, experiência e evolução geral. Os valores vêm do perfil sincronizado com o servidor."),
        "status"=>("Status detalhado","Atributos atuais e modificadores calculados. Os nós bloqueados não representam bônus ativos."),
        "attributes"=>("Atributos","Cada melhoria é individual. Passe o mouse em um atributo para ver seu valor; redefinir habilidades possui confirmação."),
        "trees"=>("Caminhos de evolução","Selecione uma escola. Os círculos representam talentos e as linhas representam pré-requisitos verdadeiros."),
        "skills"=>("Maestrias de armas","A experiência de arma é concedida por eventos reais. Habilidades exigem pontos e pré-requisitos validados pelo servidor."),
        "profs"=>("Profissões e maestrias","Mineração, agricultura, combate, exploração e outras atividades evoluem por ações confirmadas pelo servidor."),
        "bestiary"=>("Bestiário universal","Selecione uma criatura para consultar sua descrição, seus encontros e os registros de combate."),
        "inventory"=>("Bolsa de relíquias","Cada relíquia possui atributos, raridade e possíveis efeitos ativos. Equipe, examine ou recicle com confirmação."),
        "equipment"|"accessories"=>("Equipamentos e acessórios","Os poderes são aplicados somente quando o equipamento está no slot válido. Verifique a duração e a recarga."),
        "enchant"=>("Encantamentos","Bônus de equipamentos e progresso de encantamento, sujeitos às regras de acúmulo do RPG."),
        "evil"=>("Mutações","Efeitos e mutações descobertos nas criaturas; descobertas desconhecidas permanecem ocultas."),
        "achievements"=>("Conquistas","Marcos registrados no histórico e nas estatísticas de exploração e combate."),
        "history"=>("Histórico","Registro cronológico dos eventos de progresso e recompensas recentes."),
        _=>("Sistema RPG","Passe o mouse sobre um botão, item ou valor para abrir uma explicação contextual."),
    }
}
fn gear_info(g:&GearRow,prefix:&str)->Info {
    info(&format!("relic:{}",g.id),&g.title,prefix,
        "rustpg:gui/icons/attributes/icon_luck",GOLD,
        vec![format!("Slot: {} · raridade: {}",g.slot,g.rarity),
            format!("Efeito: {}",g.ability),
            format!("Atributo: {} · bônus: +{:.2}",g.stat,g.bonus),
            format!("Habilidade ativa: {}",g.power_status)])
}
pub fn talent_info(n:&window::TreeNode,available_points:u64)->Info {
    let status=match n.state {3=>"NÍVEL MÁXIMO",2=>"ADQUIRIDA",1=>"DISPONÍVEL",_=>"BLOQUEADA"};
    let mut paragraphs=vec![n.description.clone(), format!("{status} · Grau {}/{}",n.rank,n.max)];
    if n.state != 3 {
        paragraphs.push(format!("Próximo nível: {} PT · saldo: {} PT",n.cost,available_points));
        paragraphs.push(n.requirement.clone());
    }
    let mut item=info(&format!("node:{}",n.id),&n.name,"TALENTO",
    "rustpg:gui/icons/attributes/icon_strength",if n.buyable{GREEN}else{AMBER},paragraphs);
    item.can_buy=n.buyable;
    item
}

pub fn inspect(m:&MenuModel)->Info {
    let c=window::content(window::window(m.area),m.pages.len());
    let hovered_action=window::menu_hits(m).into_iter()
        .find(|(r,_)|r.contains(m.mouse)).map(|(_,a)|a);
    if let Some(ref action)=hovered_action {
        if let Some(index)=action.strip_prefix("page:").and_then(|s|s.parse::<usize>().ok()) {
            if let Some(name)=m.pages.get(index){
                let kind=crate::rustpg_view::PAGES.get(index).map_or("overview",|p|p.0);
                let (hint,detail)=page_hint(kind);
                return info(&format!("tab:{index}"),name,"ABA DO SISTEMA",
                    "rustpg:gui/hud/emblem/hud_player_emblem",CYAN,
                    vec![hint.into(),detail.into(),"Toque para abrir esta aba.".into()]);
            }
        }
    }
    if m.kind=="trees" {
        if let Some(t)=m.tree.as_ref() {
            if t.index_mode {
                if let Some((_,i))=window::constellation_categories(m).into_iter()
                    .find(|(r,_)|r.contains(m.mouse)) {
                    if let Some(name)=super::rustpg_nodes::trees().get(i) {
                        return info(&format!("school:{name}"),name,"ESCOLA DE HABILIDADES",
                            "rustpg:gui/icons/attributes/icon_intelligence",GOLD,
                            vec!["Toque para examinar as ramificações desta escola.".into(),
                                "Cada talento possui custo, grau e pré-requisitos individuais.".into()]);
                    }
                }
            } else if let Some(n)=t.nodes.get(t.selected) {
                return talent_info(n,t.available_points);
            }
        }
    }
    if m.kind=="bestiary" {
        // Pin details to selected entry rather than transient finger hover.
        if let Some(s)=m.bestiary.get(m.bestiary_sel) {
            let mut p=vec![s.description.clone(),format!("Identificador: {}",s.name),
                format!("Abates registrados: {}",s.kills),
                if s.best>0.0 {format!("Maior golpe causado nesta espécie: {:.1}",s.best)} else {"Maior golpe: ainda não medido".into()},
                format!("Classificações: {}",if s.tiers.is_empty(){"Não identificadas"}else{&s.tiers}),
                format!("Drops conhecidos: {}",if s.drops.is_empty(){"Ainda não registrados"}else{&s.drops})];
            if s.level > 0 {p.push(format!("Maior nível observado: {}",s.level));}
            let mut result=info(&format!("species:{}",s.name),&s.display_name,"ENTIDADE · BESTIÁRIO",
                "rustpg:gui/icons/professions/icon_exploration",CYAN,p);
            result.entity_kind=valid_kind(&s.name).then(||s.name.clone());
            return result;
        }
    }
    if m.kind=="inventory" {
        let per=window::gear_rows_per_page(c);
        let page=m.scroll.min(m.bag.len().div_ceil(per).max(1)-1);
        for (i,g) in m.bag.iter().skip(page*per).take(per).enumerate() {
            if window::inventory_row_rect(c,i).contains(m.mouse) {return gear_info(g,"RELÍQUIA NA BOLSA");}
        }
    }
    if m.kind=="equipment" || m.kind=="accessories" {
        let slots:&[&str]=if m.kind=="accessories" { &["ring","neck","charm"] }
            else { &["head","chest","legs","feet","weapon","ring","neck","charm"] };
        for (i,r) in window::equipped_visible_rects(c,slots.len(),m.scroll) {
            if r.contains(m.mouse) {
                if let Some(g)=m.worn.iter().find(|g|g.slot==slots[i]) {return gear_info(g,"RELÍQUIA EQUIPADA");}
                return info(&format!("slot:{}",slots[i]),slots[i],"SLOT VAZIO",
                    "rustpg:gui/icons/attributes/icon_defense",MUTED,
                    vec!["Equipe uma relíquia compatível com este slot pela bolsa de equipamentos.".into()]);
            }
        }
    }
    // Attribute cards use a grid rather than data_layout; recover their
    // exact cell geometry instead of returning a generic page description.
    if matches!(m.kind,"attributes"|"status") {
        for (i,a) in m.attrs.iter().enumerate() {
            let r=window::attribute_cell(c,m,i);
            if r.contains(m.mouse) {
                return info(&format!("attr:{}",a.name),&a.name,"ATRIBUTO DO PERSONAGEM",
                    "rustpg:gui/icons/attributes/icon_strength",CYAN,
                    vec![a.hint.clone(),format!("Valor atual: {}",a.value)]);
            }
        }
    }
    if let Some(ref act)=hovered_action {
        let detail=match act.as_str(){
            "buy"=>"Compra validada no servidor. Pontos só devem ser consumidos se o próximo nível produzir um efeito real.",
            "respec"=>"Redefinição confirmada em dois cliques. Restitui pontos respeitando a origem da compra.",
            "catalog"=>"Selecione uma escola, consulte caminhos conectados e volte à árvore sem perder seu perfil.",
            "details"=>"Exibe a descrição integral do nó selecionado, suas fórmulas e requisitos.",
            "rpage:1"=>"Ir à página seguinte desta seção; nenhum talento será descartado.",
            "rpage:-1"=>"Voltar à página anterior da seção.",
            "tree:1"|"tree:-1"=>"Percorre as escolas de habilidade mantendo as categorias independentes.",
            "tgroup:1"|"tgroup:-1"=>"Mostra mais abas do sistema sem alterar os dados do personagem.",
            a if a.starts_with("equip:")=>"Equipa uma relíquia selecionada, ativando suas regras no servidor.",
            a if a.starts_with("salvage:")=>"Recicla a relíquia com confirmação e concede somente a recompensa autorizada pelo servidor.",
            a if a.starts_with("unequip:")=>"Retira a relíquia do slot e remove seus modificadores próprios.",
            a if a.starts_with("activate:")=>"Ativa o poder da relíquia, se houver stamina suficiente e a recarga estiver concluída.",
            _=>"Interação de navegação desta tela. Clique para executar sua ação.",
        };
        return info(&format!("action:{act}"),&act.replace('_'," ").to_uppercase(),"CONTROLE INTERATIVO",
            "rustpg:gui/icons/attributes/icon_precision",CYAN,vec![detail.to_string()]);
    }
    // All informational pages use the SAME plate geometry as their renderer.
    // This also covers mastery, professions, history, enchanting and other data pages.
    for (page,r,index) in window::data_layout(c,&m.lines) {
        if page==m.scroll && r.contains(m.mouse) {
            if let Some(raw)=m.lines.get(index) {
                let (title,details)=page_hint(m.kind);
                return info(&format!("row:{}:{}",m.kind,index),raw,title,
                    "rustpg:gui/icons/attributes/icon_intelligence",CYAN,
                    vec![details.into()]);
            }
        }
    }
    // Entity viewer can follow the current live combat target from ANY page.
    if let Some(ref t)=m.live_target {
        let mut target=info(&format!("target:{}",t.entity),&t.name,"ALVO ATUAL · VISUALIZAÇÃO 3D",
            "rustpg:gui/icons/attributes/icon_defense",GREEN,
            vec![format!("Tipo: {}",t.kind),format!("Nível: {} · categoria: {}",t.level,t.tier),
                format!("HP: {:.1}/{:.1} · armadura: {:.1}",t.current_hp,t.max_hp,t.armor)]);
        target.entity_kind=valid_kind(&t.kind).then(||t.kind.clone());
        return target;
    }
    let (title,body)=page_hint(m.kind);
    info(&format!("page:{}",m.kind),title,&m.title.to_uppercase(),
        "rustpg:gui/hud/emblem/hud_player_emblem",CYAN,
        vec![body.into(),format!("Nível: {} · rank: {} · pontos livres: {}",m.level,m.rank,m.points)])
}

fn wrap(text:&str,max_width:f32)->Vec<String>{
    let mut out=Vec::new();
    let mut current=String::new();
    for paragraph in text.split('\n') {
        if !current.is_empty(){out.push(std::mem::take(&mut current));}
        for word in paragraph.split_whitespace() {
            if text_width(word)>max_width {
                if !current.is_empty(){out.push(std::mem::take(&mut current));}
                for ch in word.chars(){
                    let mut next=current.clone();next.push(ch);
                    if text_width(&next)>max_width && !current.is_empty(){out.push(std::mem::take(&mut current));}
                    current.push(ch);
                }
            }else{
                let next=if current.is_empty(){word.to_owned()} else {format!("{current} {word}")};
                if text_width(&next)>max_width && !current.is_empty(){out.push(std::mem::take(&mut current));current=word.to_owned();}
                else {current=next;}
            }
        }
        if !current.is_empty(){out.push(std::mem::take(&mut current));}
        out.push(String::new());
    }
    while out.last().is_some_and(String::is_empty){out.pop();}
    out
}
fn fit_title(text:&str,max_width:f32)->String {
    if text_width(text)<=max_width{return text.to_owned();}
    let mut out=String::new();
    let budget=max_width-text_width("…");
    for ch in text.chars(){
        if text_width(&out)+text_width(&ch.to_string())>budget {break;}
        out.push(ch);
    }
    out.push('…');
    out
}

#[derive(Clone,Debug)]
pub struct Presentation {
    pub layer:Layer,
    pub preview_kind:Option<String>,
    pub preview_rect:Option<[f32;4]>,
    pub max_scroll:usize,
    pub visible_rows:usize,
}

const PAD:f32=8.0;
const FOOT:f32=18.0;
const LINE_HEIGHT:f32=11.0;

pub fn scroll_button(rect:Rect,forward:bool)->Rect {
    Rect{x:rect.x+rect.w-PAD-if forward {21.0} else {45.0},
        y:rect.y+rect.h-FOOT-3.0,w:21.0,h:FOOT}
}

pub fn buy_button(rect:Rect)->Rect {
    Rect{x:rect.x+PAD,y:rect.y+rect.h-FOOT-3.0,w:(rect.w-2.0*PAD-49.0).min(104.0),h:FOOT}
}

fn row_count(top:f32,bottom:f32)->usize {
    if bottom-top<9.0 {0} else {1+((bottom-top-9.0)/LINE_HEIGHT).floor() as usize}
}

pub fn presentation(rect:Rect,info:&Info,scroll:usize,_tick:u32)->Presentation {
    let mut l=Layer::new();
    l.chamfer(rect,3.0,SURFACE);
    l.chamfer_outline(rect,3.0,1.0,alpha(LINE,0xAA));
    let inner=rect.inset(PAD);
    // A phone dock spends one line on the heading. Sidebars can afford a
    // wrapped title, an identifying icon and a full illustration above the copy.
    let dock=rect.w>rect.h*1.65;
    let title_x=inner.x+if dock {0.0} else {23.0};
    let title_width=inner.w-(title_x-inner.x);
    let title_lines=wrap(&info.title,title_width);
    let title_rows=title_lines.len().clamp(1,if dock {1} else {3});
    let title_y=if dock {rect.y+5.0} else {inner.y+13.0};
    if !dock {
        l.text_flat(fit_title(&info.subtitle,inner.w),inner.x,inner.y,info.accent);
        l.sprite(info.icon,[0.0,0.0,128.0,128.0],Rect{x:inner.x,y:title_y-3.0,w:17.0,h:17.0});
    }
    let mut visible_title=title_lines.iter().take(title_rows).cloned().collect::<Vec<_>>();
    if title_lines.len()>title_rows {
        let last=title_rows-1;
        visible_title[last]=fit_title(&title_lines[last..].join(" "),title_width);
    }
    for (i,line) in visible_title.iter().enumerate() {
        l.text_flat(line,title_x,title_y+i as f32*LINE_HEIGHT,WHITE);
    }
    let rule_y=title_y+title_rows as f32*LINE_HEIGHT+if dock {0.0} else {4.0};
    l.hline(inner.x,rule_y,inner.w,alpha(LINE,0x72));
    l.hline(inner.x,rule_y,inner.w.min(17.0),GOLD);
    let mut body_top=rule_y+if dock {4.0} else {8.0};
    let mut body_x=inner.x;
    let mut body_width=inner.w;
    let has_model=info.entity_kind.is_some();
    let preview=if has_model && dock {
        let size=(rect.h-22.0).clamp(30.0,76.0).min(inner.w*0.24);
        body_x+=size+7.0;
        body_width-=size+7.0;
        Some(Rect{x:inner.x,y:body_top,w:size,h:(rect.y+rect.h-4.0-body_top).min(size)})
    } else if has_model && rect.h-body_top+rect.y>=114.0 {
        let height=(rect.h*0.22).clamp(42.0,100.0);
        let pr=Rect{x:inner.x,y:body_top,w:inner.w,h:height};
        body_top+=height+7.0;
        Some(pr)
    } else {None};
    let preview_rect=preview.map(|r|{
        l.chamfer(r,2.0,alpha(0xFF131F25,0xFE));
        l.chamfer_outline(r,2.0,1.0,alpha(LINE,0x70));
        let mid=r.x+r.w*0.5;
        l.hline(mid-r.w*0.22,r.y+r.h-3.0,r.w*0.44,alpha(info.accent,0x50));
        [r.x+2.0,r.y+2.0,r.x+r.w-2.0,r.y+r.h-3.0]
    });

    let mut lines=Vec::new();
    if title_lines.len()>title_rows {
        lines.extend(wrap(&info.title,body_width));
    }
    for (index,p) in info.paragraphs.iter().enumerate() {
        if p.trim().is_empty() {continue;}
        if !dock && index>0 && !lines.is_empty() {lines.push(String::new());}
        lines.extend(wrap(p,body_width));
    }
    let bottom=rect.y+rect.h-if dock {4.0} else {PAD};
    let node=info.key.starts_with("node:");
    let overflow=lines.len()>row_count(body_top,bottom);
    let footer=node||overflow;
    let body_bottom=if footer {scroll_button(rect,true).y-2.0} else {bottom};
    let rows=row_count(body_top,body_bottom);
    let max_scroll=lines.len().saturating_sub(rows);
    let start=scroll.min(max_scroll);
    l.set_clip(Some(Rect{x:body_x,y:body_top,w:body_width,h:(body_bottom-body_top).max(0.0)}));
    for (row,line) in lines.iter().skip(start).take(rows).enumerate() {
        l.text_flat(line,body_x,body_top+row as f32*LINE_HEIGHT,ICE);
    }
    l.set_clip(None);

    if node {
        let state=if info.can_buy {super::rustpg_ui_theme::UiState::Available}
            else {super::rustpg_ui_theme::UiState::Disabled};
        l.state_button(buy_button(rect),if info.can_buy {"ADQUIRIR"} else {"INDISPONÍVEL"},state,GREEN);
    }
    if max_scroll>0 {
        if !node {
            let b=scroll_button(rect,false);
            let track=Rect{x:body_x,y:b.y+8.0,w:(b.x-8.0-body_x).max(0.0),h:2.0};
            l.rect(track,alpha(LINE,0x8A));
            let thumb=(track.w*rows as f32/lines.len() as f32).max(4.0).min(track.w);
            l.rect(Rect{x:track.x+(track.w-thumb)*start as f32/max_scroll as f32,w:thumb,..track},info.accent);
        }
        for forward in [false,true] {
            let b=scroll_button(rect,forward);
            let active=if forward {start<max_scroll} else {start>0};
            let ink=if active {ICE} else {alpha(MUTED,0x8F)};
            l.chamfer(b,2.0,alpha(LINE,if active {0x50} else {0x1A}));
            if forward {l.chevron(b.x+8.0,b.y+5.0,5.0,8.0,ink);}
            else {l.chevron_left(b.x+7.0,b.y+5.0,5.0,8.0,ink);}
        }
    }
    Presentation{layer:l,preview_kind:info.entity_kind.clone(),preview_rect,max_scroll,visible_rows:rows}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn layouts_reserve_real_sidebar_only_on_wide_screens(){
        for w in [455.0,490.0,760.0,1280.0] {
            let a=Rect{x:4.0,y:4.0,w,h:480.0};
            let l=main_area(a);let r=inspector_area(a).unwrap();
            assert!(l.w>=250.0); assert!(r.w>=152.0);assert!(l.x+l.w<r.x);
            assert!((r.x+r.w-a.x-a.w).abs()<0.02);
        }
        let phone=Rect{x:4.0,y:4.0,w:380.0,h:240.0};
        let main=main_area(phone);
        let dock=inspector_area(phone).expect("mobile inspector must be visible");
        assert!(main.y+main.h<dock.y);
        assert!(dock.h>=62.0);
        assert_eq!(dock.w,phone.w);

        let compact=Rect{x:4.0,y:4.0,w:320.0,h:180.0};
        assert_eq!(main_area(compact),compact);
        assert!(inspector_area(compact).is_none(),
            "ultra-short layouts must return the height to the primary screen");
    }
    #[test] fn text_wrap_never_breaks_panel_bounds(){
        let s="Habilidades com efeitos permanentes e descrições detalhadas para qualquer criatura";
        assert!(wrap(s,110.0).iter().all(|t|text_width(t)<=110.5));
        assert!(valid_kind("mod_name:giant/boss"));
        assert!(valid_kind("third-party.mod:giant/boss"));
        assert!(!valid_kind("evil:../../foo"));
    }
    #[test] fn mobile_description_is_immediately_readable_and_pageable(){
        use super::super::rustpg_ui_window::SpeciesRow;
        let area=Rect{x:4.0,y:4.0,w:380.0,h:310.0};
        let m=MenuModel{kind:"bestiary",bestiary:vec![SpeciesRow{
            name:"minecraft:zombie".into(),
            display_name:"Zombie".into(),
            description:"Criatura registrada pelo Bestiário.".into(),
            live_entity:None,kills:3,level:4,best:7.5,
            tiers:"Normal".into(),drops:"minecraft:rotten_flesh".into(),
            discovery:0.02}],..Default::default()};
        let selected=inspect(&m);
        assert!(selected.key.starts_with("species:"));
        assert!(selected.paragraphs.iter().any(|p|p.contains("Abates registrados: 3")));
        let rect=inspector_area(area).unwrap();
        let early=presentation(rect,&selected,0,0);
        let late=presentation(rect,&selected,0,1000);
        assert!(late.preview_rect.is_some());
        let count=|p:&Presentation|p.layer.prims.iter().filter_map(|prim|match prim{
            super::super::rustpg_ui_system::Prim::Text{text,..}=>Some(text.chars().count()),
            _=>None}).sum::<usize>();
        assert_eq!(count(&late),count(&early));
        assert!(early.visible_rows>=2);
        assert!(early.max_scroll>0);
        let last=presentation(rect,&selected,usize::MAX,0);
        assert!(last.layer.prims.iter().any(|prim|matches!(prim,
            super::super::rustpg_ui_system::Prim::Text{text,..} if text.contains("observado: 4"))));
        assert!(early.preview_rect.unwrap()[2]<scroll_button(rect,false).x);
        assert!(buy_button(rect).x+buy_button(rect).w<scroll_button(rect,false).x);
    }
    #[test] fn contextual_fallback_for_every_page(){
        for &(kind,_) in crate::rustpg_view::PAGES.iter() {
            let m=MenuModel{kind,..Default::default()};
            assert!(!inspect(&m).paragraphs.is_empty());
        }
    }
}
