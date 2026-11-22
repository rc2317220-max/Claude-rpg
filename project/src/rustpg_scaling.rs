// Mob scaling. A mob gets its level and tier once, when it first joins the world;
// the result is stored on the mob (an entity attachment) and read back on reload,
// never re-rolled or multiplied again. Stats are absolute values computed from the
// stored level and tier, so loading a mob ten times gives the same mob.
use super::rustpg_attributes::SPEED_CAP;
use super::rustpg_num::{finite, mix, unit};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Normal,
    Fortified,
    Elite,
    Champion,
    MiniBoss,
    Boss,
    Legendary,
}

impl Tier {
    pub const ALL: [Tier; 7] = [
        Tier::Normal,
        Tier::Fortified,
        Tier::Elite,
        Tier::Champion,
        Tier::MiniBoss,
        Tier::Boss,
        Tier::Legendary,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Tier::Normal => "normal",
            Tier::Fortified => "fortalecido",
            Tier::Elite => "elite",
            Tier::Champion => "campeao",
            Tier::MiniBoss => "minichefe",
            Tier::Boss => "chefe",
            Tier::Legendary => "lendario",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Tier::Normal => "Normal",
            Tier::Fortified => "Fortalecido",
            Tier::Elite => "Elite",
            Tier::Champion => "Campeão",
            Tier::MiniBoss => "Mini-chefe",
            Tier::Boss => "Chefe",
            Tier::Legendary => "Chefe lendário",
        }
    }
    pub fn from_key(k: &str) -> Option<Tier> {
        Tier::ALL.into_iter().find(|t| t.key() == k)
    }
    /// Each tier's own multipliers: (health, damage, armor, crit, loot, xp).
    fn table(self) -> (f64, f64, f64, f64, f64, f64) {
        match self {
            Tier::Normal => (1.0, 1.0, 0.0, 0.02, 1.0, 1.0),
            Tier::Fortified => (1.6, 1.2, 4.0, 0.04, 1.3, 1.5),
            Tier::Elite => (2.5, 1.5, 8.0, 0.08, 2.0, 3.0),
            Tier::Champion => (4.0, 1.9, 14.0, 0.12, 3.0, 5.0),
            Tier::MiniBoss => (7.0, 2.4, 22.0, 0.15, 5.0, 10.0),
            Tier::Boss => (10.0, 2.8, 30.0, 0.18, 8.0, 20.0),
            Tier::Legendary => (18.0, 3.5, 45.0, 0.25, 15.0, 50.0),
        }
    }
}

/// Vanilla bosses always get a boss tier.
pub fn boss_tier(kind: &str) -> Option<Tier> {
    match kind {
        "rustpg:ashen_sentinel" | "rustpg:eclipse_ogre" | "rustpg:rune_colossus" => return Some(Tier::Boss),
        "rustpg:crystal_wyrm" | "rustpg:storm_gryphon" => return Some(Tier::MiniBoss),
        "rustpg:rift_hunter" | "rustpg:grave_knight" | "rustpg:rift_mantis" | "rustpg:dusk_revenant" | "rustpg:abyss_weaver" => return Some(Tier::Elite),
        "rustpg:mana_hound" | "rustpg:ember_imp" => return Some(Tier::Normal),
        _=>{},
    }
    match kind.trim_start_matches("minecraft:") {
        "ender_dragon" | "wither" => Some(Tier::Legendary),
        "warden" | "elder_guardian" => Some(Tier::Boss),
        "ravager" | "evoker" | "piglin_brute" | "breeze" => Some(Tier::MiniBoss),
        _ => None,
    }
}

/// Mobs that scale: monsters and bosses. Passive animals stay vanilla.
pub fn scales(kind: &str) -> bool {
    if matches!(kind,
        "rustpg:ashen_sentinel"|"rustpg:rift_hunter"|"rustpg:mana_hound"|
        "rustpg:grave_knight"|"rustpg:rift_mantis"|"rustpg:crystal_wyrm"|
        "rustpg:eclipse_ogre"|"rustpg:dusk_revenant"|"rustpg:storm_gryphon"|
        "rustpg:rune_colossus"|"rustpg:ember_imp"|"rustpg:abyss_weaver"){return true;}
    matches!(
        kind.trim_start_matches("minecraft:"),
        "zombie"
            | "husk"
            | "drowned"
            | "zombie_villager"
            | "skeleton"
            | "stray"
            | "bogged"
            | "wither_skeleton"
            | "creeper"
            | "spider"
            | "cave_spider"
            | "enderman"
            | "endermite"
            | "silverfish"
            | "witch"
            | "slime"
            | "magma_cube"
            | "blaze"
            | "ghast"
            | "phantom"
            | "pillager"
            | "vindicator"
            | "evoker"
            | "vex"
            | "ravager"
            | "piglin"
            | "piglin_brute"
            | "zombified_piglin"
            | "hoglin"
            | "zoglin"
            | "guardian"
            | "elder_guardian"
            | "shulker"
            | "warden"
            | "breeze"
            | "creaking"
            | "wither"
            | "ender_dragon"
            | "illusioner"
            | "parched"
            | "camel_husk"
            | "zombie_nautilus"
    )
}

/// Vanilla max health, for effective-HP tracking (the hook API has no mob reader).
pub fn base_health(kind: &str) -> f64 {
    if kind=="rustpg:ashen_sentinel"{return 240.0}
    if kind=="rustpg:rift_hunter"{return 52.0}
    if kind=="rustpg:mana_hound"{return 38.0}
    if kind=="rustpg:grave_knight"{return 92.0}
    if kind=="rustpg:rift_mantis"{return 72.0}
    if kind=="rustpg:crystal_wyrm"{return 185.0}
    if kind=="rustpg:eclipse_ogre"{return 460.0}
    if kind=="rustpg:dusk_revenant"{return 78.0}
    if kind=="rustpg:storm_gryphon"{return 160.0}
    if kind=="rustpg:rune_colossus"{return 385.0}
    if kind=="rustpg:ember_imp"{return 27.0}
    if kind=="rustpg:abyss_weaver"{return 105.0}
    match kind.trim_start_matches("minecraft:") {
        "spider" => 16.0,
        "cave_spider" | "endermite" | "silverfish" | "vex" => 8.0,
        "enderman" => 40.0,
        "witch" => 26.0,
        "ghast" => 10.0,
        "ravager" => 100.0,
        "evoker" | "vindicator" | "pillager" => 24.0,
        "piglin_brute" => 50.0,
        "hoglin" | "zoglin" => 40.0,
        "guardian" => 30.0,
        "elder_guardian" => 80.0,
        "shulker" => 30.0,
        "warden" => 500.0,
        "breeze" => 30.0,
        "wither" => 300.0,
        "ender_dragon" => 200.0,
        "phantom" => 20.0,
        _ => 20.0,
    }
}

/// What the world has reached, stored on the level.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldProgress {
    pub bosses: Vec<String>,
    pub dimensions: Vec<String>,
}

impl WorldProgress {
    pub fn bonus(&self) -> f64 {
        self.bosses.len() as f64 * 4.0 + self.dimensions.len() as f64 * 2.0
    }
}

pub fn dimension_bonus(dimension: &str) -> f64 {
    match dimension {
        "minecraft:the_nether" => 8.0,
        "minecraft:the_end" => 18.0,
        _ => 0.0,
    }
}

/// Players near the spawn: (power, distance). Several players → between average and
/// strongest, so a strong player doesn't make mobs impossible for a weak one, and
/// more players bring a modest bump.
pub fn group_power(players: &[(f64, f64)]) -> f64 {
    let near: Vec<f64> = players
        .iter()
        .filter(|(_, d)| *d <= 96.0)
        .map(|(p, _)| finite(*p))
        .collect();
    if near.is_empty() {
        return 0.0;
    }
    let max = near.iter().cloned().fold(0.0, f64::max);
    let avg = near.iter().sum::<f64>() / near.len() as f64;
    let crowd = 1.0 + 0.1 * (near.len().min(6) as f64 - 1.0);
    finite((avg * 0.6 + max * 0.4) * crowd)
}

#[derive(Clone, Debug, PartialEq)]
pub struct MobScale {
    pub version: u32,
    pub level: u32,
    pub tier: Tier,
    pub health_mult: f64,
    pub damage_mult: f64,
    pub armor: f64,
    pub toughness: f64,
    pub resistance: f64,
    pub penetration: f64,
    pub crit_chance: f64,
    pub crit_mult: f64,
    pub crit_resist: f64,
    pub regen: f64,
    pub knockback_resist: f64,
    pub reach_bonus: f64,
    pub projectile_bonus: f64,
    pub loot_mult: f64,
    pub xp_mult: f64,
    /// Stored for the viewer; capped at +10%.
    pub speed_bonus: f64,
    /// Evil Att. mutation id (see rustpg_evil_att), already applied to the stats.
    pub mutation: Option<&'static str>,
}

/// Rank of a tier for mutation odds: Normal 0 .. Boss 5+.
fn tier_rank(t: Tier) -> u8 {
    let mut r = 0u8;
    for x in [
        Tier::Fortified,
        Tier::Elite,
        Tier::Champion,
        Tier::MiniBoss,
        Tier::Boss,
    ] {
        if t >= x {
            r += 1;
        }
    }
    r
}

/// Applies a mutation's effect on top of the (level, tier) stats. Only fields the
/// combat formulas really read are touched.
pub fn with_mutation(mut m: MobScale, id: Option<&str>) -> MobScale {
    let Some(def) = id.and_then(super::rustpg_evil_att::get) else {
        m.mutation = None;
        return m;
    };
    m.mutation = Some(def.id);
    match def.id {
        "venomous" => m.damage_mult *= 1.2,
        "flame" => {
            m.damage_mult *= 1.25;
            m.resistance = (m.resistance + 0.10).min(0.75);
        }
        "frost" => {
            m.knockback_resist = (m.knockback_resist + 0.5).min(0.95);
            m.resistance = (m.resistance + 0.05).min(0.75);
        }
        "storm" => {
            m.projectile_bonus += 0.6;
            m.damage_mult *= 1.1;
        }
        "armored" => {
            m.armor *= 1.5;
            m.toughness *= 1.5;
        }
        "vampiric" => {
            m.regen *= 3.0;
            m.damage_mult *= 1.1;
        }
        "regenerative" => m.regen *= 4.0,
        "abyssal" => {
            m.resistance = (m.resistance + 0.25).min(0.75);
            m.armor *= 1.3;
            m.crit_resist = (m.crit_resist + 0.2).min(0.7);
        }
        _ => {}
    }
    m.xp_mult *= 1.5;
    m.loot_mult *= 1.3;
    m
}

/// Rolls a mob once. `seed` must be stable for the mob (entity id + world seed) so a
/// replayed roll gives the same answer.
pub fn roll(
    kind: &str,
    dimension: &str,
    power: f64,
    world: &WorldProgress,
    day: u64,
    seed: u64,
) -> MobScale {
    let level_f =
        finite(power + dimension_bonus(dimension) + world.bonus() + (day as f64).sqrt()).max(0.0);
    let jitter = (unit(mix(seed)) * 0.3 - 0.15) * level_f;
    let level = (level_f + jitter).round().clamp(1.0, 1_000_000.0) as u32;
    let tier = boss_tier(kind).unwrap_or_else(|| {
        let r = unit(mix(seed ^ 0xA5A5));
        // Rarer tiers become possible as the level grows.
        let l = level as f64;
        let elite = (0.02 + l / 2000.0).min(0.12);
        if r < (0.002 + l / 50_000.0).min(0.01) {
            Tier::MiniBoss
        } else if r < 0.01 + (l / 5000.0).min(0.04) {
            Tier::Champion
        } else if r < 0.03 + elite {
            Tier::Elite
        } else if r < 0.15 + elite {
            Tier::Fortified
        } else {
            Tier::Normal
        }
    });
    let mutation = super::rustpg_evil_att::roll(
        level,
        tier_rank(tier),
        unit(mix(seed ^ 0x5EED_E711)),
        unit(mix(seed ^ 0x0BAD_C0DE)),
    );
    with_mutation(stats_for(level, tier), mutation)
}

/// The absolute stats of a (level, tier). Pure: same input, same mob.
pub fn stats_for(level: u32, tier: Tier) -> MobScale {
    let l = level as f64;
    let (h, d, a, c, loot, xp) = tier.table();
    // ScalingMobs' default is +3% health and damage per step; here per level.
    MobScale {
        version: 1,
        level,
        tier,
        health_mult: finite(h * (1.0 + 0.03 * l)),
        damage_mult: finite(d * (1.0 + 0.025 * l)),
        armor: finite(a + 0.4 * l),
        toughness: finite(a * 0.5 + 0.15 * l),
        resistance: (0.002 * l).min(0.5),
        penetration: (0.003 * l).min(0.6),
        crit_chance: (c + 0.001 * l).min(0.6),
        crit_mult: 0.5 + 0.005 * l,
        crit_resist: (0.001 * l).min(0.5),
        regen: 0.05 * l,
        knockback_resist: (0.004 * l).min(0.9),
        reach_bonus: (0.01 * l).min(1.5),
        projectile_bonus: 0.02 * l,
        loot_mult: finite(loot * (1.0 + 0.01 * l)),
        xp_mult: finite(xp * (1.0 + 0.02 * l)),
        speed_bonus: (0.0005 * l).min(SPEED_CAP),
        mutation: None,
    }
}

/// Boss phase from the fraction of effective health lost: 1, 2 or 3.
pub fn boss_phase(lost_fraction: f64) -> u8 {
    if lost_fraction >= 0.66 {
        3
    } else if lost_fraction >= 0.33 {
        2
    } else {
        1
    }
}

/// The extra damage factor of a boss phase.
pub fn phase_damage(phase: u8) -> f64 {
    match phase {
        3 => 1.6,
        2 => 1.25,
        _ => 1.0,
    }
}

/// Minions a boss summons when it enters a phase.
pub fn phase_minions(kind: &str, phase: u8) -> &'static [&'static str] {
    if phase < 2 {
        return &[];
    }
    match kind.trim_start_matches("minecraft:") {
        "wither" => &["minecraft:wither_skeleton", "minecraft:wither_skeleton"],
        "ender_dragon" => &["minecraft:enderman", "minecraft:phantom"],
        "warden" => &[
            "minecraft:silverfish",
            "minecraft:silverfish",
            "minecraft:silverfish",
        ],
        "elder_guardian" => &["minecraft:guardian", "minecraft:guardian"],
        _ => &["minecraft:zombie"],
    }
}

/// Anti-farming: XP factor for the n-th kill of the same kind within the window.
pub fn farm_factor(recent_same_kind: u32) -> f64 {
    (1.0 / (1.0 + recent_same_kind as f64 * 0.08)).max(0.1)
}

pub fn encode(m: &MobScale) -> String {
    match m.mutation {
        Some(id) => format!(
            "v={};lvl={};tier={};mut={id}",
            m.version,
            m.level,
            m.tier.key()
        ),
        None => format!("v={};lvl={};tier={}", m.version, m.level, m.tier.key()),
    }
}

/// Only level and tier are stored; stats are recomputed from them, so a mob can never
/// accumulate multipliers.
pub fn decode(s: &str) -> Option<MobScale> {
    let mut level = None;
    let mut tier = None;
    let mut mutation: Option<&str> = None;
    for part in s.split(';') {
        match part.split_once('=') {
            Some(("lvl", v)) => level = v.parse::<u32>().ok(),
            Some(("tier", v)) => tier = Tier::from_key(v),
            Some(("mut", v)) => mutation = Some(v),
            _ => {}
        }
    }
    Some(with_mutation(stats_for(level?.max(1), tier?), mutation))
}
