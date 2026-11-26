"""Sunrise migration adapters, based on pinned source and compiler diagnostics."""
from pathlib import Path
import re

def once(s,old,new):
    if s.count(old)!=1:raise ValueError(f"expected one occurrence, got {s.count(old)}: {old[:100]}")
    return s.replace(old,new,1)

def apply(stage):
    lib=stage/"src/lib.rs";s=lib.read_text()
    # Preview 4 requires owner declarations for attachment use, even when
    # the legacy API compiled and allowed the same static AttachmentType.
    # Match every RustPG attachment in the base source so new keys cannot be
    # accidentally omitted from the registry.
    attachments=re.findall(r'static\s+\w+\s*:\s*AttachmentType<[^;]+?=\s*AttachmentType\s*\{(.*?)\n\};',s,re.S)
    declarations=[]
    for body in attachments:
        name=re.search(r'namespace:\s*ID,\s*name:\s*"([^"]+)"',body)
        sync=re.search(r'\bsync:\s*(true|false)',body)
        if not name or not sync:raise ValueError("attachment declaration shape changed")
        declarations.append((name.group(1),sync.group(1)))
    assert len(declarations)>=3 and len({name for name,_ in declarations})==len(declarations),"missing/duplicate attachment declarations"
    header="    fn register(&self,registrar:&mut dyn Registrar,_:&RegistrationContext) {"
    generated="".join(f'\n        let _=registrar.register_attachment_type("rustpg:{name}",{sync});' for name,sync in declarations)
    s=once(s,header,header+generated)
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


    # Preview 4 replaces owned entity lists by visitor queries. Retain only
    # lightweight EntityRef handles; never hold the engine's borrow in the callback.
    s=lib.read_text()
    s=once(s,"fn draw_layer(draw:",'''fn rustpg_entities_in(level:mod_api::LevelRef,host:&dyn mod_api::Host,min:[f64;3],max:[f64;3])->Option<Vec<mod_api::EntityRef>> {
    let mut entities=Vec::new();
    level.visit_entities_in(host,min,max,&mut |entity| {
        entities.push(entity);
        if entities.len()>=4096 {core::ops::ControlFlow::Break(())}
        else {core::ops::ControlFlow::Continue(())}
    }).ok()?;
    Some(entities)
}
fn rustpg_selected_stack(player:mod_api::PlayerRef,host:&dyn mod_api::Host)->Option<mod_api::ItemStack> {
    let slot=player.selected_slot(host)? as u32;
    player.with_inventory(host,|view|view.get(slot).and_then(|stack|stack.project())).flatten()
}
fn rustpg_consume_click(host:&mut dyn ClientHost,owner:&str,id:&str)->bool {
    let Ok(mapping)=host.mod_key_mapping(owner,id) else{return false;};
    mapping.consume_click(host).unwrap_or(false)
}
fn draw_layer(draw:''')
    s=s.replace("level.entities_in(host,","rustpg_entities_in(level,host,")
    # Registry names are optional, and their owned spelling is explicitly extracted.
    s=s.replace("keys.into_iter().map(|key| key.id(host))","keys.into_iter().filter_map(|key| key.id(host).map(|id|id.to_string()))")
    s=s.replace(".kind(host).map(|k|k.id(host))",".kind(host).and_then(|k|k.id(host).map(|id|id.to_string()))")
    s=s.replace(".kind(host).map(|k|(id,k.id(host)))",".kind(host).and_then(|k|k.id(host).map(|name|(id,name.to_string())))")
    s=s.replace("let id=k.id(host);rustpg_native_entities::CREATURES.iter().any(|v|*v==id)","k.id(host).is_some_and(|id|rustpg_native_entities::CREATURES.contains(&id.as_ref()))")
    s=s.replace("ui.bestiary_instances.entry(kind.id(host)).or_insert(entity.id());","if let Some(id)=kind.id(host){ui.bestiary_instances.entry(id.to_string()).or_insert(entity.id());}")
    s=s.replace("key.id(host)==kind","key.id(host).as_deref()==Some(kind)")
    s=s.replace("k.id(host)==\"minecraft:player\"","k.id(host).as_deref()==Some(\"minecraft:player\")")
    s=s.replace("key.id(host)==rustpg_native_entities::HUNTER","key.id(host).as_deref()==Some(rustpg_native_entities::HUNTER)")
    # Weapon snapshots project only the selected slot, rather than copying an inventory.
    s=once(s,'''pl.selected_slot(host).zip(pl.inventory_slots(host))
            .and_then(|(slot,contents)|contents.iter().find(|(i,_)|*i==slot as u32)
                .and_then(|(_,stack)|rustpg_weapon_training::ranged_weapon(host,stack.item())))''',
        '''rustpg_selected_stack(pl,host)
            .and_then(|stack|rustpg_weapon_training::ranged_weapon(host,stack.item()))''')
    s=once(s,'''pl.selected_slot(host).zip(pl.inventory_slots(host))
            .and_then(|(slot,contents)|contents.into_iter()
                .find(|(i,_)|*i==slot as u32).map(|(_,stack)|stack))''',"rustpg_selected_stack(pl,host)")
    s=once(s,'''pl.selected_slot(host).zip(pl.inventory_slots(host))
                    .and_then(|(slot,items)|items.iter().find(|(i,_)|*i==slot as u32)
                        .map(|(_,stack)|stack.item()))''',"rustpg_selected_stack(pl,host).map(|stack|stack.item())")
    s=s.replace("host.consume_click(ID,","rustpg_consume_click(host,ID,")
    # Only block drops changed to a checked live view; chest and death drops
    # have other types. Restrict the conversion to the block hook.
    a=s.index("    fn block_drops(");b=s.index("    // Real mining event",a)
    block=once(s[a:b],"let originals=e.drops.clone();","let originals:Vec<_>=e.drops.iter().take(64).filter_map(|drop|drop.to_item_drop()).collect();")
    assert block.count("e.drops.first().cloned()")==2,"block drop template anchors changed"
    block=block.replace("e.drops.first().cloned()","originals.first().cloned()")
    block=once(block,"e.drops.extend(additions.into_iter().take(128));","for drop in additions.into_iter().take(128){let _=e.drops.push(drop);}")
    s=s[:a]+block+s[b:]
    # Anvil edits are now separate from immutable inputs. Preserve other listeners'
    # replacement stack, or start from vanilla when they chose Keep.
    s=s.replace("e.update.xp_cost","e.edits.xp_cost").replace("e.update.material_cost","e.edits.material_cost")
    a=s.index("        if (mastery>0 || transcendence) && e.update.output.is_some()");b=s.index("        Flow::Continue",a)
    s=s[:a]+'''        if mastery>0 || transcendence {
            if let Some(right)=e.inputs.right {
                let right_book=mod_api::__private::item_key_id(right.item())==Some("minecraft:enchanted_book");
                if right_book {
                    let chosen=match &e.edits.output {
                        mod_api::StackResult::Keep=>e.inputs.vanilla_output.cloned(),
                        mod_api::StackResult::Set(stack)=>stack.clone(),
                    };
                    if let Some(mut output)=chosen {
                        let output_book=mod_api::__private::item_key_id(output.item())==Some("minecraft:enchanted_book");
                        let _=rustpg_enchant_stack::apply_level_one_book(
                            e.inputs.left,right,&mut output,output_book,mastery,transcendence);
                        e.edits.output=mod_api::StackResult::Set(Some(output));
                    }
                }
            }
        }
'''+s[b:]
    lib.write_text(s)
    p=stage/"src/rustpg_native_entities.rs";s=p.read_text()
    p.write_text(once(s,"cx.random().next_int_bound(90).unwrap_or(1)","cx.random().next_int_bound(90)"))
