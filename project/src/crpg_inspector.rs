//! Claude RPG v4 contextual inspector. The split and hit geometry share the
//! MenuModel used for drawing and pointer dispatch; no separate fake UI state.
//! No Starling SDK dependency: all text/layout may be unit tested independently.
use super::crpg_ui_layout::Rect;
use super::crpg_ui_system::{Layer,text_width,CYAN,ICE,GOLD,GREEN,MUTED,WHITE,AMBER};
use super::crpg_ui_window::{self as window, MenuModel, GearRow};

const GAP:f32=7.0;
pub fn main_area(full:Rect)->Rect {
    if full.w < 455.0 {return full;}
    let sidebar=(full.w*0.305).clamp(152.0,280.0);
    Rect{x:full.x,y:full.y,w:full.w-sidebar-GAP,h:full.h}
}
pub fn inspector_area(full:Rect)->Option<Rect>{
    if full.w<455.0 {return None;}
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
fn valid_kind(s:&str)->bool {
    let Some((ns,path))=s.split_once(':')else{return false;};
    !ns.is_empty() && !path.is_empty() && !path.contains("..")
    && !path.starts_with('/') && !path.ends_with('/') && s.len()<150 &&
    ns.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_') &&
    path.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||matches!(b,b'_'|b'/'|b'.'|b'-'))
}
fn page_hint(kind:&str)->(&'static str,&'static str){
    match kind {
        "overview"=>("Resumo do personagem","Nível, vitalidade, stamina, pontos, experiência e evolução geral. Os valores vêm do perfil sincronizado com o servidor."),
        "status"=>("Status detalhado","Atributos atuais e modificadores calculados. Os nós bloqueados não representam bônus ativos."),
        "attributes"=>("Atributos","Cada melhoria é individual. Passe o mouse em um atributo para ver seu valor; redefinir habilidades possui confirmação."),
        "trees"=>("Caminhos de evolução","Selecione uma escola. Os círculos representam talentos e as linhas representam pré-requisitos verdadeiros."),
        "skills"=>("Maestrias de armas","A experiência de arma é concedida por eventos reais. Habilidades exigem pontos e pré-requisitos validados pelo servidor."),
        "profs"=>("Profissões e maestrias","Mineração, agricultura, combate, exploração e outras atividades evoluem por ações confirmadas pelo servidor."),
        "bestiary"=>("Bestiário universal","Espécies desbloqueadas somente após um abate confirmado pelo servidor, inclusive criaturas de mods. Aproximação não desbloqueia."),
        "inventory"=>("Bolsa de relíquias","Cada relíquia possui atributos, raridade e possíveis efeitos ativos. Equipe, examine ou recicle com confirmação."),
        "equipment"|"accessories"=>("Equipamentos e acessórios","Os poderes são aplicados somente quando o equipamento está no slot válido. Verifique a duração e a recarga."),
        "daily"=>("Missões diárias","Acompanhe progresso, metas, recompensas e limite de tentativas das missões do dia."),
        "quests"=>("Jornada e missões","Missões com progresso salvo no servidor, requisitos e recompensas de experiência."),
        "enchant"=>("Encantamentos","Bônus de equipamentos e progresso de encantamento, sujeitos às regras de acúmulo do RPG."),
        "evil"=>("Mutações","Efeitos e mutações descobertos nas criaturas; descobertas desconhecidas permanecem ocultas."),
        "achievements"=>("Conquistas","Marcos registrados no histórico e nas estatísticas de exploração e combate."),
        "history"=>("Histórico","Registro cronológico dos eventos de progresso e recompensas recentes."),
        _=>("Sistema RPG","Passe o mouse sobre um botão, item ou valor para abrir uma explicação contextual."),
    }
}
fn gear_info(g:&GearRow,prefix:&str)->Info {
    info(&format!("relic:{}",g.id),&g.title,prefix,
        "claude_rpg:gui/icons/attributes/icon_luck",GOLD,
        vec![format!("Slot: {} · raridade: {}",g.slot,g.rarity),
            format!("Efeito: {}",g.ability),
            format!("Atributo: {} · bônus: +{:.2}",g.stat,g.bonus),
            format!("Habilidade ativa: {}",g.power_status),
            "Ao equipar ou ativar, o servidor valida custo, duração e recarga. Alterações de interface não concedem poder.".into()])
}
pub fn inspect(m:&MenuModel)->Info {
    let c=window::content(window::window(m.area),m.pages.len());
    let hovered_action=window::menu_hits(m).into_iter()
        .find(|(r,_)|r.contains(m.mouse)).map(|(_,a)|a);
    if let Some(ref action)=hovered_action {
        if let Some(index)=action.strip_prefix("page:").and_then(|s|s.parse::<usize>().ok()) {
            if let Some(name)=m.pages.get(index){
                let kind=crate::crpg_view::PAGES.get(index).map_or("overview",|p|p.0);
                let (hint,detail)=page_hint(kind);
                return info(&format!("tab:{index}"),name,"ABA DO SISTEMA",
                    "claude_rpg:gui/hud/emblem/hud_player_emblem",CYAN,
                    vec![hint.into(),detail.into(),"Toque para abrir esta aba.".into()]);
            }
        }
    }
    if m.kind=="trees" {
        if let Some(t)=m.tree.as_ref() {
            if t.index_mode {
                if let Some((_,i))=window::constellation_categories(m).into_iter()
                    .find(|(r,_)|r.contains(m.mouse)) {
                    if let Some(name)=super::crpg_nodes::trees().get(i) {
                        return info(&format!("school:{name}"),name,"ESCOLA DE HABILIDADES",
                            "claude_rpg:gui/icons/attributes/icon_intelligence",GOLD,
                            vec!["Toque para examinar as ramificações desta escola.".into(),
                                "Cada talento possui custo, grau e pré-requisitos individuais.".into()]);
                    }
                }
            } else if let Some(n)=window::constellation_node_rects(m).into_iter()
                .find(|(r,_)|r.contains(m.mouse)).and_then(|(_,i)|t.nodes.get(i))
                .or_else(||t.nodes.get(t.selected)) {
                let status=match n.state {3=>"NÍVEL MÁXIMO",2=>"ADQUIRIDA",1=>"DISPONÍVEL",_=>"BLOQUEADA"};
                let mut paragraphs=vec![format!("Estado: {status} · Rank: {}/{}",n.rank,n.max),
                    format!("Custo do próximo nível: {} pontos",n.cost),n.description.clone(),
                    format!("Requisitos: {}",n.requirement)];
                if !n.buyable {paragraphs.push("Compra indisponível: não gastar pontos enquanto requisitos ou efeito estiverem ausentes.".into());}
                let mut item=info(&format!("node:{}",n.id),&n.name,"HABILIDADE · EFEITO REAL",
                    "claude_rpg:gui/icons/attributes/icon_strength",if n.buyable{GREEN}else{AMBER},paragraphs);
                item.can_buy=n.buyable;
                return item;
            }
        }
    }
    if m.kind=="bestiary" {
        let (left,_) = window::split(c);
        let rows=((left.h-26.0)/16.0).floor().max(1.0) as usize;
        let start=m.bestiary_sel.saturating_sub(rows.saturating_sub(1));
        let hit=(start..m.bestiary.len().min(start+rows)).find(|i| {
            let r=Rect{x:left.x+6.0,y:left.y+20.0+(*i-start) as f32*16.0,
                w:left.w-12.0,h:15.0};r.contains(m.mouse)
        });
        if let Some(s)=m.bestiary.get(hit.unwrap_or(m.bestiary_sel)) {
            let mut p=vec![format!("Identificador: {}",s.name),
                format!("Abates registrados: {} · maior nível visto: {}",s.kills,s.level.max(1)),
                if s.best>0.0 {format!("Maior golpe causado nesta espécie: {:.1}",s.best)} else {"Maior golpe: ainda não medido".into()},
                format!("Classificações: {}",if s.tiers.is_empty(){"Não identificadas"}else{&s.tiers}),
                format!("Drops conhecidos: {}",if s.drops.is_empty(){"Ainda não registrados"}else{&s.drops})];
            if !s.name.starts_with("minecraft:") {
                p.push("Criatura externa: o modelo será solicitado pelo ID registrado, sem lista fixa de mobs vanilla.".into());
            }
            let mut result=info(&format!("species:{}",s.name),&s.name,"ENTIDADE · BESTIÁRIO",
                "claude_rpg:gui/icons/professions/icon_exploration",CYAN,p);
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
                    "claude_rpg:gui/icons/attributes/icon_defense",MUTED,
                    vec!["Equipe uma relíquia compatível com este slot pela bolsa de equipamentos.".into()]);
            }
        }
    }
    // Attribute cards use a grid rather than data_layout; recover their
    // exact cell geometry instead of returning a generic page description.
    if m.kind=="attributes" && !m.attrs.is_empty() {
        let cols=if c.w>=300.0{2}else{1};
        let rows=m.attrs.len().div_ceil(cols).max(1);
        let cw=(c.w-16.0-6.0*(cols as f32-1.0))/cols as f32;
        let rh=((c.h-18.0-4.0-(c.y+24.0-c.y))/rows as f32).clamp(13.0,26.0);
        for (i,a) in m.attrs.iter().enumerate() {
            let col=i/rows;let row=i%rows;
            let r=Rect{x:c.x+8.0+col as f32*(cw+6.0),
                y:c.y+24.0+row as f32*rh,w:cw,h:rh-2.0};
            if r.contains(m.mouse) {
                return info(&format!("attr:{}",a.name),&a.name,"ATRIBUTO DO PERSONAGEM",
                    "claude_rpg:gui/icons/attributes/icon_strength",CYAN,
                    vec![format!("Valor atual: {}",a.value),format!("Descrição: {}",a.hint),
                        "Sua contribuição depende dos nós efetivamente implementados e dos modificadores válidos no servidor.".into()]);
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
            "claude_rpg:gui/icons/attributes/icon_precision",CYAN,vec![detail.to_string()]);
    }
    // All informational pages use the SAME plate geometry as their renderer.
    // This also covers mastery, professions, history, quest data, enchanting etc.
    for (page,r,index) in window::data_layout(c,&m.lines) {
        if page==m.scroll && r.contains(m.mouse) {
            if let Some(raw)=m.lines.get(index) {
                let (title,details)=page_hint(m.kind);
                return info(&format!("row:{}:{}",m.kind,index),raw,title,
                    "claude_rpg:gui/icons/attributes/icon_intelligence",CYAN,
                    vec![raw.clone(),details.into(),
                        "Exibido a partir do estado atual do personagem, não de uma estimativa visual.".into()]);
            }
        }
    }
    // Entity viewer can follow the current live combat target from ANY page.
    if let Some(ref t)=m.live_target {
        let mut target=info(&format!("target:{}",t.entity),&t.name,"ALVO ATUAL · VISUALIZAÇÃO 3D",
            "claude_rpg:gui/icons/attributes/icon_defense",GREEN,
            vec![format!("Tipo: {}",t.kind),format!("Nível: {} · categoria: {}",t.level,t.tier),
                format!("HP: {:.1}/{:.1} · armadura: {:.1}",t.current_hp,t.max_hp,t.armor),
                "A miniatura usa o renderizador de entidades do Starling. HP e armadura vêm dos dados sincronizados pelo servidor.".into()]);
        target.entity_kind=valid_kind(&t.kind).then(||t.kind.clone());
        return target;
    }
    let (title,body)=page_hint(m.kind);
    info(&format!("page:{}",m.kind),title,&m.title.to_uppercase(),
        "claude_rpg:gui/hud/emblem/hud_player_emblem",CYAN,
        vec![body.into(),format!("Nível: {} · rank: {} · pontos livres: {}",m.level,m.rank,m.points),
            "Passe o mouse sobre qualquer aba, talento, criatura, relíquia, controle ou valor para ver as informações completas.".into()])
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
#[derive(Clone,Debug)]
pub struct Presentation {pub layer:Layer,pub preview_kind:Option<String>,pub preview_rect:Option<[f32;4]>,pub max_scroll:usize}
pub fn buy_button(rect:Rect)->Rect {
    Rect{x:rect.x+10.0,y:rect.y+rect.h-29.0,w:rect.w-20.0,h:19.0}
}
pub fn presentation(rect:Rect,info:&Info,scroll:usize,tick:u32)->Presentation {
    let mut l=Layer::new();
    l.rect(rect,0xF4070F1A);
    l.border(rect,0xAA4B7F98);
    l.rect(Rect{x:rect.x,y:rect.y,w:3.0,h:rect.h},info.accent);
    l.rect(Rect{x:rect.x+8.0,y:rect.y+10.0,w:rect.w-16.0,h:1.0},0x4467A8B8);
    l.text("INSPECTOR",rect.x+12.0,rect.y+5.0,CYAN);
    l.sprite(info.icon,[0.0,0.0,128.0,128.0],Rect{x:rect.x+9.0,y:rect.y+21.0,w:19.0,h:19.0});
    let tw=rect.w-43.0;
    let title=wrap(&info.title,tw);
    for (i,line) in title.iter().take(4).enumerate(){l.text(line,rect.x+33.0,rect.y+22.0+i as f32*11.0,WHITE);}
    let title_end=rect.y+24.0+(title.len().min(4) as f32)*11.0;
    let subtitles=wrap(&info.subtitle,(rect.w-20.0).max(30.0));
    for (i,line) in subtitles.iter().take(3).enumerate() {
        l.text(line,rect.x+10.0,title_end+i as f32*11.0,info.accent);
    }
    let after_title=title_end+subtitles.len().min(3) as f32*11.0;
    l.rect(Rect{x:rect.x+10.0,y:after_title+3.0,w:rect.w-20.0,h:1.0},0x664C7385);
    let preview_space=if info.entity_kind.is_some() && rect.h>=240.0 {103.0}
        else if info.key.starts_with("node:") {27.0} else {0.0};
    let body=Rect{x:rect.x+10.0,y:after_title+9.0,w:rect.w-20.0,
        h:(rect.y+rect.h-20.0-preview_space-(after_title+9.0)).max(10.0)};
    let mut lines=Vec::new();
    for p in &info.paragraphs {lines.extend(wrap(p,(body.w-4.0).max(20.0)));lines.push(String::new());}
    let rows=((body.h/11.0).floor().max(1.0)) as usize;
    let max_scroll=lines.len().saturating_sub(rows);
    let start=scroll.min(max_scroll);
    l.set_clip(Some(body));
    // Typewriter VFX: emit a bounded number of Unicode characters since this
    // inspector target was selected. Scrolling instantly reveals the text, so
    // keyboard/touch navigation is never obstructed by a long animation.
    let mut visible = if scroll>0 {usize::MAX} else {(tick as usize).saturating_mul(3).saturating_add(1)};
    for (i,line) in lines.iter().skip(start).take(rows).enumerate(){
        let length=line.chars().count();
        if visible==0 {break;}
        let shown=if visible>=length {line.clone()} else {line.chars().take(visible).collect::<String>()};
        visible=visible.saturating_sub(length.saturating_add(1));
        if !shown.is_empty(){l.text(shown,body.x+2.0,body.y+i as f32*11.0,
            if i==0{ICE}else{WHITE});}
    }
    l.set_clip(None);
    if max_scroll>0 {
        let marker=format!("{}/{} · ROLE",start+1,max_scroll+1);
        l.text(&marker,rect.x+11.0,rect.y+rect.h-preview_space-16.0,MUTED);
    }
    if info.key.starts_with("node:") {
        let b=buy_button(rect);
        l.rect(b,if info.can_buy {0xFF155B5E}else{0xFF202A35});
        l.border(b,if info.can_buy{GREEN}else{MUTED});
        l.text(if info.can_buy{"ADQUIRIR NÍVEL"}else{"REQUISITOS PENDENTES"},
            b.x+6.0,b.y+5.0,if info.can_buy{WHITE}else{MUTED});
    }
    let preview_rect=if info.entity_kind.is_some() && preview_space>0.0 {
        let x0=rect.x+16.0;let y0=rect.y+rect.h-112.0;
        let w=rect.w-32.0;
        l.rect(Rect{x:x0,y:y0,w,h:96.0},0x9E0C1C2A);
        l.border(Rect{x:x0,y:y0,w,h:96.0},0x6659A2BC);
        l.text("MODELO 3D",x0+6.0,y0+5.0,GOLD);
        let phase=(tick%360) as f32;
        l.rect(Rect{x:x0+w*0.5-14.0,y:y0+85.0,w:28.0,h:1.0},0x668E9FAF);
        let _=phase;
        Some([x0+2.0,y0+19.0,x0+w-2.0,y0+88.0])
    } else {None};
    Presentation{layer:l,preview_kind:info.entity_kind.clone(),preview_rect,max_scroll}
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
        assert!(inspector_area(Rect{x:4.0,y:4.0,w:380.0,h:240.0}).is_none());
    }
    #[test] fn text_wrap_never_breaks_panel_bounds(){
        let s="Habilidades com efeitos permanentes e descrições detalhadas para qualquer criatura";
        assert!(wrap(s,110.0).iter().all(|t|text_width(t)<=110.5));
        assert!(valid_kind("mod_name:giant/boss"));
        assert!(!valid_kind("evil:../../foo"));
    }
    #[test] fn contextual_fallback_for_every_page(){
        for &(kind,_) in crate::crpg_view::PAGES.iter() {
            let m=MenuModel{kind,..Default::default()};
            assert!(!inspect(&m).paragraphs.is_empty());
        }
    }
}
