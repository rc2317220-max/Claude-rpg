//! Regras determinísticas de profissões ligadas a blocos realmente destruídos.
//! Não aplica XP geral por blocos triviais nem recompensa árvores recolocadas.
//! Tudo que concede XP é uma lista explícita; não há contains("ore").

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherReward {
    pub profession: &'static str,
    pub xp: u64,
    pub provenance_required: bool,
}

fn age(block_state: &str) -> Option<u8> {
    let start=block_state.find('[')? + 1;
    let props=block_state[start..].split(']').next()?;
    for prop in props.split(',') {
        if let Some(value)=prop.trim().strip_prefix("age=") {
            return value.parse().ok();
        }
    }
    None
}

pub fn resource(block_state: &str) -> Option<GatherReward> {
    let name=block_state.split('[').next().unwrap_or(block_state);
    // All listed logs are recognized from successful breaks, never from a click.
    let log=matches!(name,
        "minecraft:oak_log"|"minecraft:spruce_log"|"minecraft:birch_log"|
        "minecraft:jungle_log"|"minecraft:acacia_log"|"minecraft:dark_oak_log"|
        "minecraft:mangrove_log"|"minecraft:cherry_log"|"minecraft:pale_oak_log"|
        "minecraft:crimson_stem"|"minecraft:warped_stem"|
        "minecraft:stripped_oak_log"|"minecraft:stripped_spruce_log"|
        "minecraft:stripped_birch_log"|"minecraft:stripped_jungle_log"|
        "minecraft:stripped_acacia_log"|"minecraft:stripped_dark_oak_log"|
        "minecraft:stripped_mangrove_log"|"minecraft:stripped_cherry_log"|
        "minecraft:stripped_pale_oak_log"|"minecraft:stripped_crimson_stem"|
        "minecraft:stripped_warped_stem");
    if log { return Some(GatherReward {
        profession:"woodcutting", xp:4, provenance_required:true
    }); }
    if matches!(name,"minecraft:clay"|"minecraft:gravel"|
        "minecraft:sand"|"minecraft:red_sand") {
        return Some(GatherReward {profession:"excavation",
            xp:if name=="minecraft:clay" {3}else{1}, provenance_required:true});
    }
    let full_growth=match name {
        "minecraft:wheat"|"minecraft:carrots"|"minecraft:potatoes" =>7,
        "minecraft:beetroots"|"minecraft:nether_wart"|
        "minecraft:sweet_berry_bush" =>3,
        "minecraft:cocoa"=>2,
        _=>return None,
    };
    if age(block_state)==Some(full_growth) {
        Some(GatherReward{profession:"farming",xp:3,provenance_required:false})
    }else { None }
}

/// Server gate is shared between modules and cannot be bypassed by the UI.
pub fn eligible(reward:GatherReward, player_placed:bool, creative:bool)->bool {
    !creative && !(reward.provenance_required && player_placed)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn never_rewards_trivial_blocks_or_immature_crops(){
        for state in ["minecraft:dirt","minecraft:grass_block","minecraft:stone",
            "minecraft:poppy","minecraft:dandelion","minecraft:oak_planks",
            "minecraft:oak_leaves","minecraft:wheat[age=3]",
            "minecraft:beetroots[age=0]","other:oak_log"] {
            assert!(resource(state).is_none(),"{state}");
        }
    }
    #[test] fn growing_crop_only_rewards_when_mature(){
        assert_eq!(resource("minecraft:wheat[age=7]").unwrap().profession,"farming");
        assert_eq!(resource("minecraft:cocoa[facing=north,age=2]").unwrap().xp,3);
        assert!(resource("minecraft:nether_wart[age=2]").is_none());
    }
    #[test] fn player_placed_resources_never_farm_xp(){
        let log=resource("minecraft:oak_log").unwrap();
        assert!(!eligible(log,true,false));
        assert!(eligible(log,false,false));
        assert!(!eligible(log,false,true));
        assert!(eligible(resource("minecraft:carrots[age=7]").unwrap(),true,false));
    }
}
