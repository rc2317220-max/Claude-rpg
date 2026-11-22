fn crossbow_pierce(value:u8, game:&mut Game<'_>)->u8 {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    value.saturating_add(
        mod_claude_rpg::native_weapon_roll_count(uuid,"crossbow","pierce",127) as u8
    ).min(127)
}
