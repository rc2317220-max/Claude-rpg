fn tune_crossbow_projectile(game:&mut Game<'_>, arrow:&mut Projectile) {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    let speed_bonus=mod_claude_rpg::native_weapon_skill(uuid,"crossbow","projectile_speed").clamp(0.0,4.0);
    let accuracy=mod_claude_rpg::native_weapon_skill(uuid,"crossbow","accuracy").clamp(0.0,0.95);
    let speed=(arrow.velocity.x*arrow.velocity.x+arrow.velocity.y*arrow.velocity.y
        +arrow.velocity.z*arrow.velocity.z).sqrt();
    if !speed.is_finite() || speed<=1.0e-9{return;}
    let mut x=arrow.velocity.x/speed;
    let mut y=arrow.velocity.y/speed;
    let mut z=arrow.velocity.z/speed;
    if accuracy>0.0 {
        let ideal=view_vector(game.yaw,game.pitch);
        x=x*(1.0-accuracy)+ideal[0]*accuracy;
        y=y*(1.0-accuracy)+ideal[1]*accuracy;
        z=z*(1.0-accuracy)+ideal[2]*accuracy;
        let n=(x*x+y*y+z*z).sqrt();
        if n.is_finite() && n>1.0e-9{x/=n;y/=n;z/=n;}
    }
    let tuned=speed*(1.0+speed_bonus);
    arrow.velocity=Vec3::new(x*tuned,y*tuned,z*tuned);
}
