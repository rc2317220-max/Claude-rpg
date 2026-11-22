//! Weapon discipline identification. No XP is awarded here: the authoritative
//! living_drops event must confirm the kill before progress can be credited.
//! Unknown items are NOT called swords. They fall back to the existing
//! damage-source classifier when no reliable weapon can be identified.
use mod_api::{Identifier,ItemKey,ServerHost};

pub const RANGED: &[(&str,&str)] = &[
    ("minecraft:bow","bow"),("minecraft:crossbow","crossbow"),
    ("minecraft:trident","trident")
];
/// Weapon identity is recorded by Starling at firing time and read from
/// LootDrops::weapon_item, NOT guessed from the player's current hand.
pub fn ranged_weapon(host:&dyn ServerHost,item:ItemKey)->Option<&'static str> {
    RANGED.iter().find_map(|(id,profession)| {
        let key=Identifier::parse(id).ok()?;
        (host.item_key(&key)==Some(item)).then_some(*profession)
    })
}
pub const KNOWN: &[(&str,&str)]=&[
    ("minecraft:wooden_sword","sword"),("minecraft:stone_sword","sword"),
    ("minecraft:iron_sword","sword"),("minecraft:golden_sword","sword"),
    ("minecraft:diamond_sword","sword"),("minecraft:netherite_sword","sword"),
    ("minecraft:wooden_axe","axe"),("minecraft:stone_axe","axe"),
    ("minecraft:iron_axe","axe"),("minecraft:golden_axe","axe"),
    ("minecraft:diamond_axe","axe"),("minecraft:netherite_axe","axe"),
    ("minecraft:mace","mace"),
    ("minecraft:wooden_spear","spear"),("minecraft:stone_spear","spear"),
    ("minecraft:iron_spear","spear"),("minecraft:golden_spear","spear"),
    ("minecraft:diamond_spear","spear"),("minecraft:netherite_spear","spear"),
    ("minecraft:trident","trident"),
];

pub fn class_of(id:&str)->Option<&'static str> {
    KNOWN.iter().find(|(name,_)|*name==id).map(|(_,skill)|*skill)
}
/// Only None with a proven empty selected slot means bare hands. When the
/// selected inventory was unavailable, the bridge never calls this function.
pub fn from_held(host:&dyn ServerHost,item:Option<ItemKey>)->Option<&'static str> {
    let Some(item)=item else {return Some("unarmed");};
    for (id,profession) in KNOWN {
        if let Ok(key)=Identifier::parse(id) {
            if host.item_key(&key)==Some(item) {return Some(profession);}
        }
    }
    None
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn conventional_weapons_have_separate_professions(){
        assert_eq!(class_of("minecraft:iron_axe"),Some("axe"));
        assert_eq!(class_of("minecraft:mace"),Some("mace"));
        assert_eq!(class_of("minecraft:netherite_sword"),Some("sword"));
        assert_eq!(class_of("minecraft:trident"),Some("trident"));
        assert!(RANGED.iter().any(|(id,p)|*id=="minecraft:crossbow" && *p=="crossbow"));
        assert_eq!(class_of("modded:unknown_weapon"),None);
        assert_eq!(class_of("minecraft:oak_log"),None);
    }
}
