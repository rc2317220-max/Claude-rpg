fn note_fishing_cast(game:&mut Game<'_>, stack:&ItemStack) {
    if !game.is_rod(stack) || game.hook.is_some() {return;}
    let uuid=game.__horizon_mixins_claude_rpg_acting_id();
    mod_claude_rpg::native_note_fishing_hook(uuid,game.next_entity.saturating_add(1));
}
