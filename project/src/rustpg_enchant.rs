// Enchanting by repetition: each level-I book of an enchantment applied to an item
// raises that enchantment by one level, without limit once mastered. Levels are u32
// (the game's own storage type) and saturate instead of wrapping.

/// Vanilla's max level of each enchantment (26.3), the base of the mastery cap.
pub fn vanilla_max(id: &str) -> u32 {
    match id.trim_start_matches("minecraft:") {
        "sharpness" | "smite" | "bane_of_arthropods" | "efficiency" | "power" | "impaling"
        | "density" | "breach" => 5,
        "protection"
        | "fire_protection"
        | "blast_protection"
        | "projectile_protection"
        | "feather_falling"
        | "piercing" => 4,
        "unbreaking" | "fortune" | "looting" | "luck_of_the_sea" | "lure" | "respiration"
        | "depth_strider" | "thorns" | "sweeping_edge" | "quick_charge" | "loyalty" | "riptide"
        | "soul_speed" | "swift_sneak" | "wind_burst" | "lunge" => 3,
        "knockback" | "fire_aspect" | "punch" | "frost_walker" => 2,
        _ => 1,
    }
}

/// The highest level a player may reach for `id`: vanilla max + mastery levels;
/// unlimited (u32::MAX) with Transcendência.
pub fn cap_for(id: &str, mastery: f64, transcendence: bool) -> u32 {
    if transcendence {
        return u32::MAX;
    }
    // 1 mastery level per 0.02 of the stat = 50 ranks per extra level... scaled up:
    // mastery stat 1.0 → +50 levels.
    let extra = (super::rustpg_num::finite(mastery).max(0.0) * 50.0).floor();
    let extra = if extra >= u32::MAX as f64 {
        u32::MAX
    } else {
        extra as u32
    };
    vanilla_max(id).saturating_add(extra)
}

/// One book of level `book` on an item at `current`: the rule is "+1 per level-I book".
/// A book above level I counts as that many books (it can come from vanilla loot).
pub fn apply_book(current: u32, book: u32) -> u32 {
    current.saturating_add(book.max(1))
}

/// Applies `n` books in one step (bulk-safe, no loop).
pub fn apply_books(current: u32, n: u64) -> u32 {
    let n = n.min(u32::MAX as u64) as u32;
    current.saturating_add(n)
}

/// Anvil level cost for raising to `new_level`: small and logarithmic so 1000 books
/// stay affordable, never 40+ (vanilla's "Too Expensive!" in survival).
pub fn anvil_cost(new_level: u32) -> i32 {
    let c = 1.0 + (new_level as f64 + 1.0).log2();
    c.clamp(1.0, 39.0) as i32
}

/// The roman numeral (I–X) or plain number above, for tooltips.
pub fn level_label(level: u32) -> String {
    const R: [&str; 11] = [
        "", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X",
    ];
    if (1..=10).contains(&level) {
        R[level as usize].into()
    } else {
        level.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn book_merges_raise_levels_and_saturate_safely() {
        assert_eq!(apply_book(4, 1), 5);
        assert_eq!(apply_book(4, 3), 7);
        assert_eq!(apply_book(u32::MAX - 1, 3), u32::MAX);
    }
}
