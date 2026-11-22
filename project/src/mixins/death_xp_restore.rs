fn restore_death_xp(game:&mut Game<'_>, player:u128) {
    if let Some((levels,points))=mod_claude_rpg::native_take_death_xp_restore(player) {
        if let Some(target)=game.player(player) {
            if game.player_set_experience_levels(target,levels) {
                let _=game.player_set_experience_points(target,points);
            }
        }
    }
}
