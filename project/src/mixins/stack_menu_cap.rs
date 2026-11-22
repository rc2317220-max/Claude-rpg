fn stack_menu_cap(context:&SlotContext<'_>, max_stack:&mut u32) {
    if let Some(uuid)=context.player {
        *max_stack=mod_rustpg::native_personal_stack_limit(uuid,*max_stack);
    }
}
