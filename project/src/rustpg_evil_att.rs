// Mutation encyclopedia data. Discovery is driven by real scaling/mob events.

#[derive(Clone, Debug, PartialEq)]
pub struct MutationDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub rarity: &'static str,
    pub description: &'static str,
}
pub const MUTATIONS: &[MutationDef] = &[
    MutationDef {
        id: "venomous",
        name: "Peçonhento",
        category: "Elemental",
        rarity: "Incomum",
        description: "Toxina concentrada: +20% de dano.",
    },
    MutationDef {
        id: "flame",
        name: "Flamejante",
        category: "Elemental",
        rarity: "Raro",
        description: "+25% de dano e +10% de resistência.",
    },
    MutationDef {
        id: "frost",
        name: "Glacial",
        category: "Elemental",
        rarity: "Raro",
        description: "Corpo congelado: quase imune a repulsão, +5% de resistência.",
    },
    MutationDef {
        id: "storm",
        name: "Tempestuoso",
        category: "Elemental",
        rarity: "Épico",
        description: "Descargas: +60% de dano à distância e +10% de dano.",
    },
    MutationDef {
        id: "armored",
        name: "Blindado",
        category: "Defesa",
        rarity: "Incomum",
        description: "Aumenta armadura e tenacidade.",
    },
    MutationDef {
        id: "vampiric",
        name: "Vampírico",
        category: "Ofensivo",
        rarity: "Épico",
        description: "Regeneração ×3 e +10% de dano.",
    },
    MutationDef {
        id: "berserk",
        name: "Frenético",
        category: "Ofensivo",
        rarity: "Raro",
        description: "+50% de dano abaixo de metade da vida.",
    },
    MutationDef {
        id: "regenerative",
        name: "Regenerativo",
        category: "Defesa",
        rarity: "Raro",
        description: "Regeneração ×4.",
    },
    MutationDef {
        id: "abyssal",
        name: "Abissal",
        category: "Chefe",
        rarity: "Lendário",
        description: "+25% de resistência, +30% de armadura, imune a críticos fracos.",
    },
];
/// Spawn weight of a rarity (out of the total weight of the catalog).
pub fn rarity_weight(rarity: &str) -> f64 {
    match rarity {
        "Incomum" => 10.0,
        "Raro" => 5.0,
        "Épico" => 2.0,
        "Lendário" => 0.5,
        _ => 1.0,
    }
}

/// Chance that a mob of this level/tier carries a mutation at all.
/// Bosses and high tiers mutate more often, like dungeon-break elites.
pub fn mutation_chance(level: u32, tier_rank: u8) -> f64 {
    let base = 0.03 + (level as f64 / 4000.0).min(0.07);
    (base + tier_rank as f64 * 0.04).min(0.45)
}

/// Deterministic, rarity-weighted pick. `u_gate` and `u_pick` are uniform [0,1).
/// "abyssal" is reserved for elite tiers and above (tier_rank >= 2).
pub fn roll(level: u32, tier_rank: u8, u_gate: f64, u_pick: f64) -> Option<&'static str> {
    if u_gate >= mutation_chance(level, tier_rank) {
        return None;
    }
    let pool: Vec<&MutationDef> = MUTATIONS
        .iter()
        .filter(|m| m.id != "abyssal" || tier_rank >= 2)
        .collect();
    let total: f64 = pool.iter().map(|m| rarity_weight(m.rarity)).sum();
    let mut t = u_pick.clamp(0.0, 0.999_999) * total;
    for m in &pool {
        let w = rarity_weight(m.rarity);
        if t < w {
            return Some(m.id);
        }
        t -= w;
    }
    pool.last().map(|m| m.id)
}

/// Damage factor of a "berserk" mob from the fraction of health it has lost.
pub fn berserk_factor(id: Option<&str>, lost_fraction: f64) -> f64 {
    match id {
        Some("berserk") if lost_fraction >= 0.5 => 1.5,
        _ => 1.0,
    }
}

pub fn get(id: &str) -> Option<&'static MutationDef> {
    MUTATIONS.iter().find(|m| m.id == id)
}
