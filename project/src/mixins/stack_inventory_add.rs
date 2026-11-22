fn cap_inventory_add(value:u32, game:&mut Game<'_>)->u32 {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    mod_claude_rpg::native_personal_stack_limit(uuid,value)
}
