fn apply_trade_discount(game:&mut Game<'_>, session:&mut merchant::Session) {
    let uuid=game.__horizon_mixins_rustpg_acting_id();
    let discount=mod_rustpg::native_trade_discount(uuid);
    if discount<=0.0 {return;}
    for offer in &mut session.offers {
        let base=offer.base_cost_a.stack.count.max(1);
        let cut=((base as f64)*discount).floor().clamp(0.0,i32::MAX as f64) as i32;
        if cut>0 {
            offer.special_price_diff=offer.special_price_diff.saturating_sub(cut);
        }
    }
}
