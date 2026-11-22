fn note_trident(game:&mut Game<'_>, id:u32) {
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    mod_claude_rpg::native_note_projectile(uuid,id,"trident");
}
