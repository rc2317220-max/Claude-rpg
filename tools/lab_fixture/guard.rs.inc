
pub(crate) fn lab_state(dim:&str, entity:u32)->String {
    let now=current_tick();
    inner().lock().unwrap_or_else(|e|e.into_inner()).entries.get(&(dim.to_owned(),entity))
        .map(|e|format!("{}:{}:{}:{}:{}",e.stance.name(),e.started,e.until,e.cooldown,e.unused))
        .unwrap_or_else(||format!("none:{now}:0:0:false"))
}
