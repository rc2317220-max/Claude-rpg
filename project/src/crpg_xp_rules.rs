//! Server-side XP decisions. Only mining resource blocks counts as mining.
//! Pure rules permit future modules/datapacks to add tiers without granting
//! progress for dirt, flowers, wood or placed decorative blocks.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Reward {pub general:u64,pub profession:u64}

pub fn mined(block_state:&str)->Option<Reward> {
    let name=block_state.split('[').next().unwrap_or(block_state);
    // No generic `contains("ore")`: decorative mod blocks must not award XP.
    let (general,profession)=match name {
        "minecraft:coal_ore"|"minecraft:deepslate_coal_ore" =>(2,6),
        "minecraft:copper_ore"|"minecraft:deepslate_copper_ore" =>(3,8),
        "minecraft:iron_ore"|"minecraft:deepslate_iron_ore" =>(4,10),
        "minecraft:gold_ore"|"minecraft:deepslate_gold_ore"|"minecraft:nether_gold_ore" =>(5,12),
        "minecraft:redstone_ore"|"minecraft:deepslate_redstone_ore"=> (4,10),
        "minecraft:lapis_ore"|"minecraft:deepslate_lapis_ore"=> (5,12),
        "minecraft:diamond_ore"|"minecraft:deepslate_diamond_ore"=> (12,22),
        "minecraft:emerald_ore"|"minecraft:deepslate_emerald_ore"=> (12,22),
        "minecraft:nether_quartz_ore"=> (3,7),
        "minecraft:ancient_debris"=> (25,35),
        "minecraft:amethyst_cluster"=> (3,8),
        // Rare raw resource clusters; not stone, cobblestone or regular deepslate.
        "minecraft:raw_iron_block"|"minecraft:raw_copper_block"|"minecraft:raw_gold_block" =>(0,0),
        _ =>return None,
    };
    if general==0 {None} else {Some(Reward {general,profession})}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn no_xp_for_trash_or_exploitable_storage() {
        for block in ["minecraft:dirt","minecraft:grass_block","minecraft:poppy",
            "minecraft:oak_log","minecraft:stone","minecraft:raw_iron_block",
            "minecraft:diamond_block","minecraft:air", "mod:diamond_ore"] {
            assert!(mined(block).is_none(),"{block}");
        }
    }
    #[test] fn real_ores_pay_by_rarity() {
        assert_eq!(mined("minecraft:iron_ore").unwrap().general,4);
        assert!(mined("minecraft:diamond_ore").unwrap().general>mined("minecraft:coal_ore").unwrap().general);
        assert!(mined("minecraft:deepslate_diamond_ore[lit=false]").is_some());
        assert_eq!(mined("minecraft:amethyst_cluster").unwrap().profession,8);
    }
}
