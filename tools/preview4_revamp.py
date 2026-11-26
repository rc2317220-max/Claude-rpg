"""Preview 4 UI and packaging overhaul. The stable Horizon project is not modified."""
from pathlib import Path
import re, shutil, zipfile
from sunrise_compat import once

VERSION="6.2.0-alpha"
VISUAL_ID="rustpg_visuals"

def patch(stage, name, edit):
    p=stage/"src"/name
    p.write_text(edit(p.read_text()),encoding="utf-8")

def apply(stage):
    stage=Path(stage)
    def lib(s):
        # Use identical safe-area geometry for draw, scrolling, dragging and clicks.
        s=once(s,"fn menu_model(p: &Profile", """fn rustpg_screen_area(w:f32,h:f32)->Rect {
    // Starling overlays H/Mouse/Back above the game's screen.
    let top=if h>=145.0{25.0}else{4.0};
    Rect{x:4.0,y:top,w:(w-8.0).max(1.0),h:(h-top-4.0).max(1.0)}
}
fn menu_model(p: &Profile""")
        s=once(s,"let area=Rect {x:4.0,y:4.0,w:w-8.0,h:h-8.0};","let area=rustpg_screen_area(w,h);")
        s=once(s,"let area=Rect {x:4.0,y:4.0,w:w.max(200.0)-8.0,h:h.max(150.0)-8.0};","let area=rustpg_screen_area(w,h);")
        s=re.sub(r"Rect\{x:4\.0,y:4\.0,w:e\.screen\.rect\[2\]\.max\(200\.0\)-8\.0,\s*h:e\.screen\.rect\[3\]\.max\(150\.0\)-8\.0\}", "rustpg_screen_area(e.screen.rect[2],e.screen.rect[3])",s)
        # The engine owns opening eligibility; do not drop a virtual key due to
        # a transitory playing() result before the client tick processes it.
        s=once(s,'if e.action!=KeyAction::Press || !host.playing(){return;}','if e.action!=KeyAction::Press || host.local_player().is_none(){return;}')
        s=once(s,'if player_id.is_some() && host.playing() && host.mod_screen()!=Some(ID)\n            && (mapped_click || queued)', 'if player_id.is_some() && host.mod_screen()!=Some(ID)\n            && (mapped_click || queued)')
        s=once(s,'SettingSpec::toggle("touch_menu", "RustPG touch menu",false)','SettingSpec::toggle("touch_menu", "Botão de menu RustPG",true)')
        s=s.replace('setting_bool("touch_menu").unwrap_or(false)','setting_bool("touch_menu").unwrap_or(true)')
        s=once(s,'R ou /rustpg;','V ou /rustpg;')
        s=once(s,'let _=host.mod_screen_open(ID);','if host.mod_screen_open(ID) {\n                let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());\n                ui.page=0;ui.scroll=0;ui.detail_scroll=0;\n            }')
        # Dedicated tree mode. Drag and wheel translate; only +/- buttons zoom.
        a=s.index("                let old=if ui.graph_zoom>0.0")
        b=s.index("                ui.graph_zoom=next;",a)+len("                ui.graph_zoom=next;")
        s=s[:a]+"""                ui.graph_pan[0]=(ui.graph_pan[0]+e.scroll[0] as f32*18.0).clamp(-1200.0,1200.0);
                ui.graph_pan[1]=(ui.graph_pan[1]+e.scroll[1] as f32*18.0).clamp(-1200.0,1200.0);"""+s[b:]
        s=s.replace('clamp(-10000.0,10000.0)','clamp(-1200.0,1200.0)')
        s=once(s,'Some(("page",i)) => {ui.page=i.parse::<usize>().unwrap_or(0).min(rustpg_view::PAGES.len()-1);ui.scroll=0;},','''Some(("page",i)) => {
                    ui.page=i.parse::<usize>().unwrap_or(0).min(rustpg_view::PAGES.len()-1);
                    ui.scroll=0;ui.detail_scroll=0;
                    if rustpg_view::PAGES[ui.page].0=="trees" {
                        ui.tree_index=true;ui.graph_pan=[0.0;2];ui.graph_zoom=0.0;
                    }
                },
                None if a=="tree_back" => {ui.page=0;ui.scroll=0;ui.detail_scroll=0;},
                Some(("zoom",n)) => {
                    let z=if ui.graph_zoom>0.0{ui.graph_zoom}else{1.0};
                    let next=match n {"in"=>(z*1.3).min(4.0),"out"=>(z/1.3).max(0.6),_=>1.0};
                    if n=="fit"{ui.graph_pan=[0.0;2];}else{
                        for axis in 0..2{ui.graph_pan[axis]=(ui.graph_pan[axis]*next/z).clamp(-1200.0,1200.0);}
                    }
                    ui.graph_zoom=next;
                },''')
        s=once(s,'draw_chrome(&mut *e.draw,m.area,&m);','if m.kind!="trees" {draw_chrome(&mut *e.draw,m.area,&m);}')
        s=once(s,'draw_assets(&mut *e.draw,m.area,&m,host);','if m.kind!="trees" {draw_assets(&mut *e.draw,m.area,&m,host);}')
        # Live HP instead of a potentially stale saved profile HP.
        s=once(s,'ratio:(p.vitality.current/p.vitality.max.max(1.0))','ratio:(ui.vanilla_health.0/ui.vanilla_health.1.max(1.0))')
        s=once(s,'value:format!("{:.0}/{:.0}",p.vitality.current,p.vitality.max)','value:format!("{:.0}/{:.0}",ui.vanilla_health.0,ui.vanilla_health.1)')
        # Keep vanilla XP/locator/jump bar. Hide only health/food/armor/air
        # which the custom left HUD actually supplies, and only when it fits.
        s=once(s,'&& (e.layer=="minecraft:player_health" || e.layer=="minecraft:contextual_info_bar")','''&& e.draw.size()[0]>=260.0 && e.draw.size()[1]>=150.0
            && matches!(e.layer,"minecraft:player_health"|"minecraft:food_level"|"minecraft:armor_level"|"minecraft:air_level")''')
        a=s.index("        // RPG XP occupies the vanilla XP slot")
        b=s.index("        // One responsive combat surface",a)
        s=s[:a]+"""        // Independent RPG XP strip. Do not replace vanilla contextual/jump XP.
        draw.text("RPG XP",[bx+6.0*scale,by+69.0*scale],0xFFE9D990,true);
        let xp_x=bx+48.0*scale;
        draw.fill([xp_x,by+72.0*scale,81.0*scale,3.0*scale],0xAA172029);
        draw.fill([xp_x,by+72.0*scale,81.0*scale*xp_rate,3.0*scale],0xFFE2BC68);
"""+s[b:]
        s=once(s,'let y=by+(14.0+i as f32*19.0)*scale;', 'let y=by+(14.0+i as f32*19.0)*scale;')
        a=s.index("            // Cropped image bounds from original 512x64 assets.")
        b=s.index("            draw.text(value,",a)
        s=s[:a]+"""            let inner_w=(bar_w-2.0).max(1.0);
            let inner_h=(bar_h-2.0).max(1.0);
            let color=if i==0{0xFF49C77A}else{0xFF51A9ED};
            draw.fill([x,y,bar_w,bar_h],0xBB111A22);
            let ratio=if fx.initialized{fx.display}else{ratio.clamp(0.0,1.0)};
            draw_animated_bar_fx(draw,x+1.0,y+1.0,inner_w,inner_h,fx,color);
            draw.fill([x+1.0,y+1.0,inner_w*ratio,inner_h],color);
            draw.fill([x+1.0,y+1.0,inner_w*ratio,1.0],0x55FFFFFF);
"""+s[b:]
        # Hit post can occur after death processing. Preserve the impact
        # snapshot through entity leave, and recover dimension from that exact
        # hit rather than discarding the final damage.
        s=once(s,'if let Some(dim)=rt.entity_dimensions.get(&entity).cloned() {\n                    rt.impact_positions.insert((dim,entity),e.victim.position);','''let dim=rt.entity_dimensions.get(&entity).cloned().or_else(||
                    u32::try_from(e.source.causing).ok().and_then(|id|rt.entity_dimensions.get(&id).cloned()));
                if let Some(dim)=dim {
                    rt.impact_positions.insert((dim,entity),e.victim.position);''')
        s=once(s,'let Some(dimension)=rt.entity_dimensions.get(&entity).cloned() else {return;};','''let Some(dimension)=rt.entity_dimensions.get(&entity).cloned()
            .or_else(||{
                let mut matches=rt.impact_positions.keys().filter(|(_,id)|*id==entity);
                let first=matches.next()?.0.clone();
                matches.next().is_none().then_some(first)
            }) else {return;};''')
        s=once(s,'        rt.impact_positions.remove(&key);','        // Final hit may follow death/leave; the bounded per-tick snapshot survives.')
        s=once(s,'std::mem::take(&mut rt.hits)','{rt.impact_positions.clear();std::mem::take(&mut rt.hits)}')
        # Separate the actual attacker's damage summary from any target identity
        # or 3D portrait. No arbitrary portrait on the damage-only HUD.
        a=s.index('            let victim=ui.last_hit_victim.and_then')
        b=s.index('        if host.setting_bool("damage_numbers")',a)
        s=s[:a]+"""            let x=(w-124.0).max(6.0);
            let y=(target_bottom+8.0).min(h-54.0).max(30.0);
            draw.text(&format!("{} DANO",rustpg_damage_meter::display_damage(ui.damage_meter.total())),
                [x,y],rustpg_ui_theme::GOLD,true);
            draw.text(&format!("{}x COMBO",ui.damage_meter.combo()),
                [x,y+12.0],rustpg_ui_theme::WHITE,true);
            if host.setting_bool("damage_meter_list").unwrap_or(true) {
                if let Some(hit)=ui.damage_meter.recent().last(){
                    draw.text(&format!("{} {}",hit.family.label(),rustpg_damage_meter::display_damage(hit.amount as f64)),
                        [x,y+24.0],hit.family.color(),true);
                }
            }
        }
"""+s[b:]
        return s
    patch(stage,"lib.rs",lib)
    def window(s):
        # Tree owns a distinct layout and navigation state, not the generic
        # system tab strip. All render and hit geometry use the same body.
        s=once(s,"pub fn split(c: Rect)", """pub fn body(m:&MenuModel)->Rect {
    let win=window(m.area);
    if m.kind=="trees" {
        return Rect{x:win.x+8.0,y:win.y+29.0,w:(win.w-16.0).max(1.0),h:(win.h-43.0).max(1.0)};
    }
    content(win,m.pages.len())
}
pub fn split(c: Rect)""")
        s=once(s,"    chrome(&mut l, win, m);\n    tabs(&mut l, win, m);\n    let c = content(win, m.pages.len());", """    if m.kind=="trees" {
        l.state_button(Rect{x:win.x+8.0,y:win.y+5.0,w:48.0,h:19.0},"Voltar",super::rustpg_ui_theme::UiState::Normal,CYAN);
        l.text_middle("ÁRVORE DE HABILIDADES",Rect{x:win.x+59.0,y:win.y+5.0,w:(win.w-100.0).max(1.0),h:19.0},WHITE);
        l.state_button(close_rect(win),"X",super::rustpg_ui_theme::UiState::Normal,CYAN);
    }else{
        chrome(&mut l, win, m);
        tabs(&mut l, win, m);
    }
    let c = body(m);""")
        s=once(s,"    footer(&mut l, win, m);","    if m.kind!=\"trees\" {footer(&mut l, win, m);}")
        s=s.replace("let c=content(window(m.area),m.pages.len());","let c=body(m);").replace("let c = content(window(m.area), m.pages.len());","let c = body(m);")
        # Remove duplicate logo text. Real text rather than per-character
        # tracking for navigation labels.
        s=once(s,'    l.text_tracked("RUSTPG",h.x,h.y+1.0,alpha(CYAN,0xD8),0.9);\n','')
        s=once(s,'Rect{x:h.x+50.0,y:h.y-4.0,w:56.0,h:16.0}','Rect{x:h.x,y:h.y-2.0,w:56.0,h:16.0}')
        s=once(s,'l.text("VAZIO",r.x+6.0,r.y+19.0,alpha(MUTED,0x80));','l.text("Vazio",r.x+36.0,r.y+20.0,alpha(MUTED,0xC0));')
        s=once(s,'"HP" | "VIDA" => CYAN,\n        "SP" | "VIGOR" | "STA" => GREEN,','"HP" | "VIDA" => GREEN,\n        "SP" | "VIGOR" | "STA" => CYAN,')
        # Fit the whole graph at the initial camera position. Zoom remains a
        # user-controlled multiplier on top of that fit.
        s=once(s,'fn galaxy_nodes(m:&MenuModel)->Vec<(Rect,usize)> {','''fn galaxy_scale(m:&MenuModel)->f32 {
    let Some(t)=m.tree.as_ref()else{return 1.0;};
    let (g,_)=constellation_regions(m);
    let groups=galaxy_groups(t);
    let count=groups.len().max(3) as f32;
    let root_radius=(count*10.0).max(42.0);
    let max_depth=groups.iter().map(|(_,v)|v.len()).max().unwrap_or(1).saturating_sub(1);
    let radius=root_radius+max_depth as f32*28.0+18.0;
    let fit=((g.w.min(g.h-22.0)*0.45)/radius).clamp(0.5,1.0);
    (fit*t.zoom.clamp(0.6,4.0)).clamp(0.5,4.0)
}
fn galaxy_nodes(m:&MenuModel)->Vec<(Rect,usize)> {''')
        s=once(s,'    let z=t.zoom.clamp(0.12,3.0);','    let z=galaxy_scale(m);')
        s=once(s,'(16.0*t.zoom).clamp(5.0,36.0)','(16.0*galaxy_scale(m)).clamp(5.0,36.0)')
        s=once(s,'let d=d.clamp(5.0,42.0);','let d=d.clamp(11.0,30.0);')
        s=once(s,'pub fn constellation_page_arrow(m:&MenuModel,right:bool)->Rect {','''pub fn zoom_buttons(m:&MenuModel)->[(Rect,&'static str,&'static str);3] {
    let c=body(m);
    let y=c.y+19.0;
    let x=c.x+c.w-92.0;
    [(Rect{x,y,w:24.0,h:16.0},"-","zoom:out"),
     (Rect{x:x+27.0,y,w:36.0,h:16.0},"Ajustar","zoom:fit"),
     (Rect{x:x+66.0,y,w:24.0,h:16.0},"+","zoom:in")]
}
pub fn constellation_page_arrow(m:&MenuModel,right:bool)->Rect {''')
        s=once(s,'    let (g,d)=constellation_regions(m);','''    if !t.index_mode {
        for (r,label,_) in zoom_buttons(m) {
            l.state_button(r,label,super::rustpg_ui_theme::UiState::Normal,CYAN);
        }
    }
    let (g,d)=constellation_regions(m);''')
        s=once(s,'(c.w-10.0).max(35.0)),c.x+5.0,c.y+23.0,GOLD);','(c.w-104.0).max(35.0)),c.x+5.0,c.y+23.0,GOLD);')
        s=once(s,'&format!("{} {}/{}",if t.index_mode{"ESCOLAS"}else{"MAPA"},page+1,pages),','&if t.index_mode{format!("ESCOLAS {}/{}",page+1,pages)}else{"Arraste para navegar".into()},')
        s=once(s,'    for (right,enabled) in [(false,page>0),(true,page+1<pages)] {','    for (right,enabled) in [(false,page>0),(true,page+1<pages)].into_iter().filter(|_|t.index_mode) {')
        s=once(s,'    out.push((close_rect(win),"close".into()));\n    let group=','''    out.push((close_rect(win),"close".into()));
    if m.kind=="trees" {
        out.push((Rect{x:win.x+8.0,y:win.y+5.0,w:48.0,h:19.0},"tree_back".into()));
        if m.tree.as_ref().is_some_and(|t|!t.index_mode) {
            for (r,_,action) in zoom_buttons(m){out.push((r,action.into()));}
        }
    }else{
    let group=''')
        s=once(s,'    let c=content(win,m.pages.len());\n    if !matches!(m.kind,','    }\n    let c=body(m);\n    if !matches!(m.kind,')
        s=once(s,"fn paged_skill_tree_has_no_camera_controls()", "fn dedicated_skill_tree_has_explicit_camera_controls()")
        s=once(s,'        assert!(actions.iter().all(|a|!a.starts_with("zoom:") && a!="fit" && a!="center"));','''        for id in ["zoom:in","zoom:out","zoom:fit","tree_back"] {
            assert_eq!(actions.iter().filter(|a|a.as_str()==id).count(),1,"missing or duplicate control {id}");
        }
        assert!(!actions.iter().any(|a|a.starts_with("page:")));
        m.tree.as_mut().unwrap().index_mode=true;
        assert!(!menu_hits(&m).iter().any(|(_,a)|a.starts_with("zoom:")));''')
        s+="""
#[cfg(test)]
mod preview_revamp_layout_tests {
    use super::*;
    #[test] fn radial_geometry_uses_catalogue_order_and_visible_diamond_edges() {
        for tree in super::super::rustpg_nodes::trees() {
            let defs=super::super::rustpg_nodes::in_tree(&tree);
            let nodes:Vec<TreeNode>=defs.iter().map(|n|TreeNode{
                id:n.id.clone(),name:n.name.clone(),rank:0,max:"1".into(),cost:1,
                col:0,row:0,state:0,buyable:false,description:String::new(),requirement:String::new()
            }).collect();
            let index:std::collections::HashMap<&str,usize>=nodes.iter().enumerate().map(|(i,n)|(n.id.as_str(),i)).collect();
            let links:Vec<(usize,usize)>=defs.iter().enumerate().flat_map(|(i,n)|
                n.prereqs.iter().filter_map(|(id,_)|index.get(id.as_str()).map(|&p|(p,i))).collect::<Vec<_>>()).collect();
            for (w,h) in [(320.0,180.0),(560.0,300.0),(1000.0,700.0)] {
                for zoom in [0.6,1.0,2.0,4.0] {
                    let m=MenuModel{area:Rect{x:0.0,y:0.0,w,h},kind:"trees",external_inspector:true,
                        tree:Some(TreeModel{nodes:nodes.clone(),links:links.clone(),zoom,..Default::default()}),..Default::default()};
                    let (g,_)=constellation_regions(&m);
                    let c=[g.x+g.w*0.5,g.y+g.h*0.5];
                    let hub=(16.0*galaxy_scale(&m)).clamp(5.0,36.0);
                    let rects=galaxy_nodes(&m);
                    assert_eq!(rects.len(),nodes.len());
                    let positions:std::collections::HashMap<usize,Rect>=rects.into_iter().map(|(r,i)|(i,r)).collect();
                    for &(a,b) in &links {
                        let ra=positions[&a];let rb=positions[&b];
                        let pa=[ra.x+ra.w*0.5,ra.y+ra.h*0.5];
                        let pb=[rb.x+rb.w*0.5,rb.y+rb.h*0.5];
                        let dx=pb[0]-pa[0];let dy=pb[1]-pa[1];
                        let t=(((c[0]-pa[0])*dx+(c[1]-pa[1])*dy)/(dx*dx+dy*dy).max(0.001)).clamp(0.0,1.0);
                        let dist=(pa[0]+t*dx-c[0]).hypot(pa[1]+t*dy-c[1]);
                        assert!(dist>hub,"{tree}: edge crosses hub: {} -> {}",nodes[a].id,nodes[b].id);
                        let (p,q)=galaxy_wire_ends(ra,rb).expect("nonoverlapping linked nodes");
                        assert!(((p[0]-pa[0]).abs()+(p[1]-pa[1]).abs()-ra.w*0.5).abs()<0.02);
                        assert!(((q[0]-pb[0]).abs()+(q[1]-pb[1]).abs()-rb.w*0.5).abs()<0.02);
                    }
                    for (r,_) in constellation_node_rects(&m) {
                        assert!(r.w>=11.0 && r.x>=g.x && r.y>=g.y && r.x+r.w<=g.x+g.w && r.y+r.h<=g.y+g.h);
                    }
                }
            }
        }
    }
    #[test] fn dedicated_tree_has_no_general_tabs_and_zoom_controls_are_distinct() {
        let m=MenuModel{area:Rect{x:4.0,y:25.0,w:560.0,h:220.0},kind:"trees",
            pages:vec!["Resumo".into(),"Habilidades".into()],..Default::default()};
        assert!(body(&m).h>content(window(m.area),m.pages.len()).h);
        let hits=menu_hits(&m);
        assert!(hits.iter().any(|(_,a)|a=="tree_back"));
        assert!(!hits.iter().any(|(_,a)|a.starts_with("page:")));
        let buttons=zoom_buttons(&m);
        assert!(buttons[0].0.x+buttons[0].0.w<buttons[1].0.x);
        assert!(buttons[1].0.x+buttons[1].0.w<buttons[2].0.x);
    }
}
"""
        return s
    patch(stage,"rustpg_ui_window.rs",window)
    def widgets(s):
        return once(s,'let text=&fit_tracked(text,r.w-6.0,0.45);\n        let w=tracked_width(text,0.45);\n        self.text_tracked(text,r.x+(r.w-w)*0.5,r.y+(r.h-9.0)*0.5,color,0.45);','self.text_middle(fit_word(text,r.w-6.0),r,color);')
    patch(stage,"rustpg_ui_system.rs",widgets)
    patch(stage,"rustpg_inspector.rs",lambda s:once(s,'let c=window::content(window::window(m.area),m.pages.len());','let c=window::body(m);'))
    patch(stage,"rustpg_rifts.rs",lambda s:once(once(s,'RiftRank::E=>&["rustpg:ember_imp","rustpg:mana_hound","rustpg:rift_hunter"],','RiftRank::E=>&["rustpg:goblin_knife","rustpg:goblin_spear","rustpg:goblin_archer","rustpg:ember_imp","rustpg:mana_hound","rustpg:rift_hunter"],'),'RiftRank::E=>if alt{"rustpg:grave_knight"}else{"rustpg:rift_mantis"},','RiftRank::E=>if alt{"rustpg:goblin_boss"}else{"rustpg:rift_mantis"},'))
    for filename in ("sunrise-mod.toml","Cargo.toml"):
        p=stage/filename
        p.write_text(re.sub(r'(?m)^version = "[^"]+"$',f'version = "{VERSION}"',p.read_text(),count=1),encoding="utf-8")
    patch(stage,"lib.rs",lambda s:re.sub(r'const VERSION_STR: &str = "[^"]+";',f'const VERSION_STR: &str = "{VERSION}-sunrise-preview4";',s,count=1))

def split_visuals(stage,out):
    """Separate passive texture bytes without changing the rustpg entity/item IDs.
    Keep registry declarations and item models in the native package because
    those are bound to its namespace. A content-only package owns all textures.
    """
    stage=Path(stage);out=Path(out)
    visual=out/"RustPG-Visuals-preview4.smod"
    base=stage/"assets/rustpg/textures"
    assert base.is_dir()
    files=sorted(p for p in base.rglob("*") if p.is_file())
    assert files and any(p.name=="goblin_boss_authored.png" for p in files)
    manifest=f'''id = "{VISUAL_ID}"
name = "RustPG Visuals"
version = "{VERSION}"
description = "Texturas e recursos visuais independentes do código do RustPG."
authors = ["RustPG community"]
side = "both"
'''
    with zipfile.ZipFile(visual,"w",zipfile.ZIP_DEFLATED) as z:
        z.writestr("sunrise-mod.toml",manifest)
        for p in files:
            z.write(p,f"assets/{VISUAL_ID}/textures/{p.relative_to(base).as_posix()}")
    for folder in ("src","data","assets"):
        for p in (stage/folder).rglob("*"):
            if p.is_file() and p.suffix in (".rs",".json"):
                s=p.read_text(encoding="utf-8")
                t=s.replace("rustpg:gui/","rustpg_visuals:gui/").replace("rustpg:entity/","rustpg_visuals:entity/")
                if t!=s:p.write_text(t,encoding="utf-8")
    shutil.rmtree(base)
    p=stage/"sunrise-mod.toml"
    p.write_text(once(p.read_text(),"[native]",f'[dependencies]\n{VISUAL_ID} = "={VERSION}"\n\n[native]'),encoding="utf-8")
    print(f"Split visual assets: {len(files)} textures -> {visual}")
