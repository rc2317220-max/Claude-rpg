// Damage math. Outgoing: per-weapon independent stats. Incoming: per-damage-type
// independent reductions. Mob stats come from scaling.rs.
use super::crpg_num::{diminishing, finite, Rng};
use super::crpg_profile::Stats;
use super::crpg_scaling::{MobScale, Tier};

/// The weapon class of a held item id. Unknown items count as unarmed.
pub fn weapon_class(item: &str) -> &'static str {
    let i = item.trim_start_matches("minecraft:");
    if i.ends_with("_sword") {
        "sword"
    } else if i.ends_with("_axe") {
        "axe"
    } else if i == "bow" {
        "bow"
    } else if i == "crossbow" {
        "crossbow"
    } else if i == "mace" {
        "mace"
    } else if i == "trident" {
        "trident"
    } else if i.ends_with("_spear") {
        "spear"
    } else {
        "unarmed"
    }
}

pub fn is_flying(kind: &str) -> bool {
    matches!(
        kind.trim_start_matches("minecraft:"),
        "phantom"
            | "ghast"
            | "happy_ghast"
            | "blaze"
            | "vex"
            | "bat"
            | "parrot"
            | "bee"
            | "allay"
            | "breeze"
            | "ender_dragon"
            | "wither"
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub amount: f64,
    pub crit: bool,
    pub bleed: Option<(f64, u32)>,
    /// True when the player's block roll succeeded (damage reduced to the 1-point scratch).
    pub blocked: bool,
}

/// A player's hit on a mob. `base` is what the game computed (weapon, vanilla
/// enchantments, vanilla crit). `mob` is the target's scale (None: unscaled).
pub fn player_hits_mob(
    stats: &Stats,
    class: &str,
    base: f64,
    mob: Option<&MobScale>,
    target_kind: &str,
    target_hurt_fraction: f64,
    rng: &mut Rng,
) -> Hit {
    let w = |s: &str| stats.get(&format!("weapon.{class}.{s}"));
    let mut dmg = finite(base.max(0.0) + w("damage_flat"));
    dmg *= 1.0 + w("damage_pct");
    if let Some(m) = mob {
        if m.tier >= Tier::Elite && m.tier < Tier::MiniBoss {
            dmg *= 1.0 + w("vs_elite");
        }
        if m.tier >= Tier::MiniBoss {
            dmg *= 1.0 + w("vs_boss");
        }
    }
    if is_flying(target_kind) {
        dmg *= 1.0 + w("vs_flying");
    }
    if target_hurt_fraction >= 0.5 {
        dmg *= 1.0 + w("execute");
    }
    // Crit: chance above 100% doesn't stack crits; the excess is wasted by design.
    let crit_chance = w("crit_chance") - mob.map_or(0.0, |m| m.crit_resist);
    let crit = rng.chance(crit_chance);
    if crit {
        dmg *= 1.5 + w("crit_mult");
    }
    // A defesa vanilla do mob será aplicada por LivingEntity.hurtServer.
    // Não aplicar uma segunda curva de armadura (m.armor) neste hook.
    // Penetração RPG age só sobre a resistência extra do sistema; não
    // prometemos ignorar a armadura vanilla sem um hook pós-armadura.
    if let Some(m) = mob {
        let pen = w("armor_pen").clamp(0.0, 1.0);
        dmg = super::crpg_num::safe_div(dmg, m.health_mult.max(1.0));
        dmg *= 1.0 - m.resistance.clamp(0.0, 0.75) * (1.0 - pen);
        if base > 0.0 { dmg = dmg.max(0.05); }
    }
    let bleed = rng.chance(w("bleed_chance")).then(|| {
        (
            w("bleed_damage").max(0.5),
            (40.0 + w("bleed_duration") * 20.0).min(20_000.0) as u32,
        )
    });
    Hit {
        amount: finite(dmg.max(0.0)),
        crit,
        bleed,
        blocked: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpg_does_not_reapply_vanilla_armor() {
        let stats = Stats::default();
        let mut soft = super::super::crpg_scaling::stats_for(1, Tier::Normal);
        soft.health_mult = 1.0;
        soft.armor = 10.0;
        soft.toughness = 0.0;
        let mut tough = soft.clone();
        tough.toughness = 20.0;

        let soft_hit = player_hits_mob(
            &stats,
            "sword",
            10.0,
            Some(&soft),
            "zombie",
            0.0,
            &mut Rng(7),
        );
        let tough_hit = player_hits_mob(
            &stats,
            "sword",
            10.0,
            Some(&tough),
            "zombie",
            0.0,
            &mut Rng(7),
        );
        assert_eq!(tough_hit.amount, soft_hit.amount,
            "armadura e tenacidade devem ficar no cálculo vanilla, sem dupla mitigação");
    }

    #[test]
    fn mob_resistance_applies_after_health_scaling() {
        let stats = Stats::default();
        let mut mob = super::super::crpg_scaling::stats_for(1, Tier::Normal);
        mob.health_mult = 10.0;
        mob.armor = 0.0;
        mob.resistance = 0.5;

        let hit = player_hits_mob(
            &stats,
            "sword",
            10.0,
            Some(&mob),
            "zombie",
            0.0,
            &mut Rng(7),
        );
        assert!((hit.amount - 0.5).abs() < 1e-9, "{}", hit.amount);
    }
}

/// The reduction attribute for a vanilla damage type id.
pub fn reduction_attr(damage_type: &str) -> &'static str {
    let t = damage_type.trim_start_matches("minecraft:");
    match t {
        "arrow" | "trident" | "mob_projectile" | "fireball" | "wither_skull" | "spit"
        | "wind_charge" | "fireworks" => "defense.res_projectile",
        "explosion" | "player_explosion" | "bad_respawn_point" => "defense.res_explosion",
        "in_fire" | "on_fire" | "lava" | "hot_floor" | "campfire" | "unattributed_fireball" => {
            "defense.res_fire"
        }
        "magic" | "indirect_magic" | "wither" | "dragon_breath" | "sonic_boom" | "thorns" => {
            "defense.res_magic"
        }
        "fall" | "stalagmite" | "fly_into_wall" => "defense.res_fall",
        _ => "defense.res_physical",
    }
}

/// Damage a player takes. `amount` is the game's value after vanilla armor.
pub fn player_takes(
    stats: &Stats,
    damage_type: &str,
    amount: f64,
    attacker: Option<&MobScale>,
    projectile: bool,
    rng: &mut Rng,
) -> Hit {
    let mut dmg = finite(amount.max(0.0));
    let mut crit = false;
    if let Some(m) = attacker {
        dmg *= m.damage_mult.max(0.0);
        if projectile {
            dmg *= 1.0 + m.projectile_bonus;
        }
        let crit_chance = m.crit_chance * (1.0 - stats.get("defense.res_crit").clamp(0.0, 1.0));
        if rng.chance(crit_chance) {
            crit = true;
            dmg *= 1.0 + m.crit_mult;
        }
        if m.tier >= Tier::Elite && m.tier < Tier::MiniBoss {
            dmg *= 1.0 - diminishing(stats.get("defense.res_elite"), 1.0, 0.75);
        }
        if m.tier >= Tier::MiniBoss {
            dmg *= 1.0 - diminishing(stats.get("defense.res_boss"), 1.0, 0.75);
        }
        // Mob penetration ignores part of the player's RPG armor below.
    }
    let pen = attacker.map_or(0.0, |m| m.penetration.clamp(0.0, 1.0));
    let armor = (stats.get("defense.armor") * (1.0 - pen)).max(0.0);
    // Damage Threshold replaces vanilla's diminishing percentage formula for
    // physical hits. Fire, magic and fall damage use their typed resistance.
    if reduction_attr(damage_type) == "defense.res_physical" {
        dmg = (dmg - armor).max(1.0);
    }
    // Typed reduction: its own attribute, diminishing to at most 85%.
    dmg *= 1.0 - diminishing(stats.get(reduction_attr(damage_type)), 1.0, 0.85);
    // Effective max health: +X health and +Y% health divide incoming damage
    // against the vanilla 20 (no max-health setter exists in the hook API).
    let max_hp =
        (20.0 + stats.get("defense.max_health_flat")) * (1.0 + stats.get("defense.max_health_pct"));
    dmg = dmg * 20.0 / max_hp.max(20.0);
    let blocked = rng.chance(diminishing(stats.get("defense.block"), 1.0, 0.5));
    if blocked {
        dmg = 1.0;
    }
    Hit {
        amount: finite(dmg.max(1.0)),
        crit,
        bleed: None,
        blocked,
    }
}
