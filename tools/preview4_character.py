"""Preview 4 character sheet and bestiary: real player, inventory and shared hit geometry."""
from sunrise_compat import once

def apply(stage):
    p=stage/"src/rustpg_ui_window.rs";s=p.read_text()
    s=once(s,'    (((left.h - 44.0) / 16.0).floor() as usize).max(1)','    (((left.h - 71.0) / 16.0).floor().max(-1.0) as i32 + 1).max(1) as usize')
    s=once(s,'fn bestiary_page_buttons(','''fn bestiary_row_rect(left:Rect,row:usize)->Option<Rect>{
    let y=left.y+36.0+row as f32*16.0;
    (left.w>=12.0&&y+15.0<=left.y+left.h-20.0).then_some(Rect{x:left.x+6.0,y,w:left.w-12.0,h:15.0})
}
fn bestiary_page_buttons(''')
    s=once(s,'let total_kills:u64=m.bestiary.iter().map(|s|s.kills).sum();','let total_kills=m.bestiary.iter().fold(0u64,|acc,s|acc.saturating_add(s.kills));')
    s=once(s,'''        if y+15.0>left.y+left.h-20.0 {break;}
        let row=Rect{x:left.x+6.0,y,w:left.w-12.0,h:15.0};''','''        let Some(row)=bestiary_row_rect(left,i-start) else {break;};''')
    s=once(s,'let (left,_)=split(c);let rows=bestiary_rows(left);let start=(m.bestiary_sel / rows) * rows;let y=left.y+22.0;','let left=super::rustpg_ui_layout::bestiary(c).list;let rows=bestiary_rows(left);let start=(m.bestiary_sel / rows) * rows;')
    s=once(s,'out.push((Rect{x:left.x+6.0,y:y+i as f32*16.0-2.0,w:left.w-12.0,h:15.0},format!("species:{}",start+i)));','if let Some(row)=bestiary_row_rect(left,i){out.push((row,format!("species:{}",start+i)));}')
    s=once(s,'    l.text(&fit(&species.name,info_w),info_x,right.y+20.0,alpha(MUTED,0xB8));','    l.text("Registro de criatura",info_x,right.y+20.0,alpha(MUTED,0xB8));')
    # Compact summary layout (portrait in the centre, eight slots on its sides).
    s=once(s,'fn overview_page(l: &mut Layer, c: Rect, m: &MenuModel) {','''pub struct CharacterSheet {
    pub portrait:Rect,pub slots:[Rect;8],pub stats:Rect
}
pub fn character_sheet(c:Rect)->Option<CharacterSheet>{
    if c.w<250.0||c.h<91.0{return None;}
    let y=c.y+31.0;let h=c.h-32.0;
    let stat_w=(c.w*0.34).clamp(90.0,150.0);
    let left=c.w-stat_w-6.0;
    let slot_w=(left*0.28).clamp(40.0,65.0);
    let rh=(h-6.0)*0.25;
    let slots=std::array::from_fn(|i|Rect{x:if i<4{c.x}else{c.x+left-slot_w},y:y+(i%4) as f32*(rh+2.0),w:slot_w,h:rh});
    let portrait=Rect{x:c.x+slot_w+3.0,y,w:left-slot_w*2.0-6.0,h};
    let stats=Rect{x:c.x+left+6.0,y,w:stat_w,h};
    Some(CharacterSheet{portrait,slots,stats})
}
fn overview_page(l: &mut Layer, c: Rect, m: &MenuModel) {
    if let Some(sheet)=character_sheet(c){
        let head=Rect{x:c.x,y:c.y,w:c.w,h:27.0};
        l.card(head,CYAN);
        l.text(&format!("Nv. {} · {} · {} pontos",m.level,m.rank,m.points),head.x+5.0,head.y+3.0,WHITE);
        let next=if m.next_rank.is_empty(){"Patente máxima".into()}else{format!("{:.0}% → {}",m.rank_progress*100.0,m.next_rank)};
        let next=fit(&next,head.w*0.35);
        l.text(&next,head.x+head.w-text_width(&next)-5.0,head.y+3.0,GOLD);
        l.bar(Rect{x:head.x+5.0,y:head.y+17.0,w:head.w-10.0,h:5.0},unit(m.xp),unit(m.xp),GOLD);
        l.card(sheet.portrait,CYAN);
        l.card(sheet.stats,CYAN);
        for (i,r) in sheet.slots.iter().enumerate(){
            l.card(*r,CYAN);
            let label=["Cabeça","Peito","Pernas","Pés","Arma","Anel","Colar","Amuleto"][i];
            l.text(&fit(label,(r.w-18.0).max(10.0)),r.x+2.0,r.y+(r.h-8.0)*0.5,MUTED);
        }
        return;
    }
''')
    # Legacy compact fallback shows Monarca as M with a tooltip/header name.
    s=once(s,'let ranks = ["E", "D", "C", "B", "A", "S"];','let ranks = ["E", "D", "C", "B", "A", "S", "Monarca"];')
    s=once(s,'''            l.text(
                *r,
                cx - text_width(r) * 0.5,''','''            let short=if *r=="Monarca"{"M"}else{*r};
            l.text(
                short,
                cx - text_width(short) * 0.5,''')
    s+=r'''
#[cfg(test)]
mod character_sheet_regression{
    use super::*;
    #[test]
    fn slots_and_portrait_never_overlap(){
        for (w,h) in [(250.0,91.0),(320.0,120.0),(500.0,220.0)]{
            let c=Rect{x:4.0,y:10.0,w,h};
            let sheet=character_sheet(c).unwrap();
            let mut all=sheet.slots.to_vec();all.push(sheet.portrait);all.push(sheet.stats);
            for (i,r) in all.iter().enumerate(){
                assert!(r.w>0.0&&r.h>0.0&&r.x>=c.x&&r.y>=c.y&&r.x+r.w<=c.x+c.w+0.01&&r.y+r.h<=c.y+c.h+0.01);
                for q in all.iter().skip(i+1){
                    assert!(r.x+r.w<=q.x+0.01||q.x+q.w<=r.x+0.01||r.y+r.h<=q.y+0.01||q.y+q.h<=r.y+0.01);
                }
            }
        }
    }
    #[test]
    fn bestiary_clicks_use_drawn_rows(){
        for (w,h) in [(260.0,180.0),(420.0,240.0),(220.0,400.0)]{
            let m=MenuModel{kind:"bestiary",area:Rect{x:0.0,y:0.0,w,h},bestiary:vec![SpeciesRow::default();32],..Default::default()};
            let left=super::super::rustpg_ui_layout::bestiary(body(&m)).list;
            let hits=menu_hits(&m);
            for row in 0..bestiary_rows(left){
                if let Some(r)=bestiary_row_rect(left,row){
                    assert!(hits.iter().any(|(hit,a)|a==&format!("species:{row}")&&*hit==r));
                }
            }
        }
    }
}
'''
    p.write_text(s)
    p=stage/"src/rustpg_view.rs";s=p.read_text()
    s=once(s,'''pub fn overview(p: &Profile) -> Vec<String> {
    vec![''','''pub fn overview(p: &Profile) -> Vec<String> {
    let mut lines=vec![''')
    s=once(s,'''        "Fenda pronta: /rustpg rift manifest · o acesso básico não usa chave.".into(),
    ]
}''','''    ];
    if p.rift_resonance>=super::rustpg_rifts::RESONANCE_READY {
        lines.push("Fenda pronta: /rustpg rift manifest · o acesso básico não usa chave.".into());
    }
    lines
}''')
    p.write_text(s)
    p=stage/"src/lib.rs";s=p.read_text()
    s=once(s,'next_rank:"".into(),rank_progress:0.0,','next_rank:rustpg_engine_presentation::rank_progress(p.power()).0.unwrap_or("").into(),rank_progress:rustpg_engine_presentation::rank_progress(p.power()).1,')
    s=once(s,'    sprite(draw,&rank_key,[0.0,0.0,256.0,256.0],[area.x+area.w-36.0,area.y+12.0,20.0,20.0]);','    if model.kind!="bestiary" {sprite(draw,&rank_key,[0.0,0.0,256.0,256.0],[area.x+area.w-36.0,area.y+12.0,20.0,20.0]);}')
    s=once(s,'fn draw_assets(draw:',r'''fn draw_character_sheet(draw:&mut dyn mod_api::DrawContext,host:&dyn ClientHost,m:&MenuModel){
    let Some(sheet)=rustpg_ui_window::character_sheet(rustpg_ui_window::body(m)) else{return;};
    let Some(player)=host.local_player() else{return;};
    let r=sheet.portrait;
    let view=GuiEntityView::new([r.x+1.0,r.y+1.0,r.x+r.w-1.0,r.y+r.h-1.0],
        (r.h*0.40).min(r.w*0.8).max(1.0),0.0625,
        GuiEntityLook::Turned{yaw:(m.tick as f32*0.55)%360.0,pitch:0.0});
    if draw.entity(&GuiEntity::Existing(player.entity()),view).is_err(){
        draw.text("Sem prévia",[r.x+2.0,r.y+2.0],0xFFB1C9D9,true);
    }
    let selected=player.selected_slot(host).unwrap_or(0) as u32;
    let slots=[Some(39u32),Some(38),Some(37),Some(36),Some(selected),None,None,None];
    let inv=player.with_inventory(host,|view|slots.map(|i|i.and_then(|n|view.get(n)).and_then(|stack|stack.project())));
    let mut tooltip:Option<Vec<Text>>=None;
    for (i,r) in sheet.slots.iter().enumerate(){
        let relic=m.worn.iter().find(|g|g.slot==rustpg_gear::SLOTS[i].0);
        let stack=inv.as_ref().and_then(|items|items[i].as_ref());
        let size=(r.h-2.0).clamp(4.0,16.0);
        let at=[r.x+r.w-size-2.0,r.y+(r.h-size)*0.5];
        if let Some(stack)=stack {let _=draw.item_stack(stack,at,size);}
        else if relic.is_some(){
            draw.fill([at[0],at[1],size,size],0x806A531F);
            draw.text("◆",[at[0]+1.0,at[1]+1.0],0xFFFFD27C,true);
        }
        if r.contains(m.mouse){
            let mut lines=Vec::new();
            if let Some(stack)=stack{
                if let Some(els)=mod_api::ItemResource::of(stack).tooltip(host,false,false){
                    lines.extend(els.into_iter().filter_map(|el|match el{
                        mod_api::TooltipElement::Text(text,_)=>Some(text),_=>None
                    }).take(20));
                }
                if lines.is_empty(){if let Ok(name)=host.hover_name(stack){lines.push(name);}}
            }
            if let Some(g)=relic{
                let rarity=rustpg_gear::RARITIES[g.rarity.min(5) as usize];
                lines.push(Text::literal(format!("Relíquia: {} · {}",g.title,rarity)));
                if let Some(attr)=rustpg_attributes::get(&g.stat){
                    lines.push(Text::literal(format!("{}: +{}",attr.name,rustpg_view::fmt_value(attr.unit,g.bonus))));
                }
                lines.push(Text::literal(g.ability.clone()));
            }
            if lines.is_empty(){lines.push(Text::literal("Espaço vazio"));}
            tooltip=Some(lines);
        }
    }
    let r=sheet.stats;
    let mut rows:Vec<(String,String)>=m.vitals.iter().map(|v|(if v.label=="HP"{"Vida".into()}else{"Vigor".into()},v.value.clone())).collect();
    if let Some(living)=host.living(player.entity()){
        for (label,key) in [("Ataque","minecraft:attack_damage"),("Armadura","minecraft:armor"),("Tenacidade","minecraft:armor_toughness"),("Velocidade","minecraft:movement_speed"),("Sorte","minecraft:luck"),("Ataques/s","minecraft:attack_speed")]{
            if let Some(v)=living.attribute_value(host,key).filter(|v|v.is_finite()){rows.push((label.into(),format!("{v:.2}").replace('.',",")));}
        }
    }
    let step=((r.h-4.0)/rows.len().max(1) as f32).clamp(7.0,13.0);
    for (i,(label,value)) in rows.iter().enumerate(){
        let y=r.y+2.0+i as f32*step;
        if y+8.0>r.y+r.h{break;}
        let value=rustpg_ui_system::fit(value,r.w*0.45);
        let label=rustpg_ui_system::fit(label,(r.w-draw.text_width(&value)-7.0).max(1.0));
        draw.text(&label,[r.x+3.0,y],0xFFB1C9D9,false);
        draw.text(&value,[r.x+r.w-3.0-draw.text_width(&value),y],0xFFF2DBA1,true);
    }
    if let Some(lines)=tooltip{let _=draw.gui_tooltip(&lines,m.mouse);}
}

fn draw_assets(draw:''')
    # Draw after the sidebar, so hover tooltips remain on top.
    s=once(s,'''        }
    }
    fn screen_mouse_scrolled_pre''','''        }
        if m.kind=="overview"{draw_character_sheet(&mut *e.draw,host,&m);}
    }
    fn screen_mouse_scrolled_pre''')
    p.write_text(s)
