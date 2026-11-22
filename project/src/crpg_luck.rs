// Activities and loot. Each broken block maps to one activity (proficiency) and,
// for ores and crops, one specialisation. Luck is split by activity; general luck
// adds at half weight into the SAME roll, so no bonus is ever applied twice.
use super::crpg_num::{finite, Rng};
use super::crpg_profile::Stats;

#[derive(Clone, Debug, PartialEq)]
pub struct Activity {
    pub prof: &'static str,
    /// Specialisation key: an ore, a crop or a water.
    pub special: Option<&'static str>,
    pub xp: u64,
    /// The luck category whose stats apply.
    pub luck: &'static str,
}

pub const RESOURCE_BLOCK_IDS:&[&str]=&[
    "minecraft:coal_ore","minecraft:deepslate_coal_ore",
    "minecraft:copper_ore","minecraft:deepslate_copper_ore",
    "minecraft:iron_ore","minecraft:deepslate_iron_ore",
    "minecraft:gold_ore","minecraft:deepslate_gold_ore","minecraft:nether_gold_ore",
    "minecraft:redstone_ore","minecraft:deepslate_redstone_ore",
    "minecraft:lapis_ore","minecraft:deepslate_lapis_ore",
    "minecraft:diamond_ore","minecraft:deepslate_diamond_ore",
    "minecraft:emerald_ore","minecraft:deepslate_emerald_ore",
    "minecraft:nether_quartz_ore","minecraft:ancient_debris","minecraft:amethyst_cluster",
    "minecraft:stone","minecraft:deepslate","minecraft:cobblestone","minecraft:andesite",
    "minecraft:diorite","minecraft:granite","minecraft:tuff","minecraft:netherrack",
    "minecraft:blackstone","minecraft:basalt","minecraft:end_stone","minecraft:calcite",
    "minecraft:wheat","minecraft:carrots","minecraft:potatoes","minecraft:beetroots",
    "minecraft:nether_wart","minecraft:cocoa","minecraft:sweet_berry_bush",
    "minecraft:torchflower","minecraft:pitcher_crop","minecraft:melon","minecraft:pumpkin",
    "minecraft:sugar_cane",
    "minecraft:oak_log","minecraft:spruce_log","minecraft:birch_log","minecraft:jungle_log",
    "minecraft:acacia_log","minecraft:dark_oak_log","minecraft:mangrove_log","minecraft:cherry_log",
    "minecraft:pale_oak_log","minecraft:crimson_stem","minecraft:warped_stem",
    "minecraft:oak_leaves","minecraft:spruce_leaves","minecraft:birch_leaves",
    "minecraft:jungle_leaves","minecraft:acacia_leaves","minecraft:dark_oak_leaves",
    "minecraft:mangrove_leaves","minecraft:cherry_leaves","minecraft:pale_oak_leaves",
    "minecraft:dirt","minecraft:grass_block","minecraft:sand","minecraft:red_sand",
    "minecraft:gravel","minecraft:clay","minecraft:soul_sand","minecraft:soul_soil",
    "minecraft:mud","minecraft:podzol","minecraft:mycelium","minecraft:coarse_dirt",
    "minecraft:rooted_dirt","minecraft:snow_block","minecraft:suspicious_sand",
    "minecraft:suspicious_gravel"
];
pub const MATURE_CROP_STATES:&[&str]=&[
    "minecraft:wheat[age=7]","minecraft:carrots[age=7]","minecraft:potatoes[age=7]",
    "minecraft:beetroots[age=3]","minecraft:nether_wart[age=3]",
    "minecraft:cocoa[age=2,facing=north]","minecraft:cocoa[age=2,facing=south]",
    "minecraft:cocoa[age=2,facing=east]","minecraft:cocoa[age=2,facing=west]",
    "minecraft:sweet_berry_bush[age=3]","minecraft:torchflower",
    "minecraft:pitcher_crop[age=4,half=lower]","minecraft:pitcher_crop[age=4,half=upper]"
];

pub fn block_activity(block: &str, mature: bool) -> Option<Activity> {
    let b = block.trim_start_matches("minecraft:");
    let ore = |o: &'static str, xp: u64| {
        Some(Activity {
            prof: "mining",
            special: Some(o),
            xp,
            luck: "mining",
        })
    };
    if b.ends_with("_ore") || b == "ancient_debris" || b == "amethyst_cluster" {
        return match b
            .trim_start_matches("deepslate_")
            .trim_start_matches("nether_")
        {
            "coal_ore" => ore("coal", 3),
            "copper_ore" => ore("copper", 4),
            "iron_ore" => ore("iron", 6),
            "gold_ore" => ore("gold", 10),
            "redstone_ore" => ore("redstone", 8),
            "lapis_ore" => ore("lapis", 9),
            "diamond_ore" => ore("diamond", 30),
            "emerald_ore" => ore("emerald", 40),
            "quartz_ore" => ore("quartz", 6),
            "ancient_debris" => ore("netherite", 80),
            "amethyst_cluster" => ore("amethyst", 8),
            _ => ore("stone", 2),
        };
    }
    if matches!(
        b,
        "stone"
            | "deepslate"
            | "cobblestone"
            | "andesite"
            | "diorite"
            | "granite"
            | "tuff"
            | "netherrack"
            | "blackstone"
            | "basalt"
            | "end_stone"
            | "calcite"
    ) {
        return ore("stone", 1);
    }
    let crop = |c: &'static str, xp: u64| {
        if mature {
            Some(Activity {
                prof: "farming",
                special: Some(c),
                xp,
                luck: "farming",
            })
        } else {
            None
        }
    };
    match b {
        "wheat" => return crop("wheat", 4),
        "carrots" => return crop("carrots", 4),
        "potatoes" => return crop("potatoes", 4),
        "beetroots" => return crop("beetroots", 4),
        "nether_wart" => return crop("nether_wart", 6),
        "cocoa" => return crop("cocoa", 5),
        "sweet_berry_bush" => return crop("sweet_berry_bush", 3),
        "torchflower" => return crop("torchflower", 6),
        "pitcher_crop" => return crop("pitcher", 6),
        "melon" => {
            return Some(Activity {
                prof: "farming",
                special: Some("melon"),
                xp: 5,
                luck: "farming",
            });
        }
        "pumpkin" => {
            return Some(Activity {
                prof: "farming",
                special: Some("pumpkin"),
                xp: 5,
                luck: "farming",
            });
        }
        "sugar_cane" => {
            return Some(Activity {
                prof: "farming",
                special: Some("sugar_cane"),
                xp: 1,
                luck: "farming",
            });
        }
        _ => {}
    }
    if b.ends_with("_log")
        || b.ends_with("_stem") && !b.contains("melon") && !b.contains("pumpkin")
        || b.ends_with("_wood")
        || b.ends_with("_hyphae")
    {
        return Some(Activity {
            prof: "woodcutting",
            special: None,
            xp: 3,
            luck: "general",
        });
    }
    if b.ends_with("_leaves") {
        return Some(Activity {
            prof: "woodcutting",
            special: None,
            xp: 0,
            luck: "general",
        });
    }
    if matches!(
        b,
        "dirt"
            | "grass_block"
            | "sand"
            | "red_sand"
            | "gravel"
            | "clay"
            | "soul_sand"
            | "soul_soil"
            | "mud"
            | "podzol"
            | "mycelium"
            | "coarse_dirt"
            | "rooted_dirt"
            | "snow_block"
            | "suspicious_sand"
            | "suspicious_gravel"
    ) {
        return Some(Activity {
            prof: "excavation",
            special: None,
            xp: 1,
            luck: "exploration",
        });
    }
    None
}

/// The per-specialisation stat prefix for an activity.
pub fn special_prefix(a: &Activity) -> Option<String> {
    let s = a.special?;
    Some(match a.prof {
        "mining" => format!("mining.ore.{s}"),
        "farming" => format!("farming.crop.{s}"),
        _ => return None,
    })
}

/// Duplication chance for one drop of an activity: profession + specialisation +
/// that activity's luck + half of general luck, in a single sum.
pub fn duplicate_chance(stats: &Stats, a: &Activity) -> f64 {
    let mut p =
        stats.get(&format!("{}.double", a.prof)) + stats.get(&format!("{}.extra_harvest", a.prof));
    if let Some(prefix) = special_prefix(a) {
        p += stats.get(&format!("{prefix}.double")) + stats.get(&format!("{prefix}.extra"));
    }
    if a.luck != "general" {
        p += stats.get(&format!("luck.{}.duplicate", a.luck));
    }
    p += 0.5 * stats.get("luck.general.duplicate");
    finite(p)
}

/// How many extra copies of a stack of `count` drop. Above 100% chance gives
/// guaranteed copies (overflow). Triple only rolls when a double happened.
pub fn extra_copies(stats: &Stats, a: &Activity, count: u32, rng: &mut Rng) -> u64 {
    let copies = rng.overflow_count(duplicate_chance(stats, a));
    let mut extra = copies.saturating_mul(count as u64);
    if copies > 0 && rng.chance(stats.get(&format!("{}.triple", a.prof))) {
        extra = extra.saturating_add(count as u64);
    }
    // Count-based nodes are independent from the chance nodes above.
    // They use stochastic overflow so +0.02 remains meaningful instead of
    // being rounded away until a player reaches a whole extra item.
    let flat = match a.prof {
        "mining" => stats.get("mining.extra_drops"),
        "farming" => stats.get("farming.productivity"),
        _ => 0.0,
    };
    extra = extra.saturating_add(rng.overflow_count(flat).saturating_mul(count as u64));
    extra.min(4096)
}

/// XP multiplier for an activity: its own xp stat and its specialisation's, plus general.
pub fn xp_mult(stats: &Stats, a: &Activity) -> f64 {
    let mut m = 1.0 + stats.get(&format!("{}.xp", a.prof)) + stats.get("general.xp_gain");
    if let Some(prefix) = special_prefix(a) {
        m += stats.get(&format!("{prefix}.xp"));
    }
    finite(m.max(0.0))
}

/// Mob drops: extra copies from looting stats and mob luck (bosses use boss luck).
pub fn mob_extra(
    stats: &Stats,
    boss: bool,
    elite: bool,
    loot_mult: f64,
    count: u32,
    rng: &mut Rng,
) -> u64 {
    let luck = if boss { "bosses" } else { "mobs" };
    let mut p = stats.get("looting.mob_drops")
        + stats.get("hunting.extra_drops")
        + stats.get(&format!("luck.{luck}.duplicate"))
        + 0.25 * stats.get(&format!("luck.{luck}.bonus"))
        + 0.5 * stats.get("luck.general.duplicate");
    if elite {
        p += stats.get("looting.elite_drops");
    }
    p += (loot_mult - 1.0).max(0.0) * 0.25;
    rng.overflow_count(p).saturating_mul(count as u64)
}

/// Rare bonus items rolled on a break, by activity.
pub fn rare_drop(a: &Activity, stats: &Stats, rng: &mut Rng) -> Option<(&'static str, u32)> {
    let mut p = stats.get(&format!("{}.rare", a.prof))
        + if a.prof=="excavation"{stats.get("excavation.treasure")}else{0.0}
        + if a.prof=="farming"{0.25*stats.get("farming.quality")}else{0.0}
        + stats.get(&format!("luck.{}.rarity", a.luck))
        + 0.5 * stats.get("luck.general.rarity")
        + 0.20 * stats.get(&format!("luck.{}.bonus", a.luck))
        + 0.10 * stats.get(&format!("luck.{}.quality", a.luck));
    if let Some(prefix) = special_prefix(a) {
        p += stats.get(&format!("{prefix}.rare"));
    }
    if a.prof == "mining" {
        p += stats.get("mining.gem");
    }
    if !rng.chance(p.min(0.5)) {
        return None;
    }
    let extraordinary = rng.chance(
        stats.get(&format!("luck.{}.extraordinary", a.luck))
            + stats.get(&format!("luck.{}.special", a.luck))
            + 0.5 * stats.get("luck.general.extraordinary")
            + 0.25 * stats.get("luck.general.special"),
    );
    Some(match (a.prof, extraordinary) {
        ("mining", false) => ("minecraft:lapis_lazuli", 2),
        ("mining", true) => ("minecraft:diamond", 1),
        ("farming", false) => ("minecraft:golden_carrot", 1),
        ("farming", true) => ("minecraft:enchanted_golden_apple", 1),
        ("excavation", false) => ("minecraft:flint", 2),
        ("excavation", true) => ("minecraft:emerald", 1),
        (_, false) => ("minecraft:apple", 1),
        (_, true) => ("minecraft:golden_apple", 1),
    })
}

/// Resource-specific profession bonuses. These are rolled only after a real
/// eligible block break, so they cannot be farmed from UI actions.
pub fn resource_bonus(
    a:&Activity,state:&str,stats:&Stats,rng:&mut Rng
)->Vec<(&'static str,u32)> {
    let mut out=Vec::new();
    let id=state.split('[').next().unwrap_or(state).trim_start_matches("minecraft:");
    let mut roll_item=|chance:f64,item:&'static str,count:u32| {
        let n=rng.overflow_count(chance.max(0.0)).min(64) as u32;
        if n>0 {out.push((item,n.saturating_mul(count).max(1)));}
    };
    match a.prof {
        "farming"=>{
            let seed=match a.special {
                Some("wheat")=>Some("minecraft:wheat_seeds"),
                Some("beetroots")=>Some("minecraft:beetroot_seeds"),
                Some("melon")=>Some("minecraft:melon_seeds"),
                Some("pumpkin")=>Some("minecraft:pumpkin_seeds"),
                Some("torchflower")=>Some("minecraft:torchflower_seeds"),
                Some("pitcher")=>Some("minecraft:pitcher_pod"),
                _=>None,
            };
            if let Some(seed)=seed {roll_item(stats.get("farming.seeds"),seed,1);}
        }
        "woodcutting"=>{
            let sapling=if id.starts_with("oak_") {Some("minecraft:oak_sapling")}
                else if id.starts_with("spruce_") {Some("minecraft:spruce_sapling")}
                else if id.starts_with("birch_") {Some("minecraft:birch_sapling")}
                else if id.starts_with("jungle_") {Some("minecraft:jungle_sapling")}
                else if id.starts_with("acacia_") {Some("minecraft:acacia_sapling")}
                else if id.starts_with("dark_oak_") {Some("minecraft:dark_oak_sapling")}
                else if id.starts_with("mangrove_") {Some("minecraft:mangrove_propagule")}
                else if id.starts_with("cherry_") {Some("minecraft:cherry_sapling")}
                else if id.starts_with("pale_oak_") {Some("minecraft:pale_oak_sapling")}
                else {None};
            if let Some(sapling)=sapling {roll_item(stats.get("woodcutting.saplings"),sapling,1);}
            if matches!(id,"oak_log"|"oak_wood"|"oak_leaves"|"dark_oak_log"|"dark_oak_wood"|"dark_oak_leaves") {
                roll_item(stats.get("woodcutting.apples"),"minecraft:apple",1);
            }
        }
        "excavation"=>{
            if id=="gravel" {roll_item(stats.get("excavation.flint"),"minecraft:flint",1);}
            if id=="clay" {roll_item(stats.get("excavation.clay"),"minecraft:clay_ball",1);}
        }
        _=>{}
    }
    out
}

/// Which ordinary death drops are products of animal husbandry. The caller
/// additionally checks that the killed species is a domestic livestock type.
pub fn husbandry_product(id:&str)->Option<&'static str> {
    Some(match id {
        "minecraft:beef"|"minecraft:cooked_beef"|"minecraft:porkchop"|"minecraft:cooked_porkchop"|
        "minecraft:mutton"|"minecraft:cooked_mutton"|"minecraft:chicken"|"minecraft:cooked_chicken"|
        "minecraft:rabbit"|"minecraft:cooked_rabbit"=>"meat",
        "minecraft:leather"|"minecraft:rabbit_hide"|
        "minecraft:white_wool"|"minecraft:orange_wool"|"minecraft:magenta_wool"|
        "minecraft:light_blue_wool"|"minecraft:yellow_wool"|"minecraft:lime_wool"|
        "minecraft:pink_wool"|"minecraft:gray_wool"|"minecraft:light_gray_wool"|
        "minecraft:cyan_wool"|"minecraft:purple_wool"|"minecraft:blue_wool"|
        "minecraft:brown_wool"|"minecraft:green_wool"|"minecraft:red_wool"|
        "minecraft:black_wool"=>"leather",
        _=>return None,
    })
}
pub fn livestock(kind:&str)->bool {
    matches!(kind,
        "minecraft:cow"|"minecraft:mooshroom"|"minecraft:pig"|"minecraft:sheep"|
        "minecraft:chicken"|"minecraft:rabbit"|"minecraft:goat")
}
pub fn husbandry_extra(stats:&Stats,product:&str,count:u32,rng:&mut Rng)->u64 {
    let chance=match product {
        "meat"=>stats.get("husbandry.extra_meat"),
        "leather"=>stats.get("husbandry.extra_leather"),
        _=>0.0,
    };
    rng.overflow_count(chance.max(0.0)).saturating_mul(u64::from(count)).min(4096)
}

/// Fishing extras by water type: treasure and extra catch.
pub fn fishing_extra(stats: &Stats, water: &str, rng: &mut Rng) -> Vec<(&'static str, u32)> {
    let mut out = Vec::new();
    let extra = stats.get("fishing.extra_catch")
        + stats.get(&format!("fishing.water.{water}.extra"))
        + stats.get("luck.fishing.duplicate")
        + 0.25 * stats.get("luck.fishing.bonus")
        + 0.5 * stats.get("luck.general.duplicate");
    let n = rng.overflow_count(extra).min(64) as u32;
    if n > 0 {
        let fish = if rng.chance(stats.get("fishing.rare_fish")) {
            "minecraft:pufferfish"
        } else {
            "minecraft:cod"
        };
        out.push((fish, n));
    }
    let treasure = stats.get("fishing.treasure")
        + stats.get(&format!("fishing.water.{water}.treasure"))
        + stats.get("luck.fishing.rarity")
        + 0.20 * stats.get("fishing.quality")
        + 0.20 * stats.get("luck.fishing.quality")
        + 0.5 * stats.get("luck.general.rarity");
    if rng.chance(treasure.min(0.6)) {
        let t =
            if rng.chance(stats.get("fishing.special") + stats.get("luck.fishing.extraordinary")) {
                "minecraft:heart_of_the_sea"
            } else {
                "minecraft:name_tag"
            };
        out.push((t, 1));
    }
    out
}

/// The water type for fishing specialisation, from the biome name.
pub fn water_of(biome: &str) -> &'static str {
    let b = biome.trim_start_matches("minecraft:");
    if b.contains("frozen") || b.contains("snowy") {
        "frozen"
    } else if b.contains("ocean") || b.contains("beach") {
        "ocean"
    } else if b.contains("river") {
        "river"
    } else if b.contains("swamp") {
        "swamp"
    } else if b.contains("jungle") {
        "jungle"
    } else {
        "other"
    }
}

#[cfg(test)] mod catalog_integration_tests {
    use super::*;
    #[test] fn count_quality_bonus_and_hunting_nodes_change_independent_rolls() {
        let a=Activity{prof:"farming",special:Some("wheat"),xp:4,luck:"farming"};
        let mut base=Stats::default();
        let mut boosted=Stats::default();
        boosted.add("farming.productivity",1.0);
        assert_eq!(extra_copies(&base,&a,1,&mut Rng(7)),0);
        assert!(extra_copies(&boosted,&a,1,&mut Rng(7))>=1);
        base.add("looting.mob_drops",0.0);
        boosted=Stats::default();boosted.add("hunting.extra_drops",1.0);
        assert_eq!(mob_extra(&base,false,false,1.0,1,&mut Rng(9)),0);
        assert!(mob_extra(&boosted,false,false,1.0,1,&mut Rng(9))>=1);
        let mut q=Stats::default();q.add("fishing.quality",10.0);
        assert!(!fishing_extra(&q,"other",&mut Rng(1)).is_empty());
    }
}
