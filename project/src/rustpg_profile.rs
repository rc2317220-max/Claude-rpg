// The persistent RPG profile of one player, its derived stats, buying nodes, and a
// versioned text codec that survives corrupted or partial data.
use super::rustpg_attributes as attributes;
use super::rustpg_bestiary::SpeciesRecord;
use super::rustpg_engine_nutrition::Nutrition;
use super::rustpg_gear::Inventory;
use super::rustpg_relic_active::ActiveRelics;
use super::rustpg_engine_stamina::Stamina;
use super::rustpg_engine_vitality::Vitality;
use super::rustpg_nodes::{self as nodes, NodeDef};
use super::rustpg_progression as prog;
use std::collections::BTreeMap;

pub const VERSION: u32 = 7;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prof {
    pub level: u32,
    pub xp: u64,
    pub points: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PurchaseSpend {
    pub general:u64,
    pub profession:u64,
    pub tracked_ranks:u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Profile {
    pub level: u32,
    pub xp: u64,
    pub points: u64,
    /// Fractional extra general skill points from a separately purchased node.
    /// Stored as thousandths so small ranks affect future level-ups precisely.
    pub bonus_point_milli: u16,
    pub profs: BTreeMap<String, Prof>,
    pub ranks: BTreeMap<String, u64>,
    /// Exact spending by currency for refund. Missing entries are legacy v4 purchases.
    pub purchase_ledger: BTreeMap<String, PurchaseSpend>,
    /// Bestiary: kills per entity kind.
    pub kills: BTreeMap<String, u64>,
    /// Kills per tier name (normal, elite, chefe...).
    pub tier_kills: BTreeMap<String, u64>,
    pub dimensions: Vec<String>,
    pub deaths: u64,
    pub damage_dealt: f64,
    pub damage_taken: f64,
    /// Highest single hit.
    pub best_hit: f64,
    /// Recent progression lines, newest last (bounded).
    pub history: Vec<String>,
    /// Estado próprio do RPG Engine; ausente em saves antigos => defaults seguros.
    pub stamina: Stamina,
    pub vitality: Vitality,
    pub nutrition: Nutrition,
    /// Bestiary progress, keyed by the exact entity id.
    pub species: BTreeMap<String, SpeciesRecord>,
    /// Immersive progression chains with one-time bonuses, server-authored.
    pub masteries: super::rustpg_immersive::Masteries,
    /// Relíquias virtuais do Sistema, ligadas ao perfil do jogador.
    pub gear: Inventory,
    /// Estado persistente de poderes ativos e seus cooldowns (segundos UNIX).
    pub active_relics: ActiveRelics,
    /// One-time migration of pre-4.0.5 max-health base writes to named modifiers.
    pub health_modifier_migrated: bool,
    /// Server-authored Rift resonance. 100 points makes one normal Rift manifestable.
    pub rift_resonance: u32,
    /// Completed instanced dungeons, across every rank.
    pub rift_clears: u64,
    /// Best completion grade: 0 none, 1 D, 2 C, 3 B, 4 A, 5 S.
    pub rift_best_grade: u8,
    /// Highest Rift rank cleared: 0 none, 1 E ... 6 S.
    pub rift_best_rank: u8,
    /// Monotonic entrance/reward receipts plus durable cleanup journal (legacy saves default to none).
    pub rift_serial:u64,
    pub rift_rewarded_serial:u64,
    pub rift_recovery:Option<super::rustpg_rifts::RiftRecovery>,
}

#[derive(Debug, PartialEq)]
pub enum BuyError {
    UnknownNode,
    NoStatIncrease,
    MaxRank,
    Level(u32),
    Proficiency(String, u32),
    Prereq(String, u64),
    Exclusive(String),
    Points(u64),
}

impl std::fmt::Display for BuyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuyError::UnknownNode => write!(f, "node desconhecido"),
            BuyError::NoStatIncrease => write!(f, "o próximo nível não aumenta o atributo (limite atingido)"),
            BuyError::MaxRank => write!(f, "rank máximo atingido"),
            BuyError::Level(l) => write!(f, "requer nível geral {l}"),
            BuyError::Proficiency(p, l) => {
                write!(f, "requer {} nível {l}", prog::proficiency_name(p))
            }
            BuyError::Prereq(n, r) => write!(
                f,
                "requer {} rank {r}",
                nodes::get(n).map_or(n.as_str(), |d| d.name.as_str())
            ),
            BuyError::Exclusive(id) => write!(f,"ramo exclusivo já escolhido: {id}"),
            BuyError::Points(c) => write!(f, "pontos insuficientes (custa {c})"),
        }
    }
}

impl Profile {
    pub fn new() -> Self {
        Profile {
            level: 1,
            ..Default::default()
        }
    }
    pub fn record_mastery(&mut self,event:&str,tick:u64)->Vec<String>{
        let mut notices=Vec::new();
        for award in self.masteries.record(event,tick){
            self.add_general_xp(award.xp);
            let line=format!("[Maestria] {}: sequência de {} ações, +{} XP (uma vez)",
                award.event,award.milestone,award.xp);
            self.note(line.clone()); notices.push(line);
        }
        notices
    }
    /// Add bounded Rift resonance from confirmed server-side progression.
    /// Returns true only when this award crosses the ready threshold.
    pub fn add_rift_resonance(&mut self, amount:u32)->bool {
        let before=self.rift_resonance;
        self.rift_resonance=self.rift_resonance.saturating_add(amount).min(300);
        before < super::rustpg_rifts::RESONANCE_READY &&
            self.rift_resonance >= super::rustpg_rifts::RESONANCE_READY
    }
    pub fn consume_rift_resonance(&mut self)->bool {
        if self.rift_resonance < super::rustpg_rifts::RESONANCE_READY {return false;}
        self.rift_resonance-=super::rustpg_rifts::RESONANCE_READY;true
    }
    /// Reward and receipt are persisted together in the same profile write.
    pub fn award_rift_once(&mut self,serial:u64,rank:super::rustpg_rifts::RiftRank,
        result:super::rustpg_rifts::Assessment)->bool {
        if serial==0 || serial!=self.rift_serial || serial<=self.rift_rewarded_serial
            || self.rift_recovery.as_ref().map(|r|r.serial)!=Some(serial){return false;}
        self.add_general_xp(result.xp);
        self.add_rift_resonance(result.resonance);
        self.record_rift_clear(rank,result.grade);
        self.rift_rewarded_serial=serial;
        true
    }
    pub fn record_rift_clear(&mut self,rank:super::rustpg_rifts::RiftRank,
        grade:super::rustpg_rifts::Grade) {
        self.rift_clears=self.rift_clears.saturating_add(1);
        self.rift_best_rank=self.rift_best_rank.max((rank.index()+1) as u8);
        let g=match grade {super::rustpg_rifts::Grade::D=>1,super::rustpg_rifts::Grade::C=>2,
            super::rustpg_rifts::Grade::B=>3,super::rustpg_rifts::Grade::A=>4,super::rustpg_rifts::Grade::S=>5};
        self.rift_best_grade=self.rift_best_grade.max(g);
        self.note(format!("Fenda rank {} concluída · nota {}",rank.name(),grade.name()));
    }

    pub fn rank(&self, node: &str) -> u64 {
        self.ranks.get(node).copied().unwrap_or(0)
    }
    pub fn prof(&self, id: &str) -> Prof {
        self.profs.get(id).cloned().unwrap_or_default()
    }
    pub fn species(&self, kind: &str) -> SpeciesRecord {
        self.species
            .get(kind)
            .cloned()
            .unwrap_or_else(|| SpeciesRecord {
                kind: kind.to_string(),
                ..Default::default()
            })
    }
    fn note(&mut self, line: String) {
        self.history.push(line);
        let len = self.history.len();
        if len > 40 {
            self.history.drain(..len - 40);
        }
    }

    /// General XP; returns levels gained.
    pub fn add_general_xp(&mut self, amount: u64) -> u32 {
        let before = self.level;
        let bonus=self.node_stats().get("general.xp_gain");
        let amount=super::rustpg_node_effects::scaled_xp(amount,bonus);
        let gained = prog::add_xp(
            &mut self.level,
            &mut self.xp,
            amount,
            prog::general_xp_to_next,
        );
        if gained > 0 {
            let bonus=self.node_stats().get("general.skill_points");
            for l in (before + 1)..=self.level {
                self.points = self.points.saturating_add(prog::general_points_for(l));
                let (extra,carry)=super::rustpg_node_effects::point_credit(
                    bonus,self.bonus_point_milli);
                self.points=self.points.saturating_add(extra);
                self.bonus_point_milli=carry;
            }
        }
        if gained > 0 {
            self.note(format!("Nível geral {}", self.level));
        }
        gained
    }

    /// XP of one proficiency only; returns levels gained.
    pub fn add_prof_xp(&mut self, id: &str, amount: u64) -> u32 {
        // Every profession and weapon gets its own XP node, never the bonus of
        // a neighbouring tree. All server-confirmed awards pass this gateway.
        let key=if super::rustpg_attributes::WEAPONS.iter().any(|(w,_)|*w==id) {
            format!("weapon.{id}.xp")
        }else{format!("{id}.xp")};
        let bonus=self.node_stats().get(&key);
        let amount=super::rustpg_node_effects::scaled_xp(amount,bonus);
        let p = self.profs.entry(id.to_string()).or_default();
        let before = p.level;
        let gained = prog::add_xp(&mut p.level, &mut p.xp, amount, prog::prof_xp_to_next);
        if gained > 0 {for l in (before + 1)..=p.level {
            p.points = p.points.saturating_add(prog::prof_points_for(l));
        }}
        let level = p.level;
        if gained > 0 {
            self.note(format!("{} nível {level}", prog::proficiency_name(id)));
        }
        gained
    }

    pub fn check_buy(&self, node: &NodeDef) -> Result<u64, BuyError> {
        self.check_buy_with_stats(node, &self.node_stats())
    }

    pub fn check_buy_with_stats(&self, node: &NodeDef, stats: &Stats) -> Result<u64, BuyError> {
        // Prevent points being spent on nodes that only exist in the catalogue.
        // Keep existing purchases intact, so older saves remain readable.
        let rank = self.rank(&node.id);
        if node.max_rank.is_some_and(|m| rank >= m) || rank == u64::MAX {
            return Err(BuyError::MaxRank);
        }
        let current=node.formula.value(rank);
        let next=node.formula.value(rank.saturating_add(1));
        if !next.is_finite() || next<=current+1e-12 {return Err(BuyError::NoStatIncrease);}
        if let Some(cap)=attributes::get(&node.attribute).and_then(|a|a.cap) {
            if stats.get(&node.attribute)>=cap-1e-9 {
                return Err(BuyError::NoStatIncrease);
            }
        }
        if self.level < node.req_level {
            return Err(BuyError::Level(node.req_level));
        }
        let prof = nodes::tree_proficiency(&node.tree);
        if node.req_prof > 0 && prof != "general" && self.prof(prof).level < node.req_prof {
            return Err(BuyError::Proficiency(prof.to_string(), node.req_prof));
        }
        for (p, r) in &node.prereqs {
            if self.rank(p) < *r {
                return Err(BuyError::Prereq(p.clone(), *r));
            }
        }
        for exclusive in &node.exclusions {
            if self.rank(exclusive)>0 {return Err(BuyError::Exclusive(exclusive.clone()));}
        }
        let cost = node.cost_at(rank);
        let available = self.points.saturating_add(self.prof(prof).points);
        if available < cost {
            return Err(BuyError::Points(cost));
        }
        Ok(cost)
    }

    /// Buys one rank: the tree's proficiency points first, then general points.
    pub fn buy(&mut self, id: &str) -> Result<u64, BuyError> {
        let node = nodes::get(id).ok_or(BuyError::UnknownNode)?;
        let cost = self.check_buy(node)?;
        let prof = nodes::tree_proficiency(&node.tree);
        let mut left = cost;
        let mut from_prof=0u64;
        if let Some(p) = self.profs.get_mut(prof) {
            let take = p.points.min(left);
            p.points -= take;
            from_prof=take;
            left -= take;
        }
        self.points -= left;
        let spend=self.purchase_ledger.entry(id.to_string()).or_default();
        spend.general=spend.general.saturating_add(left);
        spend.profession=spend.profession.saturating_add(from_prof);
        spend.tracked_ranks=spend.tracked_ranks.saturating_add(1);
        let r = self.ranks.entry(id.to_string()).or_insert(0);
        *r = r.saturating_add(1);
        Ok(*r)
    }

    /// Buys up to `n` ranks; stops at the first refusal. Returns ranks bought and the refusal.
    pub fn buy_many(&mut self, id: &str, n: u64) -> (u64, Option<BuyError>) {
        let mut done = 0;
        while done < n {
            match self.buy(id) {
                Ok(_) => done += 1,
                Err(e) => return (done, Some(e)),
            }
        }
        (done, None)
    }

    /// Exact v5 refund: profession currency returns to its original pool,
    /// not to general points. Old v4 rank purchases lacked a ledger, and get
    /// their historical refund once; later purchases cannot repeat this exploit.
    pub fn respec(&mut self) -> u64 {
        let mut refund_general=0u64;
        let mut refund_total=0u64;
        let mut refund_profs=BTreeMap::<String,u64>::new();
        for (id, rank) in std::mem::take(&mut self.ranks) {
            let Some(n)=nodes::get(&id)else{continue;};
            let spend=self.purchase_ledger.remove(&id).unwrap_or_default();
            refund_general=refund_general.saturating_add(spend.general);
            let prof=nodes::tree_proficiency(&n.tree);
            if spend.profession>0 {
                let entry=refund_profs.entry(prof.to_string()).or_default();
                *entry=entry.saturating_add(spend.profession);
            }
            // Migration: legacy ranks have no purchased-currency trace.
            // Restore only those old ranks, then clear them permanently.
            let old=rank.saturating_sub(spend.tracked_ranks).min(1_000_000);
            for r in 0..old {refund_general=refund_general.saturating_add(n.cost_at(r));}
        }
        self.purchase_ledger.clear();
        self.points=self.points.saturating_add(refund_general);
        refund_total=refund_total.saturating_add(refund_general);
        for (prof,value) in refund_profs {
            let entry=self.profs.entry(prof).or_default();
            entry.points=entry.points.saturating_add(value);
            refund_total=refund_total.saturating_add(value);
        }
        refund_total
    }

    /// Final stats from nodes: attribute → value. Each node feeds its single attribute.
    pub fn node_stats(&self) -> Stats {
        let mut s = Stats::default();
        for (id, rank) in &self.ranks {
            if let Some(n) = nodes::get(id) {
                s.add(&n.attribute, n.formula.value(*rank));
            }
        }
        for (id, bonus) in self.gear.bonuses() { s.add(id, bonus); }
        s.apply_caps();
        s
    }

    /// A combat-power score for mob scaling: combat proficiencies, general level,
    /// combat node investment and boss kills. Never just the general level.
    pub fn power(&self) -> f64 {
        let combat: f64 = [
            "sword", "axe", "bow", "crossbow", "mace", "trident", "spear", "unarmed", "survival",
        ]
        .iter()
        .map(|p| self.prof(p).level as f64)
        .fold(0.0, f64::max);
        let invested: f64 = self
            .ranks
            .iter()
            .filter(|(id, _)| id.starts_with("weapon.") || id.starts_with("defense."))
            .map(|(_, r)| *r as f64)
            .sum();
        let bosses = self.tier_kills.get("chefe").copied().unwrap_or(0) as f64
            + 3.0 * self.tier_kills.get("lendario").copied().unwrap_or(0) as f64;
        super::rustpg_num::finite(
            self.level as f64 * 0.6
                + combat * 0.8
                + invested.sqrt() * 0.5
                + bosses * 2.0
                + self.dimensions.len() as f64 * 3.0,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stats(pub BTreeMap<String, f64>);

impl Stats {
    pub fn get(&self, id: &str) -> f64 {
        self.0.get(id).copied().unwrap_or(0.0)
    }
    pub fn add(&mut self, id: &str, v: f64) {
        let e = self.0.entry(id.to_string()).or_insert(0.0);
        *e = super::rustpg_num::finite(*e + v);
    }
    /// Applies each attribute's own cap (player movement +50%, reductions 90%).
    pub fn apply_caps(&mut self) {
        for (id, v) in self.0.iter_mut() {
            if let Some(cap) = attributes::get(id).and_then(|a| a.cap) {
                *v = v.min(cap);
            }
        }
    }
}

// ---- codec: "key=value" lines, versioned; unknown or broken lines are skipped ----

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('=', "\\e")
        .replace(';', "\\s")
        .replace(',', "\\c")
}
fn unesc(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('e') => out.push('='),
                Some('s') => out.push(';'),
                Some('c') => out.push(','),
                Some(o) => out.push(o),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn encode(p: &Profile) -> String {
    let mut s = format!(
        "v={VERSION}\nlevel={}\nxp={}\npoints={}\ndeaths={}\n",
        p.level, p.xp, p.points, p.deaths
    );
    s += &format!("bonus_point_milli={}\n",p.bonus_point_milli.min(999));
    s += &format!(
        "dealt={}\ntaken={}\nbest={}\n",
        p.damage_dealt, p.damage_taken, p.best_hit
    );
    s += &format!("rift.resonance={}\nrift.clears={}\nrift.best_grade={}\nrift.best_rank={}\n",
        p.rift_resonance,p.rift_clears,p.rift_best_grade,p.rift_best_rank);
    s += &format!("rift.serial={}\nrift.rewarded_serial={}\n",p.rift_serial,p.rift_rewarded_serial);
    if let Some(journal)=&p.rift_recovery{s+=&format!("rift.recovery={}\n",journal.encode());}
    s += &format!(
        "stamina={:.6};{:.6};{:.6};{};{};{}\n",
        p.stamina.current,
        p.stamina.max,
        p.stamina.regen_per_tick,
        p.stamina.delay_left,
        p.stamina.zero_ticks,
        if p.stamina.exhausted {1} else {0}
    );
    s += &format!(
        "vitality={:.6};{:.6};{:.6}\n",
        p.vitality.current, p.vitality.max, p.vitality.regen_per_tick
    );
    s += &format!(
        "nutrition={:.6};{:.6};{:.6};{:.6};{}\n",
        p.nutrition.satiety,
        p.nutrition.stamina_restore,
        p.nutrition.regen_bonus,
        p.nutrition.resistance_bonus,
        p.nutrition.effect_ticks
    );
    for (event,data) in p.masteries.encode() {
        s += &format!("mastery.{}={}\n",esc(&event),data);
    }
    s += &format!("hp.modifier_migrated={}\n",if p.health_modifier_migrated {1}else{0});
    s += &format!("relic.active={}\n",p.active_relics.encode());
    s += &format!("gear.next={}\n", p.gear.next_id);
    for g in &p.gear.bag { s += &format!("gear.bag={}\n", g.encode()); }
    for (slot, g) in &p.gear.worn { s += &format!("gear.worn.{}={}\n", slot, g.encode()); }
    for (k, v) in &p.profs {
        s += &format!("prof.{}={};{};{}\n", esc(k), v.level, v.xp, v.points);
    }
    for (k, v) in &p.ranks {
        s += &format!("rank.{}={v}\n", esc(k));
    }
    for (k, spend) in &p.purchase_ledger {
        s += &format!("purchase.{}={};{};{}\n",esc(k),spend.general,
            spend.profession,spend.tracked_ranks);
    }
    for (k, v) in &p.kills {
        s += &format!("kill.{}={v}\n", esc(k));
    }
    for (k, v) in &p.tier_kills {
        s += &format!("tier.{}={v}\n", esc(k));
    }
    for (k, v) in &p.species {
        let tiers = v.tiers.iter().map(|x| esc(x)).collect::<Vec<_>>().join(",");
        let mutations = v
            .mutations
            .iter()
            .map(|x| esc(x))
            .collect::<Vec<_>>()
            .join(",");
        let drops = v.drops.iter().map(|x| esc(x)).collect::<Vec<_>>().join(",");
        s += &format!(
            "species.{}={};{};{:.6};{};{};{};{};{}\n",
            esc(k),
            v.first_seen as u8,
            v.kills,
            v.best_damage,
            v.max_level,
            tiers,
            mutations,
            drops,
            v.reward_flags
        );
    }
    for d in &p.dimensions {
        s += &format!("dim={}\n", esc(d));
    }
    for h in &p.history {
        s += &format!("hist={}\n", esc(h));
    }
    s
}

/// Refuse missing/corrupt core records and unknown future versions rather than
/// replacing an unreadable profile with defaults on the next write.
pub fn decode(text: &str) -> Result<Profile, String> {
    let mut core=std::collections::HashSet::new();
    for line in text.lines() {
        if let Some((k,v))=line.split_once('=') {
            if matches!(k,"v"|"level"|"xp"|"points") {
                if !core.insert(k) {return Err(format!("campo duplicado: {k}"));}
                let n=v.parse::<u64>().map_err(|_|format!("campo inválido: {k}"))?;
                if (k=="v" && (n==0 || n>u64::from(VERSION))) ||
                   (k=="level" && (n==0 || n>u64::from(u32::MAX))) {
                    return Err(format!("campo fora dos limites: {k}"));
                }
            }
        }
    }
    if core.len()!=4 {return Err("perfil incompleto: cabeçalho obrigatório ausente".into());}
    let mut p = Profile::new();
    let mut any = false;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let num = |v: &str| v.trim().parse::<u64>().ok();
        let flt = |v: &str| v.trim().parse::<f64>().ok().map(super::rustpg_num::finite);
        match k {
            "v" => {}
            "level" => p.level = num(v).map_or(1, |n| n.min(u32::MAX as u64) as u32).max(1),
            "xp" => p.xp = num(v).unwrap_or(0),
            "points" => p.points = num(v).unwrap_or(0),
            "bonus_point_milli" => p.bonus_point_milli=num(v).unwrap_or(0).min(999) as u16,
            "deaths" => p.deaths = num(v).unwrap_or(0),
            "dealt" => p.damage_dealt = flt(v).unwrap_or(0.0),
            "taken" => p.damage_taken = flt(v).unwrap_or(0.0),
            "best" => p.best_hit = flt(v).unwrap_or(0.0),
            "rift.resonance" => p.rift_resonance=num(v).unwrap_or(0).min(300) as u32,
            "rift.clears" => p.rift_clears=num(v).unwrap_or(0),
            "rift.best_grade" => p.rift_best_grade=num(v).unwrap_or(0).min(5) as u8,
            "rift.best_rank" => p.rift_best_rank=num(v).unwrap_or(0).min(6) as u8,
            "rift.serial" => p.rift_serial=num(v).ok_or("identificador de Fenda inválido")?,
            "rift.rewarded_serial" => p.rift_rewarded_serial=num(v).ok_or("recibo de Fenda inválido")?,
            "rift.recovery" => {
                if p.rift_recovery.is_some(){return Err("registro de retorno duplicado".into());}
                p.rift_recovery=Some(super::rustpg_rifts::RiftRecovery::decode(v).ok_or("registro de retorno inválido")?);
            },
            "stamina" => {
                let f: Vec<&str> = v.split(';').collect();
                if f.len() == 5 || f.len() == 6 {
                    let parsed = (
                        f[0].parse::<f64>().ok(),
                        f[1].parse::<f64>().ok(),
                        f[2].parse::<f64>().ok(),
                        f[3].parse::<u32>().ok(),
                        f[4].parse::<u32>().ok(),
                    );
                    if let (Some(current), Some(max), Some(regen), Some(delay), Some(zero)) = parsed
                    {
                        p.stamina = Stamina {
                            current,
                            max,
                            regen_per_tick: regen,
                            regen_delay_ticks: 20,
                            delay_left: delay,
                            zero_ticks: zero,
                            exhausted: f.get(5).map_or(current<=0.0, |v| *v=="1"),
                        };
                        p.stamina.clamp();
                    }
                }
            }
            "vitality" => {
                let f: Vec<&str> = v.split(';').collect();
                if f.len() == 3 {
                    if let (Some(current), Some(max), Some(regen)) = (
                        f[0].parse::<f64>().ok(),
                        f[1].parse::<f64>().ok(),
                        f[2].parse::<f64>().ok(),
                    ) {
                        p.vitality = Vitality {
                            current: current.max(0.0),
                            max: max.max(1.0),
                            regen_per_tick: regen.max(0.0),
                        };
                        p.vitality.current = p.vitality.current.min(p.vitality.max);
                    }
                }
            }
            "nutrition" => {
                let f: Vec<&str> = v.split(';').collect();
                if f.len() == 5 || f.len() == 6 {
                    if let (Some(satiety), Some(restore), Some(regen), Some(res), Some(ticks)) = (
                        f[0].parse::<f64>().ok(),
                        f[1].parse::<f64>().ok(),
                        f[2].parse::<f64>().ok(),
                        f[3].parse::<f64>().ok(),
                        f[4].parse::<u32>().ok(),
                    ) {
                        p.nutrition = Nutrition {
                            satiety: satiety.clamp(0.0, 100.0),
                            stamina_restore: restore.clamp(0.0, 100.0),
                            regen_bonus: regen.clamp(0.0, 0.25),
                            resistance_bonus: res.clamp(0.0, 0.15),
                            effect_ticks: ticks,
                        };
                    }
                }
            }
            "hp.modifier_migrated" => p.health_modifier_migrated = v=="1",
            "relic.active" => {
                if let Some(state)=ActiveRelics::decode(v) {p.active_relics=state;}
            },
            "gear.next" => p.gear.next_id = num(v).unwrap_or(0),
            "gear.bag" => { if let Some(g) = super::rustpg_gear::Gear::decode(v) {
                if p.gear.bag.len() < super::rustpg_gear::BAG_LIMIT { p.gear.bag.push(g); }
            } },
            "dim" => p.dimensions.push(unesc(v)),
            "hist" => p.history.push(unesc(v)),
            _ => {
                if let Some(slot) = k.strip_prefix("gear.worn.") {
                    if let Some(g) = super::rustpg_gear::Gear::decode(v) {
                        if g.slot == slot { p.gear.worn.insert(slot.to_string(), g); }
                    }
                } else if let Some(event) = k.strip_prefix("mastery.") {
                    let _=p.masteries.decode_line(&unesc(event),v);
                } else if let Some(id) = k.strip_prefix("prof.") {
                    let f: Vec<u64> = v.split(';').filter_map(num).collect();
                    if f.len() == 3 {
                        p.profs.insert(
                            unesc(id),
                            Prof {
                                level: f[0].min(u32::MAX as u64) as u32,
                                xp: f[1],
                                points: f[2],
                            },
                        );
                    }
                } else if let Some(id) = k.strip_prefix("purchase.") {
                    let f=v.split(';').map(str::parse::<u64>)
                        .collect::<Result<Vec<_>,_>>().ok();
                    if let Some(f)=f.filter(|f|f.len()==3) {
                        p.purchase_ledger.insert(unesc(id),PurchaseSpend {
                            general:f[0],profession:f[1],tracked_ranks:f[2],
                        });
                    }
                } else if let Some(id) = k.strip_prefix("rank.") {
                    if let Some(n) = num(v) {
                        let id = unesc(id);
                        // Ranks above a node's max (a changed definition) are clamped.
                        let n = nodes::get(&id)
                            .and_then(|d| d.max_rank)
                            .map_or(n, |m| n.min(m));
                        p.ranks.insert(id, n);
                    }
                } else if let Some(id) = k.strip_prefix("kill.") {
                    if let Some(n) = num(v) {
                        p.kills.insert(unesc(id), n);
                    }
                } else if let Some(id) = k.strip_prefix("tier.") {
                    if let Some(n) = num(v) {
                        p.tier_kills.insert(unesc(id), n);
                    }
                } else if let Some(id) = k.strip_prefix("species.") {
                    let f: Vec<&str> = v.split(';').collect();
                    if f.len() >= 8 {
                        let list = |s: &str| {
                            s.split(',')
                                .filter(|x| !x.is_empty())
                                .map(unesc)
                                .collect::<Vec<_>>()
                        };
                        if let (Some(first), Some(kills), Some(best), Some(level), Some(flags)) = (
                            f[0].parse::<u8>().ok(),
                            f[1].parse::<u64>().ok(),
                            f[2].parse::<f64>().ok(),
                            f[3].parse::<u32>().ok(),
                            f[7].parse::<u32>().ok(),
                        ) {
                            p.species.insert(
                                unesc(id),
                                SpeciesRecord {
                                    kind: unesc(id),
                                    first_seen: first != 0,
                                    kills,
                                    best_damage: super::rustpg_num::finite(best).max(0.0),
                                    max_level: level,
                                    tiers: list(f[4]),
                                    mutations: list(f[5]),
                                    drops: list(f[6]),
                                    reward_flags: flags,
                                },
                            );
                        }
                    }
                }
            }
        }
        any = true;
    }
    if any {
        if p.rift_rewarded_serial>p.rift_serial ||
            p.rift_recovery.as_ref().is_some_and(|r|r.serial!=p.rift_serial) {
            return Err("recibo e registro de Fenda inconsistentes".into());
        }
        p.gear.sanitize();
        p.purchase_ledger.retain(|id,spend| p.ranks.get(id).is_some_and(|rank| {
            spend.tracked_ranks<=*rank && nodes::get(id).is_some()
        }));
        Ok(p)
    } else {
        Err("perfil vazio".into())
    }
}

#[cfg(test)]
mod v405_profile_migration_tests {
    use super::*;
    #[test]
    fn legacy_saves_keep_equipment_and_await_one_time_migration() {
        let mut p=Profile::new();
        p.points=37;
        let uid=p.gear.add(super::super::rustpg_gear::Gear{
            uid:0,slot:"ring".into(),rarity:4,level:50,motif:1,
        }).unwrap();
        p.gear.equip(uid).unwrap();
        let old=encode(&p).replace("v=5", "v=3")
            .replace("hp.modifier_migrated=0\n","")
            .lines().filter(|line|!line.starts_with("relic.active="))
            .collect::<Vec<_>>().join("\n");
        let decoded=decode(&old).unwrap();
        assert_eq!(decoded.points,37);
        assert_eq!(decoded.gear.worn.get("ring").unwrap().motif,1);
        assert!(!decoded.health_modifier_migrated);
        let mut saved=decoded;
        saved.health_modifier_migrated=true;
        let roundtrip=decode(&encode(&saved)).unwrap();
        assert!(roundtrip.health_modifier_migrated);
        assert_eq!(roundtrip.gear.worn.len(),1);
    }
}

#[cfg(test)]
mod currency_refund_tests {
    use super::*;
    #[test] fn respec_returns_points_to_the_original_pool() {
        let n=nodes::all().iter().find(|n|n.req_level==1 && n.req_prof==0
            && n.max_rank.unwrap_or(10)>1).unwrap();
        let prof=nodes::tree_proficiency(&n.tree).to_string();
        let mut p=Profile::new();
        p.points=8;
        p.profs.entry(prof.clone()).or_default().points=5;
        let before_general=p.points;
        let before_prof=p.prof(&prof).points;
        p.buy(&n.id).unwrap();
        assert_eq!(p.points,before_general);
        assert_eq!(p.prof(&prof).points,before_prof-1);
        let reloaded=decode(&encode(&p)).unwrap();
        let mut reloaded=reloaded;
        reloaded.respec();
        assert_eq!(reloaded.points,before_general);
        assert_eq!(reloaded.prof(&prof).points,before_prof);
        assert!(reloaded.purchase_ledger.is_empty());
    }
    #[test] fn previous_v4_ranks_can_be_refunded_once_only() {
        let n=nodes::all().iter().find(|n|n.req_level==1 && n.req_prof==0).unwrap();
        let mut p=Profile::new();
        p.ranks.insert(n.id.clone(),2);
        let first=p.respec();
        assert!(first>0);
        assert_eq!(p.respec(),0);
    }
}

#[cfg(test)]
mod xp_overflow_regression_tests {
    use super::*;
    #[test]
    fn max_general_level_does_not_overflow_on_next_xp_event() {
        let mut p=Profile::new();
        p.level=u32::MAX;
        p.add_general_xp(1);
        assert_eq!(p.level,u32::MAX);
    }
    #[test]
    fn max_profession_level_does_not_overflow_on_next_xp_event() {
        let mut p=Profile::new();
        p.profs.insert("mining".into(),Prof {level:u32::MAX,xp:0,points:0});
        p.add_prof_xp("mining",1);
        assert_eq!(p.prof("mining").level,u32::MAX);
    }
}

#[cfg(test)] mod extra_skill_points_tests {
    use super::*;
    #[test] fn independent_point_node_rewards_on_new_levels_only_and_persists() {
        let mut boosted=Profile::new();
        boosted.ranks.insert("general.skill_points#1".into(),100);
        let mut ordinary=Profile::new();
        let needed=prog::general_xp_to_next(1);
        assert_eq!(boosted.add_general_xp(needed),1);
        assert_eq!(ordinary.add_general_xp(needed),1);
        assert_eq!(boosted.points,ordinary.points+2);
        let serialized=encode(&boosted);
        let restored=decode(&serialized).expect("profile must decode");
        assert_eq!(restored.bonus_point_milli,boosted.bonus_point_milli);
        assert_eq!(restored.points,boosted.points);
    }
    #[test] fn tiny_skill_point_bonus_is_not_lost_or_fabricated() {
        let mut p=Profile::new();
        p.ranks.insert("general.skill_points#1".into(),1);
        let needed=prog::general_xp_to_next(1);
        p.add_general_xp(needed);
        assert_eq!(p.bonus_point_milli,20);
        assert_eq!(decode(&encode(&p)).unwrap().bonus_point_milli,20);
    }
}

#[cfg(test)] mod node_xp_integration_tests {
    use super::*;
    #[test] fn profession_and_weapon_bonuses_are_separate(){
        let mut p=Profile::new();
        p.ranks.insert("fishing.xp#1".into(),100);
        p.ranks.insert("weapon.sword.xp#1".into(),100);
        let base=Profile::new();
        assert_eq!(crate::rustpg_node_effects::scaled_xp(100,
            p.node_stats().get("fishing.xp")),200);
        p.add_prof_xp("fishing",100);
        p.add_prof_xp("sword",100);
        p.add_prof_xp("bow",100);
        let mut expected=base;
        expected.add_prof_xp("fishing",200);
        expected.add_prof_xp("sword",200);
        expected.add_prof_xp("bow",100);
        for id in ["fishing","sword","bow"] {
            assert_eq!(p.prof(id),expected.prof(id));
        }
    }
    #[test]fn general_xp_has_independent_bonus(){
        let mut a=Profile::new();a.ranks.insert("general.xp_gain#1".into(),100);
        let mut b=Profile::new();
        a.add_general_xp(100);b.add_general_xp(200);
        assert_eq!((a.level,a.xp),(b.level,b.xp));
    }
}

#[cfg(test)]
mod corruption_regression {
    use super::*;
    #[test] fn corrupted_core_never_decodes_as_new_profile() {
        for invalid in ["", "garbage=1", "v=7", "v=7\nlevel=oops\nxp=1\npoints=2",
            "v=999\nlevel=1\nxp=0\npoints=0","v=7\nlevel=1\nxp=1\npoints=0\npoints=1"] {
            assert!(decode(invalid).is_err(), "{invalid}");
        }
        let mut original=Profile::new();original.level=42;original.points=99;
        let loaded=decode(&encode(&original)).unwrap();
        assert_eq!(loaded.level,42);assert_eq!(loaded.points,99);
    }
}

#[cfg(test)] mod rift_receipt_tests {
    use super::*;
    use crate::rustpg_rifts::{self,RiftRank,RiftRecovery};
    #[test] fn reward_receipt_survives_reload_and_prevents_repeat(){
        let mut p=Profile::new();p.rift_serial=1;
        p.rift_recovery=Some(RiftRecovery{serial:1,level_id:"rustpg:rift/one".into(),
            origin_dimension:"minecraft:overworld".into(),origin_pos:[0.0,64.0,0.0]});
        let result=rustpg_rifts::assess(RiftRank::E,10,0.0,20.0,0);
        assert!(p.award_rift_once(1,RiftRank::E,result));
        let mut restored=decode(&encode(&p)).unwrap();
        let before=restored.clone();
        assert!(!restored.award_rift_once(1,RiftRank::E,result));
        assert_eq!(restored,before);
        assert!(!restored.award_rift_once(2,RiftRank::E,result));
        restored.rift_recovery=None;
        assert_eq!(decode(&encode(&restored)).unwrap().rift_rewarded_serial,1);
    }
}
