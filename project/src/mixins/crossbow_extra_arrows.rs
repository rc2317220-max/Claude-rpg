fn crossbow_extra_arrows(value:i32, game:&mut Game<'_>)->i32 {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    let extra=mod_claude_rpg::native_weapon_roll_count(uuid,"crossbow","extra_arrows",8) as i32;
    value.saturating_add(extra).clamp(1,11)
}
