// Levels and experience. The general level uses LevelUP's default curve
// (`0.5 × 2^(x/10) + 100`, from levelup-1.0.0's Config) up to level 100, then grows
// polynomially so very high levels stay reachable and never overflow.
// Each proficiency has its own XP and level; XP from one activity never feeds another.

pub const PROFICIENCIES: &[(&str, &str)] = &[
    ("sword", "Espadas"),
    ("axe", "Machados"),
    ("bow", "Arcos"),
    ("crossbow", "Bestas"),
    ("mace", "Maças"),
    ("trident", "Tridentes"),
    ("spear", "Lanças"),
    ("unarmed", "Desarmado"),
    ("survival", "Sobrevivência"),
    ("mining", "Mineração"),
    ("excavation", "Escavação"),
    ("woodcutting", "Lenhador"),
    ("farming", "Agricultura"),
    ("fishing", "Pesca"),
    ("hunting", "Caça"),
    ("husbandry", "Pecuária"),
    ("exploration", "Exploração"),
    ("looting", "Saqueador"),
    ("enchanting", "Encantamento"),
    ("smithing", "Ferreiro"),
    ("crafting", "Artesanato"),
    ("alchemy", "Alquimia"),
    ("cooking", "Culinária"),
    ("trading", "Comércio"),
    ("taming", "Domador"),
];

pub fn proficiency_name(id: &str) -> &str {
    PROFICIENCIES
        .iter()
        .find(|(p, _)| *p == id)
        .map_or(id, |(_, n)| n)
}

/// XP from level `level` to `level + 1` of the general level.
pub fn general_xp_to_next(level: u32) -> u64 {
    let x = level as f64;
    let v = if level <= 100 {
        0.5 * 2f64.powf(x / 10.0) + 100.0
    } else {
        // Continues from level 100's value (612) with a cubic-ish tail.
        612.0 + 40.0 * (x - 100.0).powf(1.6)
    };
    to_u64(v.round())
}

/// XP from level `level` to `level + 1` of a proficiency.
pub fn prof_xp_to_next(level: u32) -> u64 {
    let x = level as f64;
    to_u64((50.0 + 25.0 * x.powf(1.45)).round())
}

fn to_u64(v: f64) -> u64 {
    if !v.is_finite() || v >= u64::MAX as f64 {
        u64::MAX
    } else if v <= 1.0 {
        1
    } else {
        v as u64
    }
}

/// Skill points for reaching general level `level`.
pub fn general_points_for(level: u32) -> u64 {
    if level % 10 == 0 {
        5
    } else {
        2
    }
}
/// Points for reaching a proficiency's level `level` (spent only in that tree).
pub fn prof_points_for(level: u32) -> u64 {
    if level % 5 == 0 {
        3
    } else {
        1
    }
}

/// Adds `amount` XP to (level, xp) with the given curve; returns the levels gained.
/// Saturates at u32::MAX levels; never loops more than needed (bulk-safe).
pub fn add_xp(level: &mut u32, xp: &mut u64, amount: u64, curve: fn(u32) -> u64) -> u32 {
    *xp = xp.saturating_add(amount);
    let mut gained = 0u32;
    while *level < u32::MAX {
        let need = curve(*level);
        if *xp < need {
            break;
        }
        *xp -= need;
        *level += 1;
        gained += 1;
        if gained >= 100_000 {
            break; // one call never spins forever; the rest levels next time
        }
    }
    gained
}

/// XP for killing a mob: LevelUP's `maxHealth / 5`, times the tier's multiplier.
pub fn kill_xp(base_max_health: f64, xp_mult: f64) -> u64 {
    to_u64(super::crpg_num::safe_div(base_max_health, 5.0) * xp_mult.max(0.0)).max(1)
}
