fn crossbow_ammo_recovery(mut value:Ammo, game:&mut Game<'_>)->Ammo {
    if value.slot.is_none(){return value;}
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    if mod_claude_rpg::native_weapon_roll_chance(uuid,"crossbow","ammo_recovery"){
        value.slot=None;
    }
    value
}
