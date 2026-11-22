// Equipamentos R.P.G. virtuais: propriedade exclusiva do perfil salvo do jogador.
// Sem inventar NBT ou slots vanilla que o Host legado não expõe. O item só
// existe no inventário do Sistema e sua contribuição é aplicada no Stats final.
// Todas as mudanças de equipamento ocorrem nos hooks de servidor.
use super::crpg_num::{unit, mix, Rng};
use super::crpg_scaling::Tier;
use std::collections::BTreeMap;

pub const BAG_LIMIT: usize = 64;
pub const SLOTS: &[(&str, &str)] = &[
    ("head", "Elmo"), ("chest", "Peitoral"), ("legs", "Calças"),
    ("feet", "Botas"), ("weapon", "Arma"), ("ring", "Anel"),
    ("neck", "Colar"), ("charm", "Talismã"),
];
pub const RARITIES: &[&str] = &["Comum", "Incomum", "Raro", "Épico", "Lendário", "Mítico"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gear {
    pub uid: u64,
    pub slot: String,
    /// 0..5, determines colour/rarity and coefficient.
    pub rarity: u8,
    pub level: u32,
    /// Prefix/codename, generated deterministically from the drop's seed.
    pub motif: u8,
}
impl Gear {
    pub fn name(&self) -> String {
        let names = ["do Vigia", "do Eclipse", "do Abismo", "da Aurora", "do Caçador", "do Monarca"];
        let prefix = SLOTS.iter().find(|(k,_)| *k == self.slot).map_or("Relíquia", |(_,n)| *n);
        format!("{} {} {} (Nv. {})", prefix, names[self.motif.min(5) as usize],
            RARITIES[self.rarity.min(5) as usize], self.level)
    }
    /// Uma relíquia sempre reforça UMA estatística, independentemente de outras.
    pub fn stat(&self) -> (&'static str, f64) {
        let q = 1.0 + self.rarity.min(5) as f64 * 0.52;
        let lv = 1.0 + (self.level.min(10_000) as f64).sqrt() * 0.12;
        match self.slot.as_str() {
            "head" => ("defense.max_health_flat", 2.0 * q * lv),
            "chest" => ("defense.armor", 0.6 * q * lv),
            "legs" => ("defense.res_physical", 0.015 * q * lv),
            "feet" => ("defense.res_fall", 0.02 * q * lv),
            "weapon" => ("weapon.sword.damage_flat", 0.85 * q * lv),
            "ring" => ("weapon.sword.crit_chance", 0.012 * q * lv),
            "neck" => ("general.xp_gain", 0.01 * q * lv),
            "charm" => ("defense.res_magic", 0.017 * q * lv),
            _ => ("general.xp_gain", 0.0),
        }
    }
    pub fn decode(s: &str) -> Option<Self> {
        let p: Vec<_> = s.split(';').collect();
        if p.len() != 5 || !SLOTS.iter().any(|(key,_)| *key == p[1]) { return None; }
        Some(Self {
            uid: p[0].parse::<u64>().ok()?.max(1),
            slot: p[1].to_string(), rarity: p[2].parse::<u8>().ok()?.min(5),
            level: p[3].parse::<u32>().ok()?.clamp(1, 10_000),
            motif: p[4].parse::<u8>().ok()?.min(5),
        })
    }
    pub fn encode(&self) -> String {
        format!("{};{};{};{};{}", self.uid, self.slot, self.rarity, self.level, self.motif)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    pub next_id: u64,
    pub bag: Vec<Gear>,
    pub worn: BTreeMap<String, Gear>,
}
impl Inventory {
    pub fn items(&self) -> usize { self.bag.len() + self.worn.len() }
    pub fn can_store(&self) -> bool { self.bag.len() < BAG_LIMIT }
    pub fn add(&mut self, mut gear: Gear) -> Result<u64, &'static str> {
        if !self.can_store() { return Err("Bolsa do Sistema cheia (64 slots)"); }
        self.next_id = self.next_id.saturating_add(1).max(1);
        gear.uid = self.next_id;
        self.bag.push(gear);
        Ok(self.next_id)
    }
    pub fn equip(&mut self, id: u64) -> Result<String, &'static str> {
        let idx = self.bag.iter().position(|g| g.uid == id).ok_or("Item não encontrado")?;
        let gear = self.bag.remove(idx);
        let slot = gear.slot.clone();
        let name = gear.name();
        if let Some(old) = self.worn.insert(slot, gear) { self.bag.push(old); }
        Ok(format!("Equipado: {name}"))
    }
    pub fn unequip(&mut self, slot: &str) -> Result<String, &'static str> {
        if !self.can_store() { return Err("Libere um espaço na bolsa antes de remover"); }
        let old = self.worn.remove(slot).ok_or("Slot vazio")?;
        let name = old.name(); self.bag.push(old);
        Ok(format!("Removido: {name}"))
    }
    pub fn salvage(&mut self, id: u64) -> Result<u64, &'static str> {
        let idx = self.bag.iter().position(|g| g.uid == id).ok_or("Item não encontrado")?;
        let gear = self.bag.remove(idx);
        // A sucata retorna XP geral, não itens vanilla inexistentes.
        Ok((gear.level as u64).saturating_mul(3 + gear.rarity as u64).max(3))
    }
    pub fn bonuses(&self) -> Vec<(&'static str, f64)> {
        self.worn.values().map(Gear::stat).collect()
    }
    /// Repara saves truncados, colisões de IDs e itens ilegais.
    pub fn sanitize(&mut self) {
        self.bag.retain(|g| SLOTS.iter().any(|(id,_)| *id == g.slot));
        self.worn.retain(|slot,g| slot == &g.slot && SLOTS.iter().any(|(id,_)| id == slot));
        let mut taken = std::collections::BTreeSet::new();
        let mut next = self.next_id;
        for g in self.bag.iter_mut().chain(self.worn.values_mut()) {
            if g.uid == 0 || !taken.insert(g.uid) {
                next = next.saturating_add(1).max(1);
                while taken.contains(&next) && next < u64::MAX { next += 1; }
                g.uid = next; taken.insert(next);
            }
            self.next_id = self.next_id.max(g.uid);
        }
        if self.bag.len() > BAG_LIMIT { self.bag.truncate(BAG_LIMIT); }
    }
}

/// Loot com origem na morte real de um mob. Exige evento de drop do jogo:
/// chance pequena para comuns, maior para elites e chefes, sem auto farm infinita.
pub fn roll_drop(tier: Tier, mob_level: u32, hunter_luck: f64, rng: &mut Rng) -> Option<Gear> {
    let rank = Tier::ALL.iter().position(|t| *t == tier).unwrap_or(0);
    let p = ([0.015, 0.025, 0.055, 0.10, 0.20, 0.32, 0.50][rank]
        * (1.0 + hunter_luck.max(0.0).min(2.0))).min(0.90);
    if !rng.chance(p) { return None; }
    let id = rng.next_u64();
    let slot = SLOTS[(mix(id ^ 0xF00D) % SLOTS.len() as u64) as usize].0;
    let qroll = unit(mix(id ^ 0xCAFE));
    let rarity = if tier >= Tier::Legendary && qroll < 0.06 { 5 }
        else if qroll < 0.003 { 5 }
        else if qroll < 0.021 + rank as f64 * 0.006 { 4 }
        else if qroll < 0.09 + rank as f64 * 0.015 { 3 }
        else if qroll < 0.23 + rank as f64 * 0.018 { 2 }
        else if qroll < 0.53 { 1 } else { 0 };
    Some(Gear { uid: 0, slot: slot.into(), rarity, level: mob_level.clamp(1,10_000), motif: (id % 6) as u8 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slot_swap_is_atomic_and_stats_follow_worn_only() {
        let mut inv = Inventory::default();
        let a = inv.add(Gear {uid:0, slot:"ring".into(), rarity:2, level:11, motif:0}).unwrap();
        let b = inv.add(Gear {uid:0, slot:"ring".into(), rarity:5, level:11, motif:0}).unwrap();
        assert!(inv.bonuses().is_empty());
        inv.equip(a).unwrap(); assert_eq!(inv.bonuses().len(),1);
        inv.equip(b).unwrap(); assert_eq!(inv.bag.len(),1);
        assert_eq!(inv.bag[0].uid, a);
        assert!(inv.unequip("ring").is_ok());
        assert_eq!(inv.bag.len(),2);
    }
    #[test]
    fn codec_and_rolls_have_valid_ids() {
        let mut rng=Rng(14);
        let mut inv=Inventory::default();
        for _ in 0..500 {
            if let Some(g) = roll_drop(Tier::Boss,55,0.0,&mut rng) {
                // Drops have an unassigned id (0). Only the server's
                // inventory may allocate a persistent, non-zero identifier.
                assert_eq!(g.uid,0);
                if inv.can_store() {
                    let id=inv.add(g).unwrap();
                    let saved=inv.bag.last().unwrap();
                    assert_eq!(saved.uid,id);
                    assert_eq!(Gear::decode(&saved.encode()),Some(saved.clone()));
                }
            }
        }
        assert!(inv.bag.len() <= BAG_LIMIT);
        assert!(inv.bag.iter().all(|g| g.uid > 0));
    }
    #[test]
    fn illegal_items_rejected() {
        assert!(Gear::decode("1;hacker;5;100;0").is_none());
    }
}
