fn bow_draw_speed(value:i32, game:&mut Game<'_>)->i32 {
    if value<=0{return value;}
    let uuid=game.__horizon_mixins_rustpg_acting_id();
    let bonus=mod_rustpg::native_weapon_skill(uuid,"bow","draw_speed").clamp(0.0,4.0);
    ((f64::from(value))*(1.0+bonus)).round().clamp(1.0,i32::MAX as f64) as i32
}
