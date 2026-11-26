// Pages of the RPG interface as plain lines: the same text feeds the touch screen
// and the /rustpg command, so both always agree.
use super::rustpg_attributes::{self as attributes, Unit};
use super::rustpg_nodes as nodes;
use super::rustpg_profile::Profile;
use super::rustpg_progression as prog;
use super::rustpg_scaling::MobScale;

pub const PAGES: &[(&str, &str)] = &[
    ("overview", "Resumo"),
    ("status", "Status"),
    ("trees", "Habilidades"),
    ("bestiary", "Bestiário"),
    ("profs", "Profissões"),
    ("equipment", "Equipamento"),
    ("inventory", "Relíquias"),
    ("enchant", "Encantamentos"),
    ("history", "Histórico"),
];

pub fn fmt_value(unit: Unit, v: f64) -> String {
    match unit {
        Unit::Flat => format!("{v:.2}"),
        Unit::Percent | Unit::Chance | Unit::Reduction => format!("{:.1}%", v * 100.0),
        Unit::Count => format!("{v:.2}"),
    }
}

pub fn inventory(p: &Profile) -> Vec<String> {
    let mut lines = vec![format!("BOLSA DO SISTEMA · {}/{} relíquias",p.gear.bag.len(), super::rustpg_gear::BAG_LIMIT),
        "Toque em uma relíquia para equipar. O inventário vanilla não é alterado.".into()];
    for g in &p.gear.bag {
        let (stat,value) = g.stat();
        lines.push(format!("#{} · {} · {} +{:.2} · {}", g.uid, g.name(), stat, value,
            super::rustpg_relic_engine::describe(g)));
    }
    if p.gear.bag.is_empty() { lines.push("Derrote monstros para obter relíquias.".into()); }
    lines
}
pub fn equipment(p: &Profile) -> Vec<String> {
    let mut lines = vec!["EQUIPAMENTO DO SISTEMA (bonificações aplicadas ao Stats)".into()];
    for &(slot,label) in super::rustpg_gear::SLOTS {
        if let Some(g) = p.gear.worn.get(slot) {
            let (stat,value) = g.stat();
            lines.push(format!("{}: #{} {} · {} +{:.2} · {}", label,g.uid,g.name(),stat,value,
                super::rustpg_relic_engine::describe(g)));
        } else { lines.push(format!("{label}: vazio")); }
    }
    lines
}
pub fn accessories(p: &Profile) -> Vec<String> {
    let mut lines = vec!["ACESSÓRIOS".into()];
    for &(slot,label) in super::rustpg_gear::SLOTS.iter().filter(|(s,_)| ["ring","neck","charm"].contains(s)) {
        lines.push(p.gear.worn.get(slot).map_or(format!("{label}: vazio"), |g| format!("{label}: {}",g.name())));
    }
    lines
}

pub fn character(p: &Profile) -> Vec<String> {
    let need = prog::general_xp_to_next(p.level);
    let mut v = vec![
        format!("Nível geral: {} · XP {}/{}", p.level, p.xp, need),
        format!("Pontos gerais livres: {}", p.points),
        format!("Poder de combate: {:.1}", p.power()),
        format!("Nós comprados: {}", p.ranks.len()),
        format!(
            "Ranks totais: {}",
            p.ranks.values().fold(0u64, |a, b| a.saturating_add(*b))
        ),
        format!("Mortes: {}", p.deaths),
        format!("Maior golpe: {:.1}", p.best_hit),
        format!("Dano causado: {:.0}", p.damage_dealt),
        format!("Dano recebido: {:.0}", p.damage_taken),
        format!(
            "Dimensões visitadas: {}",
            if p.dimensions.is_empty() {
                "-".into()
            } else {
                p.dimensions.join(", ")
            }
        ),
    ];
    v.push("Toque em uma aba acima para ver mais.".into());
    v
}

pub fn overview(p: &Profile) -> Vec<String> {
    vec![
        format!(
            "RustPG · Rank {} · Nível {}",
            super::rustpg_engine_presentation::profile_rank(p),
            p.level
        ),
        format!(
            "Poder {:.1} · Espécies descobertas {}",
            p.power(),
            p.species.values().filter(|s| s.kills > 0).count()
        ),
        format!(
            "Vigor {:.0}/{:.0} · Vitalidade {:.0}/{:.0}",
            p.stamina.current, p.stamina.max, p.vitality.current, p.vitality.max
        ),
        format!(
            "XP {}/{} · Pontos {}",
            p.xp,
            prog::general_xp_to_next(p.level),
            p.points
        ),
        format!("Ressonância da Fenda: {}/{} · Conclusões {}",p.rift_resonance,
            super::rustpg_rifts::RESONANCE_READY,p.rift_clears),
        if p.rift_resonance>=super::rustpg_rifts::RESONANCE_READY {
            "Fenda pronta: /rustpg rift manifest · o acesso básico não usa chave.".into()
        }else{"Acumule Ressonância para manifestar uma Fenda.".into()},
    ]
}

/// Final values of every attribute the player has, with source and cap.
pub fn status(p: &Profile) -> Vec<String> {
    let s = p.node_stats();
    let mut v = vec!["Atributo: nodes = final (limite) [aplicado no jogo?]".to_string()];
    for a in attributes::all() {
        let val = s.get(&a.id);
        if val == 0.0 {
            continue;
        }
        let cap = a.cap.map_or("sem teto".into(), |c| fmt_value(a.unit, c));
        let applied = if a.applied { "sim" } else { "registrado" };
        v.push(format!(
            "{}: {} ({cap}) [{applied}]",
            a.name,
            fmt_value(a.unit, val)
        ));
    }
    if v.len() == 1 {
        v.push("Nenhum atributo ainda. Compre nodes na aba Árvores.".into());
    }
    v
}

pub fn tree_list() -> Vec<String> {
    nodes::trees()
        .iter()
        .map(|t| {
            format!(
                "{t} — {} nodes ({})",
                nodes::in_tree(t).len(),
                prog::proficiency_name(nodes::tree_proficiency(t))
            )
        })
        .collect()
}

pub fn node_line(p: &Profile, n: &nodes::NodeDef) -> String {
    let r = p.rank(&n.id);
    let unit = attributes::get(&n.attribute).map_or(Unit::Flat, |a| a.unit);
    let now = fmt_value(unit, n.formula.value(r));
    let next = fmt_value(unit, n.formula.value(r.saturating_add(1)));
    let status = match p.check_buy(n) {
        Ok(c) => format!("custo {c}"),
        Err(e) => e.to_string(),
    };
    format!(
        "{} [{}/{}] {now} → {next} · {status}",
        n.name,
        r,
        n.max_label()
    )
}

pub fn tree_page(p: &Profile, tree: &str, page: usize, per: usize) -> Vec<String> {
    let all = nodes::in_tree(tree);
    let pages = all.len().div_ceil(per.max(1)).max(1);
    let page = page.min(pages - 1);
    let mut v = vec![format!(
        "{} — página {}/{} · pontos da árvore: {} + gerais {}",
        tree,
        page + 1,
        pages,
        p.prof(nodes::tree_proficiency(tree)).points,
        p.points
    )];
    for n in all.iter().skip(page * per).take(per) {
        v.push(node_line(p, n));
    }
    v
}

pub fn proficiencies(p: &Profile) -> Vec<String> {
    prog::PROFICIENCIES
        .iter()
        .map(|(id, name)| {
            let pr = p.prof(id);
            format!(
                "{name}: nível {} · XP {}/{} · pontos {}",
                pr.level,
                pr.xp,
                prog::prof_xp_to_next(pr.level),
                pr.points
            )
        })
        .collect()
}

pub fn enchanting(p: &Profile) -> Vec<String> {
    let s = p.node_stats();
    let t = s.get("enchanting.transcendence") > 0.0;
    let m = s.get("enchanting.mastery");
    let mut v = vec![
        "Use livros de Encantamento I na bigorna: cada livro igual sobe 1 nível.".to_string(),
        format!(
            "Maestria: +{} níveis acima do máximo vanilla",
            (m * 50.0).floor()
        ),
        format!(
            "Transcendência: {}",
            if t { "ativa — sem teto" } else { "inativa" }
        ),
    ];
    for id in [
        "sharpness",
        "protection",
        "efficiency",
        "unbreaking",
        "fortune",
        "power",
        "looting",
    ] {
        let name = match id {
            "sharpness" => "Afiação",
            "protection" => "Proteção",
            "efficiency" => "Eficiência",
            "unbreaking" => "Inquebrável",
            "fortune" => "Fortuna",
            "power" => "Força",
            "looting" => "Pilhagem",
            _ => id,
        };
        v.push(format!(
            "{name}: limite atual {}",
            super::rustpg_enchant::cap_for(id, m, t)
        ));
    }
    v
}

pub fn combat(p: &Profile, class: &str) -> Vec<String> {
    let s = p.node_stats();
    let class_name = match class {
        "sword" => "Espadas",
        "axe" => "Machados",
        "bow" => "Arcos",
        "crossbow" => "Bestas",
        "mace" => "Maças",
        "trident" => "Tridentes",
        _ => class,
    };
    let mut v = vec![format!("Arma: {class_name}")];
    for (st, name, unit, _) in attributes::WEAPON_STATS {
        let val = s.get(&format!("weapon.{class}.{st}"));
        if val != 0.0 {
            v.push(format!("{name}: {}", fmt_value(*unit, val)));
        }
    }
    v.push(format!(
        "Redução física: {}",
        fmt_value(Unit::Percent, s.get("defense.res_physical"))
    ));
    v
}

pub fn bestiary(p: &Profile) -> Vec<String> {
    let mut records: Vec<_> = p.species.values().collect();
    records.sort_by(|a, b| b.kills.cmp(&a.kills).then_with(|| a.kind.cmp(&b.kind)));
    let empty = records.is_empty();
    let mut out = vec![format!(
        "Bestiário · {} espécies registradas · {} abates",
        records.len(),
        p.kills.values().sum::<u64>()
    )];
    out.extend(records.into_iter().take(30).map(|s| {
        let tiers = if s.tiers.is_empty() { "normal".to_string() } else { s.tiers.join(", ") };
        let drops = if s.drops.is_empty() { "sem drops registrados".to_string() } else { s.drops.join(", ") };
        format!("{} · {} abates · nível máx. {} · maior dano {:.1} · tiers: {} · drops: {} · recompensa #{:05b}",
            s.kind.trim_start_matches("minecraft:"), s.kills, s.max_level, s.best_damage, tiers, drops, s.reward_flags)
    }));
    if empty {
        if p.kills.is_empty() {
            out.push("Nenhuma criatura derrotada ainda.".into());
        } else {
            out.push(
                "Save legado: detalhes por espécie serão preenchidos nos próximos drops.".into(),
            );
            out.extend(p.kills.iter().take(30).map(|(kind, kills)| {
                format!(
                    "{} · {} abates",
                    kind.trim_start_matches("minecraft:"),
                    kills
                )
            }));
        }
    }
    out
}

pub fn mob(kind: &str, m: &MobScale, lost: f64) -> Vec<String> {
    let base = super::rustpg_scaling::base_health(kind);
    let eff = base * m.health_mult;
    vec![
        format!(
            "{} — {} nível {}",
            kind.trim_start_matches("minecraft:"),
            m.tier.name(),
            m.level
        ),
        format!(
            "Vida efetiva: {:.0}/{:.0} (base {base} × {:.2})",
            (eff - lost).max(0.0),
            eff,
            m.health_mult
        ),
        format!(
            "Dano: ×{:.2}  Projéteis: +{:.0}%",
            m.damage_mult,
            m.projectile_bonus * 100.0
        ),
        format!(
            "Armadura {:.1}  Tenacidade {:.1}  Resistência {:.1}%",
            m.armor,
            m.toughness,
            m.resistance * 100.0
        ),
        format!(
            "Crítico {:.1}% ×{:.2}  Resist. crítico {:.1}%",
            m.crit_chance * 100.0,
            1.0 + m.crit_mult,
            m.crit_resist * 100.0
        ),
        format!(
            "Penetração {:.1}%  Regeneração {:.2}/s",
            m.penetration * 100.0,
            m.regen
        ),
        format!(
            "Repulsão resist. {:.0}%  Alcance +{:.2}",
            m.knockback_resist * 100.0,
            m.reach_bonus
        ),
        format!("Velocidade +{:.1}% (teto 10%)", m.speed_bonus * 100.0),
        format!("Loot ×{:.2}  XP ×{:.2}", m.loot_mult, m.xp_mult),
    ]
}

pub fn help() -> Vec<String> {
    vec![
        "/rustpg — personagem".into(),
        "/rustpg status | profs | trees | bestiary | enchant | history".into(),
        "/rustpg tree <árvore> [página] — ex.: /rustpg tree weapon.sword 1".into(),
        "/rustpg buy <node> [quantidade] — ex.: /rustpg buy weapon.sword.damage_flat#1 10".into(),
        "/rustpg info <node>  /rustpg mob — último mob atingido".into(),
        "/rustpg respec — devolve todos os pontos".into(),
        "Interface por toque: Opções > Mods > RustPG > Abrir".into(),
    ]
}
