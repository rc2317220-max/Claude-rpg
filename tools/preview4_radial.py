"""Final Preview 4 radial geometry. One source of truth for drawing and hits."""
from sunrise_compat import once

def apply(stage):
    p=stage/"src/rustpg_ui_window.rs"
    s=p.read_text()
    a=s.index("fn galaxy_nodes(")
    b=s.index("pub fn constellation_categories(",a)
    s=s[:a]+'''struct GalaxyLayout {
    nodes: Vec<(Rect,usize)>,
    center: [f32;2],
    radius: f32,
}

fn galaxy_layout(m:&MenuModel)->GalaxyLayout {
    let (g,_)=constellation_regions(m);
    let Some(t)=m.tree.as_ref() else {
        return GalaxyLayout{nodes:Vec::new(),center:[0.0;2],radius:0.0};
    };
    let center=[g.x+g.w*0.5+t.pan[0],g.y+g.h*0.5+t.pan[1]];
    if t.index_mode {return GalaxyLayout{nodes:Vec::new(),center,radius:0.0};}
    // Preserve catalogue order, which is also the root prerequisite chain.
    // Do not sort by the attribute ID: that creates chords across the hub.
    let mut groups:Vec<(&str,Vec<usize>)>=Vec::new();
    for (i,n) in t.nodes.iter().enumerate() {
        let key=n.id.split('#').next().unwrap_or(&n.id);
        if let Some((_,members))=groups.iter_mut().find(|(k,_)|*k==key) {
            members.push(i);
        }else{groups.push((key,vec![i]));}
    }
    let count=groups.len().max(1) as f32;
    let root=(count*12.0).max(48.0);
    let depth=groups.iter().map(|(_,v)|v.len()).max().unwrap_or(1).saturating_sub(1);
    let extent=root+depth as f32*34.0+16.0;
    // Fit as far as legibility permits; larger maps intentionally require pan.
    let fit=((g.w.min(g.h)*0.5-8.0)/extent).clamp(0.65,1.0);
    let z=(fit*t.zoom.clamp(0.6,4.0)).clamp(0.65,4.0);
    let radius=12.0*z;
    let mut nodes=Vec::with_capacity(t.nodes.len());
    for (sector,(_,members)) in groups.into_iter().enumerate(){
        let angle=std::f32::consts::TAU*sector as f32/count.max(3.0)-std::f32::consts::FRAC_PI_2;
        let (sn,cs)=angle.sin_cos();
        for (depth,i) in members.into_iter().enumerate(){
            let r=(root+depth as f32*34.0)*z;
            let d=(if depth==0 {26.0}else{22.0})*z;
            nodes.push((Rect{x:center[0]+cs*r-d*0.5,y:center[1]+sn*r-d*0.5,w:d,h:d},i));
        }
    }
    GalaxyLayout{nodes,center,radius}
}
fn galaxy_visible(g:Rect,r:Rect)->bool {
    r.x>=g.x&&r.y>=g.y&&r.x+r.w<=g.x+g.w&&r.y+r.h<=g.y+g.h
}
pub fn constellation_node_rects(m:&MenuModel)->Vec<(Rect,usize)> {
    let (g,_)=constellation_regions(m);
    galaxy_layout(m).nodes.into_iter().filter(|(r,_)|galaxy_visible(g,*r)).collect()
}
fn node_center(r:Rect)->[f32;2]{[r.x+r.w*0.5,r.y+r.h*0.5]}
fn galaxy_edges(t:&TreeModel, visible:&[(Rect,usize)])->Vec<(usize,usize,[f32;2],[f32;2])>{
    let positions:std::collections::HashMap<usize,Rect>=visible.iter().map(|(r,i)|(*i,*r)).collect();
    t.links.iter().filter_map(|&(a,b)|{
        let (ra,rb)=(*positions.get(&a)?,*positions.get(&b)?);
        let (p,q)=(node_center(ra),node_center(rb));
        let d=[q[0]-p[0],q[1]-p[1]];
        let norm=d[0].abs()+d[1].abs();
        if norm<=0.001{return None;}
        // Diamond boundary: |x|+|y|=radius. Stop the arrow before the child.
        let ta=ra.w*0.5/norm;
        let tb=rb.w*0.5/norm;
        if ta+tb>=1.0{return None;}
        Some((a,b,[p[0]+d[0]*ta,p[1]+d[1]*ta],[q[0]-d[0]*tb,q[1]-d[1]*tb]))
    }).collect()
}
'''+s[b:]
    a=s.index("        let rects=galaxy_nodes(m);")
    b=s.index("        for (r,i) in visible {",a)
    s=s[:a]+'''        let layout=galaxy_layout(m);
        let visible=constellation_node_rects(m);
        let hovered=visible.iter().find(|(r,_)|r.contains(m.mouse)).map(|(_,i)|*i);
        for (a,b,pa,pb) in galaxy_edges(t,&visible){
            let color=if t.nodes[a].rank>0&&t.nodes[b].rank>0{GOLD}
                else if t.nodes[b].buyable{CYAN}else{alpha(MUTED,0x65)};
            constellation_wire(l,g,pa,pb,if color==GOLD{1.5}else{0.65},color);
        }
        let center=layout.center;
        let r=layout.radius;
        if galaxy_visible(g,Rect{x:center[0]-r,y:center[1]-r,w:2.0*r,h:2.0*r}){
            l.diamond(center[0],center[1],r,GOLD);
            if r>=11.0 {
                l.text_center(&t.available_points.to_string(),Rect{x:center[0]-r,y:center[1]-5.0,w:2.0*r,h:10.0},center[1]-4.0,WHITE);
            }
        }
'''+s[b:]
    s=once(s,'if t.index_mode{"VOLTAR"}else{"ESCOLAS"}','if t.index_mode{"ÁRVORE"}else{"ESCOLAS"}')
    # Both dedicated screens have the same single return action.
    s=once(s,'if m.kind=="trees" {\n        return Rect','if matches!(m.kind,"trees"|"bestiary") {\n        return Rect')
    s=once(s,'if m.kind=="trees" {\n        l.state_button','if matches!(m.kind,"trees"|"bestiary") {\n        l.state_button')
    s=once(s,'l.text_middle("ÁRVORE DE HABILIDADES",','l.text_middle(if m.kind=="trees"{"ÁRVORE DE HABILIDADES"}else{"BESTIÁRIO"},')
    s=once(s,'if m.kind!="trees" {footer(&mut l, win, m);}','if !matches!(m.kind,"trees"|"bestiary") {footer(&mut l, win, m);}')
    s=once(s,'if m.kind=="trees" {\n        out.push','if matches!(m.kind,"trees"|"bestiary") {\n        out.push')
    s+=r'''
#[cfg(test)]
mod radial_geometry_regression {
    use super::*;
    #[test]
    fn every_catalogue_tree_has_aligned_visible_edges_and_legible_nodes(){
        for id in super::super::rustpg_nodes::trees(){
            let defs=super::super::rustpg_nodes::in_tree(&id);
            let nodes:Vec<_>=defs.iter().map(|n|TreeNode{id:n.id.clone(),..Default::default()}).collect();
            let by_id:std::collections::HashMap<_,_>=nodes.iter().enumerate().map(|(i,n)|(n.id.as_str(),i)).collect();
            let mut links=Vec::new();
            for (i,n) in defs.iter().enumerate(){for (k,_) in &n.prereqs{if let Some(&j)=by_id.get(k.as_str()){links.push((j,i));}}}
            let mut m=MenuModel{kind:"trees",area:Rect{x:0.0,y:0.0,w:480.0,h:240.0},external_inspector:true,
                tree:Some(TreeModel{nodes,links,zoom:1.0,..Default::default()}),..Default::default()};
            let layout=galaxy_layout(&m);
            let t=m.tree.as_ref().unwrap();
            assert_eq!(layout.nodes.len(),t.nodes.len(),"{id}");
            let all=galaxy_edges(t,&layout.nodes);
            assert_eq!(all.len(),t.links.len(),"{id}: missing edge");
            let positions:std::collections::HashMap<_,_>=layout.nodes.iter().map(|(r,i)|(*i,*r)).collect();
            for (a,b,p,q) in all {
                let ra=positions[&a];let rb=positions[&b];
                let ca=node_center(ra);let cb=node_center(rb);
                let d=[cb[0]-ca[0],cb[1]-ca[1]];
                assert!(((p[0]-ca[0])*d[1]-(p[1]-ca[1])*d[0]).abs()<0.05,"{id}: source alignment");
                assert!(((q[0]-cb[0])*d[1]-(q[1]-cb[1])*d[0]).abs()<0.05,"{id}: destination alignment");
                assert!(((p[0]-ca[0]).abs()+(p[1]-ca[1]).abs()-ra.w*0.5).abs()<0.01);
                assert!(((q[0]-cb[0]).abs()+(q[1]-cb[1]).abs()-rb.w*0.5).abs()<0.01);
                let dx=q[0]-p[0];let dy=q[1]-p[1];
                let f=(((layout.center[0]-p[0])*dx+(layout.center[1]-p[1])*dy)/(dx*dx+dy*dy)).clamp(0.0,1.0);
                let dist=(p[0]+f*dx-layout.center[0]).hypot(p[1]+f*dy-layout.center[1]);
                assert!(dist>layout.radius,"{id}: edge crosses hub");
            }
            for &(r,i) in &layout.nodes{
                assert!(r.w>=12.0,"{id}: unreadable node");
                let c=node_center(r);
                m.tree.as_mut().unwrap().pan=[layout.center[0]-c[0],layout.center[1]-c[1]];
                let vis=constellation_node_rects(&m);
                assert!(vis.iter().any(|(_,j)|*j==i),"{id}: unreachable node");
                for (a,b,_,_) in galaxy_edges(m.tree.as_ref().unwrap(),&vis){
                    assert!(vis.iter().any(|(_,j)|*j==a)&&vis.iter().any(|(_,j)|*j==b));
                }
            }
        }
    }
    #[test]
    fn dedicated_pages_have_one_return_and_no_tabs(){
        for kind in ["trees","bestiary"]{
            let m=MenuModel{kind,area:Rect{x:0.0,y:0.0,w:480.0,h:240.0},pages:vec!["Resumo".into()],..Default::default()};
            let hits=menu_hits(&m);
            assert_eq!(hits.iter().filter(|(_,a)|a=="tree_back").count(),1);
            assert!(!hits.iter().any(|(_,a)|a.starts_with("page:")));
        }
    }
}
'''
    p.write_text(s)
    p=stage/"src/lib.rs";s=p.read_text()
    s=once(s,'if m.kind!="trees" {draw_chrome(&mut *e.draw,m.area,&m);}','if !matches!(m.kind,"trees"|"bestiary") {draw_chrome(&mut *e.draw,m.area,&m);}')
    s=s.replace("clamp(-1200.0,1200.0)","clamp(-8192.0,8192.0)")
    p.write_text(s)
