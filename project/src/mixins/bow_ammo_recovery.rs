fn bow_ammo_recovery(value:Option<usize>, game:&mut Game<'_>)->Option<usize> {
    if value.is_none(){return value;}
    let uuid=game.__horizon_mixins_rustpg_acting_id();
    if mod_rustpg::native_weapon_roll_chance(uuid,"bow","ammo_recovery"){None}else{value}
}
