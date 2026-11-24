"""Preview-only gameplay changes; stable Horizon source remains untouched."""
from pathlib import Path
import re
from sunrise_compat import once

def apply(stage):
    lib=stage/"src/lib.rs";s=lib.read_text()
    # Boss bars only text + health. Ordinary target keeps its preview.
    start=s.index("                    // The dedicated boss portrait");end=s.index("                    let sx=card.text_x;",start)
    s=s[:start]+'''                    if !boss {
                        let view=GuiEntityView::new(card.portrait,13.0,0.02,
                            GuiEntityLook::Turned{yaw:(ui.tick%360) as f32,pitch:0.0});
                        let _=draw_species_model(draw,host,&t.kind,Some(t.entity),view);
                    }
'''+s[end:]
    # Server-sampled actual attack attribute, separately from observed damage.
    s=once(s,"id:ent.id(),hp:hp.min(max_hp),max_hp,armor:armor.max(0.0),level:level_num,","id:ent.id(),hp:hp.min(max_hp),max_hp,armor:armor.max(0.0),level:level_num,\n                        attack:living.attribute_base(host,\"minecraft:attack_damage\").unwrap_or(0.0).clamp(0.0,10000.0) as f32,")
    s=s.replace('let tier="Normal".to_string();','let tier=rustpg_scaling::boss_tier(&kind).unwrap_or(rustpg_scaling::Tier::Normal).name().to_string();')
    s=once(s,'let hp_label=format!("{:.0}/{:.0} · {:.0}%",\n                        t.current_hp,t.max_hp,(health*100.0).clamp(0.0,100.0));','''let hp_label=format!("{:.0}/{:.0} · {:.0}%",
                         t.current_hp,t.max_hp,(health*100.0).clamp(0.0,100.0));
                     if let Some(sample)=ui.mob_stats.get(&t.entity) {
                         draw.text(&rustpg_ui_system::fit(&format!("ATQ {:.1} · DEF {:.1}",sample.attack,sample.armor),sw),
                             [sx,y+if boss{26.0}else{27.0}],rustpg_ui_theme::ICE,false);
                     }''')
    s=once(s,'let hp_y=if boss{y+rustpg_boss_ui::BOSS_HEALTH_BAR_Y}else{y+31.0};','let hp_y=if boss{y+rustpg_boss_ui::BOSS_HEALTH_BAR_Y}else{y+40.0};')
    s=once(s,'else{y+42.0}],','else{y+51.0}],')
    # Real input fallback: no dependency on the game mapping; physical V is caught in input_key,
    # and the world HUD supplies a separate optional button via click hook below.
    # Keep same raw hook and use actual screen state in client tick.
    # Expand the tree viewport from paged rows to a continuous graph.
    s=once(s,"tree_id:usize, node_sel:usize, tree_index:bool, tree_detail_only:bool,","tree_id:usize, node_sel:usize, tree_index:bool, tree_detail_only:bool,\n    graph_pan:[f32;2],graph_zoom:f32,")
    s=once(s,"pan:[0.0,0.0],zoom:1.0,selected,","pan:ui.graph_pan,zoom:if ui.graph_zoom>0.0{ui.graph_zoom}else{1.0},selected,")
    old='''let pages=rustpg_ui_window::constellation_pages(&m);
                if e.scroll[1]<0.0 {ui.scroll=ui.scroll.saturating_add(1).min(pages.saturating_sub(1));}
                else {ui.scroll=ui.scroll.saturating_sub(1);}'''
    s=once(s,old,'''let old=if ui.graph_zoom>0.0{ui.graph_zoom}else{1.0};
                let next=(old*(1.15_f32).powf(e.scroll[1] as f32)).clamp(0.12,3.0);
                let center=[g.x+g.w*0.5,g.y+g.h*0.5];
                for axis in 0..2 {
                    let local=e.screen.mouse[axis]-center[axis];
                    ui.graph_pan[axis]=local-(local-ui.graph_pan[axis])*next/old;
                }
                ui.graph_zoom=next;''')
    s=once(s,'''_e:&mut ScreenMouseDraggedPreEvent<'_>)->Flow {
        if host.mod_screen()!=Some(ID) {return Flow::Continue;}
        // The 5.3 skill tree is paged: no free-pan gesture, no jumpy camera.
        Flow::Continue''','''e:&mut ScreenMouseDraggedPreEvent<'_>)->Flow {
        if host.mod_screen()!=Some(ID) || e.button!=0 {return Flow::Continue;}
        let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        if rustpg_view::PAGES[ui.page.min(rustpg_view::PAGES.len()-1)].0!="trees" || ui.tree_index{return Flow::Continue;}
        let rect=Rect{x:4.0,y:4.0,w:e.screen.rect[2].max(200.0)-8.0,h:e.screen.rect[3].max(150.0)-8.0};
        let p=ui.profile.clone().unwrap_or_else(Profile::new);
        let m=menu_model(&p,&ui,rect,e.screen.mouse);
        let (g,_)=rustpg_ui_window::constellation_regions(&m);
        if !g.contains(e.screen.mouse){return Flow::Continue;}
        ui.graph_pan[0]=(ui.graph_pan[0]+e.delta[0]).clamp(-10000.0,10000.0);
        ui.graph_pan[1]=(ui.graph_pan[1]+e.delta[1]).clamp(-10000.0,10000.0);
        Flow::Cancel''')
    s=s.replace("ui.tree_id=i;ui.node_sel=0;ui.scroll=0;","ui.tree_id=i;ui.node_sel=0;ui.scroll=0;ui.graph_pan=[0.0;2];ui.graph_zoom=0.0;").replace("ui.node_sel=0;ui.scroll=0;ui.detail_scroll=0;ui.tree_detail_only=false;","ui.node_sel=0;ui.scroll=0;ui.detail_scroll=0;ui.tree_detail_only=false;ui.graph_pan=[0.0;2];ui.graph_zoom=0.0;")
    lib.write_text(s)
    p=stage/"src/rustpg_boss_ui.rs";s=p.read_text().replace("let h=if boss{60.0}else{54.0};","let h=if boss{60.0}else{66.0};")
    s=once(s,"let portrait_w=if boss {(w*0.22).clamp(46.0,58.0)}","let portrait_w=if boss {0.0}")
    s=once(s,"let text_x=portrait[2]+7.0;","let text_x=if boss{x+12.0}else{portrait[2]+7.0};")
    s=s.replace("assert!(boss.h>ordinary.h && boss.portrait[3]-boss.portrait[1]>ordinary.portrait[3]-ordinary.portrait[1]);","assert_eq!(boss.portrait[0],boss.portrait[2]);assert!(ordinary.portrait[2]>ordinary.portrait[0]);")
    p.write_text(s)
    p=stage/"src/rustpg_mob_wire.rs";s=p.read_text()
    s=s.replace("pub armor:f32,pub level:u16","pub armor:f32,pub attack:f32,pub level:u16")
    s=s.replace("self.armor>=0.0 && self.armor<=10_000.0 && self.level>=1","self.armor>=0.0 && self.armor<=10_000.0 && self.attack.is_finite() && self.attack>=0.0 && self.attack<=10_000.0 && self.level>=1")
    s=s.replace("18*self.mobs.len()","22*self.mobs.len()").replace("&[1,self.dimension.len()","&[2,self.dimension.len()").replace("18*Self::CAP || b[0]!=1","22*Self::CAP || b[0]!=2").replace("n*18","n*22").replace("chunks_exact(18)","chunks_exact(22)")
    s=s.replace("v.extend_from_slice(&m.level.to_le_bytes());","v.extend_from_slice(&m.attack.to_le_bytes());\n            v.extend_from_slice(&m.level.to_le_bytes());")
    s=s.replace("chunk[16..18]","chunk[20..22]").replace("armor:f32at(12),level","armor:f32at(12),attack:f32at(16),level")
    s=s.replace("armor:4.5,level:3","armor:4.5,attack:7.5,level:3").replace("armor:0.0,level:1","armor:0.0,attack:0.0,level:1")
    p.write_text(s)
    p=stage/"src/rustpg_scaling.rs";s=p.read_text().replace("0.03 * l","0.045 * l").replace("0.025 * l","0.035 * l").replace("armor: finite(a + 0.4 * l),","armor: finite(a * 0.35 + 8.0 * l / (l + 40.0)),").replace("toughness: finite(a * 0.5 + 0.15 * l),","toughness: finite(a * 0.10 + 3.0 * l / (l + 50.0)),").replace("resistance: (0.002 * l).min(0.5),","resistance: (0.001 * l).min(0.20),");p.write_text(s)
    p=stage/"src/rustpg_nodes.rs";s=p.read_text()
    s=once(s,'''let max = if a.id.ends_with("move_speed") {
                Some(100)
            } else {
                *max
            };''','let max = Some(1);')
    s=s.replace("if i == 1 { 25 } else { 100 }","1").replace('("enchanting.mastery#3".into(), 100)','("enchanting.mastery#3".into(), 1)').replace("cost_step: *step,","cost_step: 0,")
    # Add four unique purchasable nodes per tier instead of stacked ranks.
    # Each node has its own ID and exactly one predecessor. Branch roots connect
    # along each school's ring; buying the first node opens its next tier and neighbor.
    s=once(s,"    // Each retained attribute is its own short branch.",'''    let mut expanded=Vec::new();
    let mut roots=std::collections::BTreeMap::<String,String>::new();
    for mut node in out {
        let old_id=node.id.clone();
        let stage_no=node.id.rsplit_once('#').and_then(|(_,n)|n.parse::<u32>().ok()).unwrap_or(1);
        if stage_no==1 {
            if let Some(prior)=roots.insert(node.tree.clone(),old_id.clone()) {
                node.prereqs.push((prior,1));
            }
        } else {
            for (id,rank) in &mut node.prereqs {*id=format!("{id}.4");*rank=1;}
        }
        node.max_rank=Some(1);node.cost_step=0;
        expanded.push(node.clone());
        let mut parent=old_id.clone();
        for step in 2..=4 {
            let mut next=node.clone();next.id=format!("{old_id}.{step}");
            next.name=format!("{} · {}",node.name,step);
            next.prereqs=vec![(parent,1)];
            parent=next.id.clone();expanded.push(next);
        }
    }
    let mut out=expanded;
    // Each retained attribute is its own connected branch.''')
    s=s.replace('("enchanting.mastery#3".into(), 1)','("enchanting.mastery#3.4".into(), 1)')
    s=s.replace('"looting" => "minecraft:chest"','"looting" => "minecraft:rotten_flesh"')
    s=once(s,'icon: icon_for(&tree),','''icon: if a.id.contains("crit") {"minecraft:blaze_powder"}
                    else if a.id.contains("loot") {"minecraft:rotten_flesh"}
                    else if a.id.contains("diamond") {"minecraft:diamond"}
                    else if a.id.contains("netherite") {"minecraft:netherite_ingot"}
                    else if a.id.contains("gold") {"minecraft:gold_ingot"}
                    else if a.id.contains("iron") {"minecraft:iron_ingot"}
                    else {icon_for(&tree)},''')
    p.write_text(s)
    p=stage/"src/rustpg_ui_window.rs";s=p.read_text()
    start=s.index("pub fn constellation_node_rects(");end=s.index("pub fn constellation_categories(",start)
    s=s[:start]+'''fn galaxy_nodes(m:&MenuModel)->Vec<(Rect,usize)> {
    let Some(t)=m.tree.as_ref()else{return Vec::new();};
    if t.index_mode{return Vec::new();}
    let (g,_)=constellation_regions(m);
    let center=[g.x+g.w*0.5+t.pan[0],g.y+g.h*0.5+t.pan[1]];
    let z=t.zoom.clamp(0.12,3.0);
    let mut groups=std::collections::BTreeMap::<String,Vec<usize>>::new();
    for (i,n) in t.nodes.iter().enumerate(){
        let key=n.id.split('#').next().unwrap_or(&n.id).to_owned();
        groups.entry(key).or_default().push(i);
    }
    let count=groups.len().max(1) as f32;
    let root_radius=(count*10.0).max(42.0);
    let mut out=Vec::with_capacity(t.nodes.len());
    for (sector,(_,members)) in groups.into_iter().enumerate(){
        let angle=std::f32::consts::TAU*sector as f32/count-std::f32::consts::FRAC_PI_2;
        for (depth,i) in members.into_iter().enumerate(){
            let r=root_radius+depth as f32*28.0;
            let bend=((depth as f32)*0.42).sin()*(0.95/count.sqrt()).min(0.28);
            let (sn,cs)=(angle+bend).sin_cos();
            let d=(if depth==0||depth%4==3{20.0}else{12.0})*z;
            let d=d.clamp(5.0,42.0);
            out.push((Rect{x:center[0]+cs*r*z-d*0.5,y:center[1]+sn*r*z-d*0.5,w:d,h:d},i));
        }
    }
    out
}
pub fn constellation_node_rects(m:&MenuModel)->Vec<(Rect,usize)> {
    let (g,_)=constellation_regions(m);
    galaxy_nodes(m).into_iter().filter(|(r,_)|r.x>=g.x&&r.y>=g.y&&r.x+r.w<=g.x+g.w&&r.y+r.h<=g.y+g.h).collect()
}
'''+s[end:]
    s=once(s,"let per=constellation_rows(m).max(1);\n    constellation_segments(m).len().div_ceil(per).max(1)","1")
    start=s.index("        let visible=constellation_groups_on_page(m);");end=s.index("    l.set_clip(None);",start)
    s=s[:start]+'''        let rects=galaxy_nodes(m);
        let visible=constellation_node_rects(m);
        let hovered=visible.iter().find(|(r,_)|r.contains(m.mouse)).map(|(_,i)|*i);
        let positions:std::collections::HashMap<usize,Rect>=rects.into_iter().map(|(r,i)|(i,r)).collect();
        for &(a,b) in &t.links {
            if let (Some(ra),Some(rb))=(positions.get(&a),positions.get(&b)) {
                let pa=[ra.x+ra.w*0.5,ra.y+ra.h*0.5];let pb=[rb.x+rb.w*0.5,rb.y+rb.h*0.5];
                let color=if t.nodes[a].rank>0&&t.nodes[b].rank>0{GOLD}
                    else if t.nodes[b].buyable{CYAN}else{alpha(MUTED,0x65)};
                constellation_wire(l,g,pa,pb,if color==GOLD{1.5}else{0.65},color);
            }
        }
        let center=[g.x+g.w*0.5+t.pan[0],g.y+g.h*0.5+t.pan[1]];
        if g.contains(center){
            l.diamond(center[0],center[1],(16.0*t.zoom).clamp(5.0,36.0),GOLD);
            l.text_center(&t.available_points.to_string(),Rect{x:center[0]-18.0,y:center[1]-5.0,w:36.0,h:10.0},center[1]-4.0,WHITE);
        }
        for (r,i) in visible {
            let n=&t.nodes[i];let selected=i==t.selected;let hot=hovered==Some(i);
            let edge=if n.rank>0{GOLD}else if n.buyable{CYAN}else{alpha(MUTED,0x9A)};
            l.diamond(r.x+r.w/2.0,r.y+r.h/2.0,r.w/2.0,edge);
            l.diamond(r.x+r.w/2.0,r.y+r.h/2.0,(r.w/2.0-1.3).max(1.0),0xFF101821);
            if selected||hot {l.glow(r,2.0,1.0,alpha(edge,0xA0),1);}
            if r.w>=11.0 {
                if let Some(node)=super::rustpg_nodes::get(&n.id) {
                    let inset=r.w*0.23;
                    l.item(node.icon,Rect{x:r.x+inset,y:r.y+inset,w:r.w-2.0*inset,h:r.h-2.0*inset});
                }
            }
            if n.rank>0 {l.diamond(r.x+r.w*0.5,r.y+r.h-1.0,1.8,GOLD);}
        }
    }
'''+s[end:]
    s=s.replace('if t.index_mode{"ESCOLAS"}else{"RAMOS"}','if t.index_mode{"ESCOLAS"}else{"MAPA"}')
    p.write_text(s)

    # Add a genuine item primitive instead of a fake icon sprite.
    p=stage/"src/rustpg_ui_system.rs";s=p.read_text()
    s=once(s,"    Sprite {\n        key: &'static str,","    Item { key:&'static str, at:[f32;2], size:f32 },\n    Sprite {\n        key: &'static str,")
    s=once(s,"    pub fn sprite(&mut self, key:&'static str, src:[f32;4], r:Rect) {",'''    pub fn item(&mut self,key:&'static str,r:Rect) {
        if let Some(clipped)=self.scissor(r) {
            if clipped==r && r.w>=1.0 && r.h>=1.0 {
                self.prims.push(Prim::Item{key,at:[r.x,r.y],size:r.w.min(r.h)});
            }
        }
    }
    pub fn sprite(&mut self, key:&'static str, src:[f32;4], r:Rect) {''')
    s=s.replace("Prim::Text { .. } | Prim::Sprite { .. } => {}","Prim::Text { .. } | Prim::Sprite { .. } | Prim::Item { .. } => {}")
    p.write_text(s)
    p=stage/"src/lib.rs";s=p.read_text()
    s=once(s,"fn draw_layer(draw:&mut dyn mod_api::DrawContext, layer:&rustpg_ui_system::Layer) {","fn draw_layer(draw:&mut dyn mod_api::DrawContext,host:&dyn ClientHost,layer:&rustpg_ui_system::Layer) {")
    s=once(s,"            Prim::Sprite{key,src,dst,tint} =>",'''            Prim::Item{key,at,size} => {
                if let Ok(id)=mod_api::Identifier::parse(key) {
                    if let Some(item)=host.item_key(&id) {let _=draw.item(item,*at,*size);}
                }
            },
            Prim::Sprite{key,src,dst,tint} =>''')
    s=s.replace("draw_layer(&mut *e.draw,&layer)","draw_layer(&mut *e.draw,host,&layer)").replace("draw_layer(&mut *e.draw,&inspector.layer)","draw_layer(&mut *e.draw,host,&inspector.layer)")
    # An opt-in touch target uses GUI coordinates from the actual frame, never
    # a fixed Android pixel resolution. It cannot intercept normal play unless enabled.
    s=once(s,'let _=registrar.register_setting(SettingSpec::toggle("hud_enabled",','let _=registrar.register_setting(SettingSpec::toggle("touch_menu", "RustPG touch menu",false));\n        let _=registrar.register_setting(SettingSpec::toggle("hud_enabled",')
    s=once(s,"    fn gui_render_post(&self,host:&dyn ClientHost,e:&mut mod_api::GuiRenderPostEvent<'_>) {",'''    fn mouse_button_pre(&self,host:&mut dyn ClientHost,e:&mut mod_api::MouseButtonPreEvent)->Flow {
        if !e.pressed || e.button!=0 || !host.playing() || !host.setting_bool("touch_menu").unwrap_or(false) {return Flow::Continue;}
        let cursor=host.cursor();
        let ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        if ui.touch_rect.is_some_and(|r|r.contains(cursor)){
            OPEN_REQUEST.store(true,Ordering::Release);return Flow::Cancel;
        }
        Flow::Continue
    }
    fn gui_render_post(&self,host:&dyn ClientHost,e:&mut mod_api::GuiRenderPostEvent<'_>) {''')
    s=once(s,"    graph_pan:[f32;2],graph_zoom:f32,","    graph_pan:[f32;2],graph_zoom:f32,touch_rect:Option<Rect>,")
    # gui_render_post holds this mutex while painting; store actual button bounds in it.
    a=s.index("    fn gui_render_post(");b=s.index("        let Some(ref p)=ui.profile",a)
    part=s[a:b]
    part=once(part,"let ui=self.ui.lock()","let mut ui=self.ui.lock()")
    s=s[:a]+part+s[b:]
    a=s.index("    fn gui_render_post(");b=s.index("        let Some(ref p)=ui.profile",a)
    s=s[:b]+'''        let [gw,gh]=e.batch.size();
        ui.touch_rect=if host.setting_bool("touch_menu").unwrap_or(false)&&gw>=260.0&&gh>=150.0 {
            Some(Rect{x:gw*0.5-23.0,y:gh*0.22,w:46.0,h:18.0})
        }else{None};
        if let Some(r)=ui.touch_rect {
            e.batch.fill([r.x,r.y,r.w,r.h],0xD0122230);
            e.batch.text("RUSTPG",[r.x+4.0,r.y+4.0],0xFFF2DBA1,true);
        }
'''+s[b:]
    p.write_text(s)
    # New regression tests for the new graph and scale, not merely pixel proof.
    p=stage/"src/rustpg_nodes.rs"
    s=p.read_text()+'''
#[cfg(test)]
mod sunrise_unique_nodes {
    use super::*;
    #[test] fn every_purchase_has_one_rank_and_valid_parent() {
        assert!(all().iter().all(|n| n.max_rank==Some(1)));
        assert!(all().iter().all(|n| n.prereqs.iter().all(|(id,r)|*r==1 && get(id).is_some())));
    }
}
'''
    p.write_text(s)
    p=stage/"src/rustpg_scaling.rs"
    s=p.read_text()+'''
#[cfg(test)]
mod sunrise_scaling_tests {
    use super::*;
    #[test] fn hp_and_attack_grow_faster_than_armor() {
        let low=stats_for(1,Tier::Normal);
        let high=stats_for(100,Tier::Normal);
        assert!(high.health_mult>low.health_mult*3.0);
        assert!(high.damage_mult>low.damage_mult*3.0);
        assert!(high.armor<8.0 && high.toughness<3.0);
    }
}
'''
    p.write_text(s)
