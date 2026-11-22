// Injected at RETURN of native_player_clone. The hook queued the exact
// replacement-player XP state from OriginalPlayerRef; this point still owns
// Game and can use the internal native setters atomically on the real player.
if let Some((levels,points))=crate::native_take_death_xp_restore(player) {
    if let Some(target)=game.player(player) {
        if game.player_set_experience_levels(target,levels) {
            let _=game.player_set_experience_points(target,points);
        }
    }
}
