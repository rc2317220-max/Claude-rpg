fn note_trident(game:&mut Game<'_>, id:u32) {
    let uuid=game.__horizon_mixins_rustpg_acting_id();
    mod_rustpg::native_note_projectile(uuid,id,"trident");
}
