fn tune_bow_projectile(game:&mut Game<'_>, arrow:&mut Projectile) {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    let speed_bonus=mod_claude_rpg::native_weapon_skill(uuid,"bow","projectile_speed").clamp(0.0,4.0);
    let accuracy=mod_claude_rpg::native_weapon_skill(uuid,"bow","accuracy").clamp(0.0,0.95);
    let speed=(arrow.velocity.x*arrow.velocity.x+arrow.velocity.y*arrow.velocity.y
        +arrow.velocity.z*arrow.velocity.z).sqrt();
    if speed.is_finite() && speed>1.0e-9 {
        if accuracy>0.0 {
            let ideal=view_vector(game.yaw,game.pitch);
            let mut x=arrow.velocity.x/speed*(1.0-accuracy)+ideal[0]*accuracy;
            let mut y=arrow.velocity.y/speed*(1.0-accuracy)+ideal[1]*accuracy;
            let mut z=arrow.velocity.z/speed*(1.0-accuracy)+ideal[2]*accuracy;
            let n=(x*x+y*y+z*z).sqrt();
            if n.is_finite() && n>1.0e-9 {x/=n;y/=n;z/=n;}
            let tuned=speed*(1.0+speed_bonus);
            arrow.velocity=Vec3::new(x*tuned,y*tuned,z*tuned);
        } else if speed_bonus>0.0 {
            arrow.velocity=Vec3::new(
                arrow.velocity.x*(1.0+speed_bonus),
                arrow.velocity.y*(1.0+speed_bonus),
                arrow.velocity.z*(1.0+speed_bonus));
        }
    }
    if let Some(state)=arrow.arrow.as_mut() {
        let extra=mod_claude_rpg::native_weapon_roll_count(uuid,"bow","pierce",127) as u8;
        state.pierce=state.pierce.saturating_add(extra).min(127);
    }
    let extra=mod_claude_rpg::native_weapon_roll_count(uuid,"bow","extra_arrows",8);
    if extra==0{return;}
    let base=arrow.clone();
    for i in 0..extra {
        let mut shot=base.clone();
        let step=f64::from(i/2+1)*3.0_f64.to_radians();
        let angle=if i&1==0{-step}else{step};
        let (sin,cos)=angle.sin_cos();
        let vx=shot.velocity.x;let vz=shot.velocity.z;
        shot.velocity=Vec3::new(vx*cos-vz*sin,shot.velocity.y,vx*sin+vz*cos);
        let Some(state)=shot.arrow.as_mut() else{continue;};
        state.pickup=horizon_client::arrows::Pickup::CreativeOnly;
        let ammo=state.pickup_item.clone();
        let id=game.spawn_projectile(shot,&ammo);
        mod_claude_rpg::native_note_projectile(uuid,id,"bow");
    }
}
