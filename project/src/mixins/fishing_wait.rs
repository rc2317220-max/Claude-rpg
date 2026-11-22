fn fishing_wait(value:i32, hook:&Hook)->i32 {
    mod_rustpg::native_fishing_wait(hook.id,value)
}
