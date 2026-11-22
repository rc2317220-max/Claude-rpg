fn crossbow_ammo_recovery(mut value:Ammo, game:&mut Game<'_>)->Ammo {
    if value.slot.is_none(){return value;}
    let uuid=game.__horizon_mixins_rustpg_acting_id();
    if mod_rustpg::native_weapon_roll_chance(uuid,"crossbow","ammo_recovery"){
        value.slot=None;
    }
    value
}
