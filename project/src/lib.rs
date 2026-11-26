//! RustPG 4.7 — nativo Starling 0.9.0: atributos reais e hooks.
//! O código de domínio é compartilhado com o pacote de migração legado.
#![allow(dead_code)]
mod rustpg_actions;
mod rustpg_attributes;
mod rustpg_bestiary;
mod rustpg_combat;
mod rustpg_damage_visuals;
mod rustpg_damage_meter;
mod rustpg_mob_wire;
mod rustpg_bar_vfx;
mod rustpg_enchant;
mod rustpg_engine_attributes;
mod rustpg_engine_combat;
mod rustpg_engine_nutrition;
mod rustpg_engine_presentation;
mod rustpg_engine_stamina;
mod rustpg_engine_vitality;
mod rustpg_evil_att;
mod rustpg_gear;
mod rustpg_hud;
mod rustpg_inspector;
mod rustpg_hud_geometry;
mod rustpg_xp_rules;
mod rustpg_gathering_rules;
mod rustpg_combat_training;
mod rustpg_weapon_training;
mod rustpg_v4_core;
mod rustpg_v4_fatigue;
mod rustpg_relic_engine;
mod rustpg_combat_pipeline;
mod rustpg_relic_active;
mod rustpg_native_modifiers;
mod rustpg_v4_provenance;
mod rustpg_v4_wire;
mod rustpg_luck;
mod rustpg_nodes;
mod rustpg_node_effects;
mod rustpg_num;
mod rustpg_presentation;
mod rustpg_profile;
mod rustpg_progression;
mod rustpg_profession_plugins;
mod rustpg_script_vm;
mod rustpg_scaling;
mod rustpg_skillgraph;
mod rustpg_java_port;
mod rustpg_starling090;
mod rustpg_combo;
mod rustpg_guard;
mod rustpg_elite_encounters;
mod rustpg_enchant_stack;
mod rustpg_immersive;
mod rustpg_skeletal;
mod rustpg_boss_ui;
mod rustpg_timeline;
mod rustpg_target_hud;
mod rustpg_ui_animation;
mod rustpg_ui_core;
mod rustpg_ui_input;
mod rustpg_ui_layout;
mod rustpg_ui_system;
mod rustpg_ui_theme;
mod rustpg_ui_widgets;
mod rustpg_ui_window;
mod rustpg_view;
mod rustpg_generated_models;
mod rustpg_generated_hunter;
mod rustpg_native_entities;
mod rustpg_world_vfx;
mod rustpg_powers;
mod rustpg_hitboxes;
mod rustpg_rifts;
mod rustpg_action_combat;

use std::{sync::{Arc,Mutex,OnceLock,atomic::{AtomicBool,Ordering}}, collections::{BTreeMap,HashMap}, time::{SystemTime, UNIX_EPOCH}};
use mod_api::{self, AttachmentCodec, AttachmentError, AttachmentType, ClientHost,
    Compound, Flow, Holder, Hooks, InputKeyEvent, KeyAction, PlayerLoggedInEvent,
    ScreenMouseButtonPressedPreEvent, ScreenRenderPostEvent, ServerHost, Tag, Text,
    ServerTickEvent, ServerTickPhase, LivingDropsEvent, PlayerDestroyedBlockEvent,
    GuiEntity, GuiEntityLook, GuiEntityView, Registrar, RegistrationContext,
    IPayloadHandler, IPayloadContext, PayloadBytes, PayloadId, PayloadHandleError,
    PayloadDirection, PayloadProtocol, ItemFishedEvent, ItemCraftedEvent,
    PlayerLoggedOutEvent, EntityJoinLevelEvent, Command, CommandSource,
    ScreenMouseDraggedPreEvent, ScreenMouseScrolledPreEvent,
    RegisterCommandsEvent, RunModCommandEvent, KeyRegistration, KeyConflictContext, SettingSpec,
    PlayerTradeEvent, PlayerBrewedPotionEvent, PlayerEnchantItemEvent,
    PlayerItemSmithedEvent, ItemSmeltedEvent, EntityTameEvent};
use rustpg_profile::Profile;
use rustpg_ui_layout::Rect;
use rustpg_ui_window::{MenuModel, GearRow, Vital, SpeciesRow, AttrRow, TreeNode, TreeModel};

const ID: &str = "rustpg";
const VERSION_STR: &str = "6.1.0-alpha-starling090-rift-forged";
// A resposta do comando /rustpg e recebida na thread principal do cliente.
// O client_tick abre a GUI quando o chat ja foi fechado.
static OPEN_REQUEST: AtomicBool = AtomicBool::new(false);
// A screen mouse hook only gets &dyn ClientHost. Close on mutable client tick.
static CLOSE_REQUEST: AtomicBool = AtomicBool::new(false);
// Some public callbacks do not expose ServerHost. Cache the last authoritative
// profile stats so stable API hooks (crop growth/container loot) can read them.
static PLAYER_STATS_CACHE: OnceLock<Mutex<HashMap<u128,rustpg_profile::Stats>>> = OnceLock::new();
fn player_stats_cache()->&'static Mutex<HashMap<u128,rustpg_profile::Stats>> {
    PLAYER_STATS_CACHE.get_or_init(||Mutex::new(HashMap::new()))
}
fn sync_player_stats(uuid:u128,stats:&rustpg_profile::Stats) {
    player_stats_cache().lock().unwrap_or_else(|e|e.into_inner()).insert(uuid,stats.clone());
}

// Uma marca persistida por mob: guarda nível, tier e mutação. Carregar
// a mesma entidade jamais rola um novo tier nem multiplica o atributo de novo.
static MOB_SCALE: AttachmentType<rustpg_scaling::MobScale> = AttachmentType {
    namespace: ID, name: "mob_scale", default: || rustpg_scaling::stats_for(1,rustpg_scaling::Tier::Normal),
    codec: Some(AttachmentCodec {
        encode: |m| { let mut c=Compound::new();
            c.insert("data".into(),Tag::ByteArray(rustpg_scaling::encode(m).into_bytes())); Ok(c) },
        decode: |c| match c.get(&"data".into()) {
            Some(Tag::ByteArray(b)) => {
                let s=String::from_utf8(b.clone()).map_err(|_|AttachmentError::Decode)?;
                rustpg_scaling::decode(&s).ok_or(AttachmentError::Decode)
            },
            _ => Err(AttachmentError::Decode),
        },
    }),
    copy_on_death: false,
    sync: true,
};

// A level-scoped persistent index: placed ore blocks never award mining XP,
// even after a save/reload. All writes happen on the authoritative server.
static PLACED_ORES: AttachmentType<rustpg_v4_provenance::PlacedOres> = AttachmentType {
    namespace: ID, name: "placed_ore_provenance", default: rustpg_v4_provenance::PlacedOres::new,
    codec: Some(AttachmentCodec {
        encode: |v| {let mut c=Compound::new();
            c.insert("data".into(),Tag::ByteArray(v.encode()));Ok(c)},
        decode: |c| match c.get(&"data".into()) {
            Some(Tag::ByteArray(bytes))=>rustpg_v4_provenance::PlacedOres::decode(bytes)
                .ok_or(AttachmentError::Decode),
            _=>Err(AttachmentError::Decode),
        },
    }),
    copy_on_death:false,
    sync:false,
};

static PROFILE: AttachmentType<Profile> = AttachmentType {
    namespace: ID, name: "profile", default: Profile::new,
    codec: Some(AttachmentCodec {
        encode: |p| { let mut c=Compound::new();
            c.insert("data".into(),Tag::ByteArray(rustpg_profile::encode(p).into_bytes())); Ok(c) },
        decode: |c| match c.get(&"data".into()) {
            Some(Tag::ByteArray(b)) => {
                let s=String::from_utf8(b.clone()).map_err(|_|AttachmentError::Decode)?;
                rustpg_profile::decode(&s).map_err(|_|AttachmentError::Decode)
            }
            _ => Err(AttachmentError::Decode),
        },
    }),
    copy_on_death: true,
    // Zero per-tick custom network serialization: sends one batched attachment when changed.
    sync: true,
};

#[derive(Default)]
struct Ui {
    page: usize, bestiary_sel: usize, tick: u32, scroll: usize,
    inspector_scroll:usize, inspector_key:String, inspector_since:u32,
    tree_id:usize, node_sel:usize, tree_index:bool, tree_detail_only:bool,
    detail_scroll:usize, pending:Option<String>,
    confirm_salvage:Option<(u64,u32)>,
    confirm_respec:Option<u32>,
    local_id:Option<u128>, announced:bool,
    cached_dimension:String,
    // O hook de renderizacao tem apenas &dyn ClientHost. A API de attachments
    // exige &mut dyn ClientHost, portanto o perfil sincronizado fica em cache
    // no tick do cliente e e lido daqui pela interface.
    profile:Option<Profile>,
    target:Option<rustpg_target_hud::TargetSnapshot>,
    target_ttl:u32,
    last_hit_victim:Option<(u32,u32)>,
    hp_vfx:rustpg_bar_vfx::BarState,
    stamina_vfx:rustpg_bar_vfx::BarState,
    xp_vfx:rustpg_bar_vfx::BarState,
    target_vfx:rustpg_bar_vfx::BarState,
    target_vfx_entity:Option<u32>,
    mob_stats:HashMap<u32,rustpg_mob_wire::MobSample>,
    last_target:Option<(u32,f64)>,
    floating:Vec<DamagePopup>,
    nearby_mobs:Vec<NearbyMob>,
    external_species: Option<BTreeMap<String, EntityText>>,
    bestiary_instances: HashMap<String, u32>,
    bestiary_scan_tick: Option<u32>,
    damage_meter:rustpg_damage_meter::DamageMeter,
    vanilla_health:(f64,f64),
    hunger:i32,
    air:i32, max_air:i32,
    armor:f64,
    absorption:f64,
}

#[derive(Clone,Debug)]
struct NearbyMob {
    id:u32,point:[f64;3],name:String,kind:String,hp:f32,max_hp:f32,
    armor:f64,level:u32,tier:String,seen:u32,
}
#[derive(Clone, Debug)]
struct DamagePopup {
    point:[f64;3], amount:f64, age:u32, family:rustpg_damage_meter::DamageFamily,
}

#[derive(Clone,Debug)]
struct PendingBleed {
    dimension:String,
    victim:u32,
    attacker:u32,
    damage:f32,
    remaining:u16,
    next_tick:u64,
}

#[derive(Default)]
struct Runtime {
    tick:u64, exhausted_entities:std::collections::HashSet<(String,u32)>, rng:Option<rustpg_num::Rng>,
    // A LevelRandomHost in damage hooks has no dimension() API. The authoritative
    // entity_join_level hook records the dimension here before combat events fire.
    // Entity IDs are world-session identities; player entries are refreshed on tick.
    entity_dimensions:HashMap<u32,String>,
    /// Registry ID recorded at native spawn time for exact weapon-vs-mob nodes.
    /// This is not a list of modded mobs and never creates new creatures.
    entity_kinds:HashMap<(String,u32),String>,
    hits:Vec<rustpg_v4_wire::DamageNotice>,
    impact_positions:HashMap<(String,u32),[f64;3]>,
    relic_snapshots:HashMap<(String,u32),rustpg_relic_engine::RelicEffects>,
    // Authoritative node snapshots for damage callbacks (which have no ServerHost).
    attribute_snapshots:HashMap<(String,u32),rustpg_profile::Stats>,
    // Cache keyed by UUID and current entity identity; reset on respawn/dimension.
    native_attr_cache:HashMap<u128,((String,u32),std::collections::BTreeMap<String,f64>)>,
    last_damage_tick:HashMap<(String,u32),u64>,
    entity_by_uuid:HashMap<u128,(String,u32)>,
    // Read-only drop hooks cannot mutate level attachments, so keep the same
    // placed-resource markers mirrored per dimension for drop eligibility.
    placed_resource_blocks:HashMap<String,rustpg_v4_provenance::PlacedOres>,
    resource_block_ids:HashMap<mod_api::BlockKey,String>,
    mature_resource_states:std::collections::HashSet<mod_api::BlockState>,
    cached_item_keys:HashMap<String,mod_api::ItemKey>,
    // Proven melee attempt, keyed by (dimension, victim, attacker). Credits
    // ONLY on the confirmed death event, never on a swing.
    recent_melee:HashMap<(String,u32,u32),(&'static str,u64)>,
    // Additional item-local Sharpness ranks beyond the native enchantment reader's
    // 255-level ceiling. This is a verified server-held stack snapshot, not client input.
    melee_sharpness_overflow:HashMap<(String,u32,u32),(u64,u64)>,
    // Exact bow/crossbow/trident recorded in the death-loot source.
    recent_death_weapon:HashMap<(String,u32,u32),(&'static str,u64)>,
    // Launch-time weapon snapshot keyed by the real projectile entity id.
    // This survives weapon switching before impact and never trusts the HUD.
    projectile_weapons:HashMap<(String,u32),(&'static str,u32,rustpg_profile::Stats,u64)>,
    critical_hits:HashMap<(String,u32),(u64,bool)>,
    combo:rustpg_combo::Tracker,
    // From persisted mob scale; health snapshots follow actual living health.
    elite_combat:HashMap<(String,u32),rustpg_elite_encounters::Snapshot>,
    last_mob_regen_tick:HashMap<(String,u32),u64>,
    // Per-boss encounter summon cooldown (world-session, bounded on unload).
    boss_summons:HashMap<(String,u32),u64>,
    sprint_positions:HashMap<u128,(String,[f64;3],u64)>,
    // Life steal is deferred: damage callbacks have LevelRandomHost, not ServerHost.
    // Only the real server tick may modify a player's health.
    pending_relic_heals:Vec<(String,u32,f32)>,
    // Native damage callback lacks ServerHost: defer a bounded, authoritative
    // highest-hit aggregation to ServerTickEnd with a verified player identity.
    pending_bestiary_hits:Vec<(String,u32,String,f64)>,
    pending_bleeds:Vec<PendingBleed>,
    /// Manifested overworld Rift arches, one active portal per owner.
    rift_portals:HashMap<u128,rustpg_rifts::RiftPortal>,
    /// Instanced runtime-level dungeons, one active session per owner.
    rift_sessions:HashMap<u128,rustpg_rifts::DungeonSession>,
    /// Server-authoritative combat action timelines keyed by player entity identity.
    action_combat:HashMap<(String,u32),rustpg_action_combat::CombatState>,
}
#[derive(Default)]
struct RustPGMod { ui: Mutex<Ui>, runtime:Mutex<Runtime> }
fn now_seconds()->u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(1,|t|t.as_secs())
}

fn run_rift_commands(host:&mut dyn ServerHost, commands:&[String]) {
    for command in commands {let _=host.commands().run_command(command);}
}
fn online_player(host:&dyn ServerHost,uuid:u128)->Option<mod_api::PlayerRef>{
    host.players().server_players().into_iter().find(|p|p.uuid()==uuid)
}
fn manifest_rift(host:&mut dyn ServerHost,rt:&mut Runtime,player:mod_api::PlayerRef,
    p:&Profile,tick:u64)->Result<String,String>{
    let uuid=player.uuid();
    if rt.rift_sessions.contains_key(&uuid){return Err("você já está em uma Fenda".into());}
    if rt.rift_portals.contains_key(&uuid){return Err("uma Fenda sua já está manifestada".into());}
    if p.rift_resonance<rustpg_rifts::RESONANCE_READY{
        return Err(format!("Ressonância insuficiente: {}/{}",p.rift_resonance,rustpg_rifts::RESONANCE_READY));
    }
    let ent=player.entity();let pos=ent.position(host).ok_or("posição indisponível")?;
    let dim=ent.level(host).map(|l|l.dimension_id().to_owned()).unwrap_or_else(||host.dimension().to_owned());
    let x=pos[0].floor() as i32;let z=pos[2].floor() as i32;
    let columns=[(x+8,z),(x-8,z),(x,z+8),(x,z-8),(x+7,z+7),(x-7,z+7),(x+7,z-7),(x-7,z-7)];
    let heights=host.surface_heights(&dim,&columns,mod_api::Heightmap::MotionBlocking)
        .map_err(|e|format!("não foi possível sondar o terreno: {e}"))?;
    let mut chosen=None;
    for (column,height) in columns.into_iter().zip(heights.into_iter()){
        if let Some(y)=height {if y>-60 && y<300 {chosen=Some([column.0,y+1,column.1]);break;}}
    }
    let portal_pos=chosen.ok_or("nenhum ponto carregado e seguro para a Fenda")?;
    let rank=rustpg_rifts::rank_for_power(p.power());
    let seed=(uuid as u64)^((uuid>>64) as u64)^tick.rotate_left(17)^0xC0B4_7F1D_9E2A_6531;
    let portal=rustpg_rifts::RiftPortal{owner:uuid,dimension:dim.clone(),pos:portal_pos,rank,seed,
        created_tick:tick,expires_tick:tick+rustpg_rifts::PORTAL_LIFETIME_TICKS};
    run_rift_commands(host,&rustpg_rifts::portal_commands(&dim,portal_pos,true));
    let _=host.commands().run_command(&rustpg_rifts::portal_particle_command(&dim,portal_pos));
    rt.rift_portals.insert(uuid,portal);
    Ok(format!("Fenda Rank {} manifestada a poucos blocos. Atravesse o arco para entrar.",rank.name()))
}
fn begin_rift(host:&mut dyn ServerHost,rt:&mut Runtime,player:mod_api::PlayerRef,
    mut p:Profile,tick:u64)->Result<String,String>{
    let uuid=player.uuid();
    if rt.rift_sessions.contains_key(&uuid){return Err("sessão de Fenda já ativa".into());}
    let portal=rt.rift_portals.get(&uuid).cloned().ok_or("nenhuma Fenda manifestada")?;
    let pos=player.entity().position(host).ok_or("posição indisponível")?;
    let dim=player.entity().level(host).map(|l|l.dimension_id().to_owned()).unwrap_or_default();
    if dim!=portal.dimension || !portal.contains(pos){return Err("entre no arco da sua Fenda primeiro".into());}
    if !p.consume_rift_resonance(){return Err("a Ressonância já não é suficiente".into());}
    let level_id=format!("rustpg:rift/{:016x}",portal.seed);
    let level=mod_api::RuntimeLevel::new("minecraft:overworld",mod_api::RuntimeGenerator::Void,portal.seed as i64);
    host.runtime_levels().open(&level_id,level).map_err(|e|format!("não foi possível abrir a instância: {e:?}"))?;
    let plan=rustpg_rifts::generate(portal.seed,portal.rank);
    let session=rustpg_rifts::DungeonSession{owner:uuid,level_id:level_id.clone(),origin_dimension:portal.dimension.clone(),
        origin_pos:pos,plan,phase:rustpg_rifts::SessionPhase::Opening,current_room:1,
        room_state:rustpg_rifts::RoomState::Dormant,wave:0,started_tick:tick,
        damage_start:p.damage_taken,deaths_start:p.deaths,build_attempted:false,last_action_tick:tick};
    if PROFILE.set(host,Holder::Player(uuid),p).is_err(){
        let _=host.runtime_levels().delete(&level_id);return Err("falha ao salvar a Ressonância".into());
    }
    run_rift_commands(host,&rustpg_rifts::portal_commands(&portal.dimension,portal.pos,false));
    rt.rift_portals.remove(&uuid);rt.rift_sessions.insert(uuid,session);
    Ok(format!("Fenda Rank {} estabilizando...",portal.rank.name()))
}
fn session_creatures_alive(host:&dyn ServerHost,level:mod_api::LevelRef,session:&rustpg_rifts::DungeonSession)->Option<usize>{
    let max_x=session.plan.rooms.last().map_or(32,|r|r.center[0]+r.half+24) as f64;
    Some(level.entities_in(host,[-32.0,-16.0,-40.0],[max_x,40.0,40.0])?.into_iter()
        .filter(|e|e.kind(host).is_some_and(|k|{
            let id=k.id(host);rustpg_native_entities::CREATURES.iter().any(|v|*v==id)
        }) && host.living(*e).is_some_and(|l|l.health(host).map_or(true,|hp|hp>0.0))).count())
}
fn spawn_rift_room(host:&mut dyn ServerHost,session:&rustpg_rifts::DungeonSession)->usize{
    let Some(room)=session.room() else{return 0;};let Some(level)=host.level(&session.level_id)else{return 0;};
    let mut list=if let Some(boss)=room.boss{vec![boss]}else{room.roster.clone()};
    if session.plan.modifiers.contains(&rustpg_rifts::RiftModifier::EliteSurge) && room.boss.is_none(){
        list.push(rustpg_native_entities::DUSK_REVENANT);
    }
    if session.plan.modifiers.contains(&rustpg_rifts::RiftModifier::Echo) && room.boss.is_none() {
        if let Some(last)=list.last().copied(){list.push(last);}
    }
    let offsets=[[-3.0,0.0,-3.0],[3.0,0.0,-3.0],[-3.0,0.0,3.0],[3.0,0.0,3.0],[0.0,0.0,-4.0],[0.0,0.0,4.0],[-4.0,0.0,0.0],[4.0,0.0,0.0]];
    let mut spawned=0;
    for (i,kind) in list.into_iter().enumerate(){let o=offsets[i%offsets.len()];let c=room.center;
        if level.spawn_mob(host,kind,[c[0] as f64+0.5+o[0],c[1] as f64+1.0,c[2] as f64+0.5+o[2]],mod_api::SpawnReason::MobSummoned).is_some(){spawned+=1;}}
    if session.plan.modifiers.contains(&rustpg_rifts::RiftModifier::Frenzy) {
        run_rift_commands(host,&rustpg_rifts::frenzy_commands(&session.level_id,rustpg_native_entities::CREATURES.iter().copied()));
    }
    spawned
}
fn complete_rift(host:&mut dyn ServerHost,player:mod_api::PlayerRef,session:&rustpg_rifts::DungeonSession,tick:u64)->Result<String,String>{
    let uuid=player.uuid();let mut p=PROFILE.get(host,Holder::Player(uuid)).map_err(|_|"perfil indisponível; recompensa preservada".to_owned())?;
    let elapsed=tick.saturating_sub(session.started_tick);let damage=(p.damage_taken-session.damage_start).max(0.0);
    let deaths=p.deaths.saturating_sub(session.deaths_start);let result=rustpg_rifts::assess_plan(&session.plan,elapsed,damage,p.vitality.max,deaths);
    p.add_general_xp(result.xp);p.add_rift_resonance(result.resonance);p.record_rift_clear(session.plan.rank,result.grade);
    if PROFILE.set(host,Holder::Player(uuid),p).is_err(){return Err("falha ao salvar a recompensa".into());}
    Ok(format!("Fenda Rank {} concluída · Nota {} · +{} XP · +{} Ressonância",session.plan.rank.name(),result.grade.name(),result.xp,result.resonance))
}
fn tick_rifts(this:&RustPGMod,host:&mut dyn ServerHost,tick:u64){
    let portals={this.runtime.lock().unwrap_or_else(|e|e.into_inner()).rift_portals.values().cloned().collect::<Vec<_>>()};
    for portal in portals{
        if tick>=portal.expires_tick{
            run_rift_commands(host,&rustpg_rifts::portal_commands(&portal.dimension,portal.pos,false));
            this.runtime.lock().unwrap_or_else(|e|e.into_inner()).rift_portals.remove(&portal.owner);continue;
        }
        if tick%10==0{let _=host.commands().run_command(&rustpg_rifts::portal_particle_command(&portal.dimension,portal.pos));}
        if let Some(player)=online_player(host,portal.owner){
            let same=player.entity().level(host).is_some_and(|l|l.dimension_id()==portal.dimension);
            if same && player.entity().position(host).is_some_and(|p|portal.contains(p)){
                let Ok(p)=PROFILE.get(host,Holder::Player(portal.owner)) else{continue;};
                let result={let mut rt=this.runtime.lock().unwrap_or_else(|e|e.into_inner());begin_rift(host,&mut rt,player,p,tick)};
                if let Err(err)=result{let _=player.action_bar(host,Text::literal(format!("Fenda: {err}")));}
            }
        }
    }
    let owners={this.runtime.lock().unwrap_or_else(|e|e.into_inner()).rift_sessions.keys().copied().collect::<Vec<_>>()};
    for owner in owners{
        let Some(mut session)=({this.runtime.lock().unwrap_or_else(|e|e.into_inner()).rift_sessions.get(&owner).cloned()})else{continue;};
        let Some(player)=online_player(host,owner)else{
            // Keep the session until the instance is actually gone.
            if host.runtime_levels().delete(&session.level_id).is_ok() || !host.runtime_levels().is_open(&session.level_id) {
                this.runtime.lock().unwrap_or_else(|e|e.into_inner()).rift_sessions.remove(&owner);
            }
            continue;
        };
        match session.phase{
            rustpg_rifts::SessionPhase::Opening=>{
                if host.runtime_levels().is_open(&session.level_id){
                    if !session.build_attempted{
                        let commands=rustpg_rifts::build_commands(&session.level_id,&session.plan);run_rift_commands(host,&commands);
                        session.build_attempted=true;session.last_action_tick=tick;session.phase=rustpg_rifts::SessionPhase::Building;
                        let mods=session.plan.modifiers.iter().map(|m|m.name()).collect::<Vec<_>>().join(" · ");
                        let _=player.action_bar(host,Text::literal(format!("Fenda Rank {} · {}",session.plan.rank.name(),if mods.is_empty(){"Sem anomalias"}else{&mods})));
                    }
                }
            },
            rustpg_rifts::SessionPhase::Building=>{
                if tick.saturating_sub(session.last_action_tick)>=4{
                    let _=player.entity().set_velocity(host,[0.0,0.0,0.0]);
                    if player.entity().teleport_to_level(host,&session.level_id,[0.5,6.0,0.5],Some([90.0,0.0])){
                        session.phase=rustpg_rifts::SessionPhase::Active;session.last_action_tick=tick;
                    }
                }
            },
            rustpg_rifts::SessionPhase::Active=>{
                let in_level=player.entity().level(host).is_some_and(|l|l.dimension_id()==session.level_id);
                if !in_level{
                    // Death or external dimension transfer: abandon without completion reward.
                    session.phase=rustpg_rifts::SessionPhase::Returning;
                    session.last_action_tick=tick;
                }else{
                    if tick%40==0 {run_rift_commands(host,&rustpg_rifts::modifier_commands(&session.level_id,&session.plan));}
                    if session.finished(){
                        match complete_rift(host,player,&session,tick){Ok(msg)=>{let _=player.action_bar(host,Text::literal(msg));session.phase=rustpg_rifts::SessionPhase::Rewarded;session.last_action_tick=tick;},Err(err)=>{let _=player.action_bar(host,Text::literal(err));}}
                    }else if let Some(room)=session.room().cloned(){
                        match session.room_state{
                            rustpg_rifts::RoomState::Dormant=>{
                                let pos=player.entity().position(host).unwrap_or([9999.0;3]);let dx=pos[0]-(room.center[0] as f64+0.5);let dz=pos[2]-(room.center[2] as f64+0.5);
                                if dx*dx+dz*dz<=((room.half+2)*(room.half+2)) as f64{
                                    if tick.saturating_sub(session.last_action_tick)>=20{
                                        let spawned=spawn_rift_room(host,&session);session.wave=0;session.last_action_tick=tick;
                                        if spawned>0 {
                                            session.room_state=rustpg_rifts::RoomState::Active;
                                            let _=player.action_bar(host,Text::literal(format!("{} · onda 1 · {} inimigos",room.kind.name(),spawned)));
                                        }else{let _=player.action_bar(host,Text::literal("Falha ao gerar inimigos; a sala não foi liberada."));}
                                    }
                                }
                            },
                            rustpg_rifts::RoomState::Active=>{
                                if tick.saturating_sub(session.last_action_tick)>=5{
                                    if let Some(level)=host.level(&session.level_id){if session_creatures_alive(host,level,&session)==Some(0){
                                        if session.wave+1<room.waves{
                                            let spawned=spawn_rift_room(host,&session);session.last_action_tick=tick;
                                            if spawned>0 {session.wave+=1;let _=player.action_bar(host,Text::literal(format!("Onda {} · {} inimigos",session.wave+1,spawned)));}
                                            else {let _=player.action_bar(host,Text::literal("Falha ao gerar próxima onda; tentando novamente."));}
                                        }
                                        else{session.current_room+=1;session.wave=0;session.room_state=rustpg_rifts::RoomState::Dormant;session.last_action_tick=tick;let _=player.action_bar(host,Text::literal("Sala limpa · avance pela Fenda"));}
                                    }}
                                }
                            },
                            _=>{}
                        }
                    }
                }
            },
            rustpg_rifts::SessionPhase::Rewarded=>{
                // Reward already saved; retry only the return, never the grant.
                if tick.saturating_sub(session.last_action_tick)>=8 {
                    session.last_action_tick=tick;
                    let _=player.entity().set_velocity(host,[0.0,0.0,0.0]);
                    let outside=player.entity().level(host).is_some_and(|l|l.dimension_id()!=session.level_id);
                    if outside || player.entity().teleport_to_level(host,&session.origin_dimension,session.origin_pos,None) {
                        session.phase=rustpg_rifts::SessionPhase::Returning;
                    }
                }
            },
            rustpg_rifts::SessionPhase::Returning=>{
                if tick.saturating_sub(session.last_action_tick)>=8{
                    let _=host.commands().run_command(&format!("execute in {} run forceload remove all",session.level_id));
                    match host.runtime_levels().delete(&session.level_id){Ok(())=>session.phase=rustpg_rifts::SessionPhase::Done,Err(mod_api::RuntimeLevelError::PlayersInside)=>{},Err(_)=>{if !host.runtime_levels().is_open(&session.level_id){session.phase=rustpg_rifts::SessionPhase::Done;}}}
                }
            },
            rustpg_rifts::SessionPhase::Done=>{}
        }
        let mut rt=this.runtime.lock().unwrap_or_else(|e|e.into_inner());
        if session.phase==rustpg_rifts::SessionPhase::Done{rt.rift_sessions.remove(&owner);}else{rt.rift_sessions.insert(owner,session);}
    }
}

fn rect_of(screen: [f32; 4]) -> Rect {
    Rect { x: screen[0], y: screen[1], w: screen[2], h: screen[3] }
}
fn lines(profile: &Profile, kind: &str) -> Vec<String> {
    match kind {
        "overview" => rustpg_view::overview(profile),
        "inventory" => rustpg_view::inventory(profile),
        "status" | "attributes" => rustpg_view::status(profile),
        "equipment" => rustpg_view::equipment(profile),
        "accessories" => rustpg_view::accessories(profile),
        "char" => rustpg_view::character(profile),
        "skills" => rustpg_view::combat(profile,"sword"),
        "trees" => rustpg_view::tree_list(),
        "bestiary" => rustpg_view::bestiary(profile),
        "profs" => rustpg_view::proficiencies(profile),
        "enchant" => rustpg_view::enchanting(profile),
        "evil" => {
            let discovered=|id:&str|profile.species.values().any(|r|r.mutations.iter().any(|x|x==id));
            rustpg_evil_att::MUTATIONS.iter().map(|m| if discovered(m.id) {
                format!("{} · {} · {}",m.name,m.rarity,m.description)
            } else {format!("??? · {} · Mutações desconhecidas",m.rarity)}).collect()
        },
        "achievements" => vec![format!("Espécies: {}",profile.species.len()),
            format!("Total de abates: {}",profile.kills.values().sum::<u64>())],
        "history" => profile.history.iter().rev().cloned().collect(),
        _ => vec!["Selecione um painel do Sistema".into()],
    }
}

// Reuses the legacy tree rules, layout, costs and prerequisites with the
// modern profile attachment. Tree purchasing is sent to the server through a registered play payload.
fn attr_rows(p:&Profile)->Vec<AttrRow> {
    use rustpg_attributes::Unit;
    let stats=p.node_stats();
    let mut rows:Vec<(f64,String,String)>=stats.0.iter()
        .filter(|(_,v)|v.is_finite() && v.abs()>0.0001)
        .filter_map(|(id,v)| {
            let def=rustpg_attributes::get(id)?;
            let shown=match def.unit {Unit::Flat=>format!("{v:.1}"),
                Unit::Count=>format!("{v:.0}"), _=>format!("{:.0}%",v*100.0)};
            Some((v.abs(),def.name.clone(),shown))
        }).collect();
    rows.sort_by(|a,b|b.0.total_cmp(&a.0));
    let max=rows.first().map_or(1.0,|r|r.0).max(1.0);
    rows.into_iter().take(8).map(|(v,name,value)| AttrRow {
        name,value,ratio:(v/max).clamp(0.0,1.0) as f32,hint:String::new()
    }).collect()
}
// Catalog topology is immutable; cache only positions to avoid O(n^2)
// depth relaxation on each frame and mouse interaction.
static RPG_COMBAT:OnceLock<rustpg_combat_pipeline::CombatPipeline>=OnceLock::new();
fn combat_pipeline()->&'static rustpg_combat_pipeline::CombatPipeline {
    RPG_COMBAT.get_or_init(rustpg_combat_pipeline::builtins)
}
static RPG_RELICS:OnceLock<rustpg_relic_engine::RelicRegistry>=OnceLock::new();
fn relics()->&'static rustpg_relic_engine::RelicRegistry {
    RPG_RELICS.get_or_init(rustpg_relic_engine::builtins)
}
static PROFESSION_PACKS:OnceLock<Arc<std::sync::RwLock<rustpg_profession_plugins::Registry>>>=OnceLock::new();
fn profession_packs()->&'static Arc<std::sync::RwLock<rustpg_profession_plugins::Registry>> {
    PROFESSION_PACKS.get_or_init(||Arc::new(std::sync::RwLock::new(rustpg_profession_plugins::bundled())))
}
// Player identity and counts come ONLY from completed server events. Addon
// packs cannot award generic XP or bypass the server-side creative check.
static VM_PROGRAMS:OnceLock<Arc<std::sync::RwLock<rustpg_script_vm::Runtime>>>=OnceLock::new();
fn scripted_packs()->&'static Arc<std::sync::RwLock<rustpg_script_vm::Runtime>>{
    VM_PROGRAMS.get_or_init(||Arc::new(std::sync::RwLock::new(rustpg_script_vm::bundled())))
}
fn self_rustpg_server_tick()->u64 {rustpg_guard::current_tick()}
fn credit_professional_event(host:&mut dyn ServerHost,uuid:u128,event:&str,units:u32) {
    let mut awards=profession_packs().read().unwrap_or_else(|e|e.into_inner()).award(event,units);
    let spent=awards.iter().fold(0u64,|n,(_,xp)|n.saturating_add(*xp));
    // A plugin cannot bypass the 64-XP budget consumed by core rules.
    let extension_budget=rustpg_profession_plugins::MAX_EVENT_XP.saturating_sub(spent);
    if extension_budget>0 {
        awards.extend(scripted_packs().read().unwrap_or_else(|e|e.into_inner())
            .run(event,units,extension_budget));
    }
    let Some(pl)=host.player(uuid) else{return;};
    if pl.is_creative(host).unwrap_or(false) || pl.is_spectator(host).unwrap_or(false) {return;}
    let Ok(mut profile)=PROFILE.get(host,Holder::Player(uuid)) else{return;};
    for (career,xp) in awards {if xp>0 {let _=profile.add_prof_xp(&career,xp);}}
    let notices=profile.record_mastery(event,
        self_rustpg_server_tick());
    if PROFILE.set(host,Holder::Player(uuid),profile).is_ok() {
        for notice in notices {let _=pl.action_bar(host,Text::literal(notice));}
    }
}
// A furnace yields many materials. Classify by the ACTUAL OUTPUT registry key,
// not by assuming that every smelt is food or that all smelts are metal.
fn furnace_profession(host:&dyn ServerHost,item:mod_api::ItemKey)->Option<&'static str> {
    const FOOD:&[&str]=&[
        "minecraft:cooked_beef","minecraft:cooked_porkchop",
        "minecraft:cooked_mutton","minecraft:cooked_chicken",
        "minecraft:cooked_rabbit","minecraft:cooked_cod",
        "minecraft:cooked_salmon","minecraft:baked_potato",
        "minecraft:dried_kelp",
    ];
    const METAL:&[&str]=&[
        "minecraft:iron_ingot","minecraft:gold_ingot","minecraft:copper_ingot",
        "minecraft:netherite_scrap",
    ];
    let match_key=|raw:&str| mod_api::Identifier::parse(raw).ok()
        .and_then(|id|host.item_key(&id))==Some(item);
    if FOOD.iter().any(|id|match_key(id)) {Some("cook")}
    else if METAL.iter().any(|id|match_key(id)) {Some("smelt")}
    else {None}
}
static RPG_EXTENSIONS:OnceLock<rustpg_v4_core::ExtensionRegistry>=OnceLock::new();
fn extensions()->&'static rustpg_v4_core::ExtensionRegistry {
    RPG_EXTENSIONS.get_or_init(rustpg_v4_core::builtins)
}
static TREE_LAYOUT_CACHE:OnceLock<Mutex<HashMap<String,Vec<(usize,usize)>>>>=OnceLock::new();
fn tree_model(p:&Profile,ui:&Ui)->Option<TreeModel> {
    let trees=rustpg_nodes::trees();
    let id=trees.get(ui.tree_id % trees.len().max(1))?;
    let full=rustpg_nodes::in_tree(id);
    // The catalogue is pre-filtered to effects backed by the stable API.
    let nodes=full;
    if nodes.is_empty(){return None;}
    let by_id:HashMap<&str,usize>=nodes.iter().enumerate()
        .map(|(i,n)|(n.id.as_str(),i)).collect();
    let cache=TREE_LAYOUT_CACHE.get_or_init(||Mutex::new(HashMap::new()));
    let cells={
        let mut map=cache.lock().unwrap_or_else(|er|er.into_inner());
        if let Some(v)=map.get(id).filter(|v|v.len()==nodes.len()) {v.clone()} else {
            let parents:Vec<Vec<usize>>=nodes.iter().map(|n|n.prereqs.iter()
                .filter_map(|(name,_)|by_id.get(name.as_str()).copied()).collect()).collect();
            let cells=rustpg_skillgraph::signature_layout(id,&parents);
            map.insert(id.to_string(),cells.clone());
            cells
        }
    };
    let mut out=Vec::with_capacity(nodes.len());
    let effective_stats=p.node_stats();
    let mut links=Vec::new();
    for (i,n) in nodes.iter().enumerate() {
        let (col,row)=cells.get(i).copied().unwrap_or((0,i));
        let rank=p.rank(&n.id);
        let validation = p.check_buy_with_stats(n, &effective_stats);
        let state = if matches!(validation, Err(rustpg_profile::BuyError::MaxRank)) {3}
            else if rank > 0 {2}
            else if validation.is_ok() {1}
            else {0};
        let mut missing:Vec<String>=n.prereqs.iter()
            .filter(|(id,r)|p.rank(id)<*r)
            .map(|(id,r)|format!("{} rank {}",
                rustpg_nodes::get(id).map_or(id.as_str(),|d|d.name.as_str()),r)).collect();
        if p.level<n.req_level {missing.push(format!("Nível geral {}",n.req_level));}
        for exclusion in &n.exclusions {if p.rank(exclusion)>0 {
            missing.push(format!("Ramo exclusivo: {}",exclusion));
        }}
        let prof_id=rustpg_nodes::tree_proficiency(&n.tree);
        let prof=p.prof(prof_id);
        if n.req_prof>0 && prof_id!="general" && prof.level<n.req_prof {
            missing.push(format!("{} nível {}",rustpg_progression::proficiency_name(prof_id),n.req_prof));
        }
        if p.points.saturating_add(prof.points)<n.cost_at(rank) {
            missing.push(format!("{} pontos (gerais + perícia)",n.cost_at(rank)));
        }
        if let Err(ref e)=validation {
            if matches!(e,rustpg_profile::BuyError::NoStatIncrease) {
                missing.push(e.to_string());
            }
        }
        let requirement=if state==3 {"Nível máximo atingido".to_owned()} else if missing.is_empty() {"Requisitos satisfeitos".to_string()}
            else {format!("Falta: {}",missing.join("; "))};
        let details=rustpg_node_effects::node_detail(n,rank,&effective_stats);
        let description=details;
        out.push(TreeNode{id:n.id.clone(),name:n.name.clone(),rank,
            max:n.max_label(),cost:n.cost_at(rank),col,row,state,
            buyable:validation.is_ok(), description,requirement});
        for (prereq,_) in &n.prereqs {
            if let Some(parent)=by_id.get(prereq.as_str()) {links.push((*parent,i));}
        }
    }
    let selected=ui.node_sel.min(out.len().saturating_sub(1));
    let detail=nodes.get(selected).map(|n|{
        let rank=p.rank(&n.id);
        let current=n.formula.value(rank);
        let next=n.formula.value(rank.saturating_add(1));
        let delta=next-current;
        let cost=n.cost_at(rank);
        let mut info=vec![
            n.description.clone(),
            format!("Rank {} / {} · custo {} PT",rank,n.max_label(),cost),
            format!("Efeito atual: {:.4} · próximo: {:.4} (variação {:+.4})",current,next,delta),
        ];
        if let Some(row)=out.get(selected) {
            info.push(row.requirement.clone());
        }
        info
    }).unwrap_or_default();
    Some(TreeModel{available_points:p.points.saturating_add(p.prof(rustpg_nodes::tree_proficiency(id)).points),name:rustpg_ui_window::constellation_category_label(id),nodes:out,links,
        pan:[0.0,0.0],zoom:1.0,selected,
        index_mode:ui.tree_index,detail_only:ui.tree_detail_only,detail})
}
fn gear_row(g: &rustpg_gear::Gear,p:&Profile,now:u64,worn:bool) -> GearRow {
    let (stat,bonus)=g.stat();
    let status=if worn {p.active_relics.status(g,now)}
        else {"EQUIPE PARA ATIVAR".into()};
    let ready=worn && status=="ATIVAR" && !p.stamina.exhausted
        && p.stamina.current>=rustpg_relic_active::ACTIVE_STAMINA_COST;
    GearRow {id:g.uid,slot:g.slot.clone(),title:g.name(),
        ability:rustpg_relic_engine::describe(g).to_owned(),
        stat:stat.into(),bonus,rarity:g.rarity,power_status:status,power_ready:ready}
}
fn relic_snapshot(p:&Profile,now:u64)->rustpg_relic_engine::RelicEffects {
    relics().snapshot(&p.gear).combine(p.active_relics.bonuses(&p.gear,now))
}
fn activate_relic(p:&mut Profile,slot:&str)->Result<String,String> {
    p.active_relics.activate(&p.gear,slot,now_seconds(),&mut p.stamina)
        .map_err(|err|err.to_string())
}
fn humanize_entity_id(id:&str)->String {
    let (ns,path)=id.split_once(':').unwrap_or(("minecraft",id));
    let words=path.rsplit('/').next().unwrap_or(path).replace(['_','-']," ");
    let mut name=String::new();
    for word in words.split_whitespace() {
        if !name.is_empty(){name.push(' ');}
        let mut chars=word.chars();
        if let Some(first)=chars.next(){name.extend(first.to_uppercase());name.extend(chars);}
    }
    if name.is_empty(){name=id.to_string();}
    if ns=="minecraft" {name} else {format!("{name} · {ns}")}
}
#[derive(Default)]
struct EntityText {
    name: Option<String>,
    description: Option<String>,
}

fn entity_text(kind: &str, translate: impl Fn(&str) -> Option<String>) -> EntityText {
    let key = format!("entity.{}", kind.replace([':', '/'], "."));
    let lookup = |key: &str| translate(key).filter(|v| !v.trim().is_empty() && v != key);
    EntityText {
        name: lookup(&key),
        description: ["description", "desc", "lore"].iter()
            .find_map(|suffix| lookup(&format!("{key}.{suffix}"))),
    }
}

fn refresh_bestiary(host: &dyn ClientHost, ui: &mut Ui) {
    if ui.external_species.is_none() {
        let keys = host.entity_type_keys();
        if !keys.is_empty() {
            ui.external_species = Some(keys.into_iter().map(|key| key.id(host))
                .filter(|id| !id.starts_with("minecraft:") && !id.starts_with("rustpg:"))
                .map(|id| {let text = entity_text(&id, |key| host.translate(key)); (id,text)})
                .collect());
        }
    }
    if ui.bestiary_scan_tick.is_some_and(|last| ui.tick.wrapping_sub(last)<20) {return;}
    ui.bestiary_scan_tick = Some(ui.tick);
    ui.bestiary_instances.clear();
    let Some(player) = host.local_player().map(|p|p.entity()) else {return;};
    let (Some(level),Some(pos)) = (player.level(host),player.position(host)) else {return;};
    if let Some(entities) = level.entities_in(host,pos.map(|x|x-48.0),pos.map(|x|x+48.0)) {
        for entity in entities {
            if let Some(kind) = entity.kind(host) {
                ui.bestiary_instances.entry(kind.id(host)).or_insert(entity.id());
            }
        }
    }
}

fn bestiary_ids<'a>(p: &'a Profile, ui: &'a Ui) -> std::collections::BTreeSet<&'a str> {
    let mut ids: std::collections::BTreeSet<_> = p.species.values()
        .filter(|r| r.kills > 0).map(|r| r.kind.as_str()).collect();
    if let Some(registered) = &ui.external_species {
        ids.extend(registered.keys().map(String::as_str));
    }
    ids
}

fn menu_model(p: &Profile, ui: &Ui, area: Rect, mouse: [f32; 2]) -> MenuModel {
    let external_inspector=rustpg_inspector::inspector_area(area).is_some();
    let area=rustpg_inspector::main_area(area);
    let page=ui.page.min(rustpg_view::PAGES.len()-1);
    let (kind,title)=rustpg_view::PAGES[page];
    let species: Vec<SpeciesRow> = if kind == "bestiary" {
        bestiary_ids(p,ui).into_iter().map(|id| {
            let record = p.species.get(id);
            let live = ui.nearby_mobs.iter().find(|m| m.kind == id);
            let text = ui.external_species.as_ref().and_then(|map| map.get(id));
            let display_name = text.and_then(|t| t.name.clone())
                .or_else(|| live.map(|m| m.name.trim()).filter(|n| !n.is_empty() && *n != id && *n != "Criatura").map(str::to_owned))
                .unwrap_or_else(|| humanize_entity_id(id));
            let description = text.and_then(|t| t.description.clone()).unwrap_or_else(|| {
                let namespace = id.split(':').next().unwrap_or(id);
                format!("{display_name} pertence a {namespace}.")
            });
            SpeciesRow {
                name:id.to_owned(), display_name, description,
                live_entity:ui.bestiary_instances.get(id).copied().or_else(|| live.map(|m|m.id)),
                kills:record.map_or(0,|r|r.kills), level:record.map_or(0,|r|r.max_level),
                best:record.map_or(0.0,|r|r.best_damage),
                tiers:record.map_or_else(String::new,|r|r.tiers.join(", ")),
                drops:record.map_or_else(String::new,|r|r.drops.join(", ")),
                discovery:record.map_or(0.0,|r|r.discovery()),
            }
        }).collect()
    } else {Vec::new()};
    let bestiary_sel = ui.bestiary_sel.min(species.len().saturating_sub(1));
    let xp_need=rustpg_progression::general_xp_to_next(p.level).max(1);
    MenuModel {
        area,mouse,tick:ui.tick,page,
        pages:rustpg_view::PAGES.iter().map(|(_,n)|n.to_string()).collect(),
        kind,title:title.into(),level:p.level,rank:rustpg_engine_presentation::profile_rank(p).into(),
        next_rank:"".into(),rank_progress:0.0,xp:(p.xp as f64 / xp_need as f64).clamp(0.0,1.0) as f32,
        points:p.points,
        vitals:vec![
            Vital {label:"HP".into(), ratio:(p.vitality.current/p.vitality.max.max(1.0))
                .clamp(0.0,1.0) as f32,
                value:format!("{:.0}/{:.0}",p.vitality.current,p.vitality.max)},
            Vital {label:"SP".into(),ratio:p.stamina.fraction() as f32,
                value:format!("{:.0}/{:.0}",p.stamina.current,p.stamina.max)}],
        lines:lines(p,kind), attrs:attr_rows(p),
        tree:if kind=="trees" {tree_model(p,ui)} else {None},bestiary:species,
        bestiary_sel,detail_scroll:ui.detail_scroll,
        external_inspector,live_target:ui.target.clone(),
        bag:p.gear.bag.iter().map(|g|gear_row(g,p,now_seconds(),false)).collect(),
        worn:p.gear.worn.values().map(|g|gear_row(g,p,now_seconds(),true)).collect(),
        toast:ui.confirm_respec.filter(|when|ui.tick.wrapping_sub(*when)<100)
            .map(|_|"REDEFINIR HABILIDADES? Toque novamente em até 5s.".to_owned())
            .or_else(||ui.confirm_salvage.filter(|(_,when)|ui.tick.wrapping_sub(*when)<100)
                .map(|(uid,_)|format!("RECICLAR RELÍQUIA #{uid}? Toque REC novamente em até 5s."))),
        buy_label:"Adquirir".into(),scroll:ui.scroll,
    }
}

// The official Starling 0.9 draw API. Unlike the legacy HUD adapter, sprites
// and 3D mob model are actual engine calls; only available GUI sprites are used.
fn sprite(draw:&mut dyn mod_api::DrawContext, key:&str, src:[f32;4], dst:[f32;4]) {
    let _=draw.sprite(key,src,dst,None);
}
fn sprite_decor(draw:&mut dyn mod_api::DrawContext,key:&str,src:[f32;4],dst:[f32;4],alpha:u32) {
    let _=draw.sprite(key,src,dst,Some((alpha.clamp(0,255)<<24)|0x00FFFFFF));
}
fn draw_layer(draw:&mut dyn mod_api::DrawContext, layer:&rustpg_ui_system::Layer) {
    use rustpg_ui_system::Prim;
    for prim in &layer.prims {
        match prim {
            Prim::Rect{x,y,w,h,color} => draw.fill([*x,*y,*w,*h],*color),
            Prim::Quad{p,color} => draw.quad(*p,*color),
            Prim::Text{text,x,y,color,shadow} => draw.text(text,[*x,*y],*color,*shadow),
            Prim::Sprite{key,src,dst,tint} => {let _=draw.sprite(key,*src,*dst,*tint);},
        }
    }
}
/// Nine-slice: source corners retain their aspect ratio, unlike stretching a
/// square 256x256 panel onto a very wide 244x73 HUD. The center is reusable.
fn draw_animated_bar_fx(draw:&mut dyn mod_api::DrawContext,
    x:f32,y:f32,w:f32,h:f32,fx:&rustpg_bar_vfx::BarState,color:u32){
    if !fx.initialized||w<=0.0||h<=0.0{return;}
    if fx.ghost>fx.display+0.002 {
        // Neutral grey ghost: traditional RPG delayed-loss indicator.
        draw.fill([x+w*fx.display,y,w*(fx.ghost-fx.display),h],0xC6A7B0BC);
    } else if fx.display>fx.ghost+0.002 {
        // Healing/recovery pulse: a small light sweep over the newly filled area.
        draw.fill([x+w*fx.ghost,y,w*(fx.display-fx.ghost),h],0x6657E9B3);
    }
    if fx.flash>0.02 && fx.display>0.0 {
        let alpha=((fx.flash*145.0) as u32).min(160);
        let c=(alpha<<24)|(color&0x00FFFFFF);
        let tip=(x+w*fx.display).clamp(x,x+w);
        draw.fill([tip-1.5,y-2.0,3.0,h+4.0],c);
        draw.fill([tip-3.0,y-1.0,6.0,1.0],c);
        draw.fill([tip-3.0,y+h,6.0,1.0],c);
    }
}
fn draw_hud_nine_slice(draw:&mut dyn mod_api::DrawContext,rect:[f32;4]) {
    let key="rustpg:gui/ui/panels/ui_panel_main_compact";
    let [x,y,w,h]=rect;
    let left=12.0_f32.min(w*0.24);let right=left;
    let top=12.0_f32.min(h*0.24);let bottom=top;
    let sx=[0.0,18.0,238.0,256.0];
    let sy=[0.0,18.0,238.0,256.0];
    let dx=[x,x+left,x+w-right,x+w];
    let dy=[y,y+top,y+h-bottom,y+h];
    for i in 0..3 {for j in 0..3 {
        let src=[sx[i],sy[j],sx[i+1]-sx[i],sy[j+1]-sy[j]];
        let dst=[dx[i],dy[j],dx[i+1]-dx[i],dy[j+1]-dy[j]];
        if dst[2]>0.0 && dst[3]>0.0 {let _=draw.sprite(key,src,dst,None);}
    }}
}
/// 9-slice frame: corners keep their native proportions even when the System
/// occupies 68% of the viewport and the inspector fills the remaining width.
fn draw_nine_slice(draw:&mut dyn mod_api::DrawContext,key:&str,
    source:[f32;2],dst:[f32;4],source_border:f32,dst_border:f32,opacity:u32){
    let [x,y,w,h]=dst;
    if w<=0.0||h<=0.0{return;}
    let (sw,sh)=(source[0],source[1]);
    let edge_src=source_border.min(sw*0.25).min(sh*0.25);
    let ex=dst_border.min(w*0.25);let ey=dst_border.min(h*0.25);
    let sx=[0.0,edge_src,sw-edge_src,sw];
    let sy=[0.0,edge_src,sh-edge_src,sh];
    let dx=[x,x+ex,x+w-ex,x+w];
    let dy=[y,y+ey,y+h-ey,y+h];
    for col in 0..3 {for row in 0..3 {
        sprite_decor(draw,key,
            [sx[col],sy[row],sx[col+1]-sx[col],sy[row+1]-sy[row]],
            [dx[col],dy[row],dx[col+1]-dx[col],dy[row+1]-dy[row]],opacity);
    }}
}
fn draw_chrome(draw:&mut dyn mod_api::DrawContext, area:Rect, model:&MenuModel) {
    let win=rustpg_ui_window::window(area);
    sprite_decor(draw,"rustpg:gui/ui/panels/ui_background_holographic",
        [0.0,0.0,512.0,512.0],[win.x,win.y,win.w,win.h],38);
    draw_nine_slice(draw,"rustpg:gui/ui/panels/ui_panel_main",
        [768.0,512.0],[win.x+5.0,win.y+12.0,win.w-10.0,win.h-17.0],28.0,9.0,43);
    draw_nine_slice(draw,"rustpg:gui/ui/frames/ui_frame_main",
        [512.0,512.0],[win.x,win.y,win.w,win.h],32.0,12.0,235);
    let title=rustpg_ui_window::title_rect(win);
    sprite_decor(draw,"rustpg:gui/ui/frames/ui_frame_title",
        [0.0,0.0,512.0,96.0],[title.x,title.y,title.w,title.h],160);
    let first=rustpg_ui_window::tab_group(model.page)*rustpg_ui_window::TAB_SLOTS;
    for (i,tab) in rustpg_ui_window::tab_rects(win,model.pages.len(),model.page).into_iter().enumerate() {
        let key=if first+i==model.page {"rustpg:gui/ui/tabs/ui_tab_selected"}
            else {"rustpg:gui/ui/tabs/ui_tab_normal"};
        sprite_decor(draw,key,[0.0,0.0,256.0,64.0],[tab.x,tab.y,tab.w,tab.h],125);
    }
    let content=rustpg_ui_window::content(win,model.pages.len());
    if matches!(model.kind,"bestiary"|"inventory"|"equipment"|"accessories") {
        sprite_decor(draw,"rustpg:gui/ui/panels/ui_panel_secondary",
            [0.0,0.0,512.0,512.0],[content.x,content.y,content.w,content.h],56);
    }
}
fn draw_species_model(draw: &mut dyn mod_api::DrawContext, host: &dyn ClientHost,
    kind: &str, live: Option<u32>, mut view: GuiEntityView) -> bool {
    if !rustpg_inspector::valid_kind(kind) {return false;}
    // The pinned Starling 0.9.0 gui_entity_preview() calls the vanilla
    // mobs::render::render_with() and NEVER the mod's EntityRenderer. Its
    // "Existing" variant still paints RustPG's minecraft:zombie template.
    // For original RustPG species, display a truthful, deterministic 3D
    // portrait rasterized from the SAME authored cubes as the world renderer.
    // It is a STATIC mesh portrait; the world creature remains live/animated.
    if rustpg_native_entities::CREATURES.contains(&kind) {
        let name=kind.split(':').nth(1).unwrap_or(kind);
        let key=format!("rustpg:gui/portraits/{name}");
        let rect=[view.rect[0],view.rect[1],
                  (view.rect[2]-view.rect[0]).max(1.0),
                  (view.rect[3]-view.rect[1]).max(1.0)];
        return draw.sprite(&key,[0.0,0.0,128.0,128.0],rect,None);
    }
    // Third-party creatures have no RustPG-authored mesh for a truthful
    // sprite, so request the engine's GUI depiction instead.

    let entity=live.and_then(|id|host.entity(id))
        .filter(|e|e.kind(host).is_some_and(|key|key.id(host)==kind));
    let dims=rustpg_native_entities::preview_dimensions(kind)
        .or_else(||entity.as_ref().and_then(|ent|ent.dimensions(host))
            .map(|[w,h]|[w as f32,h as f32]));
    if let Some([width,height])=dims {
        if width.is_finite()&&height.is_finite()&&width>0.0&&height>0.0 {
            let vw=(view.rect[2]-view.rect[0]).max(1.0);
            let vh=(view.rect[3]-view.rect[1]).max(1.0);
            // Native authored mobs may be much taller/wider than the
            // supported zombie template's collision box. Fit the FULL mesh.
            view.size=(0.78*(vw/width).min(vh/height)).clamp(1.0,80.0);
        }
    }
    if let Some(ent)=entity {
        if draw.entity(&GuiEntity::Existing(ent),view).is_ok(){return true;}
    }
    draw.entity(&GuiEntity::kind(kind.to_owned()),view).is_ok()
}

fn draw_assets(draw:&mut dyn mod_api::DrawContext, area:Rect, model:&MenuModel, host:&dyn ClientHost) {
    // Rank do personagem: sprites opcionais, sem esconder os textos da interface.
    let rank_sprite=match model.rank.as_str() {
        "E"=>"rank_e", "D"=>"rank_d", "C"=>"rank_c", "B"=>"rank_b",
        "A"=>"rank_a", "S"=>"rank_s", "Monarca"|"MONARCA"=>"rank_monarch",
        _=>"rank_e",
    };
    let rank_key=format!("rustpg:gui/hud/rank/{rank_sprite}");
    sprite(draw,&rank_key,[0.0,0.0,256.0,256.0],[area.x+area.w-36.0,area.y+12.0,20.0,20.0]);
    if model.kind == "bestiary" {
        let i=model.bestiary_sel.min(model.bestiary.len().saturating_sub(1));
        if let Some(species)=model.bestiary.get(i) {
            let portrait = rustpg_ui_window::bestiary_portrait(model);
            let rect = [portrait.x+1.0, portrait.y+1.0, portrait.x+portrait.w-1.0,
                portrait.y+portrait.h-12.0];
            if portrait.h > 16.0 && rustpg_inspector::valid_kind(&species.name) {
                let view=GuiEntityView::new(rect,((portrait.h-12.0)*0.38).clamp(2.0,36.0),0.0625,
                    GuiEntityLook::Turned {yaw:(model.tick as f32 * 0.7) % 360.0,pitch:0.0});
                if !draw_species_model(draw,host,&species.name,species.live_entity,view) {
                    draw.text("Sem prévia",[rect[0]+1.0,rect[1]+2.0],0xFFB1C9D9,true);
                }
            }
        }
    }
}

// The network context is the authority: never read a player UUID from message bytes.
// Main-thread delivery is the default for this registered play channel.
struct ServerActionHandler;
impl IPayloadHandler for ServerActionHandler {
    fn handle(&self, payload:PayloadBytes, ctx:&mut dyn IPayloadContext)
        -> Result<(),PayloadHandleError> {
        if ctx.flow()!=PayloadDirection::ToServer || ctx.protocol()!=PayloadProtocol::Play {
            return Ok(());
        }
        let Some(action)=rustpg_actions::parse(&payload.0) else{return Ok(());};
        let Ok(player)=ctx.player() else{return Ok(());};
        let Ok(host)=ctx.server_host() else{return Ok(());};
        let uuid=player.uuid();
        let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{let _=player.action_bar(host,Text::literal("Perfil indisponível; ação não salva."));return Ok(());};
        let result:Result<String,String> = match action {
            rustpg_actions::Action::Equip(uid) => p.gear.equip(uid).map_err(str::to_owned),
            rustpg_actions::Action::Unequip(slot) => p.gear.unequip(&slot).map_err(str::to_owned),
            rustpg_actions::Action::Salvage(uid) => p.gear.salvage(uid)
                .map(|xp| {p.add_general_xp(xp);format!("Relíquia reciclada: +{xp} XP")})
                .map_err(str::to_owned),
            rustpg_actions::Action::Buy(id) => p.buy(&id)
                .map(|rank|format!("Habilidade {id}: rank {rank}"))
                .map_err(|e|e.to_string()),
            rustpg_actions::Action::Activate(slot) => activate_relic(&mut p,&slot),
            rustpg_actions::Action::Defend(stance) => {
                let dim=player.entity().level(host).map(|v|v.dimension_id().to_owned())
                    .unwrap_or_else(||host.dimension().to_owned());
                if player.is_creative(host).unwrap_or(false)
                    || player.is_spectator(host).unwrap_or(false) {
                    Err("Defesa RPG desativada no modo criativo/espectador".into())
                } else {
                    let stats=p.node_stats();
                    let reduction=if stance==rustpg_guard::Stance::Dodge {
                        stats.get("general.dodge_cost_reduction").clamp(0.0,0.90)
                    }else{0.0};
                    let cost=stance.cost()*(1.0-reduction);
                    rustpg_guard::activate_costed(&dim,player.entity().id(),stance,&mut p.stamina,cost)
                }
            },
            rustpg_actions::Action::Power(ability) => {
                let dim=player.entity().level(host).map(|v|v.dimension_id().to_owned())
                    .unwrap_or_else(||host.dimension().to_owned());
                if player.is_creative(host).unwrap_or(false) || player.is_spectator(host).unwrap_or(false) {
                    Err("Poder RPG indisponível nesse modo".into())
                }else{
                    let reduction=p.node_stats().get("general.skill_cost_reduction").clamp(0.0,0.90);
                    let cost=ability.cost()*(1.0-reduction);
                    rustpg_powers::activate_costed(&dim,player.entity().id(),ability,&mut p,cost)
                }
            },
            rustpg_actions::Action::Respec => {
                if p.ranks.is_empty() {Err("Nenhuma habilidade para redefinir".into())}
                else {let refund=p.respec(); Ok(format!("Árvore redefinida: +{refund} pontos"))}
            },
        };
        // Update the native inventory bridge before the next click/pickup can run.
        sync_player_stats(uuid,&p.node_stats());
        // Validation is entirely server-side: spent points, owned UID, slot and costs.
        let message=match result {
            Ok(line)=>{
                if PROFILE.set(host,Holder::Player(uuid),p).is_err(){
                    "Falha ao salvar operação".to_string()
                }else{line}
            },
            Err(msg)=>format!("Ação recusada: {msg}"),
        };
        let _=player.action_bar(host,Text::literal(message));
        Ok(())
    }
}

// Battle packet inbox is isolated from GUI lock; the handler may run on a
// network callback. ClientTick drains it on the owning render/game thread.
static HIT_INBOX:OnceLock<Mutex<Vec<rustpg_v4_wire::DamageNotice>>> = OnceLock::new();
fn take_hit_inbox()->Vec<rustpg_v4_wire::DamageNotice> {
    let mut queue=HIT_INBOX.get_or_init(||Mutex::new(Vec::new()))
        .lock().unwrap_or_else(|e|e.into_inner());
    std::mem::take(&mut *queue)
}
struct ClientDamageHandler;
impl IPayloadHandler for ClientDamageHandler {
    fn handle(&self,payload:PayloadBytes,ctx:&mut dyn IPayloadContext)
        ->Result<(),PayloadHandleError> {
        if ctx.flow()!=PayloadDirection::ToClient || ctx.protocol()!=PayloadProtocol::Play {
            return Ok(());
        }
        let Some(hit)=rustpg_v4_wire::DamageNotice::decode(&payload.0) else{return Ok(());};
        let mut queue=HIT_INBOX.get_or_init(||Mutex::new(Vec::new()))
            .lock().unwrap_or_else(|e|e.into_inner());
        if queue.len()<128 {queue.push(hit);}
        Ok(())
    }
}

// World-health replication is server-authoritative: other creatures are not
// readable through ClientHost::living() in SDK 0.9 (only local player is).
static MOB_INBOX:OnceLock<Mutex<Vec<rustpg_mob_wire::MobPacket>>> = OnceLock::new();
fn take_mob_inbox()->Vec<rustpg_mob_wire::MobPacket>{
    let mut q=MOB_INBOX.get_or_init(||Mutex::new(Vec::new()))
        .lock().unwrap_or_else(|e|e.into_inner());
    std::mem::take(&mut *q)
}
struct ClientMobHandler;
impl IPayloadHandler for ClientMobHandler {
    fn handle(&self,payload:PayloadBytes,ctx:&mut dyn IPayloadContext)
        ->Result<(),PayloadHandleError> {
        if ctx.flow()!=PayloadDirection::ToClient || ctx.protocol()!=PayloadProtocol::Play{return Ok(());}
        let Some(packet)=rustpg_mob_wire::MobPacket::decode(&payload.0) else{return Ok(());};
        let mut q=MOB_INBOX.get_or_init(||Mutex::new(Vec::new()))
            .lock().unwrap_or_else(|e|e.into_inner());
        if q.len()>=4 {q.remove(0);}
        q.push(packet);
        Ok(())
    }
}

// Durability skills run at Horizon's post-Unbreaking ItemPart hook. They alter
// only the real wear about to be committed, never repair an item after the fact.
#[derive(Clone,Copy)]
struct RustPGDurabilityPart(&'static [&'static str]);
#[mod_api::horizon_part]
impl mod_api::ItemPart for RustPGDurabilityPart {
    fn damage_item(&self,host:&mut dyn ServerHost,e:&mut mod_api::DamageItemEvent<'_>)->Flow {
        if e.amount<=0 {return Flow::Continue;}
        let Some(player)=host.players().acting_player() else{return Flow::Continue;};
        let Ok(profile)=PROFILE.get(host,Holder::Player(player.uuid())) else{return Flow::Continue;};
        let stats=profile.node_stats();
        let chances=self.0.iter().map(|key|stats.get(key).clamp(0.0,1.0)).collect::<Vec<_>>();
        let efficiency=rustpg_java_port::combine_independent_chances(&chances);
        if efficiency<=0.0 {return Flow::Continue;}
        let salt=self.0.iter().flat_map(|s|s.bytes()).fold(0u64,|h,b|h.rotate_left(5)^u64::from(b));
        let seed=(player.uuid() as u64)^rustpg_guard::current_tick().rotate_left(19)^salt;
        e.amount=rustpg_node_effects::rod_wear(e.amount,efficiency,seed);
        Flow::Continue
    }
}
// Canal independente cliente<-servidor usado por /rustpg para abrir a interface.
struct ClientOpenHandler;
impl IPayloadHandler for ClientOpenHandler {
    fn handle(&self, payload:PayloadBytes, ctx:&mut dyn IPayloadContext)
        -> Result<(),PayloadHandleError> {
        if ctx.flow()!=PayloadDirection::ToClient || ctx.protocol()!=PayloadProtocol::Play {
            return Ok(());
        }
        if payload.0.as_slice()!=b"open" {return Ok(());}
        // O canal chegou ao cliente: aguardar o chat fechar antes de abrir o menu.
        if let Ok(host)=ctx.client_host() {
            if host.local_player().is_some() {
                OPEN_REQUEST.store(true,Ordering::Relaxed);
            }
        }
        Ok(())
    }
}

#[mod_api::horizon_mod]
impl Hooks for RustPGMod {
    fn register(&self,registrar:&mut dyn Registrar,_:&RegistrationContext) {
        // Attribute catalogue and expanded vanilla bounds from Starling 0.9.0.
        rustpg_starling090::register(registrar);
        rustpg_native_entities::register_entities(registrar);
        rustpg_world_vfx::register(registrar);
        // Item-scoped wear hooks use Horizon’s public post-Unbreaking durability point.
        registrar.item_part_tag("minecraft:swords",RustPGDurabilityPart(&["weapon.sword.durability"]));
        registrar.item_part_tag("minecraft:axes",RustPGDurabilityPart(&["weapon.axe.durability","woodcutting.durability"]));
        registrar.item_part_tag("minecraft:pickaxes",RustPGDurabilityPart(&["mining.durability"]));
        registrar.item_part_tag("minecraft:spears",RustPGDurabilityPart(&["weapon.spear.durability"]));
        for (id,stats) in [
            ("minecraft:bow",&["weapon.bow.durability"][..]),
            ("minecraft:crossbow",&["weapon.crossbow.durability"][..]),
            ("minecraft:mace",&["weapon.mace.durability"][..]),
            ("minecraft:trident",&["weapon.trident.durability"][..]),
        ] {
            let key=registrar.item_key(id);
            registrar.item_part(key,RustPGDurabilityPart(stats));
        }
        registrar.register_key_mapping(KeyRegistration::new("active_relic","key.keyboard.h")
            .label("key.rustpg.active_relic")
            .category("key.categories.rustpg")
            .context(KeyConflictContext::InGame));
        for (id,key,label) in [("power_fury","key.keyboard.j","key.rustpg.power_fury"),("power_barrier","key.keyboard.k","key.rustpg.power_barrier"),("power_siphon","key.keyboard.l","key.rustpg.power_siphon")] {
            registrar.register_key_mapping(KeyRegistration::new(id,key)
                .label(label)
                .category("key.categories.rustpg")
                .context(KeyConflictContext::InGame));
        }
        registrar.register_key_mapping(KeyRegistration::new("parry","key.keyboard.b")
            .label("key.rustpg.parry")
            .category("key.categories.rustpg")
            .context(KeyConflictContext::InGame));
        registrar.register_key_mapping(KeyRegistration::new("dodge","key.keyboard.g")
            .label("key.rustpg.dodge")
            .category("key.categories.rustpg")
            .context(KeyConflictContext::InGame));
        registrar.register_key_mapping(KeyRegistration::new("open_system","key.keyboard.v")
            .label("key.rustpg.open_system")
            .category("key.categories.rustpg")
            .context(KeyConflictContext::InGame));
        // These are *real* Options > Mods native settings, saved by Starling.
        let _=registrar.register_setting(SettingSpec::toggle("hud_enabled",
            "options.rustpg.hud_enabled",true).tooltip("options.rustpg.hud_enabled.tip"));
        let _=registrar.register_setting(SettingSpec::toggle("replace_vanilla",
            "options.rustpg.replace_vanilla",true).tooltip("options.rustpg.replace_vanilla.tip"));
        let _=registrar.register_setting(SettingSpec::toggle("mob_hud",
            "options.rustpg.mob_hud",true));
        let _=registrar.register_setting(SettingSpec::toggle("power_vfx",
            "options.rustpg.power_vfx",true));
        let _=registrar.register_setting(SettingSpec::toggle("damage_numbers",
            "options.rustpg.damage_numbers",true));
        let _=registrar.register_setting(SettingSpec::toggle("damage_meter",
            "options.rustpg.damage_meter",true)
            .tooltip("options.rustpg.damage_meter.tip"));
        let _=registrar.register_setting(SettingSpec::toggle("damage_meter_list",
            "options.rustpg.damage_meter_list",true));
        let _=registrar.register_setting(SettingSpec::range("damage_meter_scale",
            "options.rustpg.damage_meter_scale",0.75,1.5,0.25,1.0));
        // The configurable input is the native KeyRegistration above: its
        // rebinding UI captures the next pressed key in Options > Controls.
        // A choice-list setting would ignore arbitrary key/mouse remappings.
        let _=registrar.register_setting(SettingSpec::choice("active_relic_slot",
            "options.rustpg.active_relic_slot",[
                "options.rustpg.slot.head", "options.rustpg.slot.chest",
                "options.rustpg.slot.legs", "options.rustpg.slot.feet",
                "options.rustpg.slot.weapon", "options.rustpg.slot.ring",
                "options.rustpg.slot.neck", "options.rustpg.slot.charm"],5));
        let _=registrar.register_setting(SettingSpec::toggle("power_hud",
            "options.rustpg.power_hud",true));
        let _=registrar.register_setting(SettingSpec::range("hud_scale",
            "options.rustpg.hud_scale",0.75,1.5,0.25,1.0));
        let _=registrar.register_setting(SettingSpec::range("hud_y_offset",
            "options.rustpg.hud_y_offset",0.0,56.0,4.0,0.0));
        // O PayloadRegistrar<'_> referencia o evento de registro.
        // Preservar o evento como variavel evita E0716 (temporario descartado).
        let mut payload_event=registrar.payload_handlers();
        let mut payloads=payload_event.registrar("2");
        payloads.play_to_server(PayloadId("rustpg:action".into()),Arc::new(ServerActionHandler));
        payloads.play_to_client(PayloadId("rustpg:open".into()),Some(Arc::new(ClientOpenHandler)));
        payloads.play_to_client(PayloadId("rustpg:hit".into()),Some(Arc::new(ClientDamageHandler)));
        payloads.play_to_client(PayloadId("rustpg:mob_hp".into()),Some(Arc::new(ClientMobHandler)));
    }
    fn register_entity_renderers(&self,_context:&RegistrationContext,
        event:&mut mod_api::RegisterEntityRenderersEvent<'_>) {
        rustpg_native_entities::register_renderers(event);
    }
    // Runtime-selected datapack content, not compiled executable plugins.
    // Path: data/<namespace>/rustpg/professions/<name>.rustpg
    fn add_server_reload_listeners(&self,_host:&dyn mod_api::Host,
        event:&mut mod_api::AddServerReloadListenersEvent<'_>) {
        let id=mod_api::Identifier::parse("rustpg:profession_content");
        let folder=mod_api::FileToIdConverter::new("rustpg/professions",".rustpg");
        if let (Ok(id),Ok(folder))=(id,folder) {
            let listener=Arc::new(rustpg_profession_plugins::ProfessionPackReload {
                shared:profession_packs().clone(),
            });
            let _=event.listeners.add_listener(id,folder,listener);
        }
        // Executable *sandboxed* .rustpgx scripts from datapacks. They are
        // interpreted instructions, not arbitrary native dynamic libraries.
        let id=mod_api::Identifier::parse("rustpg:script_runtime");
        let folder=mod_api::FileToIdConverter::new("rustpg/behaviors",".rustpgx");
        if let (Ok(id),Ok(folder))=(id,folder) {
            let listener=Arc::new(rustpg_script_vm::ScriptReload {
                shared:scripted_packs().clone(),
            });
            let _=event.listeners.add_listener(id,folder,listener);
        }
    }
    fn register_commands(&self,_host:&mut dyn ServerHost,e:&mut RegisterCommandsEvent<'_>) {
        for usage in ["rustpg", "rustpg open", "rustpg help", "rustpg status", "rustpg debug",
            "rustpg rift", "rustpg rift status", "rustpg rift manifest", "rustpg rift enter", "rustpg rift leave",
            "rustpg power head", "rustpg power chest", "rustpg power legs", "rustpg power feet",
            "rustpg power weapon", "rustpg power ring", "rustpg power neck", "rustpg power charm"] {
            e.commands.push(Command::new(usage));
        }
    }
    fn run_mod_command(&self,host:&mut dyn ServerHost,e:&mut RunModCommandEvent<'_>)->Flow {
        if e.command.literals.first().map(String::as_str)!=Some("rustpg") {return Flow::Continue;}
        let sub=e.command.literals.get(1).map(String::as_str).unwrap_or("open");
        match sub {
            "help" => {
                let _=host.commands().feedback(Text::literal(
                    "RustPG: V abre o menu. /rustpg rift mostra Ressonância; /rustpg rift manifest cria uma Fenda quando chegar a 100. B/G = aparar/esquivar."));
            },
            "rift" => {
                if let CommandSource::Player { player,.. }=&e.source {
                    let uuid=player.uuid();
                    let action=e.command.literals.get(2).map(String::as_str).unwrap_or("status");
                    let Ok(p)=PROFILE.get(host,Holder::Player(uuid)) else{let _=host.commands().feedback(Text::literal("Perfil indisponível."));return Flow::Cancel;};
                    let result:Result<String,String>=match action {
                        "manifest"=>{let tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());manifest_rift(host,&mut rt,*player,&p,tick)},
                        "enter"=>{let tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());begin_rift(host,&mut rt,*player,p.clone(),tick)},
                        "leave"=>{
                            let session=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).rift_sessions.get(&uuid).cloned();
                            if let Some(mut session)=session {
                                let _=player.entity().set_velocity(host,[0.0,0.0,0.0]);
                                if player.entity().teleport_to_level(host,&session.origin_dimension,session.origin_pos,None){
                                    session.phase=rustpg_rifts::SessionPhase::Returning;session.last_action_tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;
                                    self.runtime.lock().unwrap_or_else(|er|er.into_inner()).rift_sessions.insert(uuid,session);
                                    Ok("Fenda abandonada. Nenhuma recompensa de conclusão foi concedida.".into())
                                }else{Err("não foi possível sair da instância".into())}
                            }else{Err("você não está em uma Fenda".into())}
                        },
                        _=>{
                            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                            let state=if let Some(s)=rt.rift_sessions.get(&uuid){format!("Dentro de uma Fenda Rank {} · sala {}/{}",s.plan.rank.name(),s.current_room+1,s.plan.rooms.len())}
                                else if let Some(portal)=rt.rift_portals.get(&uuid){format!("Fenda Rank {} manifestada",portal.rank.name())}
                                else if p.rift_resonance>=rustpg_rifts::RESONANCE_READY{"Pronta para manifestar".into()}else{"Acumulando Ressonância".into()};
                            Ok(format!("Ressonância {}/300 · {} · conclusões {}",p.rift_resonance,state,p.rift_clears))
                        }
                    };
                    let _=host.commands().feedback(Text::literal(result.unwrap_or_else(|e|format!("Fenda: {e}"))));
                }
            },
            "power" => {
                let slot=e.command.literals.get(2).map(String::as_str).unwrap_or("");
                if let CommandSource::Player { player,.. }=&e.source {
                    let uuid=player.uuid();
                    let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{let _=host.commands().feedback(Text::literal("Perfil indisponível."));return Flow::Cancel;};
                    let message=if let Ok(line)=activate_relic(&mut p,slot) {
                        if PROFILE.set(host,Holder::Player(uuid),p).is_ok() {line}
                        else {"Falha ao salvar ativação".into()}
                    }else {"Falha: relíquia ausente, sem stamina ou em recarga".into()};
                    let _=host.commands().feedback(Text::literal(message));
                }
            },
            "status" | "debug" => {
                if let CommandSource::Player { player, .. } = &e.source {
                    let uuid=player.uuid();
                    let p=PROFILE.get_existing(host,Holder::Player(uuid)).ok().flatten();
                    let msg=match p {
                        Some(p)=>format!("RustPG {VERSION_STR}: nivel {}, XP {}, pontos {}, habilidades {}, relíquias {}. Perfil carregado.",
                            p.level,p.xp,p.points,p.ranks.len(),p.gear.items()),
                        None=>format!("RustPG {VERSION_STR}: perfil ainda nao sincronizado; tente novamente em alguns segundos."),
                    };
                    let _=host.commands().feedback(Text::literal(msg));
                } else {
                    let _=host.commands().feedback(Text::literal("RustPG ativo no servidor. Para consultar o personagem, use o comando dentro do jogo."));
                }
                if sub=="debug" {
                    let tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;
                    let _=host.commands().feedback(Text::literal(format!(
                        "Diagnostico: servidor ativo, tick={tick}, perfil em attachment, motor de relíquias ABI {} com {} famílias, pacotes profissões: {} ({}) / falhas {}. V abre; H ativa relíquia; dano e cura são autoritativos.",
                        rustpg_relic_engine::RELIC_ABI,relics().count(),profession_packs().read().unwrap_or_else(|e|e.into_inner()).pack_count(),
                        profession_packs().read().unwrap_or_else(|e|e.into_inner()).rule_count(),
                        profession_packs().read().unwrap_or_else(|e|e.into_inner()).errors().len())));
                    let vm=scripted_packs().read().unwrap_or_else(|e|e.into_inner());
                    let _=host.commands().feedback(Text::literal(format!(
                        "Script VM ABI {}: {} programas executaveis confinados; {} falhas de validacao.",
                        rustpg_script_vm::ABI,vm.programs.len(),vm.errors.len())));
                    if let Some(err)=vm.errors.first() {
                        let _=host.commands().feedback(Text::literal(format!("Script rejeitado: {err}")));
                    }
                    drop(vm);
                    let pack_error=profession_packs().read()
                        .unwrap_or_else(|e|e.into_inner()).errors().first().cloned();
                    if let Some(err)=pack_error {
                        let _=host.commands().feedback(Text::literal(format!(
                            "Erro em pacote de profissoes: {err}")));
                    }
                }
            },
            _ => {
                if let CommandSource::Player { player, .. } = &e.source {
                    match player.send_payload(host,&PayloadId("rustpg:open".into()),
                        PayloadBytes(b"open".to_vec())) {
                        Ok(_)=>{let _=host.commands().feedback(Text::literal("RustPG: solicitando abertura do menu no cliente..."));},
                        Err(err)=>{let _=host.commands().feedback(Text::literal(format!(
                            "RustPG: cliente nao recebeu o pedido ({err:?}). Confira se o mod esta instalado nos dois lados.")));},
                    }
                } else {
                    let _=host.commands().feedback(Text::literal("Use /rustpg no chat do jogador, nao no console."));
                }
            },
        }
        Flow::Cancel
    }
    fn player_logged_in(&self, host:&mut dyn ServerHost, e:&mut PlayerLoggedInEvent<'_>) {
        let Ok(p)=PROFILE.get(host,Holder::Player(e.player)) else{
            if let Some(player)=host.player(e.player) {
                let _=player.action_bar(host,Text::literal("Perfil RustPG indisponível. Dados existentes preservados."));
            }
            return;
        };
        sync_player_stats(e.player,&p.node_stats());
        if PROFILE.set(host,Holder::Player(e.player),p).is_err(){
            if let Some(player)=host.player(e.player) {
                let _=player.action_bar(host,Text::literal("Falha ao salvar perfil RustPG."));
            }
            return;
        }
        if let Some(player)=host.player(e.player) {
            let _=player.action_bar(host,Text::literal("RustPG: V abre o menu; /rustpg debug verifica a conexão."));
        }
    }
    // Vanilla CriticalHitEvent is authoritative; the event provides the final
    // decision after other listeners. We store only a short-lived flag for the
    // attacking player and correlate it with the final damage post-hook.
    fn critical_hit(&self,host:&mut dyn ServerHost,e:&mut mod_api::CriticalHitEvent<'_>) {
        let Some(player)=host.players().acting_player() else{return;};
        let dim=player.entity().level(host).map(|lvl|lvl.dimension_id().to_owned())
            .unwrap_or_else(||host.dimension().to_owned());
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let now=rt.tick;
        rt.critical_hits.insert((dim,player.entity().id()),(now,e.critical));
        if rt.critical_hits.len()>256 {rt.critical_hits.retain(|_,(at,_)|now.saturating_sub(*at)<=3);}
    }
    // Orb XP is a genuine mutable server event. Vanilla Mending and the
    // actual experience award consume the adjusted orb; no visual-only bonus.
    fn pickup_xp(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::PickupXpEvent<'_>)->Flow {
        if let Ok(p)=PROFILE.get(host,Holder::Player(e.player)) {
            if p.rank("general.orb_xp#1")>0 ||
                p.ranks.keys().any(|id|id.starts_with("general.orb_xp#")) {
                let bonus=p.node_stats().get("general.orb_xp");
                e.value=rustpg_node_effects::orb_value(e.value,bonus);
            }
        }
        Flow::Continue
    }
    // Food loss is authoritative: no persistent effect if no actual hunger
    // change. Upward changes (eating) are never reduced or multiplied.
    fn food_level_changed(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::FoodLevelChangedEvent<'_>)->Flow {
        if e.to>=e.from {return Flow::Continue;}
        let Some(pl)=host.player(e.player) else{return Flow::Continue;};
        if pl.is_creative(host).unwrap_or(false){return Flow::Continue;}
        let Ok(p)=PROFILE.get(host,Holder::Player(e.player))else{return Flow::Continue;};
        let reduction=p.node_stats().get("survival.hunger");
        if reduction>0.0 {
            let tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;
            let seed=(e.player as u64).wrapping_add(tick.rotate_left(17));
            e.to=rustpg_node_effects::food_after(e.from,e.to,reduction,seed);
        }
        Flow::Continue
    }
    // Harmful-effect resistance uses the actual addEffect applicability decision.
    // Beneficial and neutral effects are never blocked.
    fn mob_effect_applicable(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::MobEffectApplicableEvent)->Option<mod_api::EffectApplicable> {
        if e.effect.category(host)!=Some(mod_api::EffectCategory::Harmful) {return None;}
        let entity=e.entity.entity().id();
        let player=host.players().server_players().into_iter()
            .find(|p|p.entity().id()==entity)?;
        let p=PROFILE.get(host,Holder::Player(player.uuid())).ok()?;
        let resist=p.node_stats().get("defense.res_debuff").clamp(0.0,0.90);
        if resist<=0.0 {return None;}
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let rng=rt.rng.get_or_insert_with(||rustpg_num::Rng(0xDEB0_FF55));
        rng.chance(resist).then_some(mod_api::EffectApplicable::Deny)
    }
    // Mining speed hook fires on BOTH client and server: client preview and
    // authoritative server calculation share one 1/3 multiplier.
    fn player_break_speed(&self,host:mod_api::HostSide<'_>,
        e:&mut mod_api::PlayerBreakSpeedEvent)->Flow {
        let (exhausted,relic_bonus,mining_speed,tool_eff)=match host {
            mod_api::HostSide::Server(server) => {
                let key=(server.dimension().to_owned(),e.player);
                let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                let stats=rt.attribute_snapshots.get(&key);
                (rt.exhausted_entities.contains(&key),
                    rt.relic_snapshots.get(&key).copied().unwrap_or_default(),
                    stats.map_or(0.0,|s|s.get("mining.speed")),
                    stats.map_or(0.0,|s|s.get("mining.tool_eff")))
            },
            mod_api::HostSide::Client(client) => {
                let local=client.local_player().map(|p|p.entity().id());
                let ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
                if local==Some(e.player) {
                    let stats=ui.profile.as_ref().map(|p|p.node_stats());
                    (ui.profile.as_ref().is_some_and(|p|p.stamina.exhausted),
                     ui.profile.as_ref().map(|p|relic_snapshot(p,now_seconds())).unwrap_or_default(),
                     stats.as_ref().map_or(0.0,|s|s.get("mining.speed")),
                     stats.as_ref().map_or(0.0,|s|s.get("mining.tool_eff")))
                }else{(false,Default::default(),0.0,0.0)}
            },
        };
        let speed=rustpg_v4_fatigue::adjusted_break_speed(e.new_speed,exhausted);
        let node_speed=rustpg_java_port::scaled_break_speed(speed,mining_speed,tool_eff);
        e.new_speed=relic_bonus.mining_speed(node_speed,exhausted);
        Flow::Continue
    }
    // Bow/crossbow launch identity: Horizon reports the projectile after it
    // entered the level. Store the actual selected weapon and a server profile
    // snapshot now, so switching hotbar slots before impact cannot change perks.
    fn entity_shoot_bow(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::EntityShootBowEvent<'_>) {
        let Some(pl)=host.player(e.player) else{return;};
        if pl.is_creative(host).unwrap_or(false) || pl.is_spectator(host).unwrap_or(false){return;}
        let weapon=pl.selected_slot(host).zip(pl.inventory_slots(host))
            .and_then(|(slot,contents)|contents.iter().find(|(i,_)|*i==slot as u32)
                .and_then(|(_,stack)|rustpg_weapon_training::ranged_weapon(host,stack.item())));
        let Some(weapon)=weapon.filter(|w|matches!(*w,"bow"|"crossbow")) else{return;};
        let Ok(profile)=PROFILE.get(host,Holder::Player(e.player)) else{return;};
        let dimension=pl.entity().level(host).map(|l|l.dimension_id().to_owned())
            .unwrap_or_else(||host.dimension().to_owned());
        let attacker=pl.entity().id();
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let now=rt.tick;
        rt.projectile_weapons.insert((dimension,e.projectile),
            (weapon,attacker,profile.node_stats(),now));
        if rt.projectile_weapons.len()>1024 {
            rt.projectile_weapons.retain(|_,(_,_,_,at)|now.saturating_sub(*at)<=1200);
        }
    }

    // Spending stamina on a real player attack (not a client-side animation).
    // Kept separate from the damage hook: each only mutates its own resource.
    fn attack_entity(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::AttackEntityEvent)->Flow {
        let Some(pl)=host.players().acting_player() else {return Flow::Continue;};
        if pl.is_creative(host).unwrap_or(false) || pl.is_spectator(host).unwrap_or(false) {
            return Flow::Continue;
        }
        // Source-based kill events cannot tell axes, maces and spears apart.
        // Snapshot selected hotbar item when the server accepts the attack.
        // An absent inventory is unknown; an empty confirmed slot is unarmed.
        let held=pl.selected_slot(host).zip(pl.inventory_slots(host))
            .and_then(|(slot,contents)|contents.into_iter()
                .find(|(i,_)|*i==slot as u32).map(|(_,stack)|stack));
        let weapon=rustpg_weapon_training::from_held(host,held.as_ref().map(|stack|stack.item()));
        let sharpness_overflow=held.as_ref().map_or(0,|stack|
            rustpg_enchant_stack::overflow_level(stack,"minecraft:sharpness"));
        let uuid=pl.uuid();
        let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{return Flow::Cancel;};
        let relic_bonus=relic_snapshot(&p,now_seconds());
        let node_stats=p.node_stats();
        let dim=pl.entity().level(host).map(|lvl|lvl.dimension_id().to_owned())
            .unwrap_or_else(||host.dimension().to_owned());
        let action_ticket=if let Some(discipline)=weapon {
            let mastery=(p.prof(discipline).level as f64/100.0).clamp(0.0,1.0);
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let tick=rt.tick;
            let state=rt.action_combat.entry((dim.clone(),pl.entity().id())).or_default();
            match state.attempt(discipline,tick,mastery) {
                Ok(ticket)=>{if rustpg_guard::take_counter(&dim,pl.entity().id()){state.boost_last(1.20);}Some(ticket)},
                Err(_)=>return Flow::Cancel,
            }
        }else{None};
        let efficiency=node_stats.get("general.stamina_efficiency").clamp(0.0,0.90);
        let heavy=weapon.is_some_and(|w|matches!(w,"axe"|"mace"|"spear"));
        let heavy_reduction=if heavy {
            node_stats.get("general.heavy_attack_cost_reduction").clamp(0.0,0.90)
        }else{0.0};
        let action_cost=action_ticket.map_or(1.0,|t|t.stamina_mult);
        let attack_cost=relic_bonus.attack_cost(rustpg_v4_fatigue::NORMAL_ATTACK_COST)
            *(1.0-efficiency)*(1.0-heavy_reduction)*action_cost;
        // Authoritative gate: failed/empty stamina never permits a free hit.
        // Do not register melee snapshots until the engine has paid the cost.
        if !p.stamina.consume(attack_cost.max(0.05)) {
            let _=PROFILE.set(host,Holder::Player(uuid),p);
            return Flow::Cancel;
        }
        let exhausted=p.stamina.exhausted;
        let _=PROFILE.set(host,Holder::Player(uuid),p);
        // The Starling attack event exposes an i32, while combat cache keys use u32.
        // Ignore negative entity identifiers without panicking.
        if let (Some(discipline), Ok(target_id))=(weapon,u32::try_from(e.entity)) {
            let valid_target=pl.entity().position(host).zip(host.entity(target_id)
                .and_then(|ent|ent.position(host)))
                .is_some_and(|(a,b)|rustpg_hitboxes::within_melee_reach(a,b,6.0));
            if valid_target {
                let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                if rt.recent_melee.len()>512 {rt.recent_melee.clear();}
                let tick=rt.tick;
                let key=(host.dimension().to_owned(),target_id,pl.entity().id());
                rt.recent_melee.insert(key.clone(),(discipline,tick));
                if rt.melee_sharpness_overflow.len()>512 {
                    rt.melee_sharpness_overflow.clear();
                }
                if sharpness_overflow>0 {
                    rt.melee_sharpness_overflow.insert(key,(sharpness_overflow,tick));
                }else{
                    rt.melee_sharpness_overflow.remove(&key);
                }
            }
        }

        let key=(dim,pl.entity().id());
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        if exhausted {rt.exhausted_entities.insert(key);}
        else {rt.exhausted_entities.remove(&key);}
        Flow::Continue
    }
    // Survival/exploration nodes are applied inside Horizon's actual
    // breathing, drowning and landing funnels. Server snapshots are the source
    // of truth; the client path is prediction only for living_fall.
    fn living_breathe(&self,context:&dyn mod_api::LoopContext,
        e:&mut mod_api::LivingBreatheEvent<'_>) {
        if e.kind!="minecraft:player" || !matches!(context.side(),mod_api::LoopSide::Server){return;}
        let bonus={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.entity_dimensions.get(&e.entity).and_then(|dim|
                rt.attribute_snapshots.get(&(dim.clone(),e.entity)))
                .map_or(0.0,|s|s.get("survival.drown")).clamp(0.0,0.95)
        };
        if bonus<=0.0{return;}
        if !e.can_breathe && e.consume>0 {
            let tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;
            let roll=rustpg_num::unit(rustpg_num::mix(
                tick ^ (u64::from(e.entity)<<32) ^ 0xB8EA_7E));
            if roll<bonus {e.consume=0;}
        } else if e.can_breathe && e.refill>0 {
            e.refill=(f64::from(e.refill)*(1.0+bonus))
                .round().clamp(0.0,i32::MAX as f64) as i32;
        }
    }
    fn living_drown(&self,context:&dyn mod_api::LoopContext,
        e:&mut mod_api::LivingDrownEvent<'_>)->Flow {
        if e.kind!="minecraft:player" || !matches!(context.side(),mod_api::LoopSide::Server){
            return Flow::Continue;
        }
        let bonus={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.entity_dimensions.get(&e.entity).and_then(|dim|
                rt.attribute_snapshots.get(&(dim.clone(),e.entity)))
                .map_or(0.0,|s|s.get("survival.drown")).clamp(0.0,0.95)
        };
        if bonus>0.0 && e.damage_amount.is_finite() && e.damage_amount>0.0 {
            e.damage_amount=(f64::from(e.damage_amount)*(1.0-bonus)).max(0.0) as f32;
        }
        Flow::Continue
    }
    fn living_fall(&self,context:&dyn mod_api::LoopContext,
        e:&mut mod_api::LivingFallEvent<'_>)->Flow {
        if e.kind!="minecraft:player" {return Flow::Continue;}
        let server_bonus={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.entity_dimensions.get(&e.entity).and_then(|dim|
                rt.attribute_snapshots.get(&(dim.clone(),e.entity)))
                .map(|s|s.get("exploration.fall_save"))
        };
        let bonus=server_bonus.unwrap_or_else(||{
            if !matches!(context.side(),mod_api::LoopSide::Client){return 0.0;}
            self.ui.lock().unwrap_or_else(|er|er.into_inner()).profile.as_ref()
                .map_or(0.0,|p|p.node_stats().get("exploration.fall_save"))
        }).clamp(0.0,0.90);
        if bonus>0.0 && e.distance.is_finite() && e.distance>0.0 {
            e.distance*=1.0-bonus;
            e.multiplier=(f64::from(e.multiplier.max(0.0))*(1.0-bonus))
                .clamp(0.0,f32::MAX as f64) as f32;
        }
        Flow::Continue
    }

    // Puffish-style healing attribute, now backed by Starling 0.9's real
    // LivingHeal hook. The same deterministic multiplier is used on both
    // sides; the server snapshot wins, while the client uses the synchronized
    // profile for prediction when it has no server-side entity cache.
    fn living_heal(&self,context:&dyn mod_api::LoopContext,
        e:&mut mod_api::LivingHealEvent)->Flow {
        if !e.amount.is_finite() || e.amount<=0.0 {return Flow::Continue;}
        let entity=e.entity;
        let server_bonus={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.entity_dimensions.get(&entity).and_then(|dim|
                rt.attribute_snapshots.get(&(dim.clone(),entity)))
                .map(|stats|(stats.get("general.healing_bonus"),
                              stats.get("defense.healing_received")))
        };
        // No server fallback to the local UI profile: it belongs to a
        // different player and could accidentally boost a mob's heal.
        let (general,received)=server_bonus.unwrap_or_else(||{
            if !matches!(context.side(),mod_api::LoopSide::Client){return (0.0,0.0);}
            self.ui.lock().unwrap_or_else(|er|er.into_inner()).profile.as_ref()
                .map_or((0.0,0.0),|p|{
                    let stats=p.node_stats();
                    (stats.get("general.healing_bonus"),stats.get("defense.healing_received"))
                })
        });
        // These are two independently bought nodes; multiplicative ordering
        // is fixed and both sides apply the same math to real heal events.
        e.amount=(f64::from(e.amount)*(1.0+general.clamp(0.0,3.0))*
            (1.0+received.clamp(0.0,3.0))).clamp(0.0,1_000_000.0) as f32;
        Flow::Continue
    }

    // Starling's jump-power hook runs before the vanilla velocity write and
    // on replayed client ticks too. This replaces the old catalogue-only
    // Puffish jump node with a real gameplay effect.
    fn living_jump_power(&self,context:&dyn mod_api::LoopContext,
        e:&mut mod_api::LivingJumpPowerEvent<'_>) {
        if e.kind!="minecraft:player" || !e.power.is_finite() || e.power<=0.0 {return;}
        let entity=e.entity;
        let server_bonus={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.entity_dimensions.get(&entity).and_then(|dim|
                rt.attribute_snapshots.get(&(dim.clone(),entity)))
                .map(|stats|stats.get("general.jump_bonus"))
        };
        let bonus=server_bonus.unwrap_or_else(||{
            if !matches!(context.side(),mod_api::LoopSide::Client){return 0.0;}
            self.ui.lock().unwrap_or_else(|er|er.into_inner()).profile.as_ref()
                .map_or(0.0,|p|p.node_stats().get("general.jump_bonus"))
        }).clamp(0.0,1.5);
        e.power=(f64::from(e.power)*(1.0+bonus)).clamp(0.0,32.0) as f32;
    }

    // Damage calculated AFTER armor/shields, BEFORE health is subtracted.
    // No extra hit, no client-side guessing: reduce the one genuine hit by 25%.
    fn living_damage_pre(&self,_host:&mut dyn mod_api::LevelRandomHost,
        e:&mut mod_api::LivingDamagePreEvent<'_>) {
        let attacker=u32::try_from(e.source.causing).ok();
        let victim=u32::try_from(e.entity).ok();
        let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        // The level-only callback cannot query ConfigHost::dimension(). Use
        // the dimension recorded when the victim entered the server level.
        // Never guess on an unknown ID: skip RPG modifiers, keep vanilla hit.
        let dimension=victim.and_then(|id|rt.entity_dimensions.get(&id)).cloned();
        let attacker=attacker.filter(|id|dimension.as_ref().is_some_and(|dim|
            rt.entity_dimensions.get(id)==Some(dim)));
        let exhausted=attacker.is_some_and(|id|dimension.as_ref().is_some_and(|dim|
            rt.exhausted_entities.contains(&(dim.clone(),id))));
        let attack=attacker.and_then(|id|dimension.as_ref().and_then(|dim|
            rt.relic_snapshots.get(&(dim.clone(),id)).copied())).unwrap_or_default();
        let defense=victim.and_then(|id|dimension.as_ref().and_then(|dim|
            rt.relic_snapshots.get(&(dim.clone(),id)).copied())).unwrap_or_default();
        // Better Combat-style bounded combos. A server-confirmed attack_entity
        // record is required; projectiles, magic and ambient DoT cannot stack.
        let damage_now=e.container.new_damage();
        let accepted_melee=damage_now.is_finite() && damage_now>0.0
            && !matches!(e.source.damage_type.rsplit(':').next().unwrap_or(""),
                "arrow"|"trident"|"fireball"|"indirect_magic"|"magic"|"thorns"|
                "explosion"|"fall"|"on_fire"|"in_fire"|"lava"|"wither");
        let pending_combo=if accepted_melee {
            if let (Some(a),Some(v),Some(ref dim))=(attacker,victim,dimension.as_ref()) {
                rt.recent_melee.get(&(dim.to_string(),v,a))
                    .and_then(|(weapon,at)| (rt.tick.saturating_sub(*at)<=2)
                        .then(|| (a,v,dim.clone(),*weapon,rt.tick)))
            } else {None}
        }else{None};
        // A ranged hit is tied to the exact launched entity, not to whatever
        // weapon happens to be in the player's hand when the arrow lands.
        let direct=u32::try_from(e.source.direct).ok();
        let projectile_weapon=direct.and_then(|direct|dimension.as_ref().and_then(|dim|
                rt.projectile_weapons.get(&(dim.clone(),direct))))
            .and_then(|(weapon,owner,stats,at)|{
                (attacker==Some(*owner) && rt.tick.saturating_sub(*at)<=1200)
                    .then(||(*weapon,stats.clone()))
            });
        let critical_attack=attacker.and_then(|a|dimension.as_ref().map(|d|(d,a)))
            .and_then(|(d,a)|rt.critical_hits.get(&(d.clone(),a)))
            .is_some_and(|(at,critical)|*critical && rt.tick.saturating_sub(*at)<=2);
        // Only a recent, stamina-paid melee attack may carry extra levels.
        // This snapshot is safe even if the player switched hotbar slots.
        let action_multiplier=if accepted_melee {
            match (attacker,dimension.as_ref()) {
                (Some(a),Some(dim))=>rt.action_combat.get(&(dim.clone(),a))
                    .map_or(1.0,|state|state.damage_multiplier(rt.tick)),
                _=>1.0,
            }
        }else{1.0};
        let overflow_sharpness=if pending_combo.is_some() {
            match (attacker,victim,dimension.as_ref()) {
                (Some(a),Some(v),Some(dim))=>rt.melee_sharpness_overflow
                    .get(&(dim.clone(),v,a))
                    .filter(|(_,seen)|rt.tick.saturating_sub(*seen)<=2)
                    .map_or(0,|(levels,_)|*levels),
                _=>0,
            }
        }else{0};
        // Snapshot before releasing the Runtime mutex. The callback never
        // reaches into inventory or levels in a LevelRandomHost-only context.
        drop(rt);
        let confirmed_melee=pending_combo.is_some();
        let confirmed_weapon=projectile_weapon.as_ref().map(|(weapon,_)|*weapon)
            .or_else(||pending_combo.as_ref().map(|(_,_,_,weapon,_)|*weapon));
        let bonus=if let Some((a,v,dim,weapon,tick))=pending_combo {
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let points=rt.attribute_snapshots.get(&(dim.clone(),a))
                .map_or(0.0,|s|s.get(&format!("weapon.{weapon}.combo_damage")));
            rt.combo.landed(&dim,a,v,tick,points).1
        } else {1.0};
        let before=e.container.new_damage();
        // Imported Puffish combat attributes act on server-confirmed damage.
        // Melee requires the existing verified attack_entity record. Ranged
        // damage is restricted to known ranged source types, not arbitrary DoT.
        let reference_bonus=if let Some(ref dim)=dimension {
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let attacker_multiplier=attacker.and_then(|a|rt.attribute_snapshots.get(&(dim.clone(),a)))
                .map_or(1.0,|stats| {
                    let ranged=matches!(e.source.damage_type.rsplit(':').next().unwrap_or(""),
                        "arrow"|"trident"|"mob_projectile");
                    if ranged {
                        1.0+stats.get("general.ranged_damage").clamp(0.0,2.0)
                    }else if confirmed_melee {
                        1.0+stats.get("general.melee_damage").clamp(0.0,2.0)
                    }else{1.0}
                });
            let defense_multiplier=victim.and_then(|v|rt.attribute_snapshots.get(&(dim.clone(),v)))
                .map_or(1.0,|stats| 1.0-stats.get("general.resistance_all").clamp(0.0,0.75));
            (attacker_multiplier*defense_multiplier) as f32
        }else{1.0};
        let (mob_attack,mob_defense)=if let Some(ref dim)=dimension {
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            (attacker.and_then(|id|rt.elite_combat.get(&(dim.clone(),id)).copied()),
             victim.and_then(|id|rt.elite_combat.get(&(dim.clone(),id)).copied()))
        }else{(None,None)};
        let defense_reaction=if let (Some(v),Some(ref dim))=(victim,dimension.as_ref()) {
            rustpg_guard::on_hit(dim,v,e.source.damage_type)
        } else {1.0};
        let resolved=combat_pipeline().resolve(rustpg_combat_pipeline::CombatContext{
            base_damage:before,attacker_exhausted:exhausted,attacker:attack,defender:defense});
        if resolved.is_finite(){
            let powers=if let Some(ref dim)=dimension {
                let atk=attacker.filter(|_|confirmed_melee).map_or(1.0,|id|rustpg_powers::outgoing(dim,id));
                let def=victim.map_or(1.0,|id|rustpg_powers::incoming(dim,id));
                atk*def
            }else{1.0};
            let with_elites=rustpg_elite_encounters::resolve(
                resolved*bonus*reference_bonus*powers*action_multiplier,
                mob_attack,mob_defense,e.source.damage_type);
            // Real independent node effects. Only an authoritative melee
            // attempt can select weapon modifiers; typed defenses apply to
            // the actual player's post-armor incoming damage, never twice.
            let (attack_stats,defense_stats,target_kind,now)={
                let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                let attack_stats=projectile_weapon.as_ref().map(|(_,stats)|stats.clone())
                    .or_else(||attacker.and_then(|a|dimension.as_ref().and_then(|d|
                        rt.attribute_snapshots.get(&(d.clone(),a)).cloned())));
                let defense_stats=victim.and_then(|v|dimension.as_ref().and_then(|d|
                    rt.attribute_snapshots.get(&(d.clone(),v)).cloned()));
                let target_kind=victim.and_then(|v|dimension.as_ref().and_then(|d|
                    rt.entity_kinds.get(&(d.clone(),v)).cloned())).unwrap_or_default();
                (attack_stats,defense_stats,target_kind,rt.tick)
            };
            let seed=now.rotate_left(17) ^
                (u64::from(attacker.unwrap_or(0))<<32) ^ u64::from(victim.unwrap_or(0));
            let computed=rustpg_node_effects::damage_after_armor(
                (with_elites*defense_reaction).max(0.0),
                e.container.reduction(mod_api::Reduction::Armor),
                confirmed_weapon.and_then(|w|attack_stats.as_ref().map(|s|(s,w))),
                defense_stats.as_ref(),
                e.source.damage_type,
                &target_kind,
                mob_defense.map(|s|s.tier),
                mob_attack.map(|s|s.tier),
                mob_defense.map(|s|s.health_fraction),
                critical_attack,
                seed);
            // Vanilla's get_enchantment_level sees up to 255. Preserve its
            // ordinary Sharpness damage, then add only the stored overflow.
            let overflowed=if confirmed_melee {
                rustpg_enchant_stack::sharpness_overflow_damage(computed,overflow_sharpness)
            }else{computed};
            e.container.set_new_damage(overflowed.max(0.0));
        }
    }
    fn living_incoming_damage(&self,_host:&mut dyn mod_api::LevelRandomHost,
        e:&mut mod_api::LivingIncomingDamageEvent<'_>)->Flow {
        if e.kind!="minecraft:player" {
            if let Ok(entity)=u32::try_from(e.entity) {
                let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                if rt.impact_positions.len()>=512 {rt.impact_positions.clear();}
                if let Some(dim)=rt.entity_dimensions.get(&entity).cloned() {
                    rt.impact_positions.insert((dim,entity),e.victim.position);
                }
            }
        }
        Flow::Continue
    }
    // Fired after armor, enchantment, absorption and shield reductions.
    // The underlying Starling generator takes event fields from the native
    // HitLevel::native_living_damage_post signature, not from the short catalog
    // declaration. Never change damage here, only enqueue final health loss.
    fn living_damage_post(&self,_host:&mut dyn mod_api::LevelRandomHost,
        e:&mut mod_api::LivingDamagePostEvent<'_>) {
        let Ok(entity)=u32::try_from(e.entity) else {return;};
        let attacker=u32::try_from(e.source.causing).ok();
        let actual=e.container.health_damage();
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let Some(dimension)=rt.entity_dimensions.get(&entity).cloned() else {return;};
        if e.kind=="minecraft:player" && actual.is_finite() && actual>0.0 {
            let now=rt.tick;
            rt.last_damage_tick.insert((dimension.clone(),entity),now);
        }
        // Bleed is tied to the exact weapon that produced this confirmed hit.
        // Delayed bleed uses generic damage and has no recent weapon record,
        // so it cannot recursively apply itself.
        if actual.is_finite() && actual>0.0 {
            if let Some(attacker)=attacker.filter(|id|*id!=entity &&
                rt.entity_dimensions.get(id)==Some(&dimension)) {
                let melee=rt.recent_melee.get(&(dimension.clone(),entity,attacker))
                    .and_then(|(weapon,seen)|(rt.tick.saturating_sub(*seen)<=2)
                        .then(||(*weapon,rt.attribute_snapshots
                            .get(&(dimension.clone(),attacker)).cloned().unwrap_or_default())));
                let direct=u32::try_from(e.source.direct).ok();
                let ranged=direct.and_then(|direct|rt.projectile_weapons
                    .get(&(dimension.clone(),direct))
                    .and_then(|(weapon,owner,stats,seen)|(*owner==attacker &&
                        rt.tick.saturating_sub(*seen)<=1200)
                        .then(||(*weapon,stats.clone()))));
                if let Some((weapon,stats))=ranged.or(melee) {
                    let seed=rt.tick.rotate_left(29)^((u64::from(attacker))<<32)
                        ^u64::from(entity)^u64::from(e.source.direct.max(0) as u32);
                    if let Some((damage,remaining))=
                        rustpg_node_effects::bleed_plan(&stats,weapon,seed) {
                        if rt.pending_bleeds.len()<256 {
                            let next_tick=rt.tick.saturating_add(20);
                            rt.pending_bleeds.push(PendingBleed{
                                dimension:dimension.clone(),victim:entity,attacker,
                                damage,remaining,next_tick,
                            });
                        }
                    }
                }
            }
        }
        // Use final HP damage (after armor/absorption) for life steal. No heals
        // from friendly fire, absorbed hits or self-damage; the server tick applies it.
        if e.kind!="minecraft:player" {
            if let Some(attacker)=attacker.filter(|id|*id!=entity &&
                rt.entity_dimensions.get(id)==Some(&dimension)) {
                let relic_heal=rt.relic_snapshots.get(&(dimension.clone(),attacker))
                    .copied().unwrap_or_default().heal_from_hit(actual);
                // Only a recent *verified melee attempt* can trigger weapon
                // lifesteal; ranged/DoT/projectiles need a launch-time snapshot.
                let melee_kind=rt.recent_melee.get(&(dimension.clone(),entity,attacker))
                    .and_then(|(weapon,tick)|
                        (rt.tick.saturating_sub(*tick)<=2).then_some(*weapon));
                let node_heal=melee_kind.and_then(|weapon|rt.attribute_snapshots
                    .get(&(dimension.clone(),attacker)).map(|stats|
                        rustpg_java_port::life_steal(actual,
                            stats.get(&format!("weapon.{weapon}.lifesteal")))))
                    .unwrap_or(0.0);
                let power_heal=if melee_kind.is_some(){rustpg_powers::siphon(&dimension,attacker,actual)}else{0.0};
                let heal=(relic_heal+node_heal+power_heal).min(2048.0);
                if heal>0.0 && rt.pending_relic_heals.len()<128 {
                    rt.pending_relic_heals.push((dimension.clone(),attacker,heal));
                }
            }
        }
        if e.kind=="minecraft:player" {return;}
        // Successful dealt damage, including hits without a damage popup.
        if actual.is_finite() && actual>0.0 && actual<=100_000.0 {
            if let Some(id)=attacker.filter(|id|*id!=entity &&
                rt.entity_dimensions.get(id)==Some(&dimension)) {
                if rt.pending_bestiary_hits.len()<256 && e.kind.contains(':') {
                    rt.pending_bestiary_hits.push((dimension.clone(),id,e.kind.to_owned(),actual as f64));
                }
            }
        }
        let Some(hit)=rustpg_v4_wire::DamageHit::new(entity,actual) else {return;};
        let Some(at)=rt.impact_positions.remove(&(dimension.clone(),entity)) else {return;};
        let at=[at[0],at[1]+1.45,at[2]];
        let attacker=e.source.causing.max(0);
        let mut family=rustpg_damage_meter::DamageFamily::classify(e.source.damage_type);
        if family==rustpg_damage_meter::DamageFamily::Physical {
            if let Some(attacker)=u32::try_from(e.source.causing).ok() {
                if rt.critical_hits.get(&(dimension.clone(),attacker))
                    .is_some_and(|(at,is_critical)|*is_critical && rt.tick.saturating_sub(*at)<=2){
                    family=rustpg_damage_meter::DamageFamily::Critical;
                }
            }
        }
        if let Some(notice)=rustpg_v4_wire::DamageNotice::new(&dimension,hit,at,attacker,family) {
            if rt.hits.len()<256 {rt.hits.push(notice);}
        }
    }
    // Authoritative server tick: actually modifies vanilla movement/health.
    // Client profile is a synced attachment, not a client-invented gameplay value.
    fn server_tick(&self,host:&mut dyn ServerHost,e:&mut ServerTickEvent<'_>) {
        if !matches!(e.phase,ServerTickPhase::ServerTickEnd{..}) {return;}
        let tick={let mut st=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            st.tick=st.tick.wrapping_add(1);st.tick};
        tick_rifts(self,host,tick);
        rustpg_guard::set_tick(tick);
        rustpg_powers::cleanup(tick);
        // Resolve registry identities once through ServerHost. Read-only
        // LevelHost hooks can then classify exact blocks and create bonus
        // stacks without calling APIs intentionally absent from LevelHost.
        let need_registry_cache={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.resource_block_ids.is_empty() || rt.cached_item_keys.is_empty()
        };
        if need_registry_cache {
            let mut blocks=HashMap::new();
            for id in rustpg_luck::RESOURCE_BLOCK_IDS {
                if let Some(state)=host.block_state_id(id) {
                    blocks.entry(state.block()).or_insert_with(||(*id).to_owned());
                }
            }
            let mut mature=std::collections::HashSet::new();
            for id in rustpg_luck::MATURE_CROP_STATES {
                if let Some(state)=host.block_state_id(id){mature.insert(state);}
            }
            let mut items=HashMap::new();
            for id in ["minecraft:lapis_lazuli","minecraft:diamond","minecraft:golden_carrot",
                "minecraft:enchanted_golden_apple","minecraft:flint","minecraft:emerald",
                "minecraft:apple","minecraft:golden_apple","minecraft:pufferfish",
                "minecraft:cod","minecraft:heart_of_the_sea","minecraft:name_tag",
                "minecraft:wheat_seeds","minecraft:beetroot_seeds","minecraft:melon_seeds",
                "minecraft:pumpkin_seeds","minecraft:torchflower_seeds","minecraft:pitcher_pod",
                "minecraft:oak_sapling","minecraft:spruce_sapling","minecraft:birch_sapling",
                "minecraft:jungle_sapling","minecraft:acacia_sapling","minecraft:dark_oak_sapling",
                "minecraft:mangrove_propagule","minecraft:cherry_sapling","minecraft:pale_oak_sapling",
                "minecraft:clay_ball","minecraft:beef","minecraft:cooked_beef","minecraft:porkchop",
                "minecraft:cooked_porkchop","minecraft:mutton","minecraft:cooked_mutton",
                "minecraft:chicken","minecraft:cooked_chicken","minecraft:rabbit","minecraft:cooked_rabbit",
                "minecraft:leather","minecraft:rabbit_hide","minecraft:white_wool","minecraft:orange_wool",
                "minecraft:magenta_wool","minecraft:light_blue_wool","minecraft:yellow_wool",
                "minecraft:lime_wool","minecraft:pink_wool","minecraft:gray_wool",
                "minecraft:light_gray_wool","minecraft:cyan_wool","minecraft:purple_wool",
                "minecraft:blue_wool","minecraft:brown_wool","minecraft:green_wool",
                "minecraft:red_wool","minecraft:black_wool"] {
                if let Ok(parsed)=mod_api::Identifier::parse(id) {
                    if let Some(key)=host.item_key(&parsed){items.insert(id.to_owned(),key);}
                }
            }
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            if rt.resource_block_ids.is_empty(){rt.resource_block_ids=blocks;}
            if rt.mature_resource_states.is_empty(){rt.mature_resource_states=mature;}
            if rt.cached_item_keys.is_empty(){rt.cached_item_keys=items;}
        }
        if tick%20==0 {
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.recent_melee.retain(|_,(_,seen)|tick.saturating_sub(*seen)<=60);
            rt.recent_death_weapon.retain(|_,(_,seen)|tick.saturating_sub(*seen)<=20);
        }
        // Flush confirmed post-armor damage as best hit per species/player.
        // Only a valid online player in the same dimension earns this record.
        let pending={let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            std::mem::take(&mut rt.pending_bestiary_hits)};
        if !pending.is_empty() {
            let players=host.players().server_players();
            let mut grouped:HashMap<(u128,String),f64>=HashMap::new();
            for (dimension,attacker,species,damage) in pending {
                let Some(level)=host.level(&dimension) else {continue;};
                if let Some(pl)=players.iter().find(|p|p.entity().id()==attacker &&
                    p.entity().level(host)==Some(level)) {
                    let value=grouped.entry((pl.uuid(),species)).or_insert(0.0);
                    *value=value.max(damage);
                }
            }
            for ((uuid,species),damage) in grouped {
                let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{continue;};
                if (p.species.len()<512 || p.species.contains_key(&species)) && damage>0.0 {
                    let rec=p.species.entry(species.clone()).or_insert_with(||
                        rustpg_bestiary::SpeciesRecord{kind:species,..Default::default()});
                    if damage>rec.best_damage {
                        rec.best_damage=damage;
                        let _=PROFILE.set(host,Holder::Player(uuid),p);
                    }
                }
            }
        }
        // Near-range replication: send real final hits only to players in
        // range of the affected entity. The client must still resolve that
        // entity locally before displaying it (dimension and stale ID guard).
        let hits={
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            std::mem::take(&mut rt.hits)
        };
        if !hits.is_empty() {
            let players=host.players().server_players();
            for hit in hits {
                let pos=hit.at;
                let Some(event_level)=host.level(&hit.dimension) else {continue;};
                let payload=PayloadBytes(hit.encode());
                for pl in &players {
                    // Distinct worlds can share coordinates and entity IDs.
                    if pl.entity().level(host)!=Some(event_level) {continue;}
                    let Some(watching)=pl.entity().position(host) else {continue;};
                    let sq=(pos[0]-watching[0]).powi(2)
                         +(pos[1]-watching[1]).powi(2)
                         +(pos[2]-watching[2]).powi(2);
                    if sq<=64.0*64.0 {
                        let _=pl.send_payload(host,&PayloadId("rustpg:hit".into()),payload.clone());
                    }
                }
            }
        }
        // A API de attachments devolve snapshots. O antigo tick processava
        // vigor 20 vezes/s, mas persistia apenas em 1 dos 5 ticks, perdendo
        // quatro quintos das alteracoes. Simulamos cinco passos e persistimos
        // o estado 4 vezes/s, evitando salvar/sincronizar a cada frame.
        // One-second bleed pulses use the engine's real entity_hurt path: armor,
        // invulnerability, death hooks and other mods still see the damage.
        let due_bleeds={
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let mut due=Vec::new();
            let mut later=Vec::with_capacity(rt.pending_bleeds.len());
            for bleed in std::mem::take(&mut rt.pending_bleeds) {
                if bleed.next_tick<=tick {due.push(bleed)} else {later.push(bleed)}
            }
            rt.pending_bleeds=later;
            due
        };
        if !due_bleeds.is_empty() {
            let mut repeat=Vec::new();
            for mut bleed in due_bleeds {
                let Some(level)=host.level(&bleed.dimension) else{continue;};
                let Some(victim)=host.entity(bleed.victim) else{continue;};
                if victim.level(host)!=Some(level){continue;}
                let causing=host.entity(bleed.attacker)
                    .filter(|a|a.level(host)==Some(level));
                let taken=victim.hurt(host,"minecraft:generic",bleed.damage,None,causing)
                    .unwrap_or(false);
                if taken && bleed.remaining>1 {
                    bleed.remaining-=1;
                    bleed.next_tick=tick.saturating_add(20);
                    repeat.push(bleed);
                }
            }
            if !repeat.is_empty() {
                let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                let room=256usize.saturating_sub(rt.pending_bleeds.len());
                rt.pending_bleeds.extend(repeat.into_iter().take(room));
            }
        }
        // Every tick: queued heal events must not wait for the next profile save.
        let heals={let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            std::mem::take(&mut rt.pending_relic_heals)};
        if !heals.is_empty() {
            let players=host.players().server_players();
            for (dimension,id,value) in heals {
                let Some(level)=host.level(&dimension) else {continue;};
                if let Some(pl)=players.iter().find(|p|p.entity().id()==id &&
                    p.entity().level(host)==Some(level)) {
                    if let Some(l)=host.living(pl.entity()) {
                        let health=l.health(host).unwrap_or(0.0);
                        let max=l.max_health(host).unwrap_or(0.0);
                        if health>0.0 && max>health && value.is_finite() && value>0.0 {
                            let _=l.set_health(host,(health+value).min(max));
                        }
                    }
                }
            }
        }
        // Five authoritative HP updates per second (previously 2 Hz).
        // At most 24 mobs x 18 bytes: about 2.2 KB/s of payload per player.
        // The client never extrapolates HP between these confirmed samples.
        if tick%4==0 {
            let players=host.players().server_players();
            for pl in players {
                let Some(level)=pl.entity().level(host) else {continue;};
                let Some(pos)=pl.entity().position(host) else {continue;};
                let min=[pos[0]-25.0,pos[1]-16.0,pos[2]-25.0];
                let max=[pos[0]+25.0,pos[1]+16.0,pos[2]+25.0];
                let Some(entities)=level.entities_in(host,min,max) else {continue;};
                let mut nearby=Vec::new();
                for ent in entities {
                    if ent.id()==pl.entity().id(){continue;}
                    let kind=ent.kind(host).map(|k|k.id(host)).unwrap_or_default();
                    if kind=="minecraft:player" || kind=="minecraft:armor_stand" {continue;}
                    let Some(living)=host.living(ent) else {continue;};
                    let (Some(hp),Some(max_hp))=(living.health(host),living.max_health(host))
                        else {continue;};
                    let armor=living.attribute_value(host,"minecraft:armor").unwrap_or(0.0) as f32;
                    // Custom encounter: in phase II the Ashen Sentinel calls
                    // nearby Rift Hunters at most once every 300 ticks. The
                    // number of living minions is capped to prevent floods.
                    if kind==rustpg_native_entities::SENTINEL && hp>0.0 && max_hp>0.0
                        && hp/max_hp<=0.50 && tick%200==0 {
                        let boss_key=(level.dimension_id().to_owned(),ent.id());
                        let permitted={
                            let rt=self.runtime.lock().unwrap_or_else(|e|e.into_inner());
                            rt.boss_summons.get(&boss_key)
                                .is_none_or(|last|tick.saturating_sub(*last)>=300)
                        };
                        if permitted {
                            if let Some(origin)=ent.position(host){
                                let near=level.entities_in(host,
                                    [origin[0]-12.0,origin[1]-5.0,origin[2]-12.0],
                                    [origin[0]+12.0,origin[1]+5.0,origin[2]+12.0]);
                                let living_minions=near.unwrap_or_default().iter().filter(|other|
                                    other.kind(host).is_some_and(|key|
                                        key.id(host)==rustpg_native_entities::HUNTER)
                                    && host.living(**other).is_some_and(|l|
                                        l.health(host).unwrap_or(0.0)>0.0)).count();
                                if living_minions<3 {
                                    let target=[origin[0]+2.0,origin[1],origin[2]+2.0];
                                    if level.spawn_mob(host,rustpg_native_entities::HUNTER,target,
                                        mod_api::SpawnReason::MobSummoned).is_some(){
                                        self.runtime.lock().unwrap_or_else(|e|e.into_inner())
                                            .boss_summons.insert(boss_key,tick);
                                    }
                                }
                            }
                        }
                    }
                    // The server samples actual HP, never an assumed boss phase.
                    // Multiple nearby players may see the same mob: regen is
                    // limited to one application per mob per real server tick.
                    let mob_key=(level.dimension_id().to_owned(),ent.id());
                    let should_regen=if tick%20==0 {
                        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                        if let Some(snapshot)=rt.elite_combat.get_mut(&mob_key) {
                            snapshot.update_health(hp,max_hp);
                        }
                        if rt.elite_combat.contains_key(&mob_key) &&
                            rt.last_mob_regen_tick.get(&mob_key).copied()!=Some(tick) {
                            rt.last_mob_regen_tick.insert(mob_key.clone(),tick);true
                        }else{false}
                    } else {
                        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                        if let Some(snapshot)=rt.elite_combat.get_mut(&mob_key) {
                            snapshot.update_health(hp,max_hp);
                        }
                        false
                    };
                    if should_regen && hp>0.0 && hp<max_hp {
                        let heal=self.runtime.lock().unwrap_or_else(|er|er.into_inner())
                            .elite_combat.get(&mob_key).copied()
                            .map_or(0.0,|s|s.heal_per_second(max_hp));
                        if heal>0.0 {let _=living.set_health(host,(hp+heal).min(max_hp));}
                    }
                    let level_num=MOB_SCALE.get_existing(host,Holder::Entity(ent.id()))
                        .ok().flatten().map_or(1,|m|m.level).clamp(1,65535) as u16;
                    let sample=rustpg_mob_wire::MobSample {
                        id:ent.id(),hp:hp.min(max_hp),max_hp,armor:armor.max(0.0),level:level_num,
                    };
                    if sample.valid(){
                        if let Some(at)=ent.position(host) {
                            let d2=(pos[0]-at[0]).powi(2)+(pos[1]-at[1]).powi(2)
                                +(pos[2]-at[2]).powi(2);
                            if d2.is_finite(){nearby.push((d2,sample,kind));}
                        }
                    }
                }
                nearby.sort_by(|a,b|a.0.total_cmp(&b.0));
                // Nearby mobs are only replicated to the target HUD;
                // their mere presence must never reveal a bestiary entry.
                let samples=nearby.into_iter().take(rustpg_mob_wire::MobPacket::CAP)
                    .map(|(_,sample,_)|sample).collect();
                if let Some(packet)=rustpg_mob_wire::MobPacket::new(level.dimension_id(),samples) {
                    let _=pl.send_payload(host,&PayloadId("rustpg:mob_hp".into()),
                        PayloadBytes(packet.encode()));
                }
            }
        }
        if tick%5!=0 {return;}
        let players=host.players().server_players();
        for pl in players {
            let uuid=pl.uuid();
            let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{continue;};
            let previous=p.clone();
            let mut stats=p.node_stats();
            sync_player_stats(uuid,&stats);
            let attrs=rustpg_engine_attributes::EngineAttributes::from_stats(&stats);
            let relic_bonus=relic_snapshot(&p,now_seconds());
            p.stamina.max=attrs.stamina_max.max(1.0);
            p.stamina.regen_per_tick=(attrs.stamina_regen.max(0.0)+relic_bonus.stamina_regen)/20.0;
            let dimension=pl.entity().level(host).map(|lvl|lvl.dimension_id().to_owned())
                .unwrap_or_else(||host.dimension().to_owned());
            let cache_key=(dimension.clone(),pl.entity().id());
            {let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                rt.entity_dimensions.insert(pl.entity().id(),dimension.clone());
                rt.relic_snapshots.insert(cache_key.clone(),relic_bonus);
                rt.entity_by_uuid.insert(uuid,cache_key.clone());}

            let creative=pl.is_creative(host).unwrap_or(false)
                || pl.is_spectator(host).unwrap_or(false);
            // A new realm is paid once per persistent profile, even across reloads.
            // No XP for walking in circles or repeatedly entering a portal.
            if !creative && !dimension.starts_with("rustpg:rift/")
                && !p.dimensions.contains(&dimension) && !dimension.is_empty() {
                p.dimensions.push(dimension.clone());
                let _=p.add_prof_xp("exploration",30);
                let _=p.add_general_xp(5);
                let ready=p.add_rift_resonance(12);
                if ready {let _=pl.action_bar(host,Text::literal(
                    "Ressonância 100% · uma Fenda pode ser manifestada"));}
            }
            // Some mobile launcher keymaps send fast movement without reliably
            // exposing the sprint flag. Use both the canonical server flag and
            // a conservative 5-tick horizontal displacement, rejecting teleports.
            let sprint_flag=pl.entity().is_sprinting(host).unwrap_or(false);
            let position=pl.entity().position(host);
            let moving_fast={
                let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                let previous=rt.sprint_positions.remove(&uuid);
                if let Some(at)=position {rt.sprint_positions.insert(uuid,(dimension.clone(),at,tick));}
                previous.zip(position).is_some_and(|((old_dim,at,old_tick),next)| {
                    let dt=tick.saturating_sub(old_tick);
                    let dx=next[0]-at[0];let dz=next[2]-at[2];
                    let dist2=dx*dx+dz*dz;
                    old_dim==dimension && dt>=4 && dt<=7 &&
                        dist2>=0.58*0.58 && dist2<=2.2*2.2
                })
            };
            let sprinting=(sprint_flag || moving_fast) && !creative;
            let on_ground=pl.entity().on_ground(host).unwrap_or(false);
            for _ in 0..5 {
                if sprinting && !p.stamina.exhausted {
                    let cost=relic_bonus.attack_cost(
                        rustpg_engine_combat::cost(rustpg_engine_combat::Action::Sprint,attrs))
                        * rustpg_v4_fatigue::SPRINT_COST_MULTIPLIER; // 10% more sprint time
                    let _=p.stamina.consume(cost);
                    // Sprinting delays regeneration rather than regenerating
                    // stamina on the same tick as the drain.
                } else {p.stamina.tick_while_grounded(on_ground);}
                p.nutrition.tick();
            }
            let exhausted=p.stamina.exhausted && !creative;
            {
                let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                if exhausted {rt.exhausted_entities.insert(cache_key.clone());}
                else {rt.exhausted_entities.remove(&cache_key);}
            }
            // 0.9.0: sync acquired skill attributes into LivingRef's native map.
            // The UUID cache also records the entity identity: stale data from
            // death/dimension switches cannot suppress reapplication.
            let previous_attrs={
                let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                rt.native_attr_cache.get(&uuid)
                    .filter(|(identity,_)|identity==&cache_key)
                    .map(|(_,items)|items.clone()).unwrap_or_default()
            };
            if let Some(l)=host.living(pl.entity()) {
                let accepted=rustpg_starling090::reconcile(host,l,&stats,&previous_attrs);
                rustpg_starling090::effective_in_hooks(host,l,&mut stats);
                let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
                rt.native_attr_cache.insert(uuid,(cache_key.clone(),accepted));
            }
            // Read the attributes' final values again: external attribute modifiers
            // can now participate in stamina and vitality calculations too.
            let attrs=rustpg_engine_attributes::EngineAttributes::from_stats(&stats);
            p.stamina.max=attrs.stamina_max.max(1.0);
            p.stamina.regen_per_tick=(attrs.stamina_regen.max(0.0)+relic_bonus.stamina_regen)/20.0;
            // Damage hooks have LevelRandomHost, not ServerHost: one coherent
            // snapshot after native reconciliation is authoritative for the tick.
            self.runtime.lock().unwrap_or_else(|er|er.into_inner())
                .attribute_snapshots.insert(cache_key.clone(),stats.clone());
            // Replace the *named* movement modifier, never PlayerAbilities' base.
            // This composes with other mods' speed modifiers and is removed on recovery.
            if let Some(l)=host.living(pl.entity()) {
                if let Ok(id)=mod_api::Identifier::parse(
                    rustpg_native_modifiers::FATIGUE_MODIFIER_ID) {
                    if let Some(penalty)=rustpg_native_modifiers::speed_penalty(exhausted) {
                        let _=l.replace_attribute_modifier(host,"minecraft:movement_speed",
                            mod_api::AttributeModifier::new(id,penalty,
                                mod_api::AttributeOperation::AddMultipliedTotal),
                            mod_api::ModifierLifetime::Transient);
                    }else{
                        let _=l.remove_attribute_modifier(host,"minecraft:movement_speed",&id);
                    }
                }
            }
            if let Some(l)=host.living(pl.entity()) {
                // Player mobility supports the imported 10 x 5% Puffish tree.
                // Mob scaling retains its own independent 10% cap.
                if let Ok(id)=mod_api::Identifier::parse("rustpg:node_move_speed") {
                    let bonus=stats.get("general.move_speed").clamp(0.0,0.50);
                    if bonus>0.000001 {
                        let _=l.replace_attribute_modifier(host,"minecraft:movement_speed",
                            mod_api::AttributeModifier::new(id,bonus,
                                mod_api::AttributeOperation::AddMultipliedTotal),
                            mod_api::ModifierLifetime::Transient);
                    }else{let _=l.remove_attribute_modifier(host,"minecraft:movement_speed",&id);}
                }
                // Dedicated Luck modifier: composes with vanilla and other mods.
                if let Ok(id)=mod_api::Identifier::parse("rustpg:reference_luck") {
                    let bonus=stats.get("general.luck_bonus").clamp(0.0,128.0);
                    if bonus>0.000001 {
                        let _=l.replace_attribute_modifier(host,"minecraft:luck",
                            mod_api::AttributeModifier::new(id,bonus,
                                mod_api::AttributeOperation::AddValue),
                            mod_api::ModifierLifetime::Transient);
                    }else{let _=l.remove_attribute_modifier(host,"minecraft:luck",&id);}
                }
                // Armor, toughness and knockback resistance are independent
                // vanilla gameplay attributes, with isolated modifier IDs.
                // No raw base writes and no effect on other players or mobs.
                for (stat,vanilla,owner,cap) in [
                    ("defense.armor","minecraft:armor","rustpg:node_armor",2048.0),
                    ("defense.toughness","minecraft:armor_toughness","rustpg:node_toughness",2048.0),
                    ("defense.res_knockback","minecraft:knockback_resistance","rustpg:node_knockback",0.95),
                ] {
                    if let Ok(id)=mod_api::Identifier::parse(owner) {
                        let amount=stats.get(stat).clamp(0.0,cap);
                        if amount>0.000001 {
                            let _=l.replace_attribute_modifier(host,vanilla,
                                mod_api::AttributeModifier::new(id,amount,
                                    mod_api::AttributeOperation::AddValue),
                                mod_api::ModifierLifetime::Transient);
                        }else{let _=l.remove_attribute_modifier(host,vanilla,&id);}
                    }
                }
                // Selected melee discipline only. Do not apply sword nodes while
                // holding a pickaxe, and do not guess the identity of modded items.
                let selected_item=pl.selected_slot(host).zip(pl.inventory_slots(host))
                    .and_then(|(slot,items)|items.iter().find(|(i,_)|*i==slot as u32)
                        .map(|(_,stack)|stack.item()));
                let discipline=rustpg_weapon_training::from_held(host,selected_item);
                if let Ok(id)=mod_api::Identifier::parse("rustpg:node_attack_speed") {
                    let amount=(stats.get("general.attack_speed") +
                        discipline.map(|w|stats.get(&format!("weapon.{w}.attack_speed")))
                        .unwrap_or(0.0)).clamp(0.0,2.0);
                    if amount>0.000001 {
                        let _=l.replace_attribute_modifier(host,"minecraft:attack_speed",
                            mod_api::AttributeModifier::new(id,amount,
                                mod_api::AttributeOperation::AddMultipliedTotal),
                            mod_api::ModifierLifetime::Transient);
                    }else{let _=l.remove_attribute_modifier(host,"minecraft:attack_speed",&id);}
                }
                // Absorption capacity is a native vanilla attribute. This does not
                // fabricate golden hearts; it raises the real cap consumed by effects/items.
                if let Ok(id)=mod_api::Identifier::parse("rustpg:node_absorption") {
                    let amount=stats.get("defense.absorption").clamp(0.0,2048.0);
                    if amount>0.000001 {
                        let _=l.replace_attribute_modifier(host,"minecraft:max_absorption",
                            mod_api::AttributeModifier::new(id,amount,
                                mod_api::AttributeOperation::AddValue),
                            mod_api::ModifierLifetime::Transient);
                    }else{let _=l.remove_attribute_modifier(host,"minecraft:max_absorption",&id);}
                }
                // Mining reach follows the same server/client-synced vanilla
                // block_interaction_range that validates real digs. It applies
                // only while an item from #minecraft:pickaxes is selected.
                if let Ok(id)=mod_api::Identifier::parse("rustpg:mining_reach") {
                    let pickaxe=selected_item.is_some_and(|item|{
                        mod_api::Identifier::parse("minecraft:pickaxes").ok()
                            .and_then(|tag|host.tags().key(&tag))
                            .and_then(|tag|host.tags().is_in_tag(item,tag))
                            .unwrap_or(false)
                    });
                    let amount=if pickaxe {stats.get("mining.reach").clamp(0.0,16.0)} else {0.0};
                    if amount>0.000001 {
                        let _=l.replace_attribute_modifier(host,"minecraft:block_interaction_range",
                            mod_api::AttributeModifier::new(id,amount,
                                mod_api::AttributeOperation::AddValue),
                            mod_api::ModifierLifetime::Transient);
                    }else{let _=l.remove_attribute_modifier(host,"minecraft:block_interaction_range",&id);}
                }
                // A named additive attribute modifier replaces the old base overwrite.
                // Attribute ownership makes other mods' bonuses composable. The legacy
                // migration runs at most once per profile, and only on a precise match.
                let desired=attrs.max_health.clamp(20.0,2048.0);
                let old=l.max_health(host).unwrap_or(20.0) as f64;
                let before_hp=l.health(host).unwrap_or(old as f32);
                // Install our modifier FIRST. Never clear the old base if the
                // engine rejects the modifier (e.g. unsupported attribute).
                let old_base=l.attribute_base(host,"minecraft:max_health").unwrap_or(20.0);
                let migrate=!p.health_modifier_migrated &&
                    rustpg_native_modifiers::can_migrate_old_base(old_base,old,desired);
                if let Ok(id)=mod_api::Identifier::parse(
                    rustpg_native_modifiers::HEALTH_MODIFIER_ID) {
                    let bonus=rustpg_native_modifiers::bonus_health(desired);
                    let installed=if bonus>0.001 {
                        l.replace_attribute_modifier(host,"minecraft:max_health",
                            mod_api::AttributeModifier::new(id.clone(),bonus,
                                mod_api::AttributeOperation::AddValue),
                            mod_api::ModifierLifetime::Transient)
                    }else{
                        let _=l.remove_attribute_modifier(host,"minecraft:max_health",&id);
                        true
                    };
                    if installed {
                        if migrate {
                            if l.set_attribute_base(host,"minecraft:max_health",20.0) {
                                p.health_modifier_migrated=true;
                            }else{
                                // Abort migration rather than multiplying the old RPG base.
                                let _=l.remove_attribute_modifier(host,"minecraft:max_health",&id);
                            }
                        }else{p.health_modifier_migrated=true;}
                    }
                }
                let after=l.max_health(host).unwrap_or(old as f32);
                if before_hp>0.0 && after>0.0 {
                    // Recover HP clamped during a legacy-base migration and preserve
                    // missing hearts when max health really increases (20/20->21/21).
                    let target=(before_hp + (after as f64-old).max(0.0) as f32).min(after);
                    let current=l.health(host).unwrap_or(before_hp);
                    if (target-current).abs()>0.05 {
                        let _=l.set_health(host,target);
                    }
                }
                let max=l.max_health(host).unwrap_or(20.0);
                let health=l.health(host).unwrap_or(max);
                let in_combat=self.runtime.lock().unwrap_or_else(|er|er.into_inner())
                    .last_damage_tick.get(&cache_key).is_some_and(|last|
                        tick.saturating_sub(*last)<=120);
                let contextual=if in_combat {stats.get("defense.regen_combat")}
                    else {stats.get("defense.regen_out_combat")};
                let regen=rustpg_java_port::regen_for_elapsed_ticks(
                    attrs.health_regen+contextual,5);
                if regen>0.0 && health>0.0 && health<max && tick%5==0 {
                    // Vanilla health write happens on server; no phantom health bars.
                    let _=l.set_health(host,(health+regen).min(max));
                }
                p.vitality.max=max as f64;
                p.vitality.current=l.health(host).unwrap_or(health) as f64;
            }
            if p!=previous {let _=PROFILE.set(host,Holder::Player(uuid),p);}
        }
        if tick%40==0 {
            let mut state=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let online:std::collections::HashSet<u128>=host.players().server_players()
                .iter().map(|p|p.uuid()).collect();
            let live:std::collections::HashSet<(String,u32)>=host.players().server_players()
                .iter().filter_map(|p|p.entity().level(host).map(|lvl|
                    lvl.dimension_id().to_owned())
                    .map(|dim|(dim,p.entity().id()))).collect();
            state.exhausted_entities.retain(|id|live.contains(id));
            state.relic_snapshots.retain(|id,_|live.contains(id));
            state.attribute_snapshots.retain(|id,_|live.contains(id));
            state.last_damage_tick.retain(|id,_|live.contains(id));
            state.combo.purge(tick);
            for action in state.action_combat.values_mut(){action.reset_if_idle(tick);}
            state.action_combat.retain(|key,_|live.contains(key));
            state.entity_by_uuid.retain(|uuid,_|online.contains(uuid));
            state.native_attr_cache.retain(|uuid,(identity,_)|online.contains(uuid) && live.contains(identity));
        }
    }

    // API nativa Starling 0.9: mob existente no mundo recebe uma marca salva.
    // Atributos são valores absolutos: nunca recalcular a partir de um valor já escalado.
    fn entity_join_level(&self,host:&mut dyn ServerHost,e:&mut EntityJoinLevelEvent<'_>)->Flow {
        // Every entity (including passive mobs, pets and players) must have a
        // dimension recorded for LevelRandomHost damage callbacks. Previously
        // only scaled mobs were processed here.
        {
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            if rt.entity_dimensions.len()>65536 {rt.entity_dimensions.clear();}
            rt.entity_dimensions.insert(e.entity,e.dimension.to_owned());
            if rt.entity_kinds.len()>65536 {rt.entity_kinds.clear();}
            rt.entity_kinds.insert((e.dimension.to_owned(),e.entity),e.kind.to_owned());
        }
        if !rustpg_scaling::scales(e.kind) {return Flow::Continue;}
        let holder=Holder::Entity(e.entity);
        let old=MOB_SCALE.get_existing(host,holder).ok().flatten();
        let mob=if let Some(saved)=old.as_ref(){saved.clone()}else{
            // O poder considera apenas jogadores próximos. Spawn sem jogador = base normal.
            let power=host.entity(e.entity)
                .and_then(|ent|ent.position(host))
                .map(|pos|{
                    let players=host.players().server_players();
                    let nearby:Vec<(f64,f64)>=players.into_iter().filter_map(|pl|{
                        let here=pl.entity().position(host)?;
                        let dx=here[0]-pos[0];let dy=here[1]-pos[1];let dz=here[2]-pos[2];
                        let distance=(dx*dx+dy*dy+dz*dz).sqrt();
                        if distance>96.0 {return None;}
                        let p=PROFILE.get_existing(host,Holder::Player(pl.uuid())).ok().flatten();
                        Some((p.map_or(1.0,|p|p.level as f64),distance))
                    }).collect();
                    rustpg_scaling::group_power(&nearby)
                }).unwrap_or(1.0);
            let seed=(u64::from(e.entity)<<32) ^ self.runtime.lock()
                .unwrap_or_else(|er|er.into_inner()).tick ^ 0xC1A0_DE32;
            let rolled=rustpg_scaling::roll(e.kind,e.dimension,power,
                &rustpg_scaling::WorldProgress::default(),0,seed);
            // Se não podemos salvar no mob, não mudamos seus atributos.
            if MOB_SCALE.set(host,holder,rolled.clone()).is_err(){return Flow::Continue;}
            rolled
        };
        // The snapshot is derived only from the saved MobScale, so a chunk
        // reload does not re-roll mutations or change the mob's archetype.
        {let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            if rt.elite_combat.len()>65536 {rt.elite_combat.clear();}
            rt.elite_combat.insert((e.dimension.to_owned(),e.entity),
                rustpg_elite_encounters::Snapshot::from_scale(&mob));
        }
        let Some(ent)=host.entity(e.entity) else{return Flow::Continue;};
        let Some(liv)=host.living(ent) else{return Flow::Continue;};
        let target=(rustpg_scaling::base_health(e.kind)*mob.health_mult).clamp(1.0,2048.0);
        if (liv.max_health(host).unwrap_or(target as f32) as f64-target).abs()>0.05 {
            let applied=liv.set_attribute_base(host,"minecraft:max_health",target);
            if applied && old.is_none() && !e.loaded_from_disk {
                // Vida inicial cheia, mas nunca curar mobs persistentes ao recarregar chunks.
                let actual=liv.max_health(host).unwrap_or(target as f32);
                let _=liv.set_health(host,actual);
            }
        }
        if old.is_none() && !e.loaded_from_disk {
            // Base, não valor efetivo, para evitar duplicar modificadores de equipamento.
            let armor=mob.armor.clamp(0.0,100.0);
            let _=liv.set_attribute_base(host,"minecraft:armor",armor);
            if let Some(base)=liv.attribute_base(host,"minecraft:attack_damage") {
                let _=liv.set_attribute_base(host,"minecraft:attack_damage",
                    (base*mob.damage_mult).clamp(0.0,2048.0));
            }
            if let Some(base)=liv.attribute_base(host,"minecraft:movement_speed") {
                let _=liv.set_attribute_base(host,"minecraft:movement_speed",
                    (base*(1.0+mob.speed_bonus)).clamp(0.0,1.0));
            }
        }
        Flow::Continue
    }

    // Purge world-session IDs when an entity despawns, dies, changes dimension
    // or its chunk unloads. The hook is the inverse of entity_join_level.
    fn entity_leave_level(&self,_host:&mut dyn ServerHost,
        e:&mut mod_api::EntityLeaveLevelEvent<'_>) {
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        if rt.entity_dimensions.get(&e.entity).is_some_and(|d|d.as_str()==e.dimension) {
            rt.entity_dimensions.remove(&e.entity);
        }
        let key=(e.dimension.to_owned(),e.entity);
        rt.exhausted_entities.remove(&key);
        rt.relic_snapshots.remove(&key);
        rt.attribute_snapshots.remove(&key);
        rt.entity_kinds.remove(&key);
        rt.projectile_weapons.remove(&key);
        rt.last_damage_tick.remove(&key);
        rt.impact_positions.remove(&key);
        rt.elite_combat.remove(&key);
        rt.last_mob_regen_tick.remove(&key);
        rt.boss_summons.remove(&key);
        rt.action_combat.remove(&key);
        rt.pending_bleeds.retain(|b|b.dimension!=e.dimension ||
            (b.victim!=e.entity && b.attacker!=e.entity));
        rustpg_guard::remove(e.dimension,e.entity);
        rustpg_powers::remove(e.dimension,e.entity);
    }

    // A block placed by any entity must never be mineable for RPG XP.
    // Mark ore positions in the persistent per-level attachment before the
    // game writes them. This is conservative if another mod later cancels.
    fn entity_place(&self,host:&mut dyn ServerHost,e:&mut mod_api::EntityPlaceEvent)->Flow {
        let Some(name)=host.block_state_string(e.state) else{return Flow::Continue;};
        let track_gather=rustpg_gathering_rules::resource(&name)
            .is_some_and(|r|r.provenance_required);
        if rustpg_xp_rules::mined(&name).is_none() && !track_gather {
            return Flow::Continue;
        }
        let pos=[e.pos.x,e.pos.y,e.pos.z];
        let mut placed=PLACED_ORES.get(host,Holder::Level)
            .unwrap_or_else(|_|rustpg_v4_provenance::PlacedOres::new());
        placed.mark(pos);
        let _=PLACED_ORES.set(host,Holder::Level,placed.clone());
        self.runtime.lock().unwrap_or_else(|er|er.into_inner())
            .placed_resource_blocks.entry(host.dimension().to_owned()).or_default().mark(pos);
        Flow::Continue
    }

    // Farming growth is player-specific without changing vanilla crop data:
    // a crop random tick may be accelerated by the strongest recently-present
    // farmer within 32 blocks in the same dimension. An explicit deny from an
    // earlier mod is respected; this only turns Default into True.
    fn crop_grow_pre(&self,_host:&mut dyn mod_api::LevelRandomHost,
        e:&mut mod_api::CropGrowPreEvent<'_>) {
        if !matches!(*e.result,mod_api::TriState::Default){return;}
        let dimension=match e.dimension {
            mod_api::LevelDimension::Overworld=>"minecraft:overworld".to_owned(),
            mod_api::LevelDimension::Nether=>"minecraft:the_nether".to_owned(),
            mod_api::LevelDimension::End=>"minecraft:the_end".to_owned(),
            mod_api::LevelDimension::Custom(key)=>key.id().to_owned(),
        };
        let (tick,nearby)={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let tick=rt.tick;
            let list=rt.sprint_positions.iter().filter_map(|(uuid,(dim,pos,seen))|{
                if dim!=&dimension || tick.saturating_sub(*seen)>20{return None;}
                let dx=pos[0]-(f64::from(e.pos.x)+0.5);
                let dy=pos[1]-(f64::from(e.pos.y)+0.5);
                let dz=pos[2]-(f64::from(e.pos.z)+0.5);
                (dx*dx+dy*dy+dz*dz<=32.0*32.0).then_some(*uuid)
            }).collect::<Vec<_>>();
            (tick,list)
        };
        if nearby.is_empty(){return;}
        let growth={
            let all=player_stats_cache().lock().unwrap_or_else(|er|er.into_inner());
            nearby.iter().filter_map(|uuid|all.get(uuid))
                .map(|stats|stats.get("farming.growth"))
                .fold(0.0f64,f64::max).clamp(0.0,4.0)
        };
        if growth<=0.0{return;}
        // Convert +growth speed into an extra random-tick success chance while
        // keeping it bounded. +100% -> 50% bonus chance, +300% -> 75%.
        let chance=(growth/(1.0+growth)).clamp(0.0,0.80);
        let seed=tick.rotate_left(21) ^ ((e.pos.x as i64 as u64).rotate_left(7))
            ^ ((e.pos.y as i64 as u64).rotate_left(19)) ^ (e.pos.z as i64 as u64);
        if rustpg_num::unit(rustpg_num::mix(seed))<chance {*e.result=mod_api::TriState::True;}
    }

    // Rolled chest/container loot is mutable before vanilla shuffles it into
    // slots. Only a real player opener can apply personal looting/chest luck.
    fn container_loot_filled(&self,level:&dyn mod_api::LevelHost,
        e:&mut mod_api::ContainerLootFilledEvent<'_>) {
        let Some(uuid)=e.context.opener else{return;};
        let stats={
            let all=player_stats_cache().lock().unwrap_or_else(|er|er.into_inner());
            let Some(stats)=all.get(&uuid) else{return;};
            stats.clone()
        };
        let duplicate=(stats.get("looting.chest")
            +stats.get("luck.chests.duplicate")
            +0.25*stats.get("luck.chests.bonus")
            +0.10*stats.get("luck.chests.quality")
            +0.50*stats.get("luck.general.duplicate")).max(0.0);
        let rare=(stats.get("luck.chests.rarity")
            +0.35*stats.get("luck.chests.special")
            +0.20*stats.get("luck.chests.extraordinary")
            +0.25*stats.get("luck.general.rarity")).max(0.0);
        if duplicate<=0.0 && rare<=0.0{return;}
        let originals=e.drops.clone();
        let mut additions=Vec::new();
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let rng=rt.rng.get_or_insert_with(||rustpg_num::Rng(0xC4E5_71A7));
        for stack in originals.iter().take(54) {
            let copies=rng.overflow_count(duplicate).min(16);
            for _ in 0..copies {
                additions.push(stack.clone());
                if additions.len()>=128{break;}
            }
            if additions.len()>=128{break;}
        }
        if rng.chance(rare.min(0.75)) {
            let extraordinary=rng.chance((stats.get("luck.chests.extraordinary")
                +0.5*stats.get("luck.chests.special")).clamp(0.0,0.50));
            let id=if extraordinary{"minecraft:diamond"}else{"minecraft:emerald"};
            let key=rt.cached_item_keys.get(id).copied();
            if let Some(key)=key {
                if let Some(stack)=mod_api::ItemStack::new(key,1){additions.push(stack);}
            }
        }
        drop(rt);
        e.drops.extend(additions.into_iter().take(128));
    }

    // Loot effects run on the authoritative BlockDropsEvent before item
    // entities spawn. Stats come from the server-tick snapshot of the actual
    // breaker; the GUI cannot inject a roll or a count.
    fn block_drops(&self,level:&dyn mod_api::LevelHost,
        e:&mut mod_api::BlockDropsEvent<'_>)->Flow {
        let Some(uuid)=e.breaker else{return Flow::Continue;};
        let (identity,stats)={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let Some(identity)=rt.entity_by_uuid.get(&uuid).cloned() else{return Flow::Continue;};
            let Some(stats)=rt.attribute_snapshots.get(&identity).cloned() else{return Flow::Continue;};
            (identity,stats)
        };
        let (state,mature)={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let Some(state)=rt.resource_block_ids.get(&e.state.block()).cloned() else{return Flow::Continue;};
            (state,rt.mature_resource_states.contains(&e.state))
        };
        let Some(activity)=rustpg_luck::block_activity(&state,mature) else{return Flow::Continue;};
        // Positions marked as player-placed in this session do not generate
        // resource duplication. The persistent attachment still guards XP in
        // player_destroyed_block; this mirror closes the drop path itself.
        let pos=[e.pos.x,e.pos.y,e.pos.z];
        let placed={
            let rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            rt.placed_resource_blocks.get(&identity.0).is_some_and(|v|v.is_placed(pos))
        };
        if placed && activity.prof!="farming" {return Flow::Continue;}
        let originals=e.drops.clone();
        let mut additions=Vec::new();
        let (rare,specific)={
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let rng=rt.rng.get_or_insert_with(||rustpg_num::Rng(0xC10D_DA7A));
            for drop in originals.iter().take(64) {
                let count=drop.stack.count().max(1);
                let mut left=rustpg_luck::extra_copies(&stats,&activity,count,rng).min(4096);
                let mut guard=0usize;
                while left>0 && guard<64 {
                    let take=(left.min(u64::from(count))) as u32;
                    let mut copy=drop.clone();
                    if copy.stack.set_count(take).is_ok(){additions.push(copy);}
                    left=left.saturating_sub(u64::from(take));guard+=1;
                }
            }
            (rustpg_luck::rare_drop(&activity,&stats,rng),
             rustpg_luck::resource_bonus(&activity,&state,&stats,rng))
        };
        if let Some((id,count))=rare {
            let key=self.runtime.lock().unwrap_or_else(|er|er.into_inner())
                .cached_item_keys.get(id).copied();
            if let (Some(template),Some(key))=(e.drops.first().cloned(),key) {
                if let Some(stack)=mod_api::ItemStack::new(key,count) {
                    let mut extra=template;extra.stack=stack;additions.push(extra);
                }
            }
        }
        if let Some(template)=e.drops.first().cloned() {
            for (id,count) in specific {
                let key=self.runtime.lock().unwrap_or_else(|er|er.into_inner())
                    .cached_item_keys.get(id).copied();
                if let Some(key)=key {
                    if let Some(stack)=mod_api::ItemStack::new(key,count) {
                        let mut extra=template.clone();extra.stack=stack;additions.push(extra);
                    }
                }
            }
        }
        e.drops.extend(additions.into_iter().take(128));
        Flow::Continue
    }

    // Real mining event, after successful block destruction (not client clicks).
    fn player_destroyed_block(&self,host:&mut dyn ServerHost,
        e:&mut PlayerDestroyedBlockEvent) {
        let Some(name)=host.block_state_string(e.state) else {return;};
        let ore=rustpg_xp_rules::mined(&name);
        let gathered=rustpg_gathering_rules::resource(&name);
        let block=name.split('[').next().unwrap_or(&name);
        let log=gathered.is_some_and(|r|r.profession=="woodcutting");
        let stone=matches!(block,"minecraft:stone"|"minecraft:cobblestone"|
            "minecraft:deepslate"|"minecraft:cobbled_deepslate");
        if ore.is_none() && gathered.is_none() && !stone {return;}
        let track=gathered.is_some_and(|r|r.provenance_required) || ore.is_some();
        let was_placed=if track {
            let mut placed=PLACED_ORES.get(host,Holder::Level)
                .unwrap_or_else(|_|rustpg_v4_provenance::PlacedOres::new());
            let was=placed.take_placed(e.pos);
            if was {let _=PLACED_ORES.set(host,Holder::Level,placed);}
            let mirror=self.runtime.lock().unwrap_or_else(|er|er.into_inner())
                .placed_resource_blocks.entry(host.dimension().to_owned()).or_default()
                .take_placed(e.pos);
            was || mirror
        }else{false};
        let Some(pl)=host.players().acting_player() else{return;};
        let creative=pl.is_creative(host).unwrap_or(false)
            || pl.is_spectator(host).unwrap_or(false);
        if creative {return;}
        let uuid=pl.uuid();
        let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{return;};
        if ore.is_some() && !was_placed {
            let reward=extensions().calculate(&rustpg_v4_core::ProgressionEvent::Mine {
                block_state:&name,player_placed:false,
            });
            if reward.general>0 || reward.profession>0 {
                let adjusted=rustpg_node_effects::resource_xp(
                    &p.node_stats(),"mining",&name,reward.profession);
                let _=p.add_prof_xp("mining",adjusted);
                let _=p.add_general_xp(reward.general);
            }
        }else if let Some(reward)=gathered {
            if rustpg_gathering_rules::eligible(reward,was_placed,creative) {
                let adjusted=rustpg_node_effects::resource_xp(
                    &p.node_stats(),reward.profession,&name,reward.xp);
                let _=p.add_prof_xp(reward.profession,adjusted);
            }
        }
        let _=PROFILE.set(host,Holder::Player(uuid),p);
        if ore.is_some() && !was_placed {credit_professional_event(host,uuid,"mine",1);}
        else if !was_placed && log {credit_professional_event(host,uuid,"woodcut",1);}
        else if !was_placed && gathered.is_some_and(|r|r.profession=="farming") {
            credit_professional_event(host,uuid,"harvest",1);
        }
    }

    // LootDrops precedes LivingDrops in the supplied Starling 0.9.0 engine.
    // Unlike checking the held item at death, this context retains the bow,
    // crossbow or trident actually used when the projectile was launched.
    fn loot_drops(&self,host:&mut dyn ServerHost,e:&mut mod_api::LootDropsEvent<'_>) {
        let Some((victim,_))=e.context.entity else {return;};
        let Some(source)=e.context.damage_source else {return;};
        let Some(player_id)=e.context.player else {return;};
        let Ok(causing)=u32::try_from(source.causing) else {return;};
        if causing==0 {return;}
        // A remembered kill credit is not always the entity that caused death.
        // Do not attribute environmental deaths or someone else's projectile.
        let Some(attacker)=host.entity(causing) else {return;};
        if attacker.uuid(host)!=Some(player_id) {return;}
        let Some(weapon)=e.context.weapon_item(host) else {return;};
        let Some(kind)=rustpg_weapon_training::ranged_weapon(host,weapon.item()) else {
            return;
        };
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let tick=rt.tick;
        rt.recent_death_weapon.insert((e.context.dimension.to_string(),victim,causing),(kind,tick));
    }

    // Drops are awarded once after a mob's death loot is collected. No fabricated
    // vanilla ItemStack: relics are stored in the player's authoritative profile.
    fn living_drops(&self,host:&mut dyn ServerHost,e:&mut LivingDropsEvent<'_>)->Flow {
        if e.kind=="minecraft:player" {return Flow::Continue;}
        let Ok(causing)=u32::try_from(e.source.causing) else{return Flow::Continue;};
        let Some(cause)=host.entity(causing) else{return Flow::Continue;};
        let Some(uuid)=cause.uuid(host) else{return Flow::Continue;};
        let Some(pl)=host.player(uuid) else{return Flow::Continue;};
        if pl.is_creative(host)!=Some(false) || pl.is_spectator(host)!=Some(false) {return Flow::Continue;}
        let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{return Flow::Continue;};
        let count=p.kills.entry(e.kind.to_string()).or_default();
        *count=count.saturating_add(1);
        let rec=p.species.entry(e.kind.to_string()).or_insert_with(||rustpg_bestiary::SpeciesRecord{
            kind:e.kind.to_string(),..Default::default() });
        rec.first_seen=true;rec.kills=rec.kills.saturating_add(1);rec.max_level=rec.max_level.max(1);
        let scaled=MOB_SCALE.get_existing(host,Holder::Entity(e.entity)).ok().flatten();
        let tier=scaled.as_ref().map_or(rustpg_scaling::Tier::Normal,|m|m.tier);
        let mob_level=scaled.as_ref().map_or(p.level.max(1),|m|m.level);
        let tier_count=p.tier_kills.entry(tier.key().to_string()).or_default();
        *tier_count=tier_count.saturating_add(1);
        let reward=extensions().calculate(&rustpg_v4_core::ProgressionEvent::Kill {
            kind:e.kind, natural_health:rustpg_scaling::base_health(e.kind),
            xp_multiplier:scaled.as_ref().map_or(1.0,|m|m.xp_mult),
        });
        let recent_melee={
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let now=rt.tick;
            rt.recent_melee.remove(&(host.dimension().to_owned(),e.entity,causing))
                .and_then(|(skill,tick)|(now.saturating_sub(tick)<=60).then_some(skill))
        };
        let damage_kind=e.source.damage_type.rsplit(':').next().unwrap_or("");
        let projectile=matches!(damage_kind,"arrow"|"trident"|"fireball"|"indirect_magic");
        let recorded_weapon={
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let tick=rt.tick;
            rt.recent_death_weapon.remove(&(host.dimension().to_owned(),e.entity,causing))
                .and_then(|(career,at)|(tick.saturating_sub(at)<=20).then_some(career))
        };
        let training=if projectile {
            recorded_weapon.unwrap_or_else(||
                rustpg_combat_training::kill_profession(e.kind,e.source.damage_type))
        } else {
            recent_melee.or(recorded_weapon).unwrap_or_else(||
                rustpg_combat_training::kill_profession(e.kind,e.source.damage_type))
        };
        let training_xp=rustpg_combat_training::kill_profession_xp(e.kind,reward.profession);
        if training_xp>0 {
            // Passive creatures count as hunting regardless of the weapon;
            // a real confirmed melee weapon also trains its own discipline.
            if rustpg_v4_core::classify_living(e.kind)==rustpg_v4_core::CreatureClass::Passive {
                let _=p.add_prof_xp("hunting",training_xp);
                if let Some(weapon)=recent_melee {
                    if weapon!="hunting" {let _=p.add_prof_xp(weapon,2);}
                }
            } else {
                let _=p.add_prof_xp(training,training_xp);
            }
        }
        if reward.general>0 {let _=p.add_general_xp(reward.general);}
        let notices=p.record_mastery("kill",self_rustpg_server_tick());
        if let Some(scale)=scaled.as_ref() {
            if let Some(record)=p.species.get_mut(e.kind) {
                record.max_level=record.max_level.max(scale.level);
                let tier_name=scale.tier.name().to_owned();
                if !record.tiers.contains(&tier_name){record.tiers.push(tier_name);}
                if let Some(mutation)=scale.mutation {
                    let key=mutation.to_string();
                    if !record.mutations.contains(&key){record.mutations.push(key);}
                }
            }
        }
        // Duplicate only stacks that the real death already produced.
        // This preserves item components/equipment rolls and never fabricates
        // another kill. Hunting/looting/luck nodes feed the same bounded roll.
        let stats=p.node_stats();
        let mut drop_additions=Vec::new();
        let product_keys=self.runtime.lock().unwrap_or_else(|er|er.into_inner())
            .cached_item_keys.iter().filter_map(|(id,key)|
                rustpg_luck::husbandry_product(id).map(|product|(*key,product))).collect::<Vec<_>>();
        let relic=if rustpg_scaling::scales(e.kind) {let mut state=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let rng=state.rng.get_or_insert_with(||rustpg_num::Rng(0xCC0D_E101));
            let boss=tier>=rustpg_scaling::Tier::MiniBoss;
            let elite=tier>=rustpg_scaling::Tier::Elite;
            let loot_mult=scaled.as_ref().map_or(1.0,|m|m.loot_mult);
            let livestock=rustpg_luck::livestock(e.kind);
            for drop in e.drops.iter().cloned().take(64) {
                let count=drop.stack.count().max(1);
                let mut left=rustpg_luck::mob_extra(&stats,boss,elite,loot_mult,count,rng).min(4096);
                if livestock {
                    if let Some((_,product))=product_keys.iter().find(|(key,_)|*key==drop.stack.item()) {
                        left=left.saturating_add(rustpg_luck::husbandry_extra(&stats,product,count,rng));
                    }
                }
                let mut guard=0usize;
                while left>0 && guard<64 {
                    let take=(left.min(u64::from(count))) as u32;
                    let mut copy=drop.clone();
                    if copy.stack.set_count(take).is_ok(){drop_additions.push(copy);}
                    left=left.saturating_sub(u64::from(take));guard+=1;
                }
            }
            let category=if boss{"bosses"}else{"mobs"};
            let hunter_luck=(stats.get("luck.general.rarity")
                +stats.get(&format!("luck.{category}.rarity"))
                +0.25*stats.get(&format!("luck.{category}.bonus"))
                +0.15*stats.get(&format!("luck.{category}.quality"))
                +0.30*stats.get(&format!("luck.{category}.special"))
                +0.45*stats.get(&format!("luck.{category}.extraordinary"))
                +0.5*stats.get("hunting.trophy")).clamp(0.0,4.0);
            rustpg_gear::roll_drop(tier,mob_level,hunter_luck,rng)
        } else {None};
        e.drops.extend(drop_additions.into_iter().take(128));
        if let Some(g)=relic { let _=p.gear.add(g); }
        let resonance=rustpg_rifts::resonance_for_kill(e.kind,tier.key());
        let rift_ready=p.add_rift_resonance(resonance);
        let _=PROFILE.set(host,Holder::Player(uuid),p);
        credit_professional_event(host,uuid,"kill",1);
        for n in notices{let _=pl.action_bar(host,Text::literal(n));}
        if rift_ready {let _=pl.action_bar(host,Text::literal(
            "Ressonância 100% · /rustpg rift manifest para abrir uma Fenda"));}
        Flow::Continue
    }

    // Hook real do servidor: o resultado precisa existir; lançar a vara sozinho não conta.
    fn item_fished(&self,host:&mut dyn ServerHost,e:&mut ItemFishedEvent<'_>)->Flow {
        if e.drops.is_empty(){return Flow::Continue;}
        let Some(pl)=host.player(e.player) else{return Flow::Continue;};
        if pl.is_creative(host).unwrap_or(false){return Flow::Continue;}
        let uuid=pl.uuid();
        let Ok(mut p)=PROFILE.get(host,Holder::Player(uuid)) else{return Flow::Continue;};
        // A separate fishing stat changes the exact durability loss the
        // ItemFishedEvent version 2 passes to the vanilla rod.
        let saved_wear=p.node_stats().get("fishing.rod_eff");
        if saved_wear>0.0 {
            let tick=self.runtime.lock().unwrap_or_else(|er|er.into_inner()).tick;
            let seed=(uuid as u64).wrapping_add(tick.rotate_left(9));
            let old=i32::try_from(e.rod_damage).unwrap_or(i32::MAX);
            e.rod_damage=rustpg_node_effects::rod_wear(old,saved_wear,seed) as _;
        }
        let stats=p.node_stats();
        let water=pl.entity().level(host)
            .and_then(|level|{
                let pos=e.fished.position;
                level.biome_at(host,[pos[0].floor() as i32,pos[1].floor() as i32,pos[2].floor() as i32])
            })
            .map_or("other",|biome|rustpg_luck::water_of(&biome));
        let extras={
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let rng=rt.rng.get_or_insert_with(||rustpg_num::Rng(0xF15A_CA7C));
            rustpg_luck::fishing_extra(&stats,water,rng)
        };
        for (id,count) in extras {
            if let Ok(id)=mod_api::Identifier::parse(id) {
                if let Some(key)=host.item_key(&id) {
                    if let Some(stack)=mod_api::ItemStack::new(key,count){e.drops.push(stack);}
                }
            }
        }
        let junk_reduction=stats.get("fishing.less_junk").clamp(0.0,0.95);
        if junk_reduction>0.0 {
            let junk=["minecraft:lily_pad","minecraft:bowl","minecraft:leather",
                "minecraft:leather_boots","minecraft:rotten_flesh","minecraft:stick",
                "minecraft:string","minecraft:bone","minecraft:ink_sac","minecraft:tripwire_hook"];
            let junk_keys=junk.iter().filter_map(|id|mod_api::Identifier::parse(id).ok()
                .and_then(|id|host.item_key(&id))).collect::<Vec<_>>();
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            let rng=rt.rng.get_or_insert_with(||rustpg_num::Rng(0x1E55_A11A));
            e.drops.retain(|stack|!junk_keys.contains(&stack.item()) || !rng.chance(junk_reduction));
        }
        let _=PROFILE.set(host,Holder::Player(uuid),p);
        credit_professional_event(host,uuid,"fish",1);
        Flow::Continue
    }

    // Every vanilla/Horizon enchantment read sees the persisted portion of
    // Transcendence that fits the native 0..=255 reader. The complete counter
    // remains on the item for RustPG overflow consumers.
    fn get_enchantment_level(&self,e:&mut mod_api::GetEnchantmentLevelEvent<'_>) {
        rustpg_enchant_stack::expose_to_reader(e);
    }

    // Enchanting-table offers use the real server player and the exact
    // three costs/row level Horizon is about to send to the menu.
    fn prepare_item_enchant(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::PrepareItemEnchantEvent<'_>) {
        let Ok(p)=PROFILE.get(host,Holder::Player(e.player)) else{return;};
        let reduction=p.node_stats().get("enchanting.cost").clamp(0.0,0.90);
        if reduction<=0.0{return;}
        for cost in e.costs.iter_mut() {
            if *cost>0 {
                *cost=(f64::from(*cost)*(1.0-reduction)).ceil()
                    .clamp(1.0,i32::MAX as f64) as i32;
            }
        }
    }
    fn enchantment_level_set(&self,host:&mut dyn ServerHost,
        e:&mut mod_api::EnchantmentLevelSetEvent<'_>) {
        let Ok(p)=PROFILE.get(host,Holder::Player(e.offer.player)) else{return;};
        let power=p.node_stats().get("enchanting.table_power").max(0.0);
        if power<=0.0{return;}
        // Count stats preserve fractional ranks: +0.02/rank becomes +1
        // offer level at 50 effective ranks instead of a hidden boolean.
        let extra=power.floor().clamp(0.0,255.0) as i32;
        if extra>0 && e.level>0 {
            e.level=e.level.saturating_add(extra).max(e.offer.row+1);
        }
    }
    fn anvil_update(&self,side:mod_api::HostSide<'_>,
        e:&mut mod_api::AnvilUpdateEvent<'_>)->Flow {
        let profile=match side {
            mod_api::HostSide::Server(server)=>PROFILE.get(server,Holder::Player(e.player)).ok(),
            mod_api::HostSide::Client(_)=>self.ui.lock()
                .unwrap_or_else(|er|er.into_inner()).profile.clone(),
        };
        let Some(profile)=profile else{return Flow::Continue;};
        let stats=profile.node_stats();
        let transcendence=profile.rank("enchanting.transcendence")>0;
        let enchant=stats.get("enchanting.cost").clamp(0.0,0.90);
        let smith=stats.get("smithing.anvil_cost").clamp(0.0,0.90);
        let repair=stats.get("smithing.repair").clamp(0.0,0.90);
        if e.update.xp_cost>0 {
            e.update.xp_cost=(f64::from(e.update.xp_cost)*(1.0-enchant)*(1.0-smith))
                .ceil().clamp(1.0,i32::MAX as f64) as i32;
        }
        if e.update.material_cost>0 && repair>0.0 {
            e.update.material_cost=(f64::from(e.update.material_cost)*(1.0-repair))
                .ceil().clamp(1.0,i32::MAX as f64) as i32;
        }
        // The vanilla anvil must first produce a legal output. Mastery then
        // permits a finite number of level-I book cap bypasses; Transcendence
        // stores further levels on the item itself and never depends on the
        // player's profile after the craft.
        let mastery=stats.get("enchanting.mastery").floor().clamp(0.0,1_000_000.0) as u64;
        if (mastery>0 || transcendence) && e.update.output.is_some() {
            if let Some(right)=e.update.right.clone() {
                let right_book=mod_api::__private::item_key_id(right.item())
                    ==Some("minecraft:enchanted_book");
                if right_book {
                    let left=e.update.left.clone();
                    if let Some(output)=e.update.output.as_mut() {
                        let output_book=mod_api::__private::item_key_id(output.item())
                            ==Some("minecraft:enchanted_book");
                        let _=rustpg_enchant_stack::apply_level_one_book(
                            &left,&right,output,output_book,mastery,transcendence);
                    }
                }
            }
        }
        Flow::Continue
    }
    // Craft rewards use the stable post-craft event only. RustPG does not alter
    // vanilla stack limits or crafting output behind the engine's public API.
    fn item_crafted(&self,host:&mut dyn ServerHost,e:&mut ItemCraftedEvent<'_>) {
        if e.stack.count()==0{return;}
        let Some(pl)=host.player(e.player) else{return;};
        if pl.is_creative(host).unwrap_or(false){return;}
        credit_professional_event(host,pl.uuid(),"craft",1);
    }

    fn entity_tame(&self,host:&mut dyn ServerHost,e:&mut EntityTameEvent<'_>) {
        // Unlike animal_tame (a pre-event), this fires AFTER ownership is set.
        // Only the actual owner receives taming XP.
        if let Some(owner)=e.owner {
            credit_professional_event(host,owner,"tame",1);
        }
    }
    fn player_trade(&self,host:&mut dyn ServerHost,e:&mut PlayerTradeEvent<'_>) {
        credit_professional_event(host,e.player,"trade",1);
    }
    fn player_brewed_potion(&self,host:&mut dyn ServerHost,e:&mut PlayerBrewedPotionEvent<'_>) {
        if e.stack.count()>0 {credit_professional_event(host,e.player,"brew",1);}
    }
    fn player_enchant_item(&self,host:&mut dyn ServerHost,e:&mut PlayerEnchantItemEvent<'_>) {
        if !e.enchantments.is_empty() {
            credit_professional_event(host,e.player,"enchant",1);
        }
    }
    fn player_item_smithed(&self,host:&mut dyn ServerHost,e:&mut PlayerItemSmithedEvent<'_>) {
        if e.result.count()>0 {credit_professional_event(host,e.player,"smith",1);}
    }
    fn item_smelted(&self,host:&mut dyn ServerHost,e:&mut ItemSmeltedEvent<'_>) {
        if e.amount_removed==0 {return;}
        let Some(ref stack)=e.stack else{return;};
        // Single output events can represent a stack moved by shift-click.
        // The data-pack rule caps accepted units per completed event.
        if let Some(kind)=furnace_profession(host,stack.item()) {
            credit_professional_event(host,e.player,kind,e.amount_removed);
        }
    }

    fn player_logged_out(&self,host:&mut dyn ServerHost,e:&mut PlayerLoggedOutEvent<'_>) {
        // Transient modifiers disappear on entity unload. Explicit removal also
        // covers orderly disconnects and prevents an orphan penalty during handoff.
        if let Some(pl)=host.player(e.player) {
            if let Some(l)=host.living(pl.entity()) {
                if let Ok(id)=mod_api::Identifier::parse(rustpg_native_modifiers::FATIGUE_MODIFIER_ID) {
                    let _=l.remove_attribute_modifier(host,"minecraft:movement_speed",&id);
                }
            }
        }
        player_stats_cache().lock().unwrap_or_else(|er|er.into_inner()).remove(&e.player);
        let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
        let abandoned_portal=rt.rift_portals.remove(&e.player);
        let abandoned_session=rt.rift_sessions.get(&e.player).cloned();
        rt.sprint_positions.remove(&e.player);
        rt.native_attr_cache.remove(&e.player);
        if let Some(key)=rt.entity_by_uuid.remove(&e.player) {
            rt.relic_snapshots.remove(&key);
            rt.exhausted_entities.remove(&key);
            rt.pending_relic_heals.retain(|(dimension,pid,_)|
                !(dimension==&key.0 && *pid==key.1));
            rt.recent_melee.retain(|(dimension,_,attacker),_|
                !(dimension==&key.0 && *attacker==key.1));
            rt.melee_sharpness_overflow.retain(|(dimension,_,attacker),_|
                !(dimension==&key.0 && *attacker==key.1));
            rt.recent_death_weapon.retain(|(dimension,_,attacker),_|
                !(dimension==&key.0 && *attacker==key.1));
        }
        drop(rt);
        if let Some(portal)=abandoned_portal {run_rift_commands(host,&rustpg_rifts::portal_commands(&portal.dimension,portal.pos,false));}
        if let Some(mut session)=abandoned_session {
            // Try to leave while the player is still available, then retain cleanup state on failure.
            if let Some(pl)=host.player(e.player) {
                let _=pl.entity().teleport_to_level(host,&session.origin_dimension,session.origin_pos,None);
            }
            let deleted=host.runtime_levels().delete(&session.level_id).is_ok() || !host.runtime_levels().is_open(&session.level_id);
            let mut rt=self.runtime.lock().unwrap_or_else(|er|er.into_inner());
            if deleted {rt.rift_sessions.remove(&e.player);}
            else {session.phase=rustpg_rifts::SessionPhase::Returning;rt.rift_sessions.insert(e.player,session);}
        }
    }

    fn client_level_unload(&self,_host:&mut dyn ClientHost,
        _e:&mut mod_api::ClientLevelUnloadEvent<'_>) {
        // A world switch invalidates cached entity IDs, HUD profile and old hits.
        // Never let a packet from the previous dimension contaminate the new world.
        let _=take_hit_inbox();
        let _=take_mob_inbox();
        OPEN_REQUEST.store(false,Ordering::Release);
        let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        ui.profile=None;
        ui.target=None;
        ui.last_target=None;
        ui.floating.clear();
        ui.nearby_mobs.clear();
        ui.external_species=None;
        ui.bestiary_instances.clear();
        ui.bestiary_scan_tick=None;
        ui.mob_stats.clear();
        ui.damage_meter.clear();
        ui.local_id=None;
        ui.announced=false;
    }
    // The local movement gate runs before keys/axes become a sprint packet.
    // The server still owns stamina drain and movement-speed penalties; the
    // client only suppresses sprint while its synchronized profile is exhausted.
    fn movement_input(&self,host:&dyn ClientHost,
        e:&mut mod_api::MovementInputEvent) {
        if !host.playing() || host.local_player().is_none() {return;}
        let exhausted=self.ui.lock().unwrap_or_else(|er|er.into_inner())
            .profile.as_ref().is_some_and(|p|p.stamina.exhausted);
        if exhausted {e.sprint=false;}
    }

    // Native remappable mappings are consumed in client_tick_pre. input_key also
    // supplies a physical/layout V fallback because some Android launchers do not
    // deliver the registered mapping click reliably.
    fn input_key(&self,host:&mut dyn ClientHost,e:&mut InputKeyEvent<'_>) {
        if e.action!=KeyAction::Press || !host.playing() || host.local_player().is_none(){return;}
        let v=e.key=="key.keyboard.v" || e.layout_key==Some("key.keyboard.v");
        if v && host.mod_screen()!=Some(ID) {OPEN_REQUEST.store(true,Ordering::Release);}
    }

    fn client_resources_reloaded(&self, _: &mut dyn ClientHost, _: &mut mod_api::ClientResourcesReloadedEvent) {
        self.ui.lock().unwrap_or_else(|e| e.into_inner()).external_species = None;
    }

    fn client_tick_pre(&self, host:&mut dyn ClientHost, _e:&mut mod_api::ClientTickPreEvent) {
        // AttachmentType::get_existing requires a mutable AttachmentStore.
        // Render and mouse hooks get &dyn ClientHost, so read only here.
        let player_id=host.local_player().map(|pl|pl.uuid());
        let local_entity=host.local_player().map(|pl|pl.entity().id());
        if player_id.is_none() {let _=take_hit_inbox();}
        // Only engine-resolved mapping (plus /rustpg fallback) opens the menu.
        // Atomic swap consumes the request; a stale V press cannot open the
        // menu later after another screen is closed.
        let mapped_click=host.consume_click(ID,"open_system");
        let active_click=host.consume_click(ID,"active_relic");
        let parry_click=host.consume_click(ID,"parry");
        let dodge_click=host.consume_click(ID,"dodge");
        let power_fury=host.consume_click(ID,"power_fury");
        let power_barrier=host.consume_click(ID,"power_barrier");
        let power_siphon=host.consume_click(ID,"power_siphon");
        if player_id.is_some() && host.playing() {
            let input=if parry_click {Some("defend:parry")}
                else if dodge_click {Some("defend:dodge")} else {None};
            if let Some(cmd)=input {let _=host.send_payload_to_server(
                &PayloadId("rustpg:action".into()),PayloadBytes(cmd.as_bytes().to_vec()));}
        }
        if player_id.is_some() && host.playing() {
            let power=if power_fury{Some("power:fury")}else if power_barrier{Some("power:barrier")}
                else if power_siphon{Some("power:siphon")}else{None};
            if let Some(cmd)=power {let _=host.send_payload_to_server(
                &PayloadId("rustpg:action".into()),PayloadBytes(cmd.as_bytes().to_vec()));}
        }
        if CLOSE_REQUEST.swap(false,Ordering::AcqRel) && host.mod_screen()==Some(ID){
            host.mod_screen_close(ID);
        }
        let queued=OPEN_REQUEST.swap(false,Ordering::AcqRel);
        if player_id.is_some() && host.playing() && host.mod_screen()!=Some(ID)
            && (mapped_click || queued) {
            let _=host.mod_screen_open(ID);
        }
        let profile=player_id.and_then(|id|
            PROFILE.get_existing(host,Holder::Player(id)).ok().flatten());
        if let (Some(uuid),Some(p))=(player_id,profile.as_ref()) {
            sync_player_stats(uuid,&p.node_stats());
        }
        let active_slot=rustpg_gear::SLOTS.get(host.setting_choice("active_relic_slot")
            .unwrap_or(5) as usize).map(|(slot,_)|*slot);
        let new_mobs=take_mob_inbox().into_iter()
            .filter(|packet|packet.dimension==host.dimension()).last();
        let cached_mobs={
            let ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
            if let Some(ref packet)=new_mobs {
                packet.mobs.iter().map(|s|(s.id,*s)).collect::<HashMap<_,_>>()
            } else if ui.cached_dimension==host.dimension(){ui.mob_stats.clone()}
            else {HashMap::new()}
        };
        // Client SDK reads remote entity position and identity, but NOT remote
        // health; actual HP/armor come from the server's mob_hp packet.
        let observed=match host.crosshair_target() {
            Some(mod_api::CrosshairTarget::Entity {entity,..}) => {
                cached_mobs.get(&entity.id()).map(|stats| {
                    let name=entity.name(host).unwrap_or_else(||"Criatura".into());
                    let kind=entity.kind(host).map(|k|k.id(host)).unwrap_or_default();
                    let tier="Normal".to_string();
                    rustpg_target_hud::TargetSnapshot {
                        entity:entity.id(),kind,name,level:stats.level as u32,
                        rank:tier.clone(),current_hp:stats.hp as f64,
                        max_hp:stats.max_hp as f64,armor:stats.armor as f64,tier,
                        mutations:Vec::new(),badges:Vec::new(),
                    }.sanitized()
                })
            },_=>None,
        };
        // World entity bars: one bounded client scan per 8 ticks (~2.5/s),
        // not an O(entities) query or draw-call generator every frame.
        let refresh_mobs=self.ui.lock().unwrap_or_else(|er|er.into_inner()).tick%8==0;
        let nearby=if refresh_mobs {
            let mut found=Vec::new();
            for stats in cached_mobs.values() {
                if found.len()>=rustpg_mob_wire::MobPacket::CAP{break;}
                let Some(ent)=host.entity(stats.id) else {continue;};
                let Some(at)=ent.position(host) else {continue;};
                let name=ent.name(host).unwrap_or_else(||"Criatura".to_owned());
                let kind=ent.kind(host).map(|k|k.id(host)).unwrap_or_default();
                found.push(NearbyMob {
                    id:stats.id,point:[at[0],at[1]+1.35,at[2]],name,kind,
                    hp:stats.hp,max_hp:stats.max_hp,armor:stats.armor as f64,
                    level:stats.level as u32,tier:"Normal".into(),seen:0,
                });
            }
            Some(found)
        } else {None};
        let vanilla_health=host.local_player().and_then(|pl| {
            let living=host.living(pl.entity())?;
            Some((living.health(host)? as f64,living.max_health(host)? as f64))
        }).unwrap_or((0.0,20.0));
        let hunger=host.local_player().and_then(|pl|pl.food(host)).map_or(20,|f|f.level);
        let (air,max_air)=host.local_player().map(|pl| {
            (pl.entity().air_supply(host).unwrap_or(300),
             pl.entity().max_air_supply(host).unwrap_or(300))
        }).unwrap_or((300,300));
        let (armor,absorption)=host.local_player().and_then(|pl|host.living(pl.entity()))
            .map(|l|(l.attribute_value(host,"minecraft:armor").unwrap_or(0.0),
                     l.absorption(host).unwrap_or(0.0) as f64)).unwrap_or((0.0,0.0));
        let (action,announce)={
            let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
            if ui.local_id!=player_id || ui.cached_dimension!=host.dimension() {
                ui.bestiary_instances.clear();
                ui.bestiary_scan_tick=None;
                ui.cached_dimension=host.dimension().to_string();
                ui.local_id=player_id;
                ui.inspector_scroll=0;ui.inspector_key.clear();ui.inspector_since=ui.tick;
                ui.announced=false;
                ui.pending=None;
                ui.confirm_salvage=None;
                ui.confirm_respec=None;
                ui.target=None;
                ui.target_ttl=0;
                ui.last_hit_victim=None;
                ui.hp_vfx=Default::default();
                ui.stamina_vfx=Default::default();
                ui.xp_vfx=Default::default();
                ui.target_vfx=Default::default();
                ui.target_vfx_entity=None;
                        ui.last_target=None;
                ui.floating.clear();
                ui.nearby_mobs.clear();
                ui.mob_stats.clear();
                ui.damage_meter.clear();
            }
            ui.tick=ui.tick.wrapping_add(1);
            let current_tick=ui.tick;
            ui.profile=profile;
            if let Some(ref packet)=new_mobs {
                ui.mob_stats.clear();
                for stats in &packet.mobs {ui.mob_stats.insert(stats.id,*stats);}
                let active:std::collections::HashSet<u32>=ui.mob_stats.keys().copied().collect();
                ui.nearby_mobs.retain(|mob|active.contains(&mob.id));
            }
            if let Some(list)=nearby {
                // Merge rather than drop snapshots seen in render_living_post.
                // Otherwise an unrelated spatial scan could erase a mob this frame.
                let seen_tick=ui.tick;
                for mut mob in list {
                    mob.seen=seen_tick;
                    if let Some(old)=ui.nearby_mobs.iter_mut().find(|m|m.id==mob.id) { *old=mob; }
                    else if ui.nearby_mobs.len()<24 {ui.nearby_mobs.push(mob);}
                }
            }
            if let Some(ref obs)=observed {
                if !ui.nearby_mobs.iter().any(|m|m.id==obs.entity) {
                    if let Some(ent)=host.entity(obs.entity) {
                        if let Some(at)=ent.position(host) {
                            if ui.nearby_mobs.len()>=20 {ui.nearby_mobs.pop();}
                            ui.nearby_mobs.push(NearbyMob {
                                id:obs.entity,point:[at[0],at[1]+1.35,at[2]],
                                name:obs.name.clone(),kind:obs.kind.clone(),
                                hp:obs.current_hp as f32,max_hp:obs.max_hp as f32,
                                armor:obs.armor,level:obs.level,tier:obs.tier.clone(),seen:current_tick,
                            });
                        }
                    }
                }
            }
            ui.vanilla_health=vanilla_health;
            let hp_ratio=(vanilla_health.0/vanilla_health.1.max(1.0)) as f32;
            ui.hp_vfx.step(hp_ratio);
            if let Some(ref profile)=ui.profile {
                let stamina_ratio=profile.stamina.fraction() as f32;
                let xp_req=rustpg_progression::general_xp_to_next(profile.level).max(1) as f64;
                let xp_ratio=((profile.xp as f64)/xp_req) as f32;
                ui.stamina_vfx.step(stamina_ratio);
                ui.xp_vfx.step(xp_ratio);
            }
            ui.hunger=hunger;
            ui.air=air;ui.max_air=max_air;
            ui.armor=armor;
            ui.absorption=absorption;
            for popup in &mut ui.floating {popup.age=popup.age.saturating_add(1);}
            ui.floating.retain(|p|p.age<30);
            // Damage popups are now fed exclusively by final server damage
            // events; do not infer damage from crosshair HP changes.
            ui.last_target=observed.as_ref().map(|t|(t.entity,t.current_hp));
            for hit in take_hit_inbox() {
                // Never render or count hits received from a different dimension.
                if hit.dimension!=host.dimension() {continue;}
                // If the id is still loaded, it must refer to the same physical
                // entity: reject stale IDs that were recycled for another mob.
                if let Some(ent)=host.entity(hit.hit.victim) {
                    if let Some(at)=ent.position(host) {
                        let dsq=(at[0]-hit.at[0]).powi(2)
                            +(at[1]+1.45-hit.at[1]).powi(2)
                            +(at[2]-hit.at[2]).powi(2);
                        if dsq>24.0*24.0 {continue;}
                        if let Some(stats)=ui.mob_stats.get(&ent.id()).copied() {
                            let name=ent.name(host).unwrap_or_else(||"Criatura".to_string());
                            let kind=ent.kind(host).map(|k|k.id(host)).unwrap_or_default();
                            let point=[at[0],at[1]+1.35,at[2]];
                            if let Some(mob)=ui.nearby_mobs.iter_mut().find(|m|m.id==ent.id()) {
                                mob.point=point;mob.name=name.clone();mob.kind=kind.clone();
                                mob.hp=stats.hp;mob.max_hp=stats.max_hp;
                                mob.armor=stats.armor as f64;mob.seen=current_tick;
                            }else if ui.nearby_mobs.len()<24 {
                                ui.nearby_mobs.push(NearbyMob {
                                    id:ent.id(),point,name:name.clone(),kind:kind.clone(),
                                    hp:stats.hp,max_hp:stats.max_hp,armor:stats.armor as f64,
                                    level:stats.level as u32,tier:"Normal".into(),seen:current_tick,
                                });
                            }
                            if u32::try_from(hit.attacker).ok()==local_entity && local_entity.is_some() {
                                let tier="Normal".to_string();
                                ui.target=Some(rustpg_target_hud::TargetSnapshot {
                                    entity:ent.id(),kind,name,level:stats.level as u32,
                                    rank:tier.clone(),current_hp:stats.hp as f64,
                                    max_hp:stats.max_hp as f64,armor:stats.armor as f64,tier,
                                    mutations:Vec::new(),badges:Vec::new(),
                                }.sanitized());
                                ui.target_ttl=100;
                            }
                        }
                    }
                }
                if host.setting_bool("power_vfx").unwrap_or(true) {
                    rustpg_world_vfx::impact(host,hit.at,
                        hit.family==rustpg_damage_meter::DamageFamily::Critical);
                }
                // Meter counts only hits attributed by the authoritative source
                // to this player, never environmental or other players' hits.
                if u32::try_from(hit.attacker).ok()==local_entity && local_entity.is_some() {
                    // The entity can disappear between server damage and the client tick.
                    // Prefer the last real snapshot instead of showing a blank COMBATE card.
                    if ui.target.as_ref().is_none_or(|t|t.entity!=hit.hit.victim) {
                        // Exact victim ID first. World positions only select a fallback
                        // if a tracked entity is within 3 blocks of the impact.
                        let mob=ui.nearby_mobs.iter().find(|m|m.id==hit.hit.victim)
                            .or_else(||ui.nearby_mobs.iter().filter(|m| {
                                let dx=m.point[0]-hit.at[0];
                                let dy=m.point[1]-hit.at[1];
                                let dz=m.point[2]-hit.at[2];
                                dx*dx+dy*dy+dz*dz<9.0
                            }).next()).cloned();
                        if let Some(mob)=mob {
                            ui.target=Some(rustpg_target_hud::TargetSnapshot {
                                entity:mob.id,kind:mob.kind,name:mob.name,level:mob.level,
                                rank:mob.tier.clone(),current_hp:mob.hp as f64,max_hp:mob.max_hp as f64,
                                armor:mob.armor,tier:mob.tier,mutations:Vec::new(),badges:Vec::new(),
                            }.sanitized());
                        }
                    }
                    if ui.target.is_some() {ui.target_ttl=100;}
                    ui.last_hit_victim=Some((hit.hit.victim,ui.tick));
                    let now=ui.tick;
                    ui.damage_meter.record(now,hit.hit.damage,hit.family);
                }
                ui.floating.push(DamagePopup {
                    point:hit.at,amount:hit.hit.damage as f64,age:0,family:hit.family
                });
            }
            if ui.floating.len()>64 {
                let extra=ui.floating.len()-64;
                ui.floating.drain(..extra);
            }
            if let Some(obs)=observed.clone() {
                ui.target=Some(obs);
                ui.target_ttl=100;
            } else if ui.target_ttl>0 {
                ui.target_ttl=ui.target_ttl.saturating_sub(1);
                if ui.target_ttl==0 { ui.target=None; }
            } else {
                ui.target=None;
            }
            if let Some((_,then))=ui.last_hit_victim {
                if ui.tick.wrapping_sub(then)>100 {ui.last_hit_victim=None;}
            }
            let target_for_anim=ui.target.as_ref().map(|t|(t.entity,t.fraction()));
            if let Some((entity,ratio))=target_for_anim {
                if ui.target_vfx_entity!=Some(entity) {
                    ui.target_vfx=Default::default();
                    ui.target_vfx_entity=Some(entity);
                }
                ui.target_vfx.step(ratio);
            } else {ui.target_vfx_entity=None;}
            let seen_tick=ui.tick;
            ui.nearby_mobs.retain(|mob|seen_tick.wrapping_sub(mob.seen)<=24);
            // No per-entity bar animation work: the combat card has a single
            // target VFX state, and all mob HP remains server-authoritative.
            let announce=player_id.is_some() && host.playing() && !ui.announced;
            if announce {ui.announced=true;}
            if player_id.is_some() && host.playing() && active_click && ui.pending.is_none() {
                if let Some(slot)=active_slot {ui.pending=Some(format!("activate:{slot}"));}
            }
            (ui.pending.take(),announce)
        };
        if announce {
            let keys=["rustpg:gui/ui/panels/ui_panel_main",
                "rustpg:gui/hud/health/hud_health_fill",
                "rustpg:gui/ui/panels/ui_tree_node_available"];
            let loaded=keys.iter().filter(|key|{
                mod_api::Identifier::parse(key).ok()
                    .and_then(|id|host.gui_sprite(&id)).is_some()
            }).count();
            let _=host.chat_line(Text::literal(format!(
                "[RustPG {VERSION_STR}] Cliente conectado. R ou /rustpg; sprites:{loaded}/3; extensoes:{}/1; pacotes de profissao:{}/{}; dano+combo: servidor.",extensions().count(),
                profession_packs().read().unwrap_or_else(|e|e.into_inner()).pack_count(),
                profession_packs().read().unwrap_or_else(|e|e.into_inner()).rule_count())));
        }
        // The mod's own screen is open here; host.playing() can be false
        // even while the player is in a world. Send the queued UI action
        // whenever a local player exists (never from the read-only hook).
        if player_id.is_some() {
            if let Some(action)=action {
                let _=host.send_payload_to_server(
                    &PayloadId("rustpg:action".into()),PayloadBytes(action.into_bytes()));
            }
        }
    }
    // Replace the overlapping vanilla rows, leaving hotbar and launcher controls intact.
    fn gui_layer_pre(&self,host:&dyn ClientHost,e:&mut mod_api::GuiLayerPreEvent<'_>)->Flow {
        if host.playing() && host.setting_bool("hud_enabled").unwrap_or(true)
            && host.setting_bool("replace_vanilla").unwrap_or(true)
            && self.ui.lock().unwrap_or_else(|er|er.into_inner()).profile.is_some()
            && (e.layer=="minecraft:player_health" || e.layer=="minecraft:contextual_info_bar") {
            return Flow::Cancel;
        }
        Flow::Continue
    }
    // Render-state-driven discovery, from mobs the engine actually draws.
    // This complements the bounded client scan and fixes missing head bars
    // when entities_in yields no tracked result during the scan.
    fn render_living_post(&self,host:&dyn ClientHost,
        e:&mut mod_api::RenderLivingPostEvent<'_>) {
        let id=e.entity.id;
        if host.local_player().is_some_and(|pl|pl.entity().id()==id){return;}
        let Some(ent)=host.entity(id) else {return;};
        let Some(at)=ent.position(host) else {return;};
        let name=ent.name(host).unwrap_or_else(||"Criatura".to_owned());
        let kind=ent.kind(host).map(|k|k.id(host)).unwrap_or_default();
        let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        let Some(stats)=ui.mob_stats.get(&id).copied() else{return;};
        let (hp,max_hp,armor)=(stats.hp,stats.max_hp,stats.armor as f64);
        let point=[at[0],at[1]+1.35,at[2]];
        let seen_tick=ui.tick;
        if let Some(mob)=ui.nearby_mobs.iter_mut().find(|m|m.id==id) {
            mob.seen=seen_tick;
            mob.point=point;mob.hp=hp;mob.max_hp=max_hp;
            mob.name=name.clone();mob.kind=kind.clone();mob.armor=armor;
        } else {
            if ui.nearby_mobs.len()>=24 {ui.nearby_mobs.remove(0);}
            ui.nearby_mobs.push(NearbyMob {
                id,point,name:name.clone(),kind:kind.clone(),hp,max_hp,
                armor,level:stats.level as u32,tier:"Normal".into(),seen:seen_tick,
            });
        }
        // A hit can arrive before the living entity becomes client-queryable.
        // Once this entity renders, populate the target card with real HP.
        if ui.last_hit_victim.is_some_and(|(victim,then)|victim==id && ui.tick.wrapping_sub(then)<=100) {
            let (level,tier)=ui.nearby_mobs.iter().find(|m|m.id==id)
                .map(|m|(m.level,m.tier.clone())).unwrap_or((1,"Normal".into()));
            ui.target=Some(rustpg_target_hud::TargetSnapshot {
                entity:id,kind,name,level,rank:tier.clone(),current_hp:hp as f64,
                max_hp:max_hp as f64,armor,tier,mutations:Vec::new(),badges:Vec::new(),
            }.sanitized());
            ui.target_ttl=100;
        }
    }
    fn gui_render_post(&self,host:&dyn ClientHost,e:&mut mod_api::GuiRenderPostEvent<'_>) {
        if e.gui_hidden || !host.playing() || host.mod_screen()==Some(ID)
            || !host.setting_bool("hud_enabled").unwrap_or(true) {return;}
        let ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        let Some(ref p)=ui.profile else {return;};
        let draw=&mut *e.batch;
        let [w,h]=draw.size();
        if w<260.0 || h<150.0 {return;}
        // Native RGB atlas sprites are trimmed, so their active bounds map
        // exactly to the progress ratio, without stretching transparent 512px margins.
        // HP and stamina remain left; XP uses the vanilla hotbar position.
        let requested=host.setting_f64("hud_scale").unwrap_or(1.0) as f32;
        let offset=host.setting_f64("hud_y_offset").unwrap_or(0.0) as f32;
        let Some(bounds)=rustpg_hud_geometry::compute(w,h,requested,offset) else {return;};
        let (bx,by,bw,bh,scale)=(bounds.x,bounds.y,bounds.w,bounds.h,bounds.scale);
        // LEFT HUD: only the trimmed individual bars, never a large rectangle.
        // Leave the world transparent between the health/stamina/XP elements.
        let rank=rustpg_engine_presentation::profile_rank(p);
        draw.text(&format!("LV {} · {} · {} PT",p.level,rank,p.points),
            [bx+7.0*scale,by+3.0*scale],0xFFCAEEFF,true);
        let exhausted=p.stamina.exhausted;
        if exhausted {
            draw.text("EXAUSTO",[bx+bw-62.0*scale,by+3.0*scale],0xFFFFAA55,true);
        }
        let hp=ui.vanilla_health;
        let hp_rate=if hp.0.is_finite() && hp.1.is_finite() {
            (hp.0/hp.1.max(1.0)).clamp(0.0,1.0) as f32
        } else {0.0};
        let vig_rate=p.stamina.fraction() as f32;
        let xp_need=rustpg_progression::general_xp_to_next(p.level).max(1) as f64;
        let xp_rate=((p.xp as f64)/xp_need).clamp(0.0,1.0) as f32;
        let bar_w=rustpg_hud_geometry::VITAL_BAR_WIDTH*scale;
        let bar_h=rustpg_hud_geometry::hp_stamina_height(scale);
        for (i,(name,fill,frame,ratio,value,fx)) in [
            ("HP","rustpg:gui/hud/health/hud_health_fill_trim","rustpg:gui/hud/health/hud_health_frame_trim",hp_rate,
                if ui.absorption>0.0 {format!("{:.0}/{:.0}+{:.0}",hp.0,hp.1,ui.absorption)}
                else {format!("{:.0}/{:.0}",hp.0,hp.1)},&ui.hp_vfx),
            ("SP","rustpg:gui/hud/stamina/hud_stamina_fill_trim","rustpg:gui/hud/stamina/hud_stamina_frame_trim",vig_rate,
                format!("{:.0}/{:.0}",p.stamina.current,p.stamina.max),&ui.stamina_vfx),
        ].iter().enumerate() {
            let y=by+(14.0+i as f32*19.0)*scale; // keep touch/text lanes while bars become slim
            draw.text(name,[bx+6.0*scale,y],0xFFE7F5FF,true);
            let x=bx+25.0*scale;
            // Cropped image bounds from original 512x64 assets.
            // Original frame bbox x=3..509, y=6..58 (506x52).
            // Original fill bbox x=22..490, y=20..44 (468x24).
            let pad_x=19.0/506.0*bar_w;
            let pad_y=14.0/52.0*bar_h;
            let inner_w=bar_w-2.0*pad_x;
            let inner_h=24.0/52.0*bar_h;
            draw.fill([x+pad_x,y+pad_y,inner_w,inner_h],0xFF102330);
            let ratio=if fx.initialized {fx.display} else {ratio.clamp(0.0,1.0)};
            // Draw the damage/heal afterimage behind the real fill.
            draw_animated_bar_fx(draw,x+pad_x,y+pad_y,inner_w,inner_h,fx,
                if i==0 {0xFFE96975} else {0xFF72B9F4});
            if ratio>0.0 {
                let _=draw.sprite(fill,[0.0,0.0,468.0*ratio,24.0],
                    [x+pad_x,y+pad_y,inner_w*ratio,inner_h],None);
            }
            let _=draw.sprite(frame,[0.0,0.0,506.0,52.0],
                [x,y,bar_w,bar_h],None);
            draw.text(value,[x+bar_w+4.0*scale,y-2.0*scale],0xFFEDF6FF,true);
        }
        // Survival indicators remain visible when vanilla hearts, oxygen and
        // food layer is replaced. Use native values, not custom phantom stats.
        let hy=by+55.0*scale;
        let hx=bx+8.0*scale;
        let hunger_ratio=(ui.hunger as f32/20.0).clamp(0.0,1.0);
        draw.text("FOME",[hx,hy-1.0*scale],0xFFF7D58E,true);
        draw.fill([hx+33.0*scale,hy+2.0*scale,54.0*scale,4.0*scale],0xFF392E28);
        draw.fill([hx+33.0*scale,hy+2.0*scale,54.0*scale*hunger_ratio,4.0*scale],0xFFF4BD68);
        let _=draw.sprite("rustpg:gui/icons/attributes/icon_defense",
            [0.0,0.0,128.0,128.0],[bx+99.0*scale,hy-3.0*scale,12.0*scale,12.0*scale],None);
        draw.text(&format!("{:.0}",ui.armor.max(0.0)),
            [bx+112.0*scale,hy-1.0*scale],0xFFB9D7E4,true);
        if ui.max_air>0 && ui.air<ui.max_air {
            let _=draw.sprite("rustpg:gui/icons/elements/icon_air",
                [0.0,0.0,128.0,128.0],[bx+150.0*scale,hy-3.0*scale,12.0*scale,12.0*scale],None);
            draw.text(&format!("{}%",(ui.air.max(0)*100/ui.max_air.max(1)).clamp(0,100)),
                [bx+163.0*scale,hy-1.0*scale],0xFF84D6FF,true);
        }
        // Active relic status is integrated into the SAME HUD header.
        // A separate rectangular overlay above the hotbar duplicated the UI.
        if host.setting_bool("power_hud").unwrap_or(true) {
            let selected=host.setting_choice("active_relic_slot").unwrap_or(5);
            if let Some((slot,_))=rustpg_gear::SLOTS.get(selected as usize) {
                if let Some(g)=p.gear.worn.get(*slot) {
                    let status=p.active_relics.status(g,now_seconds());
                    let icon=match g.motif {
                        0=>"rustpg:gui/hud/buffs/hud_buff_fortify",
                        1=>"rustpg:gui/hud/buffs/hud_buff_regeneration",
                        2=>"rustpg:gui/hud/buffs/hud_buff_focus",
                        3=>"rustpg:gui/hud/buffs/hud_buff_regeneration",
                        4=>"rustpg:gui/hud/buffs/hud_buff_haste",
                        _=>"rustpg:gui/icons/elements/icon_stamina",
                    };
                    let _=draw.sprite(icon,[0.0,0.0,128.0,128.0],
                        [bx+bw-33.0*scale,by+2.0*scale,11.0*scale,11.0*scale],None);
                    draw.text(if status.starts_with("ATIVO") {"ON"}
                        else if status=="ATIVAR" {"H"} else {"CD"},
                        [bx+bw-20.0*scale,by+4.0*scale],
                        if status.starts_with("ATIVO") {0xFF79F5BA}else{0xFFE4EFFA},true);
                }
            }
        }
        if exhausted {
            let progress=p.stamina.recovery_progress() as f32;
            draw.fill([bx+25.0*scale,by+69.0*scale,104.0*scale,2.0*scale],0xFF172E44);
            draw.fill([bx+25.0*scale,by+69.0*scale,104.0*scale*progress,2.0*scale],0xFF4D9BEE);
        }
        // RPG XP occupies the vanilla XP slot, just above the hotbar.
        // The vanilla contextual_info_bar layer is hidden when replace_vanilla is enabled.
        if host.setting_bool("replace_vanilla").unwrap_or(true) {
            let xp_w=(w-18.0).min(216.0);
            let xp_h=rustpg_hud_geometry::XP_BAR_HEIGHT * scale;
            let xp_x=(w-xp_w)*0.5;
            let xp_y=(h-24.0-xp_h).max(8.0);
            let fill_x=xp_x+xp_w*19.0/506.0;
            let fill_w=xp_w*468.0/506.0;
            let fill_y=xp_y+xp_h*14.0/52.0;
            let fill_h=xp_h*24.0/52.0;
            draw.fill([fill_x,fill_y,fill_w,fill_h],0xBB0C1C20);
            draw_animated_bar_fx(draw,fill_x,fill_y,fill_w,fill_h,&ui.xp_vfx,0xFFFFE09B);
            let progress=if ui.xp_vfx.initialized {ui.xp_vfx.display}else{xp_rate};
            if progress>0.0 {
                let _=draw.sprite("rustpg:gui/hud/experience/hud_experience_fill_trim",
                    [0.0,0.0,468.0*progress,24.0],
                    [fill_x,fill_y,fill_w*progress,fill_h],None);
            }
            let _=draw.sprite("rustpg:gui/hud/experience/hud_experience_frame_trim",
                [0.0,0.0,506.0,52.0],[xp_x,xp_y,xp_w,xp_h],None);
            if h>=185.0 {
                draw.text(&format!("LV {} · {:.0}%",p.level,xp_rate*100.0),
                    [xp_x+xp_w*0.5-22.0,xp_y-10.0],0xFFE9D990,true);
            }
        }
        // One responsive combat surface owns target, boss and damage telemetry.
        // The world remains visible behind it; hierarchy comes from rails,
        // typography and bar geometry rather than a floating opaque card.
        let meter_active=host.setting_bool("damage_meter").unwrap_or(true)
            && ui.damage_meter.active(ui.tick);
        let mob_hud=host.setting_bool("mob_hud").unwrap_or(true);
        let external_focus=if mob_hud {
            match host.crosshair_target() {
                Some(mod_api::CrosshairTarget::Entity{entity,..})
                    if !ui.target.as_ref().is_some_and(|t|t.entity==entity.id()) => {
                    entity.kind(host).map(|k|k.id(host))
                        .filter(|id|!id.starts_with("minecraft:")&&!id.starts_with("rustpg:"))
                        .map(|kind|(entity,kind))
                }
                _=>None,
            }
        }else{None};
        let target_active=mob_hud&&(external_focus.is_some()||ui.target.is_some());

        // Both boss and ordinary target cards use the game's actual entity
        // renderer, not a sprite guessed from an entity name.
        let mut target_bottom=7.0;
        if target_active {
            if let Some((entity,kind))=external_focus {
                if let Some(card)=rustpg_boss_ui::card_layout(w,false) {
                    let (x,y,cw,ch)=(card.x,card.y,card.w,card.h);
                    target_bottom=y+ch;
                    draw.fill([x+2.0,y,cw-2.0,ch],0x8210171F);
                    draw.fill([x,y+3.0,2.0,ch-6.0],rustpg_ui_theme::AETHER);
                    let view=GuiEntityView::new(card.portrait,12.0,0.02,
                        GuiEntityLook::Turned{yaw:(ui.tick%360) as f32,pitch:0.0});
                    if !draw_species_model(draw,host,&kind,Some(entity.id()),view) {
                        draw.text("SEM 3D",[card.portrait[0],y+20.0],
                            rustpg_ui_theme::MUTED,false);
                    }
                    let name=host.translate(&format!("entity.{}",kind.replace([':', '/'],".")))
                        .or_else(||entity.name(host)).unwrap_or_else(||humanize_entity_id(&kind));
                    draw.text(&rustpg_ui_system::fit_word(&name,card.text_w),
                        [card.text_x,y+5.0],rustpg_ui_theme::WHITE,true);
                    draw.text("ENTIDADE EXTERNA",[card.text_x,y+18.0],
                        rustpg_ui_system::alpha(rustpg_ui_theme::MUTED,0xC0),false);
                    draw.text(&rustpg_ui_system::fit("HP NÃO SINCRONIZADO",card.text_w),
                        [card.text_x,y+40.0],rustpg_ui_theme::MUTED,false);
                }
            } else if let Some(t)=ui.target.as_ref() {
                let palette=rustpg_boss_ui::style(&t.kind,&t.tier,t.fraction(),ui.tick);
                let boss=palette.boss;
                if let Some(card)=rustpg_boss_ui::card_layout(w,boss) {
                    let (x,y,cw,ch)=(card.x,card.y,card.w,card.h);
                    target_bottom=y+ch;
                    draw.fill([x+2.0,y,cw-2.0,ch],
                        if boss{0xA10A0D13}else{0x8210171F});
                    draw.fill([x,y+3.0,2.0,ch-6.0],palette.border);
                    if boss {
                        draw.fill([x+8.0,y,cw*0.25,1.0],
                            rustpg_ui_system::alpha(palette.border,0xD0));
                        draw.fill([x+cw*0.75,y+ch-1.0,cw*0.20,1.0],
                            rustpg_ui_system::alpha(rustpg_ui_theme::GOLD,0x70));
                    }
                    // The dedicated boss portrait uses its *original*
                    // RustPG mesh-derived texture. This Starling SDK routes
                    // GuiEntity previews through vanilla renderers, so native
                    // custom models use an accurate static portrait until
                    // the engine exposes custom renderers for GUI previews.
                    let size=if boss{20.0}else{13.0};
                    let view=GuiEntityView::new(card.portrait,size,0.02,
                        GuiEntityLook::Turned{yaw:(ui.tick%360) as f32,pitch:0.0});
                    if !draw_species_model(draw,host,&t.kind,Some(t.entity),view) {
                        draw.text("SEM 3D",[card.portrait[0],y+20.0],
                            rustpg_ui_theme::MUTED,false);
                    }
                    let sx=card.text_x;
                    let sw=card.text_w;
                    draw.text(&rustpg_ui_system::fit_word(&t.name.to_uppercase(),sw),
                        [sx,y+5.0],rustpg_ui_theme::WHITE,true);
                    // BossPalette.phase is an HP colour band, not a real
                    // gameplay phase. Never invent phase transitions in HUD.
                    // One compact metadata row: a separate tier line would
                    // overlap the subtitle on a 60px Android boss card.
                    let sub=if boss{
                        format!("CHEFE · {} · NV {} · {}",t.rank,t.level,t.tier)
                    }else{
                        format!("{} · NV {} · {}",t.rank,t.level,t.tier)
                    };
                    draw.text(&rustpg_ui_system::fit(&sub,sw),
                        [sx,y+if boss{rustpg_boss_ui::BOSS_SUBTITLE_LINE_Y}else{18.0}],
                        rustpg_ui_system::alpha(rustpg_ui_theme::MUTED,0xD0),false);
                    let hp_y=if boss{y+rustpg_boss_ui::BOSS_HEALTH_BAR_Y}else{y+31.0};
                    let hp_h=if boss{rustpg_boss_ui::BOSS_HEALTH_BAR_H}else{6.0};
                    draw.fill([sx,hp_y,sw,hp_h],0xCC070A0E);
                    let health=t.fraction();
                    if health>0.0 {
                        draw.fill([sx,hp_y,sw*health,hp_h],palette.fill);
                        draw.fill([sx,hp_y,sw*health,1.0],
                            rustpg_ui_system::alpha(
                                rustpg_ui_system::lighten(palette.fill,0.45),0xD0));
                    }
                    if boss {
                        for i in 1..palette.segments {
                            let cut=sx+sw*f32::from(i)/f32::from(palette.segments);
                            draw.fill([cut,hp_y,1.0,hp_h],0xD50A0D13);
                        }
                        let a=(0x35u32+(palette.pulse*65.0) as u32).min(0x90);
                        draw.fill([sx,hp_y-1.0,sw,1.0],
                            (a<<24)|(palette.border&0x00FF_FFFF));
                    }
                    let hp_label=format!("{:.0}/{:.0} · {:.0}%",
                        t.current_hp,t.max_hp,(health*100.0).clamp(0.0,100.0));
                    draw.text(&rustpg_ui_system::fit(&hp_label,sw),
                        [sx,if boss{y+rustpg_boss_ui::BOSS_HEALTH_LABEL_Y}else{y+42.0}],
                        rustpg_ui_theme::MOON,true);
                }
            }
        }
        if meter_active {
            // The last DAMAGE VICTIM is the source of truth for the damage
            // panel, not whichever unrelated animal is under the crosshair.
            let victim=ui.last_hit_victim.and_then(|(id,at)|
                (ui.tick.wrapping_sub(at)<=
                    rustpg_damage_meter::COMBO_GAP_TICKS+
                    rustpg_damage_meter::SUMMARY_TICKS).then_some(id));
            let identified=victim.and_then(|id| {
                host.entity(id).and_then(|entity|
                    entity.kind(host).map(|k|(id,k.id(host))))
                .or_else(||ui.nearby_mobs.iter().find(|mob|mob.id==id)
                    .map(|mob|(id,mob.kind.clone())))
            }).filter(|(_,kind)|rustpg_inspector::valid_kind(kind));
            let scale=host.setting_f64("damage_meter_scale").unwrap_or(1.0) as f32;
            let available=((h-target_bottom-12.0)/46.0).max(0.75);
            if let Some([dx,dy,dw,dh])=rustpg_boss_ui::damage_card(w,target_bottom,
                scale.min(available)) {
                // A thin translucent identity rail; the rest of the game
                // remains unobscured. The old large damage panel is removed.
                draw.fill([dx+2.0,dy,dw-2.0,dh],0x7610171F);
                draw.fill([dx,dy+3.0,2.0,dh-6.0],rustpg_ui_theme::GOLD);
                let s=dh/46.0;
                let preview_w=if identified.is_some(){(33.0*s).min(dw*0.25)}else{0.0};
                let tx=dx+6.0+preview_w+if preview_w>0.0{5.0}else{0.0};
                let tw=(dx+dw-6.0-tx).max(25.0);
                if let Some((id,kind))=identified {
                    let view=GuiEntityView::new(
                        [dx+4.0,dy+3.0,dx+4.0+preview_w,dy+dh-3.0],
                        12.0,0.02,GuiEntityLook::Turned{
                            yaw:(ui.tick%360) as f32,pitch:0.0});
                    if !draw_species_model(draw,host,&kind,Some(id),view){
                        draw.text("3D?",[dx+6.0,dy+16.0],
                            rustpg_ui_theme::MUTED,false);
                    }
                }
                draw.text(&rustpg_ui_system::fit(
                    &format!("{} DANO",rustpg_damage_meter::display_damage(ui.damage_meter.total())),tw),
                    [tx,dy+5.0*s],rustpg_ui_theme::GOLD,true);
                draw.text(&rustpg_ui_system::fit(
                    &format!("{}x COMBO",ui.damage_meter.combo()),tw),
                    [tx,dy+17.0*s],rustpg_ui_theme::WHITE,true);
                if host.setting_bool("damage_meter_list").unwrap_or(true) {
                    if let Some(hit)=ui.damage_meter.recent().last() {
                        draw.text(&rustpg_ui_system::fit(
                            &format!("{} {}",hit.family.label(),rustpg_damage_meter::display_damage(hit.amount as f64)),tw),
                            [tx,dy+30.0*s],hit.family.color(),false);
                    }
                }
            }
        }
        if host.setting_bool("damage_numbers").unwrap_or(true) {
            for (i,pop) in ui.floating.iter().enumerate() {
                // 3D world position projected to 2D HUD coordinates. Works for
                // mobs anywhere in the camera frustum, not only under crosshair.
                let Some([nx,ny,_depth])=host.project(pop.point) else {continue;};
                if nx.abs()>1.05 || ny.abs()>1.05 {continue;}
                let drift=pop.age as f32*0.75;
                let x=(nx+1.0)*0.5*w+(i as f32%3.0-1.0)*7.0;
                let y=(1.0-ny)*0.5*h-drift;
                let remaining=30_u32.saturating_sub(pop.age);
                let a=(remaining.min(20)*255/20).min(255);
                let tint=pop.family.color() & 0x00FFFFFF;
                let color=(a<<24)|tint;
                let prefix=match pop.family {
                    rustpg_damage_meter::DamageFamily::Critical=>"★ ",
                    rustpg_damage_meter::DamageFamily::Wither=>"WITHER ",
                    rustpg_damage_meter::DamageFamily::Fire=>"FOGO ",
                    rustpg_damage_meter::DamageFamily::Poison=>"VENENO ",
                    _=>"",
                };
                let label=format!("{}-{:.1}",prefix,pop.amount);
                draw.text(&label,[x+1.0,y+1.0],(a<<24)|0x00101820,false);
                draw.text(&label,[x,y],color,true);
                // Event-driven, deterministic sparkle VFX; no extra network traffic.
                let sparks=if pop.family==rustpg_damage_meter::DamageFamily::Critical {6}else{2};
                let t=pop.age as f32;
                let sparkle=rustpg_timeline::spark(pop.age);
                for j in 0..sparks {
                    let angle=(j as f32*6.2831853/sparks as f32)+i as f32*0.77;
                    let radius=(2.0+t*0.28).min(12.0)*sparkle;
                    let px=x+angle.cos()*radius;
                    let py=y+angle.sin()*radius*0.64;
                    let size=if j%2==0 {2.0}else{1.0};
                    draw.fill([px,py,size,size],color);
                }
            }
        }
        // The combat telemetry is included in the unified target card above.
    }
    fn screen_render_post(&self, host:&dyn ClientHost, e:&mut ScreenRenderPostEvent<'_>) {
        if host.mod_screen()!=Some(ID) {return;}
        let [w,h]=e.draw.size();
        let area=Rect {x:4.0,y:4.0,w:w-8.0,h:h-8.0};
        let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        if rustpg_view::PAGES.get(ui.page).is_some_and(|(kind,_)| *kind=="bestiary") {
            refresh_bestiary(host,&mut ui);
        }
        let fallback=Profile::new();
        let p=ui.profile.as_ref().unwrap_or(&fallback);
        let m=menu_model(p,&ui,area,e.screen.mouse);
        let inspected=rustpg_inspector::inspect(&m);
        if inspected.key!=ui.inspector_key {
            ui.inspector_key=inspected.key.clone();
            ui.inspector_scroll=0;
            ui.inspector_since=ui.tick;
        }
        let inspector_scroll=ui.inspector_scroll;
        let typewriter_age=ui.tick.wrapping_sub(ui.inspector_since);
        let inspector_live_entity=inspected.entity_kind.as_ref().and_then(|kind|
            ui.bestiary_instances.get(kind).copied().or_else(|| ui.nearby_mobs.iter().find(|m|&m.kind==kind).map(|m|m.id)));
        drop(ui);
        let layer=rustpg_ui_window::draw_menu(&m);
        // Draw only within the recomputed left viewport. Native sprites retain
        // their proportions while an independent sidebar owns all descriptions.
        draw_chrome(&mut *e.draw,m.area,&m);
        draw_layer(&mut *e.draw,&layer);
        draw_assets(&mut *e.draw,m.area,&m,host);
        if let Some(side)=rustpg_inspector::inspector_area(area) {
            let inspector=rustpg_inspector::presentation(side,&inspected,inspector_scroll,typewriter_age);
            draw_layer(&mut *e.draw,&inspector.layer);
            if let (Some(kind),Some(rect))=(inspector.preview_kind,inspector.preview_rect) {
                // Use the game’s GUI preview for the selected registry entry.
                let size=if kind.contains("dragon")||kind.contains("wither"){18.0}
                    else if kind.contains("golem")||kind.contains("ravager"){25.0}
                    else{35.0};
                let angle=(m.tick%360) as f32;
                let view=GuiEntityView::new(rect,size,0.02,
                    GuiEntityLook::Turned{yaw:angle,pitch:0.0});
                if !draw_species_model(&mut *e.draw,host,&kind,inspector_live_entity,view) {
                    e.draw.text("Sem prévia",[rect[0]+2.0,rect[1]+2.0],0xFFB1C9D9,true);
                }
            }
        }
    }
    fn screen_mouse_scrolled_pre(&self,host:&dyn ClientHost,
        e:&mut ScreenMouseScrolledPreEvent<'_>)->Flow {
        if host.mod_screen()!=Some(ID) {return Flow::Continue;}
        let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        let full=Rect{x:4.0,y:4.0,w:e.screen.rect[2].max(200.0)-8.0,
            h:e.screen.rect[3].max(150.0)-8.0};
        if rustpg_inspector::inspector_area(full)
            .is_some_and(|r|r.contains(e.screen.mouse)) {
            if e.scroll[1]<0.0 {ui.inspector_scroll=ui.inspector_scroll.saturating_add(2).min(1024);}
            else if e.scroll[1]>0.0 {ui.inspector_scroll=ui.inspector_scroll.saturating_sub(2);}
            return Flow::Cancel;
        }
        let (kind,_)=rustpg_view::PAGES[ui.page.min(rustpg_view::PAGES.len()-1)];
        if kind=="bestiary" {
            let len=ui.profile.as_ref().map_or(0,|p|bestiary_ids(p,&ui).len());
            if e.scroll[1]<0.0 {ui.bestiary_sel=ui.bestiary_sel.saturating_add(1).min(len.saturating_sub(1));}
            else if e.scroll[1]>0.0 {ui.bestiary_sel=ui.bestiary_sel.saturating_sub(1);}
            return Flow::Cancel;
        }
        if kind=="trees" {
            let fallback=Profile::new();
            let profile=ui.profile.as_ref().unwrap_or(&fallback);
            let rect=Rect{x:4.0,y:4.0,w:e.screen.rect[2].max(200.0)-8.0,
                h:e.screen.rect[3].max(150.0)-8.0};
            let m=menu_model(profile,&ui,rect,e.screen.mouse);
            let (g,detail)=rustpg_ui_window::constellation_regions(&m);
            if detail.contains(e.screen.mouse) {
                if e.scroll[1]<0.0 {ui.detail_scroll=ui.detail_scroll.saturating_add(2).min(512);}
                else if e.scroll[1]>0.0 {ui.detail_scroll=ui.detail_scroll.saturating_sub(2);}
            } else if ui.tree_index {
                if e.scroll[1]<0.0 {ui.scroll=ui.scroll.saturating_add(1)
                    .min(rustpg_ui_window::constellation_pages(&m).saturating_sub(1));}
                else if e.scroll[1]>0.0 {ui.scroll=ui.scroll.saturating_sub(1);}
            } else if g.contains(e.screen.mouse) && e.scroll[1]!=0.0 {
                let pages=rustpg_ui_window::constellation_pages(&m);
                if e.scroll[1]<0.0 {ui.scroll=ui.scroll.saturating_add(1).min(pages.saturating_sub(1));}
                else {ui.scroll=ui.scroll.saturating_sub(1);}
            }
        } else if e.scroll[1]!=0.0 {
            let fallback = Profile::new();
            let m = menu_model(ui.profile.as_ref().unwrap_or(&fallback), &ui, full, e.screen.mouse);
            let last = rustpg_ui_window::page_count(&m).saturating_sub(1);
            let page = ui.scroll.min(last);
            ui.scroll = if e.scroll[1]<0.0 {page.saturating_add(1).min(last)} else {page.saturating_sub(1)};
        }
        Flow::Cancel
    }
    fn screen_mouse_dragged_pre(&self,host:&dyn ClientHost,
        _e:&mut ScreenMouseDraggedPreEvent<'_>)->Flow {
        if host.mod_screen()!=Some(ID) {return Flow::Continue;}
        // The 5.3 skill tree is paged: no free-pan gesture, no jumpy camera.
        Flow::Continue
    }
    fn screen_mouse_button_pressed_pre(&self,host:&dyn ClientHost,
        e:&mut ScreenMouseButtonPressedPreEvent<'_>)->Flow {
        if host.mod_screen()!=Some(ID) || e.button!=0 {return Flow::Continue;}
        let [w,h]=e.screen.rect[2..4].try_into().unwrap_or([400.0,250.0]);
        let area=Rect {x:4.0,y:4.0,w:w.max(200.0)-8.0,h:h.max(150.0)-8.0};
        let mut ui=self.ui.lock().unwrap_or_else(|er|er.into_inner());
        // Clone to allow menu actions to mutate ui while reading the profile.
        let p=ui.profile.clone().unwrap_or_else(Profile::new);
        let m=menu_model(&p,&ui,area,e.screen.mouse);
        if let Some(side)=rustpg_inspector::inspector_area(area) {
            if rustpg_inspector::buy_button(side).contains(e.screen.mouse) {
                if let Some(n)=m.tree.as_ref().and_then(|t|t.nodes.get(t.selected)) {
                    if n.buyable {ui.pending=Some(format!("buy:{}",n.id));}
                    return Flow::Cancel;
                }
            }
        }
        if let Some((_,a))=rustpg_ui_window::menu_hits(&m).into_iter().find(|(r,_)|r.contains(e.screen.mouse)) {
            match a.split_once(':') {
                Some(("page",i)) => {ui.page=i.parse::<usize>().unwrap_or(0).min(rustpg_view::PAGES.len()-1);ui.scroll=0;},
                Some(("rpage",n)) => {
                    let last = rustpg_ui_window::page_count(&m).saturating_sub(1);
                    let page = ui.scroll.min(last);
                    ui.scroll = if n=="1" {page.saturating_add(1).min(last)} else {page.saturating_sub(1)};
                },
                Some(("species",i)) => ui.bestiary_sel=i.parse().unwrap_or(0),
                Some(("tgroup",d)) => {
                    let groups=rustpg_ui_window::tab_groups(rustpg_view::PAGES.len());
                    let group=rustpg_ui_window::tab_group(ui.page);
                    // The tab strip dims unavailable arrows; navigation
                    // must agree with that visual disabled state.
                    let next=if d=="1" {(group+1).min(groups.saturating_sub(1))}
                        else {group.saturating_sub(1)};
                    ui.page=(next*rustpg_ui_window::TAB_SLOTS).min(rustpg_view::PAGES.len()-1);
                    ui.scroll=0;
                },
                Some(("treecat",i)) => {
                    let n=rustpg_nodes::trees().len();
                    if let Ok(i)=i.parse::<usize>() {
                        if i<n {ui.tree_id=i;ui.node_sel=0;ui.scroll=0;
                            ui.detail_scroll=0;ui.tree_index=false;ui.tree_detail_only=false;}
                    }
                },
                Some(("tree",d)) => {
                    let len=rustpg_nodes::trees().len().max(1);
                    ui.tree_id=if d=="1" {(ui.tree_id+1)%len}
                        else {(ui.tree_id+len-1)%len};
                    ui.node_sel=0;ui.scroll=0;ui.detail_scroll=0;ui.tree_detail_only=false;
                },
                Some(("node",i)) => {
                    ui.node_sel=i.parse().unwrap_or(0);
                    ui.detail_scroll=0;
                },
                Some(("nsel",d)) => {
                    let len=m.tree.as_ref().map_or(1,|t|t.nodes.len()).max(1);
                    ui.node_sel=if d=="1" {(ui.node_sel+1)%len}
                        else {(ui.node_sel+len-1)%len};
                    ui.detail_scroll=0;
                },
                Some(("equip",id)) => {
                    if let Ok(id)=id.parse::<u64>() {
                        if p.gear.bag.iter().any(|g|g.uid==id) {
                            ui.pending=Some(format!("equip:{id}"));
                        }
                    }
                },
                Some(("activate",slot)) => {
                    if p.gear.worn.contains_key(slot) {
                        ui.pending=Some(format!("activate:{slot}"));
                    }
                },
                Some(("unequip",slot)) => {
                    if p.gear.worn.contains_key(slot) {
                        ui.pending=Some(format!("unequip:{slot}"));
                    }
                },
                Some(("salvage",id)) => {
                    if let Ok(id)=id.parse::<u64>() {
                        if p.gear.bag.iter().any(|g|g.uid==id) {
                            match ui.confirm_salvage {
                                Some((selected,at)) if selected==id && ui.tick.wrapping_sub(at)<100 => {
                                    ui.pending=Some(format!("salvage:{id}"));
                                    ui.confirm_salvage=None;
                                },
                                _=>ui.confirm_salvage=Some((id,ui.tick)),
                            }
                        }
                    }
                },
                None if a=="close" => {
                    CLOSE_REQUEST.store(true,Ordering::Release);
                },
                None if a=="respec" => {
                    match ui.confirm_respec {
                        Some(at) if ui.tick.wrapping_sub(at)<100 => {
                            ui.pending=Some("respec:confirm".into());
                            ui.confirm_respec=None;
                        },
                        _=>ui.confirm_respec=Some(ui.tick),
                    }
                },
                None if a=="buy" => {
                    if let Some(tree)=m.tree.as_ref() {
                        if let Some(node)=tree.nodes.get(tree.selected).filter(|n|n.buyable) {
                            ui.pending=Some(format!("buy:{}",node.id));
                        }
                    }
                },
                None if a=="catalog" => {
                    ui.tree_index=!ui.tree_index;
                    ui.tree_detail_only=false;
                    ui.scroll=0;ui.detail_scroll=0;
                },
                Some(("mode",m)) => {ui.tree_index=m=="index";ui.scroll=0;},
                _=>{}, // Unsupported input is intentionally ignored.
            }
            if m.kind=="trees" && a.starts_with("rpage:") && !ui.tree_index {
                let new_model=menu_model(&p,&ui,area,e.screen.mouse);
                if let Some((_,i))=rustpg_ui_window::constellation_node_rects(&new_model).first() {
                    ui.node_sel=*i;ui.detail_scroll=0;
                }
            }
        }
        Flow::Cancel
    }
}

#[cfg(test)]
mod v405_regression {
    use super::*;
    #[test]
    fn relic_api_registered_and_speed_modifier_has_owned_id() {
        assert_eq!(relics().count(),6);
        assert_eq!(rustpg_native_modifiers::FATIGUE_MODIFIER_ID,
            "rustpg:exhausted_movement");
        assert_eq!(rustpg_native_modifiers::bonus_health(21.0),1.0);
    }
}


#[cfg(test)]
mod external_entity_tests {
    use super::*;

    #[test]
    fn external_registry_entries_are_visible_before_a_kill_and_merge_by_full_id() {
        let mut p = Profile::new();
        let mut ui = Ui::default();
        ui.page = rustpg_view::PAGES.iter().position(|(id,_)| *id=="bestiary").unwrap();
        let kind = "other-mod:forest/sentinel";
        let text = entity_text(kind, |key| match key {
            "entity.other-mod.forest.sentinel" => Some("Sentinela".into()),
            "entity.other-mod.forest.sentinel.description" => Some("Protege a floresta.".into()),
            _ => None,
        });
        ui.external_species = Some(BTreeMap::from([(kind.into(),text),
            ("another:forest/sentinel".into(),EntityText::default())]));
        ui.bestiary_instances.insert(kind.into(),91);
        let area = Rect{x:0.0,y:0.0,w:520.0,h:320.0};
        let m = menu_model(&p,&ui,area,[0.0;2]);
        let row = m.bestiary.iter().find(|s|s.name==kind).unwrap();
        assert_eq!((row.display_name.as_str(),row.description.as_str()),("Sentinela","Protege a floresta."));
        assert_eq!((row.kills,row.live_entity),(0,Some(91)));
        p.species.insert(kind.into(),rustpg_bestiary::SpeciesRecord{kind:kind.into(),kills:2,..Default::default()});
        let m = menu_model(&p,&ui,area,[0.0;2]);
        assert_eq!(m.bestiary.len(),2);
        assert_eq!(m.bestiary.iter().find(|s|s.name==kind).unwrap().kills,2);
    }
}
