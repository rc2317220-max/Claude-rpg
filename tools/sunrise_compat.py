"""Sunrise migration adapters, based on pinned source and compiler diagnostics."""
from pathlib import Path
import re

def once(s,old,new):
    if s.count(old)!=1:raise ValueError(f"expected one occurrence, got {s.count(old)}: {old[:100]}")
    return s.replace(old,new,1)

def apply(stage):
    lib=stage/"src/lib.rs";s=lib.read_text()
    s=once(s,"RegisterCommandsEvent, RunModCommandEvent,", "RegisterCommandsEvent, CommandInvocation,")
    s=once(s,"runtime:Mutex<Runtime>", "runtime:Arc<Mutex<Runtime>>")
    # New commands are retained executors. Capture owned runtime, not a borrowed Hooks receiver.
    s=once(s,"e.commands.push(Command::new(usage));",'''let runtime=Arc::clone(&self.runtime);
            let command=if usage.starts_with("rustpg lab "){Command::op(usage,2)}else{Command::new(usage)};
            e.commands.push(command.executes(move |host,source,command| rustpg_command(&runtime,host,source,command)));''')
    start=s.index("    fn run_mod_command(");end=s.index("    fn player_logged_in(",start)
    command=s[start:end]
    command=once(command,"    fn run_mod_command(&self,host:&mut dyn ServerHost,e:&mut RunModCommandEvent<'_>)->Flow {","fn rustpg_command(runtime:&Mutex<Runtime>,host:&mut dyn ServerHost,source:&CommandSource,command:&CommandInvocation)->Option<i32> {")
    command=command.replace("e.command.","command.").replace("&e.source","source").replace("self.runtime","runtime").replace("return Flow::Continue;","return None;").replace("Flow::Cancel","Some(1)")
    s=s[:start]+s[end:];s+="\n"+command
    s=once(s,"fn movement_input(&self,host:&dyn ClientHost,\n        e:&mut mod_api::MovementInputEvent)","fn movement_input_update(&self,host:&dyn ClientHost,\n        e:&mut mod_api::MovementInputUpdateEvent<'_>)")
    s=once(s,"if exhausted {e.sprint=false;}","if exhausted {e.input.key_presses.sprint=false;}")
    s=once(s,"fn add_server_reload_listeners(&self,_host:&dyn mod_api::Host,","fn add_server_reload_listeners(&self,_host:&dyn mod_api::ReloadContext,")
    s=once(s,"fn client_resources_reloaded(&self, _: &mut dyn ClientHost,","fn client_resources_reloaded(&self, _: &dyn mod_api::ClientReloadHost,")
    for hook in ("living_damage_pre","living_incoming_damage","living_damage_post"):
        s=once(s,f"fn {hook}(&self,_host:&mut dyn mod_api::LevelRandomHost,",f"fn {hook}(&self,_host:&mut dyn mod_api::HitHost,")
    start=s.index("    fn living_breathe(");end=s.index("    fn living_drown(",start)
    body=s[start:end]
    body=once(body,"context:&dyn mod_api::LoopContext,\n        e:&mut mod_api::LivingBreatheEvent<'_>","host:&mut dyn ServerHost,\n        e:&mut mod_api::LivingBreatheEvent")
    body=once(body,'if e.kind!="minecraft:player" || !matches!(context.side(),mod_api::LoopSide::Server){return;}','let entity=e.entity.entity();\n        if !entity.kind(host).is_some_and(|k|k.id(host)=="minecraft:player"){return;}\n        let entity_id=entity.id();')
    body=body.replace("&e.entity","&entity_id").replace("e.entity)))","entity_id)))").replace("u64::from(e.entity)","u64::from(entity_id)")
    s=s[:start]+body+s[end:]
    # Capture V at raw physical/layout input, plus the normal mapping and
    # optional touch button (see GUI patch). No local-player read in key hook;
    # tick owns open/close decision, including world transitions.
    s=once(s,'if e.action!=KeyAction::Press || !host.playing() || host.local_player().is_none(){return;}','if e.action!=KeyAction::Press || !host.playing(){return;}')
    s=once(s,'if v && host.mod_screen()!=Some(ID) {OPEN_REQUEST.store(true,Ordering::Release);}','if v && host.mod_screen()!=Some(ID) {OPEN_REQUEST.store(true,Ordering::Release);}')
    lib.write_text(s)
    for n in ("rustpg_profession_plugins.rs","rustpg_script_vm.rs"):
        f=stage/"src"/n;s=f.read_text().replace("Box<dyn mod_api::ReloadApply>","Box<dyn FnOnce() + Send + 'static>").replace("<dyn mod_api::ReloadApply>::new(","Box::new(");f.write_text(s)
