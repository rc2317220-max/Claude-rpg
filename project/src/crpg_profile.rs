// The persistent RPG profile of one player, its derived stats, buying nodes, and a
// versioned text codec that survives corrupted or partial data.
use super::crpg_attributes as attributes;
use super::crpg_bestiary::SpeciesRecord;
use super::crpg_campaign::{self,Campaign};
use super::crpg_engine_nutrition::Nutrition;
use super::crpg_gear::Inventory;
use super::crpg_relic_active::ActiveRelics;
use super::crpg_engine_stamina::Stamina;
use super::crpg_engine_vitality::Vitality;
use super::crpg_nodes::{self as nodes, NodeDef};
use super::crpg_progression as prog;
use super::crpg_quests::{self, Activity, QuestEvent, QuestState, StoryAction};
use std::collections::BTreeMap;

pub const VERSION: u32 = 5;

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
    /// Auto-credited daily + story quest progress, saved with the player.
    pub quests: QuestState,
    /// FTB-style mission graph: state and rewards saved in this same profile.
    pub campaign: Campaign,
    /// Immersive progression chains with one-time bonuses, server-authored.
    pub masteries: super::crpg_immersive::Masteries,
    /// Relíquias virtuais do Sistema, ligadas ao perfil do jogador.
    pub gear: Inventory,
    /// Estado persistente de poderes ativos e seus cooldowns (segundos UNIX).
    pub active_relics: ActiveRelics,
    /// One-time migration of pre-4.0.5 max-health base writes to named modifiers.
    pub health_modifier_migrated: bool,
}

#[derive(Debug, PartialEq)]
pub enum BuyError {
    UnknownNode,
    UnimplementedEffect,
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
            BuyError::UnimplementedEffect => write!(f, "efeito ainda não implementado no jogo"),
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
    pub fn refresh_quests(&mut self, now: u64) -> Vec<String> {
        match self.quests.refresh(now) {
            Some(QuestEvent::DailyFailed) => {
                // A bounded penalty, never negative XP or removal of earned levels.
                let lost = self.xp / 50;
                self.xp = self.xp.saturating_sub(lost);
                self.stamina.current = (self.stamina.current * 0.9).max(0.0);
                let msg = format!("[Sistema] Quest diária falhou: -{lost} XP e -10% da stamina atual.");
                self.note(msg.clone());
                vec![msg]
            }
            _ => Vec::new(),
        }
    }

    pub fn record_quest(&mut self, activity: Activity, amount: u32, now: u64) -> Vec<String> {
        let mut lines = self.refresh_quests(now);
        for event in self.quests.record(activity, amount) {
            match event {
                QuestEvent::DailyReward => {
                    self.add_general_xp(250);
                    self.points = self.points.saturating_add(3);
                    let msg = "[Sistema] Quest diária concluída! +250 XP e +3 pontos.".to_string();
                    self.note(msg.clone());
                    lines.push(msg);
                }
                QuestEvent::ChapterReward(chapter) => {
                    let xp = 300 * u64::from(chapter);
                    self.add_general_xp(xp);
                    self.points = self.points.saturating_add(u64::from(chapter));
                    let msg = format!("[Sistema] Capítulo {chapter} concluído! +{xp} XP e +{chapter} ponto(s).");
                    self.note(msg.clone());
                    lines.push(msg);
                }
                QuestEvent::StoryReward(_) => {} // story events handled separately
                QuestEvent::DailyFailed => {} // only refresh may fail
            }
        }
        lines
    }

    pub fn record_story(&mut self,activity:StoryAction,amount:u32)->Vec<String>{
        let Some(QuestEvent::StoryReward(stage))=self.quests.record_story(activity,amount)
            else {return Vec::new()};
        let xp=150u64.saturating_mul(u64::from(stage));
        let pts=if stage==8 {4} else {1};
        self.add_general_xp(xp);
        self.points=self.points.saturating_add(pts);
        let msg=format!("[Sistema] Missão principal {stage}/8 concluída! +{xp} XP e +{pts} ponto(s).");
        self.note(msg.clone());
        vec![msg]
    }
    pub fn record_campaign(&mut self, action:crpg_campaign::Action,quantity:u32)->Vec<String>{
        let mut lines=Vec::new();
        for reward in self.campaign.record(action,quantity){
            self.add_general_xp(reward.xp);
            self.points=self.points.saturating_add(reward.points);
            let message=format!("[Campanha] {reward}");
            self.note(message.clone());lines.push(message);
        }
        lines
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
        let gained = prog::add_xp(
            &mut self.level,
            &mut self.xp,
            amount,
            prog::general_xp_to_next,
        );
        if gained > 0 {for l in (before + 1)..=self.level {
            self.points = self.points.saturating_add(prog::general_points_for(l));
        }}
        if gained > 0 {
            self.note(format!("Nível geral {}", self.level));
        }
        gained
    }

    /// XP of one proficiency only; returns levels gained.
    pub fn add_prof_xp(&mut self, id: &str, amount: u64) -> u32 {
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
        // Prevent points being spent on nodes that only exist in the catalogue.
        // Keep existing purchases intact, so older saves remain readable.
        if !nodes::runtime_supported(node) {return Err(BuyError::UnimplementedEffect);}
        let rank = self.rank(&node.id);
        let current=node.formula.value(rank);
        let next=node.formula.value(rank.saturating_add(1));
        if !next.is_finite() || next<=current+1e-12 {return Err(BuyError::NoStatIncrease);}
        if let Some(cap)=attributes::get(&node.attribute).and_then(|a|a.cap) {
            if self.node_stats().get(&node.attribute)>=cap-1e-9 {
                return Err(BuyError::NoStatIncrease);
            }
        }
        if node.max_rank.is_some_and(|m| rank >= m) || rank == u64::MAX {
            return Err(BuyError::MaxRank);
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
        super::crpg_num::finite(
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
        *e = super::crpg_num::finite(*e + v);
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
    s += &format!(
        "dealt={}\ntaken={}\nbest={}\n",
        p.damage_dealt, p.damage_taken, p.best_hit
    );
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
    s += &format!("quests={}\n", p.quests.encode());
    s += &format!("campaign={}\n", p.campaign.encode());
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

/// Decodes a profile. A newer version or garbage lines don't lose the rest; numbers
/// out of range are clamped, never wrapped. Err only when nothing usable is there.
pub fn decode(text: &str) -> Result<Profile, String> {
    let mut p = Profile::new();
    let mut any = false;
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let num = |v: &str| v.trim().parse::<u64>().ok();
        let flt = |v: &str| v.trim().parse::<f64>().ok().map(super::crpg_num::finite);
        match k {
            "v" => {}
            "level" => p.level = num(v).map_or(1, |n| n.min(u32::MAX as u64) as u32).max(1),
            "xp" => p.xp = num(v).unwrap_or(0),
            "points" => p.points = num(v).unwrap_or(0),
            "deaths" => p.deaths = num(v).unwrap_or(0),
            "dealt" => p.damage_dealt = flt(v).unwrap_or(0.0),
            "taken" => p.damage_taken = flt(v).unwrap_or(0.0),
            "best" => p.best_hit = flt(v).unwrap_or(0.0),
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
            "quests" => { if let Some(q) = super::crpg_quests::QuestState::decode(v) { p.quests = q; } },
            "campaign" => {if let Some(c)=Campaign::decode(v) {p.campaign=c;}},
            "hp.modifier_migrated" => p.health_modifier_migrated = v=="1",
            "relic.active" => {
                if let Some(state)=ActiveRelics::decode(v) {p.active_relics=state;}
            },
            "gear.next" => p.gear.next_id = num(v).unwrap_or(0),
            "gear.bag" => { if let Some(g) = super::crpg_gear::Gear::decode(v) {
                if p.gear.bag.len() < super::crpg_gear::BAG_LIMIT { p.gear.bag.push(g); }
            } },
            "dim" => p.dimensions.push(unesc(v)),
            "hist" => p.history.push(unesc(v)),
            _ => {
                if let Some(slot) = k.strip_prefix("gear.worn.") {
                    if let Some(g) = super::crpg_gear::Gear::decode(v) {
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
                                    best_damage: super::crpg_num::finite(best).max(0.0),
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
        let uid=p.gear.add(super::super::crpg_gear::Gear{
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
