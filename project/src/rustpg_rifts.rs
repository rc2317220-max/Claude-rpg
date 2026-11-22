//! Instanced Rift/Dungeon domain for RustPG 6.0.
//! Inspired by the uploaded Solo Leveling mod's portal/room state machines,
//! rewritten as an original deterministic Rust system for Horizon runtime levels.
use std::collections::BTreeSet;

pub const RESONANCE_READY:u32=100;
pub const PORTAL_LIFETIME_TICKS:u64=20*60*5;

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub enum RiftRank{E,D,C,B,A,S}
impl RiftRank{
    pub const ALL:[Self;6]=[Self::E,Self::D,Self::C,Self::B,Self::A,Self::S];
    pub fn index(self)->u32{match self{Self::E=>0,Self::D=>1,Self::C=>2,Self::B=>3,Self::A=>4,Self::S=>5}}
    pub fn name(self)->&'static str{match self{Self::E=>"E",Self::D=>"D",Self::C=>"C",Self::B=>"B",Self::A=>"A",Self::S=>"S"}}
    pub fn expected_ticks(self)->u64{(6+2*self.index() as u64)*60*20}
    pub fn rooms(self)->usize{4+self.index() as usize}
    pub fn wave_base(self)->usize{2+(self.index() as usize/2)}
}
pub fn rank_for_power(power:f64)->RiftRank{
    if power>=115.0{RiftRank::S}else if power>=75.0{RiftRank::A}else if power>=48.0{RiftRank::B}
    else if power>=28.0{RiftRank::C}else if power>=14.0{RiftRank::D}else{RiftRank::E}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RiftModifier{Umbral,Frenzy,HeavyGravity,EliteSurge,FracturedSpace,UnstableGround,Mist,Countdown,Echo}
impl RiftModifier{
    pub fn name(self)->&'static str{match self{
        Self::Umbral=>"Umbra",Self::Frenzy=>"Frenesi",Self::HeavyGravity=>"Gravidade Densa",
        Self::EliteSurge=>"Surto de Elite",Self::FracturedSpace=>"Espaço Fraturado",
        Self::UnstableGround=>"Solo Instável",Self::Mist=>"Névoa Arcana",
        Self::Countdown=>"Relógio da Fenda",Self::Echo=>"Eco de Batalha"}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomKind{Entrance,Hunt,Ambush,Shrine,Trap,Elite,Boss,Exit}
impl RoomKind{pub fn name(self)->&'static str{match self{Self::Entrance=>"Entrada",Self::Hunt=>"Caçada",Self::Ambush=>"Emboscada",Self::Shrine=>"Santuário",Self::Trap=>"Armadilha",Self::Elite=>"Elite",Self::Boss=>"Chefe",Self::Exit=>"Saída"}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomState{Dormant,Active,Cleared,Reward}

#[derive(Clone,Debug,PartialEq)]
pub struct RoomPlan{
    pub kind:RoomKind,pub center:[i32;3],pub half:i32,pub waves:u8,pub roster:Vec<&'static str>,pub boss:Option<&'static str>
}
#[derive(Clone,Debug,PartialEq)]
pub struct DungeonPlan{pub seed:u64,pub rank:RiftRank,pub modifiers:Vec<RiftModifier>,pub rooms:Vec<RoomPlan>}

#[derive(Clone,Copy,Debug)]struct Rng(u64);
impl Rng{fn next(&mut self)->u64{let mut x=self.0;if x==0{x=0x9E3779B97F4A7C15}x^=x<<13;x^=x>>7;x^=x<<17;self.0=x;x}fn pick<T:Copy>(&mut self,v:&[T])->T{v[(self.next() as usize)%v.len()]}}

pub fn roster(rank:RiftRank)->&'static [&'static str]{match rank{
    RiftRank::E=>&["rustpg:ember_imp","rustpg:mana_hound","rustpg:rift_hunter"],
    RiftRank::D=>&["rustpg:mana_hound","rustpg:abyss_weaver","rustpg:dusk_revenant"],
    RiftRank::C=>&["rustpg:abyss_weaver","rustpg:rift_mantis","rustpg:grave_knight","rustpg:dusk_revenant"],
    RiftRank::B=>&["rustpg:rift_mantis","rustpg:grave_knight","rustpg:dusk_revenant","rustpg:storm_gryphon"],
    RiftRank::A=>&["rustpg:grave_knight","rustpg:storm_gryphon","rustpg:crystal_wyrm","rustpg:rune_colossus"],
    RiftRank::S=>&["rustpg:storm_gryphon","rustpg:crystal_wyrm","rustpg:rune_colossus","rustpg:ashen_sentinel"],
}}
pub fn boss_for(rank:RiftRank,seed:u64)->&'static str{let alt=seed&1==1;match rank{
    RiftRank::E=>if alt{"rustpg:grave_knight"}else{"rustpg:rift_mantis"},
    RiftRank::D=>if alt{"rustpg:dusk_revenant"}else{"rustpg:grave_knight"},
    RiftRank::C=>if alt{"rustpg:storm_gryphon"}else{"rustpg:crystal_wyrm"},
    RiftRank::B=>if alt{"rustpg:crystal_wyrm"}else{"rustpg:rune_colossus"},
    RiftRank::A=>if alt{"rustpg:rune_colossus"}else{"rustpg:ashen_sentinel"},
    RiftRank::S=>if alt{"rustpg:eclipse_ogre"}else{"rustpg:ashen_sentinel"},}}

pub fn generate(seed:u64,rank:RiftRank)->DungeonPlan{
    let mut rng=Rng(seed^0xA7F4_D36C_51E2_990B);
    let mod_pool=[RiftModifier::Umbral,RiftModifier::Frenzy,RiftModifier::HeavyGravity,
        RiftModifier::EliteSurge,RiftModifier::FracturedSpace,RiftModifier::UnstableGround,
        RiftModifier::Mist,RiftModifier::Countdown,RiftModifier::Echo];
    let mod_count=(rank.index()/2) as usize + usize::from(rank==RiftRank::S);
    let mut chosen=BTreeSet::new();while chosen.len()<mod_count{chosen.insert((rng.next() as usize)%mod_pool.len());}
    let modifiers=chosen.into_iter().map(|i|mod_pool[i]).collect::<Vec<_>>();
    let n=rank.rooms();let pool=roster(rank);let mut rooms=Vec::with_capacity(n);
    rooms.push(RoomPlan{kind:RoomKind::Entrance,center:[0,5,0],half:7,waves:0,roster:vec![],boss:None});
    for i in 1..n-1{
        let elite=i==n-2 || (rank.index()>=3 && i==n/2);
        let kind=if elite{RoomKind::Elite}else{match rng.next()%5{0=>RoomKind::Ambush,1=>RoomKind::Trap,2=>RoomKind::Shrine,_=>RoomKind::Hunt}};
        let half=if elite{9}else{7+(rng.next()%2) as i32};
        let waves=if matches!(kind,RoomKind::Shrine|RoomKind::Trap){1}else{1+(rank.index()>=3) as u8};
        let count=rank.wave_base()+usize::from(elite);
        let mut mobs=Vec::new();for _ in 0..count{mobs.push(rng.pick(pool));}
        rooms.push(RoomPlan{kind,center:[(i as i32)*26,5,0],half,waves,roster:mobs,boss:None});
    }
    let bi=n-1;rooms.push(RoomPlan{kind:RoomKind::Boss,center:[(bi as i32)*26,5,0],half:12,waves:1,roster:vec![],boss:Some(boss_for(rank,seed))});
    DungeonPlan{seed,rank,modifiers,rooms}
}

/// Countdown has a real consequence: the grading clock is 22% tighter.
pub fn expected_ticks(plan:&DungeonPlan)->u64 {
    let base=plan.rank.expected_ticks();
    if plan.modifiers.contains(&RiftModifier::Countdown) {(base as f64*0.78) as u64}else{base}
}

/// Low-frequency effects for modifiers whose challenge is persistent.
pub fn modifier_commands(dimension:&str,plan:&DungeonPlan)->Vec<String>{
    let mut out=Vec::new();
    for modifier in &plan.modifiers {match modifier {
        RiftModifier::Umbral=>out.push(format!("execute in {dimension} run effect give @a minecraft:darkness 3 0 true")),
        RiftModifier::Mist=>out.push(format!("execute in {dimension} run effect give @a minecraft:blindness 2 0 true")),
        RiftModifier::HeavyGravity=>{
            out.push(format!("execute in {dimension} run effect give @a minecraft:slowness 3 0 true"));
            out.push(format!("execute in {dimension} run effect give @a minecraft:mining_fatigue 3 0 true"));
        },
        RiftModifier::FracturedSpace=>{
            out.push(format!("execute in {dimension} run effect give @a minecraft:nausea 2 0 true"));
            out.push(format!("execute in {dimension} run particle minecraft:reverse_portal ~ ~1 ~ 4 1 4 0.02 18 force @a"));
        },
        RiftModifier::Frenzy|RiftModifier::EliteSurge|RiftModifier::UnstableGround|RiftModifier::Countdown|RiftModifier::Echo=>{},
    }}
    out
}
pub fn frenzy_commands(dimension:&str,kinds:impl IntoIterator<Item=&'static str>)->Vec<String>{
    kinds.into_iter().flat_map(|kind|[
        format!("execute in {dimension} run effect give @e[type={kind}] minecraft:speed 999999 0 true"),
        format!("execute in {dimension} run effect give @e[type={kind}] minecraft:strength 999999 0 true"),
    ]).collect()
}

pub fn portal_commands(dimension:&str,pos:[i32;3],open:bool)->Vec<String>{
    let [x,y,z]=pos;let mut v=Vec::new();
    if open{
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:crying_obsidian",x-2,y,z,x-2,y+4,z));
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:crying_obsidian",x+2,y,z,x+2,y+4,z));
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:amethyst_block",x-2,y+4,z,x+2,y+4,z));
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:polished_blackstone_bricks",x-2,y,z,x+2,y,z));
    }else{
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:air replace minecraft:crying_obsidian",x-2,y,z,x+2,y+4,z));
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:air replace minecraft:amethyst_block",x-2,y,z,x+2,y+4,z));
        v.push(format!("execute in {dimension} run fill {} {} {} {} {} {} minecraft:air replace minecraft:polished_blackstone_bricks",x-2,y,z,x+2,y+4,z));
    }v
}
pub fn portal_particle_command(dimension:&str,pos:[i32;3])->String{let[x,y,z]=pos;format!("execute in {dimension} run particle minecraft:portal {} {} {} 1.3 1.8 0.12 0.05 28 force",x as f64+0.5,y as f64+2.0,z as f64+0.5)}

fn shell_commands(d:&str,r:&RoomPlan,wall:&str,accent:&str)->Vec<String>{
    let [x,y,z]=r.center;let h=r.half;let mut v=vec![
        format!("execute in {d} run fill {} {} {} {} {} {} {wall}",x-h,y-1,z-h,x+h,y+7,z+h),
        format!("execute in {d} run fill {} {} {} {} {} {} minecraft:air",x-h+1,y,z-h+1,x+h-1,y+6,z+h-1),
        format!("execute in {d} run fill {} {} {} {} {} {} {accent}",x-h+2,y-1,z-h+2,x+h-2,y-1,z+h-2),
    ];
    v.push(format!("execute in {d} run fill {} {} {} {} {} {} minecraft:air",x-h,y,z-1,x-h,y+3,z+1));
    v.push(format!("execute in {d} run fill {} {} {} {} {} {} minecraft:air",x+h,y,z-1,x+h,y+3,z+1));
    if matches!(r.kind,RoomKind::Boss|RoomKind::Elite){v.push(format!("execute in {d} run setblock {x} {} {z} minecraft:lodestone",y-1));}
    match r.kind {
        RoomKind::Shrine=>{
            v.push(format!("execute in {d} run setblock {x} {y} {z} minecraft:enchanting_table"));
            v.push(format!("execute in {d} run setblock {} {y} {z} minecraft:amethyst_block",x-3));
            v.push(format!("execute in {d} run setblock {} {y} {z} minecraft:amethyst_block",x+3));
        },
        RoomKind::Trap=>{
            v.push(format!("execute in {d} run fill {} {} {} {} {} {} minecraft:magma_block",x-2,y-1,z-2,x+2,y-1,z+2));
            v.push(format!("execute in {d} run setblock {x} {} {z} minecraft:polished_blackstone",y-1));
        },
        RoomKind::Ambush=>{
            for (px,pz) in [(x-4,z-4),(x+4,z-4),(x-4,z+4),(x+4,z+4)] {
                v.push(format!("execute in {d} run fill {px} {y} {pz} {px} {} {pz} minecraft:chiseled_deepslate",y+3));
            }
        },
        RoomKind::Boss=>{
            v.push(format!("execute in {d} run fill {} {y} {} {} {y} {} minecraft:crying_obsidian",x-3,z-3,x+3,z+3));
            v.push(format!("execute in {d} run fill {} {y} {} {} {y} {} {accent}",x-2,z-2,x+2,z+2));
        },
        _=>{}
    }
    v
}
pub fn build_commands(dimension:&str,plan:&DungeonPlan)->Vec<String>{
    let (wall,accent)=match plan.rank{RiftRank::E=>("minecraft:stone_bricks","minecraft:cracked_stone_bricks"),RiftRank::D=>("minecraft:deepslate_bricks","minecraft:polished_deepslate"),RiftRank::C=>("minecraft:polished_blackstone_bricks","minecraft:chiseled_polished_blackstone"),RiftRank::B=>("minecraft:deepslate_tiles","minecraft:sculk"),RiftRank::A=>("minecraft:obsidian","minecraft:amethyst_block"),RiftRank::S=>("minecraft:reinforced_deepslate","minecraft:crying_obsidian")};
    let max_x=plan.rooms.last().map_or(48,|r|r.center[0]+r.half+16);
    let mut v=vec![format!("execute in {dimension} run forceload add -16 -16 {max_x} 16")];
    for (i,r) in plan.rooms.iter().enumerate(){
        v.extend(shell_commands(dimension,r,wall,accent));
        if plan.modifiers.contains(&RiftModifier::UnstableGround) && !matches!(r.kind,RoomKind::Entrance|RoomKind::Boss) {
            let [x,y,z]=r.center;
            for (ox,oz) in [(-4,-4),(4,-4),(-4,4),(4,4),(0,0)] {
                v.push(format!("execute in {dimension} run setblock {} {} {} minecraft:magma_block",x+ox,y-1,z+oz));
            }
        }
        if i>0{
            let a=plan.rooms[i-1].center;let b=r.center;let x1=a[0]+plan.rooms[i-1].half;let x2=b[0]-r.half;
            v.push(format!("execute in {dimension} run fill {x1} 4 -3 {x2} 10 3 {wall}"));
            v.push(format!("execute in {dimension} run fill {x1} 5 -2 {x2} 9 2 minecraft:air"));
        }
    }
    v
}

pub fn resonance_for_kill(kind:&str,tier:&str)->u32{
    let mut base=if tier.to_ascii_lowercase().contains("boss")||tier.to_ascii_lowercase().contains("chefe"){16}else if tier.to_ascii_lowercase().contains("mini"){10}else if tier.to_ascii_lowercase().contains("elite"){6}else{2};
    if kind.starts_with("rustpg:"){base+=2;}base
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Grade{S,A,B,C,D}
impl Grade{pub fn name(self)->&'static str{match self{Self::S=>"S",Self::A=>"A",Self::B=>"B",Self::C=>"C",Self::D=>"D"}}pub fn xp_mult(self)->f64{match self{Self::S=>2.0,Self::A=>1.5,Self::B=>1.0,Self::C=>0.65,Self::D=>0.35}}}
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Assessment{pub grade:Grade,pub xp:u64,pub resonance:u32,pub time_ratio:f64,pub damage_ratio:f64}
pub fn assess_with_expected(rank:RiftRank,expected:u64,elapsed:u64,damage_taken:f64,max_health:f64,deaths:u64)->Assessment{
    let tr=elapsed as f64/expected.max(1) as f64;let dr=if max_health>0.0{(damage_taken/max_health).max(0.0)}else{99.0};
    let grade=if deaths==0&&tr<=0.85&&dr<=1.0{Grade::S}else if deaths==0&&tr<=1.15&&dr<=2.5{Grade::A}else if deaths<=1&&tr<=1.55&&dr<=5.0{Grade::B}else if tr<=2.2{Grade::C}else{Grade::D};
    let xp=((rank.index()+1) as f64*90.0*grade.xp_mult()).round() as u64;
    let resonance=(8+rank.index()*3)+(match grade{Grade::S=>10,Grade::A=>7,Grade::B=>4,Grade::C=>2,Grade::D=>0});
    Assessment{grade,xp,resonance,time_ratio:tr,damage_ratio:dr}
}
pub fn assess(rank:RiftRank,elapsed:u64,damage_taken:f64,max_health:f64,deaths:u64)->Assessment{
    assess_with_expected(rank,rank.expected_ticks(),elapsed,damage_taken,max_health,deaths)
}
pub fn assess_plan(plan:&DungeonPlan,elapsed:u64,damage_taken:f64,max_health:f64,deaths:u64)->Assessment{
    assess_with_expected(plan.rank,expected_ticks(plan),elapsed,damage_taken,max_health,deaths)
}
#[cfg(test)]mod tests{use super::*;
#[test]fn deterministic_plan_has_entry_and_boss(){for rank in RiftRank::ALL{let a=generate(42,rank);let b=generate(42,rank);assert_eq!(a,b);assert_eq!(a.rooms.len(),rank.rooms());assert_eq!(a.rooms[0].kind,RoomKind::Entrance);assert_eq!(a.rooms.last().unwrap().kind,RoomKind::Boss);}}
#[test]fn ranks_and_assessment_scale(){assert_eq!(rank_for_power(5.0),RiftRank::E);assert_eq!(rank_for_power(130.0),RiftRank::S);assert_eq!(assess(RiftRank::B,RiftRank::B.expected_ticks()/2,0.0,20.0,0).grade,Grade::S);assert_eq!(assess(RiftRank::B,RiftRank::B.expected_ticks()*3,200.0,20.0,4).grade,Grade::D);}
#[test]fn commands_are_bounded(){let p=generate(9,RiftRank::S);let c=build_commands("rustpg:rift/test",&p);assert!(c.len()<120);assert!(c.iter().all(|s|s.len()<300));}
#[test]fn countdown_really_tightens_the_clock(){let mut p=generate(1,RiftRank::A);p.modifiers=vec![RiftModifier::Countdown];assert!(expected_ticks(&p)<RiftRank::A.expected_ticks());}
}

#[derive(Clone,Debug)]
pub struct RiftPortal{
    pub owner:u128,pub dimension:String,pub pos:[i32;3],pub rank:RiftRank,pub seed:u64,
    pub created_tick:u64,pub expires_tick:u64,
}
impl RiftPortal{
    pub fn contains(&self,point:[f64;3])->bool{let dx=point[0]-(self.pos[0] as f64+0.5);let dy=point[1]-(self.pos[1] as f64+1.5);let dz=point[2]-(self.pos[2] as f64+0.5);dx*dx+dz*dz<=4.0&&dy.abs()<=3.0}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum SessionPhase{Opening,Building,Active,Returning,Done}
#[derive(Clone,Debug)]
pub struct DungeonSession{
    pub owner:u128,pub level_id:String,pub origin_dimension:String,pub origin_pos:[f64;3],
    pub plan:DungeonPlan,pub phase:SessionPhase,pub current_room:usize,pub room_state:RoomState,
    pub wave:u8,pub started_tick:u64,pub damage_start:f64,pub deaths_start:u64,
    pub build_attempted:bool,pub last_action_tick:u64,
}
impl DungeonSession{
    pub fn room(&self)->Option<&RoomPlan>{self.plan.rooms.get(self.current_room)}
    pub fn finished(&self)->bool{self.current_room>=self.plan.rooms.len()}
}
