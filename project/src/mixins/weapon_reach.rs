fn weapon_reach(mut value:mod_api::ItemAttackRange, game:&mut Game<'_>)->mod_api::ItemAttackRange {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    let weapon=match game.held() {
        None=>"unarmed",
        Some(stack)=>{
            let items=game.res.items();
            let id=stack.item.as_str();
            if items.item_in_tag("minecraft:swords",id) {"sword"}
            else if items.item_in_tag("minecraft:axes",id) {"axe"}
            else if items.item_in_tag("minecraft:spears",id) {"spear"}
            else if id=="minecraft:mace" {"mace"}
            else if id=="minecraft:trident" {"trident"}
            else {return value;}
        }
    };
    let bonus=mod_claude_rpg::native_weapon_skill(uuid,weapon,"reach").clamp(0.0,16.0) as f32;
    value.max=(value.max+bonus).min(64.0);
    value
}
