fn boost_crafting_output(game:&mut Game<'_>, player:u128, result:&mut Option<ItemStack>) {
    let Some(stack)=result.as_mut() else{return;};
    let max=game.res.items().get(&stack.item).map_or(64,|item|item.max_stack);
    let next=mod_rustpg::native_crafting_output_count(
        player,&stack.item,stack.count,max
    );
    stack.count=next.min(max).max(1);
}
