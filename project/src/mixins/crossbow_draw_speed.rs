fn crossbow_draw_speed(value:f32, game:&mut Game<'_>)->f32 {
    let uuid=game.__horizon_mixins_rustpg_acting_id();
    let bonus=mod_rustpg::native_weapon_skill(uuid,"crossbow","draw_speed").clamp(0.0,4.0) as f32;
    (value/(1.0+bonus)).max(1.0)
}
