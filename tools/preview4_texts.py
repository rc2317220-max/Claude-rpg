"""Player-facing one-purchase node wording for the actual Preview 4 catalogue."""
from sunrise_compat import once

def apply(stage):
    p=stage/"src/rustpg_nodes.rs";s=p.read_text()
    s=once(s,'    let mut out=expanded;','''    let mut out=expanded;
    // Preserve persistent identifiers and costs; change presentation only.
    let titles=["Despertar","Fundamento","Lapidação","Firmeza",
        "Disciplina","Refinamento","Maestria","Ascensão",
        "Excelência","Perfeição","Supremacia","Legado"];
    for n in &mut out {
        if let Some((_,suffix))=n.id.rsplit_once('#'){
            let mut parts=suffix.split('.');
            let stage=parts.next().and_then(|v|v.parse::<usize>().ok()).unwrap_or(1);
            let part=parts.next().and_then(|v|v.parse::<usize>().ok()).unwrap_or(1);
            let title=titles[(stage.saturating_sub(1)*4+part.saturating_sub(1)).min(titles.len()-1)];
            if let Some(a)=attributes::get(&n.attribute){
                n.name=format!("{title} · {}",a.name);
                n.description=format!("{}: +{}.",a.name,crate::rustpg_node_effects::fmt_value(n.formula.value(1),a.unit));
            }
        }
    }''')
    p.write_text(s)
    p=stage/"src/rustpg_node_effects.rs";s=p.read_text()
    a=s.index("pub fn fmt_value(value:f64,unit:Unit)");b=s.index("/// Bounded vanilla XP orb",a)
    s=s[:a]+'''pub fn fmt_value(value:f64,unit:Unit)->String {
    let v=if value.is_finite(){value.max(0.0)}else{0.0};
    let scaled=match unit{Unit::Percent|Unit::Chance|Unit::Reduction=>v*100.0,_=>v};
    let num=format!("{scaled:.3}");
    let num=num.trim_end_matches('0').trim_end_matches('.').replace('.',",");
    match unit{
        Unit::Percent|Unit::Chance|Unit::Reduction=>format!("{num}%"),
        Unit::Count=>format!("{num} un."),
        Unit::Flat=>format!("{num} pontos"),
    }
}
pub fn node_detail(node:&crate::rustpg_nodes::NodeDef,rank:u64,stats:&Stats)->String{
    let Some(def)=rustpg_attributes::get(&node.attribute)else{return node.description.clone();};
    let global=stats.get(&node.attribute);
    let full=node.formula.value(1);
    let remaining=def.cap.map_or(full,|cap|(cap-global).max(0.0).min(full));
    let effect=if rank>0{full}else{remaining};
    format!("{}\\n{}: +{}\\nTotal atual: {}\\n{}",
        if rank>0{"Habilidade adquirida."}else{"Adquira para melhorar este atributo."},
        def.name,fmt_value(effect,def.unit),fmt_value(global,def.unit),
        if rank>0{"Compra única; não há outro grau neste nó."}else if remaining<=0.0{"Limite do atributo atingido."}else{"Compra única."})
}
'''+s[b:]
    s=once(s,'assert!(detail.contains("Próximo:")); assert!(detail.contains("No máximo:"));','assert!(detail.contains("Total atual:")); assert!(!detail.contains(&n.attribute));')
    p.write_text(s)
    p=stage/"src/lib.rs";s=p.read_text()
    s=once(s,'''            .map(|(id,r)|format!("{} rank {}",
                rustpg_nodes::get(id).map_or(id.as_str(),|d|d.name.as_str()),r)).collect();''','''            .map(|(id,_)|format!("Adquirir {}",rustpg_nodes::get(id).map_or("habilidade anterior",|d|d.name.as_str()))).collect();''')
    s=once(s,'missing.push(format!("Ramo exclusivo: {}",exclusion));','missing.push(format!("Ramo exclusivo: {}",rustpg_nodes::get(exclusion).map_or("outra habilidade",|d|d.name.as_str())));')
    a=s.index("        let current=n.formula.value(rank);",s.index("    let detail=nodes.get(selected)"))
    b=s.index("        if let Some(row)=out.get(selected)",a)
    s=s[:a]+'''        let mut info=vec![rustpg_node_effects::node_detail(n,rank,&effective_stats),
            if rank>0{"Adquirida".into()}else{format!("Custo: {} pontos",n.cost_at(rank))}];
'''+s[b:]
    s=once(s,'.map(|rank|format!("Habilidade {id}: rank {rank}"))','.map(|_|format!("Habilidade adquirida: {}",rustpg_nodes::get(&id).map_or("Talento",|n|n.name.as_str())))')
    p.write_text(s)
    p=stage/"src/rustpg_ui_window.rs";s=p.read_text()
    s=once(s,'let subtitle=format!("{state} · {}/{} · {} PT",n.rank,n.max,n.cost);','let subtitle=if n.rank>0{state.to_owned()}else{format!("{state} · {} pontos",n.cost)};')
    s=once(s,'let label=if can{"ADQUIRIR NÍVEL"}else if n.state==3{"NÍVEL MÁXIMO"}else{"REQUISITOS PENDENTES"};','let label=if can{"ADQUIRIR"}else if n.rank>0{"ADQUIRIDA"}else{"REQUISITOS PENDENTES"};')
    p.write_text(s)
    p=stage/"src/rustpg_inspector.rs";s=p.read_text()
    s=once(s,'let mut paragraphs=vec![format!("{status} · Rank {}/{}",n.rank,n.max), n.description.clone()];','let mut paragraphs=vec![status.to_owned(), n.description.clone()];')
    s=once(s,'Próximo nível: {} PT · saldo: {} PT','Custo: {} pontos · saldo: {} pontos')
    s=s.replace('"Identificador: {}"','"Registro: {}"')
    s=once(s,'s.description.clone(),format!("Registro: {}",s.name),','s.description.clone(),')
    s=once(s,'format!("Slot: {} · raridade: {}",g.slot,g.rarity),','format!("Espaço: {} · raridade: {}",crate::rustpg_gear::SLOTS.iter().find(|(id,_)|*id==g.slot).map_or("Equipamento",|(_,label)|*label),crate::rustpg_gear::RARITIES[g.rarity.min(5) as usize]),')
    s=once(s,'format!("Atributo: {} · bônus: +{:.2}",g.stat,g.bonus),','crate::rustpg_attributes::get(&g.stat).map_or("Bônus não identificado".into(),|a|format!("{}: +{}",a.name,crate::rustpg_node_effects::fmt_value(g.bonus,a.unit))),')
    s=once(s,'return info(&format!("action:{act}"),&act.replace(\'_\'," ").to_uppercase(),"CONTROLE INTERATIVO",','''let label=match act.split(':').next().unwrap_or("") {
            "buy"=>"Adquirir","respec"=>"Redistribuir pontos","catalog"=>"Escolas de habilidades",
            "details"=>"Descrição","rpage"=>"Mudar página","tree"=>"Mudar escola",
            "tgroup"=>"Mudar abas","equip"=>"Equipar","salvage"=>"Reciclar",
            "unequip"=>"Retirar","activate"=>"Ativar poder","tree_back"=>"Voltar",
            "zoom"=>"Ajustar visão","close"=>"Fechar",_=>"Selecionar"};
        return info(&format!("action:{act}"),label,"CONTROLE INTERATIVO",''')
    s=s.replace("Drops conhecidos","Recompensas conhecidas").replace("stamina","vigor").replace(" · rank: "," · patente: ")
    p.write_text(s)
