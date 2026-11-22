// The attribute catalogue. Every attribute is independent: a node changes exactly the
// one attribute it names, and no attribute is derived from another. Categories only
// group them on screen.
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// A flat amount (damage points, armor points, health points).
    Flat,
    /// A fraction shown as a percentage (0.25 = +25%).
    Percent,
    /// A probability; above 1.0 it overflows into guaranteed extra (drops, targets).
    Chance,
    /// A count (extra targets, extra arrows, enchantment levels).
    Count,
    /// Ticks or seconds reduced (shown as %).
    Reduction,
}

#[derive(Clone, Debug)]
pub struct AttrDef {
    pub id: String,
    pub name: String,
    pub category: &'static str,
    pub unit: Unit,
    /// Hard cap on the final value, if any (player movement 0.50).
    pub cap: Option<f64>,
    /// Whether the game can apply it today (see docs/LIMITACOES.md); shown in the viewer.
    pub applied: bool,
}

/// Mob movement scaling ceiling; player movement has a separate 50% cap.
pub const SPEED_CAP: f64 = 0.10;

pub const WEAPONS: &[(&str, &str)] = &[
    ("sword", "Espada"),
    ("axe", "Machado"),
    ("bow", "Arco"),
    ("crossbow", "Besta"),
    ("mace", "Maça"),
    ("trident", "Tridente"),
    ("spear", "Lança"),
    ("unarmed", "Desarmado"),
];

/// Per-weapon stats. Each (weapon, stat) is its own attribute.
pub const WEAPON_STATS: &[(&str, &str, Unit, bool)] = &[
    ("damage_flat", "Dano-base", Unit::Flat, true),
    ("damage_pct", "Dano percentual", Unit::Percent, true),
    ("crit_chance", "Chance crítica", Unit::Chance, true),
    ("crit_mult", "Multiplicador crítico", Unit::Percent, true),
    ("armor_pen", "Penetração de armadura", Unit::Percent, true),
    ("vs_elite", "Dano contra elites", Unit::Percent, true),
    ("vs_boss", "Dano contra chefes", Unit::Percent, true),
    ("vs_flying", "Dano contra voadores", Unit::Percent, true),
    ("bleed_chance", "Chance de sangramento", Unit::Chance, true),
    ("bleed_damage", "Dano do sangramento", Unit::Flat, true),
    ("bleed_duration", "Duração do sangramento", Unit::Flat, true),
    ("lifesteal", "Roubo de vida", Unit::Percent, false),
    ("attack_speed", "Velocidade de ataque", Unit::Percent, false),
    ("combo_damage", "Dano por sequência de golpes", Unit::Percent, true),
    (
        "durability",
        "Eficiência de durabilidade",
        Unit::Chance,
        false,
    ),
    ("reach", "Alcance", Unit::Flat, false),
    ("sweep_width", "Largura do ataque", Unit::Flat, false),
    ("extra_targets", "Alvos adicionais", Unit::Count, false),
    (
        "projectile_speed",
        "Velocidade do projétil",
        Unit::Percent,
        false,
    ),
    ("accuracy", "Precisão", Unit::Percent, false),
    ("draw_speed", "Velocidade de puxada", Unit::Percent, false),
    ("pierce", "Inimigos atravessados", Unit::Count, false),
    ("extra_arrows", "Flechas adicionais", Unit::Count, false),
    ("ammo_recovery", "Recuperar munição", Unit::Chance, false),
    (
        "precise_hit",
        "Dano em acertos precisos",
        Unit::Percent,
        true,
    ),
    ("execute", "Dano em alvos feridos", Unit::Percent, true),
    ("xp", "Experiência da arma", Unit::Percent, true),
];

pub const DEFENSE: &[(&str, &str, Unit, bool)] = &[
    ("max_health_flat", "Vida máxima fixa", Unit::Flat, true),
    (
        "max_health_pct",
        "Vida máxima percentual",
        Unit::Percent,
        true,
    ),
    ("regen", "Regeneração", Unit::Flat, false),
    ("regen_combat", "Regeneração em combate", Unit::Flat, false),
    (
        "regen_out_combat",
        "Regeneração fora de combate",
        Unit::Flat,
        false,
    ),
    ("healing_received", "Cura recebida", Unit::Percent, false),
    ("healing_done", "Cura produzida", Unit::Percent, false),
    ("potion_healing", "Cura de poções", Unit::Percent, false),
    ("absorption", "Absorção", Unit::Flat, false),
    ("armor", "Armadura", Unit::Flat, true),
    ("toughness", "Tenacidade", Unit::Flat, true),
    (
        "res_physical",
        "Redução de dano físico",
        Unit::Percent,
        true,
    ),
    (
        "res_projectile",
        "Redução de projéteis",
        Unit::Percent,
        true,
    ),
    ("res_explosion", "Redução de explosões", Unit::Percent, true),
    ("res_fire", "Resistência a fogo", Unit::Percent, true),
    (
        "res_magic",
        "Resistência a dano mágico",
        Unit::Percent,
        true,
    ),
    ("res_fall", "Redução de queda", Unit::Percent, true),
    ("res_crit", "Resistência a críticos", Unit::Percent, true),
    (
        "res_knockback",
        "Resistência a repulsão",
        Unit::Percent,
        false,
    ),
    (
        "res_debuff",
        "Resistência a efeitos negativos",
        Unit::Percent,
        false,
    ),
    ("res_elite", "Redução contra elites", Unit::Percent, true),
    ("res_boss", "Redução contra chefes", Unit::Percent, true),
    ("block", "Chance de bloqueio", Unit::Chance, true),
    ("thorns", "Espinhos", Unit::Percent, false),
];

pub const ORES: &[(&str, &str)] = &[
    ("coal", "Carvão"),
    ("copper", "Cobre"),
    ("iron", "Ferro"),
    ("gold", "Ouro"),
    ("redstone", "Redstone"),
    ("lapis", "Lápis-lazúli"),
    ("diamond", "Diamante"),
    ("emerald", "Esmeralda"),
    ("quartz", "Quartzo"),
    ("netherite", "Detritos antigos"),
    ("amethyst", "Ametista"),
    ("stone", "Pedra"),
];

pub const CROPS: &[(&str, &str)] = &[
    ("wheat", "Trigo"),
    ("carrots", "Cenoura"),
    ("potatoes", "Batata"),
    ("beetroots", "Beterraba"),
    ("nether_wart", "Fungo do Nether"),
    ("melon", "Melancia"),
    ("pumpkin", "Abóbora"),
    ("cocoa", "Cacau"),
    ("sweet_berry_bush", "Bagas doces"),
    ("sugar_cane", "Cana-de-açúcar"),
    ("torchflower", "Flor-tocha"),
    ("pitcher", "Planta-jarro"),
];

pub const WATERS: &[(&str, &str)] = &[
    ("ocean", "Oceano"),
    ("river", "Rio"),
    ("swamp", "Pântano"),
    ("frozen", "Gelado"),
    ("jungle", "Selva"),
    ("other", "Outros"),
];

/// Profession-wide stats; each (profession, stat) is its own attribute.
pub const PROFESSION_STATS: &[(&str, &str, &[(&str, &str, Unit, bool)])] = &[
    (
        "mining",
        "Mineração",
        &[
            ("speed", "Velocidade de mineração", Unit::Percent, false),
            ("reach", "Alcance de mineração", Unit::Flat, false),
            ("xp", "Experiência", Unit::Percent, true),
            ("double", "Chance de duplicação", Unit::Chance, true),
            ("triple", "Chance de triplicação", Unit::Chance, true),
            ("extra_drops", "Drops extras", Unit::Count, true),
            ("vein", "Mineração de veios", Unit::Count, false),
            ("gem", "Chance de gemas", Unit::Chance, true),
            ("rare", "Drops raros", Unit::Chance, true),
            ("durability", "Durabilidade", Unit::Chance, false),
            (
                "tool_eff",
                "Eficiência de ferramentas",
                Unit::Percent,
                false,
            ),
        ],
    ),
    (
        "farming",
        "Agricultura",
        &[
            ("growth", "Velocidade de crescimento", Unit::Percent, false),
            ("extra_harvest", "Colheitas adicionais", Unit::Chance, true),
            ("seeds", "Sementes extras", Unit::Chance, true),
            ("quality", "Qualidade das colheitas", Unit::Percent, false),
            ("rare", "Drops raros", Unit::Chance, true),
            ("xp", "Experiência por colheita", Unit::Percent, true),
            ("productivity", "Produtividade", Unit::Count, true),
        ],
    ),
    (
        "fishing",
        "Pesca",
        &[
            ("catch_time", "Tempo de captura", Unit::Reduction, false),
            ("treasure", "Chance de tesouros", Unit::Chance, true),
            ("rare_fish", "Peixes raros", Unit::Chance, true),
            ("extra_catch", "Quantidade de capturas", Unit::Chance, true),
            ("quality", "Qualidade dos peixes", Unit::Percent, false),
            ("less_junk", "Redução de lixo", Unit::Percent, false),
            ("special", "Recompensas especiais", Unit::Chance, true),
            ("rod_eff", "Eficiência da vara", Unit::Chance, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "woodcutting",
        "Lenhador",
        &[
            ("double", "Toras duplicadas", Unit::Chance, true),
            ("saplings", "Mudas extras", Unit::Chance, true),
            ("apples", "Maçãs extras", Unit::Chance, true),
            ("xp", "Experiência", Unit::Percent, true),
            ("speed", "Velocidade de corte", Unit::Percent, false),
            ("treefell", "Derrubar árvore", Unit::Count, false),
        ],
    ),
    (
        "excavation",
        "Escavação",
        &[
            ("double", "Duplicação", Unit::Chance, true),
            ("treasure", "Tesouros", Unit::Chance, true),
            ("flint", "Sílex", Unit::Chance, true),
            ("clay", "Argila", Unit::Chance, true),
            ("xp", "Experiência", Unit::Percent, true),
            ("speed", "Velocidade de escavação", Unit::Percent, false),
        ],
    ),
    (
        "husbandry",
        "Pecuária",
        &[
            ("extra_meat", "Carne extra", Unit::Chance, true),
            ("extra_leather", "Couro/lã extra", Unit::Chance, true),
            ("twins", "Filhotes gêmeos", Unit::Chance, false),
            (
                "breed_cooldown",
                "Recarga de reprodução",
                Unit::Reduction,
                false,
            ),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "hunting",
        "Caça",
        &[
            ("extra_drops", "Drops de caça extras", Unit::Chance, true),
            ("trophy", "Troféus", Unit::Chance, true),
            ("xp", "Experiência", Unit::Percent, true),
            ("tracking", "Rastreamento", Unit::Flat, false),
        ],
    ),
    (
        "exploration",
        "Exploração",
        &[
            ("xp", "Experiência", Unit::Percent, true),
            ("discovery", "Descobertas", Unit::Percent, true),
            ("fall_save", "Queda reduzida", Unit::Percent, true),
            ("stamina", "Resistência física", Unit::Percent, false),
        ],
    ),
    (
        "looting",
        "Saqueador",
        &[
            ("mob_drops", "Drops de mobs extras", Unit::Chance, true),
            ("elite_drops", "Drops de elites extras", Unit::Chance, true),
            ("chest", "Itens de baús", Unit::Chance, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "enchanting",
        "Encantamento",
        &[
            (
                "mastery",
                "Maestria (limite de nível extra)",
                Unit::Count,
                true,
            ),
            ("cost", "Redução de custo", Unit::Reduction, false),
            ("table_power", "Poder da mesa", Unit::Count, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "smithing",
        "Ferreiro",
        &[
            ("repair", "Eficiência de reparo", Unit::Percent, false),
            ("anvil_cost", "Custo da bigorna", Unit::Reduction, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "crafting",
        "Artesanato",
        &[
            ("extra_output", "Produção extra", Unit::Chance, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "alchemy",
        "Alquimia",
        &[
            ("duration", "Duração de poções", Unit::Percent, false),
            ("potency", "Potência", Unit::Percent, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "cooking",
        "Culinária",
        &[
            ("saturation", "Saturação", Unit::Percent, false),
            ("extra_output", "Produção extra", Unit::Chance, true),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "trading",
        "Comércio",
        &[
            ("discount", "Desconto", Unit::Percent, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "taming",
        "Domador",
        &[
            ("pet_damage", "Dano de mascotes", Unit::Percent, false),
            ("pet_health", "Vida de mascotes", Unit::Percent, false),
            ("xp", "Experiência", Unit::Percent, true),
        ],
    ),
    (
        "survival",
        "Sobrevivência",
        &[
            ("hunger", "Fome reduzida", Unit::Reduction, false),
            ("drown", "Fôlego", Unit::Percent, false),
            ("xp", "Experiência", Unit::Percent, true),
            ("death_keep", "Manter XP ao morrer", Unit::Percent, true),
        ],
    ),
];

pub const LUCKS: &[(&str, &str)] = &[
    ("general", "Sorte geral"),
    ("mining", "Sorte de mineração"),
    ("fishing", "Sorte de pesca"),
    ("farming", "Sorte de agricultura"),
    ("mobs", "Sorte de mobs"),
    ("bosses", "Sorte de chefes"),
    ("chests", "Sorte de baús"),
    ("exploration", "Sorte de exploração"),
    ("crafting", "Sorte de fabricação"),
];
pub const LUCK_EFFECTS: &[(&str, &str, Unit, bool)] = &[
    ("rarity", "Chance de raridade", Unit::Chance, true),
    ("duplicate", "Chance de duplicação", Unit::Chance, true),
    ("bonus", "Recompensas adicionais", Unit::Chance, true),
    ("special", "Drops especiais", Unit::Chance, true),
    ("quality", "Qualidade dos itens", Unit::Percent, false),
    (
        "extraordinary",
        "Resultados extraordinários",
        Unit::Chance,
        true,
    ),
];

pub const GENERAL: &[(&str, &str, Unit, bool)] = &[
    ("xp_gain", "Experiência geral", Unit::Percent, true),
    (
        "move_speed",
        "Velocidade de movimento",
        Unit::Percent,
        true,
    ),
    ("stamina_max_flat", "Stamina máxima fixa", Unit::Flat, true),
    (
        "stamina_max_pct",
        "Stamina máxima percentual",
        Unit::Percent,
        true,
    ),
    (
        "stamina_regen_flat",
        "Regeneração de stamina",
        Unit::Flat,
        true,
    ),
    (
        "stamina_regen_pct",
        "Regeneração percentual de stamina",
        Unit::Percent,
        true,
    ),
    (
        "stamina_efficiency",
        "Eficiência de stamina",
        Unit::Reduction,
        true,
    ),
    (
        "heavy_attack_cost_reduction",
        "Redução de custo de ataque pesado",
        Unit::Reduction,
        true,
    ),
    (
        "dodge_cost_reduction",
        "Redução de custo de esquiva",
        Unit::Reduction,
        true,
    ),
    (
        "skill_cost_reduction",
        "Redução de custo de habilidade",
        Unit::Reduction,
        true,
    ),
    (
        "skill_points",
        "Pontos adicionais por nível",
        Unit::Count,
        false,
    ),
    ("orb_xp", "Experiência de orbes", Unit::Percent, false),
];

/// Whether the mod applies the attribute in the game today (crpg_main wires it). The
/// others are bought, saved and shown, waiting for a hook point (docs/LIMITACOES.md).
/// Gameplay readiness MUST match a verified callsite in our pinned native
/// adapter, not merely registration of the attribute in Horizon's registry.
/// An effect which has no engine consumer stays in the catalog, but cannot
/// consume skill points until an actual Part/hook/native bridge is installed.
pub fn wired(id: &str) -> bool {
    const MELEE_STATS:&[&str]=&[
        "damage_flat","damage_pct","combo_damage",
        "crit_chance","crit_mult","vs_elite","vs_boss",
        "vs_flying","execute","lifesteal","attack_speed","xp",
    ];
    const DEFENSE:&[&str]=&[
        "max_health_flat","max_health_pct","regen",
        "armor","toughness",
        "res_physical","res_projectile","res_explosion","res_fire",
        "res_magic","res_fall","res_elite","res_boss","res_knockback",
        "block","healing_received","regen_combat","regen_out_combat",
    ];
    const EXACT:&[&str]=&[
        "general.xp_gain","general.orb_xp","general.skill_points",
        "general.move_speed","general.melee_damage","general.ranged_damage",
        "general.resistance_all","general.attack_speed","general.luck_bonus",
        "general.healing_bonus","general.jump_bonus",
        "general.stamina_max_flat","general.stamina_max_pct",
        "general.stamina_regen_flat","general.stamina_regen_pct",
        "general.stamina_efficiency","general.heavy_attack_cost_reduction",
        "general.dodge_cost_reduction","general.skill_cost_reduction",
        "mining.speed","mining.tool_eff","mining.double","mining.triple",
        "mining.extra_drops","mining.gem","mining.rare",
        "farming.extra_harvest","farming.rare","farming.productivity",
        "fishing.treasure","fishing.rare_fish","fishing.extra_catch",
        "fishing.quality","fishing.less_junk","fishing.special","fishing.rod_eff",
        "woodcutting.double","excavation.double","excavation.treasure",
        "hunting.extra_drops","hunting.trophy","looting.mob_drops","looting.elite_drops",
        "survival.hunger","survival.drown","exploration.fall_save","exploration.stamina",
        "luck.general.rarity",
    ];
    if let Some(rest)=id.strip_prefix("weapon.") {
        return rest.split_once('.').is_some_and(|(weapon,stat)|{
            // Launch-time snapshots do not yet exist; ranged disciplines can
            // earn weapon XP but cannot sell melee-only projectile perks.
            if matches!(weapon,"bow"|"crossbow") {return stat=="xp";}
            matches!(weapon,"sword"|"axe"|"mace"|"spear"|"trident"|"unarmed")
                && MELEE_STATS.contains(&stat)
        });
    }
    if let Some(stat)=id.strip_prefix("defense."){
        return DEFENSE.contains(&stat);
    }
    if let Some(raw)=id.strip_prefix("mining.ore.") {
        return raw.rsplit_once('.').is_some_and(|(ore,stat)|
            ORES.iter().any(|(key,_)|*key==ore) && matches!(stat,"xp"|"double"|"rare"));
    }
    if let Some(raw)=id.strip_prefix("farming.crop.") {
        return raw.rsplit_once('.').is_some_and(|(crop,stat)|
            CROPS.iter().any(|(key,_)|*key==crop) && matches!(stat,"xp"|"extra"|"rare"));
    }
    if let Some(raw)=id.strip_prefix("fishing.water.") {
        return raw.rsplit_once('.').is_some_and(|(water,stat)|
            WATERS.iter().any(|(key,_)|*key==water) && matches!(stat,"treasure"|"extra"));
    }
    if let Some(raw)=id.strip_prefix("luck.") {
        return raw.split_once('.').is_some_and(|(category,effect)|
            LUCKS.iter().any(|(key,_)|*key==category) &&
            matches!(effect,"rarity"|"duplicate"|"bonus"|"special"|"quality"|"extraordinary"));
    }
    if let Some((career,stat))=id.split_once('.') {
        if stat=="xp" && PROFESSION_STATS.iter().any(|(key,_,_)|*key==career) {
            return true; // central add_prof_xp intercepts all server XP
        }
    }
    EXACT.contains(&id)
}

fn push(
    out: &mut Vec<AttrDef>,
    id: String,
    name: String,
    category: &'static str,
    unit: Unit,
    _declared: bool,
) {
    let applied = wired(&id);
    let cap = match unit {
        _ if id == "general.move_speed" => Some(0.50),
        _ if id == "general.skill_points" => Some(100.0),
        _ if id == "general.orb_xp" => Some(100.0),
        _ if id == "defense.res_knockback" => Some(0.95),
        _ if id == "fishing.rod_eff" => Some(1.0),
        _ if id == "general.resistance_all" => Some(0.75),
        _ if id == "general.healing_bonus" => Some(3.0),
        _ if id == "general.jump_bonus" => Some(1.5),
        _ if id == "general.melee_damage" || id == "general.ranged_damage"
            || id == "general.attack_speed" => Some(2.0),
        _ if id.ends_with("move_speed") => Some(SPEED_CAP),
        _ if id.ends_with("combo_damage") => Some(0.05),
        Unit::Reduction => Some(0.90),
        _ => None,
    };
    out.push(AttrDef {
        id,
        name,
        category,
        unit,
        cap,
        applied,
    });
}

fn build() -> Vec<AttrDef> {
    let mut out = Vec::new();
    for (g, n, u, a) in GENERAL {
        push(
            &mut out,
            format!("general.{g}"),
            (*n).into(),
            "general",
            *u,
            *a,
        );
    }
    for (w, wn) in WEAPONS {
        for (s, sn, u, a) in WEAPON_STATS {
            push(
                &mut out,
                format!("weapon.{w}.{s}"),
                format!("{sn} ({wn})"),
                "combat",
                *u,
                *a,
            );
        }
    }
    for (s, sn, u, a) in DEFENSE {
        push(
            &mut out,
            format!("defense.{s}"),
            (*sn).into(),
            "defense",
            *u,
            *a,
        );
    }
    for (p, pn, stats) in PROFESSION_STATS {
        for (s, sn, u, a) in *stats {
            push(
                &mut out,
                format!("{p}.{s}"),
                format!("{sn} ({pn})"),
                "profession",
                *u,
                *a,
            );
        }
    }
    for (o, on) in ORES {
        for (s, sn) in [
            ("double", "Duplicação"),
            ("xp", "Experiência"),
            ("rare", "Drops raros"),
        ] {
            push(
                &mut out,
                format!("mining.ore.{o}.{s}"),
                format!("{sn}: {on}"),
                "profession",
                if s == "xp" {
                    Unit::Percent
                } else {
                    Unit::Chance
                },
                true,
            );
        }
    }
    for (c, cn) in CROPS {
        for (s, sn) in [
            ("extra", "Colheita extra"),
            ("xp", "Experiência"),
            ("rare", "Drops raros"),
        ] {
            push(
                &mut out,
                format!("farming.crop.{c}.{s}"),
                format!("{sn}: {cn}"),
                "profession",
                if s == "xp" {
                    Unit::Percent
                } else {
                    Unit::Chance
                },
                true,
            );
        }
    }
    for (w, wn) in WATERS {
        for (s, sn) in [("treasure", "Tesouros"), ("extra", "Captura extra")] {
            push(
                &mut out,
                format!("fishing.water.{w}.{s}"),
                format!("{sn}: {wn}"),
                "profession",
                Unit::Chance,
                *w == "other" || true,
            );
        }
    }
    for (l, ln) in LUCKS {
        for (e, en, u, a) in LUCK_EFFECTS {
            push(
                &mut out,
                format!("luck.{l}.{e}"),
                format!("{en} ({ln})"),
                "luck",
                *u,
                *a,
            );
        }
    }
    // Independently-addressable modifiers used by the imported Puffish nodes.
    // Healing and jump are consumed by Starling 0.9 native hooks
    // (living_heal and living_jump_power), so these are real runtime effects.
    for (id, title, unit) in [
        ("general.melee_damage", "Dano corpo a corpo", Unit::Percent),
        ("general.ranged_damage", "Dano à distância", Unit::Percent),
        ("general.resistance_all", "Resistência geral", Unit::Percent),
        ("general.attack_speed", "Velocidade geral de ataque", Unit::Percent),
        ("general.luck_bonus", "Sorte", Unit::Flat),
        ("general.healing_bonus", "Cura", Unit::Percent),
        ("general.jump_bonus", "Altura de salto", Unit::Percent),
    ] {
        push(&mut out, id.to_string(), title.into(), "reference", unit, true);
    }
    out
}

pub fn all() -> &'static [AttrDef] {
    static ALL: OnceLock<Vec<AttrDef>> = OnceLock::new();
    ALL.get_or_init(build)
}

pub fn get(id: &str) -> Option<&'static AttrDef> {
    // The Skill Tree queries hundreds of attributes per frame. Keep lookups
    // indexed instead of repeatedly scanning the complete catalogue.
    static INDEX: OnceLock<std::collections::BTreeMap<&'static str,usize>>=OnceLock::new();
    let index=INDEX.get_or_init(||all().iter().enumerate()
        .map(|(i,a)|(a.id.as_str(),i)).collect());
    index.get(id).map(|&i|&all()[i])
}

#[cfg(test)] mod availability_gate_tests {
    use super::*;
    #[test]fn effect_availability_requires_actual_server_adapter(){
        for implemented in [
            "general.skill_points","general.orb_xp","general.xp_gain",
            "fishing.xp","fishing.rod_eff","survival.hunger","survival.drown",
            "exploration.fall_save","exploration.stamina",
            "weapon.sword.damage_flat","weapon.sword.xp",
            "weapon.sword.lifesteal","defense.res_fire",
            "defense.block","defense.res_knockback",
            "defense.healing_received","mining.ore.diamond.xp",
            "farming.crop.wheat.xp",
        ] {assert!(wired(implemented),"{implemented}");}
        for implemented in [
            "general.dodge_cost_reduction","general.skill_cost_reduction",
            "general.heavy_attack_cost_reduction","fishing.treasure",
            "mining.double","luck.mobs.duplicate",
            "mining.ore.diamond.double","farming.crop.wheat.extra",
            "mining.ore.stone.xp","farming.crop.melon.xp",
            "fishing.water.ocean.treasure",
        ] {assert!(wired(implemented),"{implemented}");}
        for unimplemented in [
            "weapon.bow.damage_flat","weapon.sword.bleed_chance",
            "weapon.sword.armor_pen","enchanting.mastery","defense.res_crit",
        ] {assert!(!wired(unimplemented),"{unimplemented}");}
    }
}
