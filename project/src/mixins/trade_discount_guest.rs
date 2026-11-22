fn apply_guest_trade_discount(uuid:[i32;4], merchant:&mut GuestMerchant) {
    let uuid=((uuid[0] as u32 as u128)<<96)
        |((uuid[1] as u32 as u128)<<64)
        |((uuid[2] as u32 as u128)<<32)
        |(uuid[3] as u32 as u128);
    let discount=mod_rustpg::native_trade_discount(uuid);
    if discount<=0.0 {return;}
    for offer in &mut merchant.merchant.session.offers {
        let base=offer.base_cost_a.stack.count.max(1);
        let cut=((base as f64)*discount).floor().clamp(0.0,i32::MAX as f64) as i32;
        if cut>0 {
            offer.special_price_diff=offer.special_price_diff.saturating_sub(cut);
        }
    }
}
