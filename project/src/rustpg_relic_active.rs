//! RustPG v4: active relic effects with persistent cooldowns.
//! Pure domain logic: no ClientHost/ServerHost, networking or world access.
//! Activation requests are validated and applied only by the server bridge.
use super::rustpg_engine_stamina::Stamina;
use super::rustpg_gear::{Inventory, Gear, SLOTS};
use super::rustpg_relic_engine::RelicEffects;

pub const ACTIVE_RELIC_ABI: u16 = 1;
pub const FAMILY_COUNT: usize = 6;
pub const ACTIVE_DURATION_SECS: u64 = 12;
pub const ACTIVE_COOLDOWN_SECS: u64 = 65;
pub const ACTIVE_STAMINA_COST: f64 = 18.0;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ActiveRelics {
    // UNIX seconds, not server ticks: survive world reloads and offline periods.
    pub cooldown_until: [u64; FAMILY_COUNT],
    pub empowered_until: [u64; FAMILY_COUNT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivateError {
    UnknownSlot,
    NotEquipped,
    Tired,
    Cooldown(u64),
}
impl std::fmt::Display for ActivateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSlot => write!(f, "Slot de relíquia desconhecido"),
            Self::NotEquipped => write!(f, "Não há uma relíquia equipada nesse slot"),
            Self::Tired => write!(f, "Stamina insuficiente ou personagem exausto"),
            Self::Cooldown(seconds) => write!(f, "Relíquia recarregando: {seconds}s"),
        }
    }
}

fn valid_worn<'a>(items:&'a Inventory, slot:&str)->Option<&'a Gear> {
    if !SLOTS.iter().any(|(id,_)| *id==slot) {return None;}
    items.worn.get(slot).filter(|g|g.slot==slot && usize::from(g.motif)<FAMILY_COUNT)
}

pub fn label(motif:u8)->&'static str {
    match motif {
        0=>"Bastião do Vigia", 1=>"Pacto do Eclipse", 2=>"Fúria do Abismo",
        3=>"Segundo Fôlego", 4=>"Extração do Caçador",5=>"Graça do Monarca",
        _=>"Relíquia desconhecida",
    }
}
impl ActiveRelics {
    /// Only an equipped, recognized motif can activate; one cooldown per family.
    pub fn activate(&mut self, items:&Inventory, slot:&str, now:u64,
        stamina:&mut Stamina)->Result<String,ActivateError> {
        if !SLOTS.iter().any(|(id,_)|*id==slot) {return Err(ActivateError::UnknownSlot);}
        let item=valid_worn(items,slot).ok_or(ActivateError::NotEquipped)?;
        let i=usize::from(item.motif);
        if now < self.cooldown_until[i] {
            return Err(ActivateError::Cooldown(self.cooldown_until[i]-now));
        }
        // Precheck the full price. Failing to activate must not drain to zero.
        if stamina.exhausted || !stamina.current.is_finite()
            || stamina.current+1e-9 < ACTIVE_STAMINA_COST {
            return Err(ActivateError::Tired);
        }
        if !stamina.consume(ACTIVE_STAMINA_COST) {return Err(ActivateError::Tired);}
        self.empowered_until[i]=now.saturating_add(ACTIVE_DURATION_SECS);
        self.cooldown_until[i]=now.saturating_add(ACTIVE_COOLDOWN_SECS);
        Ok(format!("{} ativado por {}s · recarga {}s",label(item.motif),
            ACTIVE_DURATION_SECS,ACTIVE_COOLDOWN_SECS))
    }
    pub fn remaining(&self, motif:u8, now:u64)->(u64,u64) {
        let i=usize::from(motif);
        if i>=FAMILY_COUNT {return (0,0);}
        (self.empowered_until[i].saturating_sub(now),
         self.cooldown_until[i].saturating_sub(now))
    }
    pub fn status(&self, item:&Gear, now:u64)->String {
        let (active,cooldown)=self.remaining(item.motif,now);
        if active>0 {format!("ATIVO {active}s")}
        else if cooldown>0 {format!("RECARGA {cooldown}s")}
        else {"ATIVAR".into()}
    }
    /// Never expose an active effect after unequipping the corresponding motif.
    pub fn bonuses(&self, items:&Inventory, now:u64)->RelicEffects {
        let mut out=RelicEffects::default();
        for (slot,item) in &items.worn {
            if valid_worn(items,slot).is_none() {continue;}
            let i=usize::from(item.motif);
            if i>=FAMILY_COUNT || self.empowered_until[i]<=now {continue;}
            match i {
                0=>out.guard=out.guard.max(0.14),
                1=>out.life_steal=out.life_steal.max(0.09),
                2=>out.attack_bonus=out.attack_bonus.max(0.12),
                3=>out.stamina_regen=out.stamina_regen.max(2.4),
                4=>out.mining_bonus=out.mining_bonus.max(0.18),
                5=>out.stamina_discount=out.stamina_discount.max(0.20),
                _=>{},
            }
        }
        out
    }
    pub fn encode(&self)->String {
        self.cooldown_until.iter().chain(self.empowered_until.iter())
            .map(u64::to_string).collect::<Vec<_>>().join(";")
    }
    /// Reject invalid or partial saves atomically, rather than mixing cooldowns.
    pub fn decode(value:&str)->Option<Self> {
        if value.len()>256 {return None;}
        let values=value.split(';').map(str::parse::<u64>)
            .collect::<Result<Vec<_>,_>>().ok()?;
        if values.len()!=FAMILY_COUNT*2 {return None;}
        let mut out=Self::default();
        out.cooldown_until.copy_from_slice(&values[..FAMILY_COUNT]);
        out.empowered_until.copy_from_slice(&values[FAMILY_COUNT..]);
        for i in 0..FAMILY_COUNT {
            // Expired/past dates are harmless; future buffs may not extend
            // beyond their corresponding cooldown deadline.
            if out.empowered_until[i]>out.cooldown_until[i] {return None;}
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wearing(motif:u8)->Inventory {
        let mut i=Inventory::default();
        let item=Gear{uid:1,slot:"ring".into(),rarity:3,level:12,motif};
        i.worn.insert("ring".into(),item);
        i
    }
    #[test] fn all_six_actives_have_real_distinct_effects() {
        for motif in 0..6 {
            let mut stamina=Stamina::new(100.0,0.0,0);
            let items=wearing(motif);
            let mut active=ActiveRelics::default();
            assert!(active.activate(&items,"ring",100,&mut stamina).is_ok());
            let b=active.bonuses(&items,101);
            let flags=[b.guard>0.0,b.life_steal>0.0,b.attack_bonus>0.0,
                b.stamina_regen>0.0,b.mining_bonus>0.0,b.stamina_discount>0.0];
            assert_eq!(flags.iter().filter(|&&x|x).count(),1);
        }
    }
    #[test] fn cooldown_survives_roundtrip_and_rejects_early_retrigger() {
        let items=wearing(2);
        let mut stamina=Stamina::new(100.0,0.0,0);
        let mut active=ActiveRelics::default();
        active.activate(&items,"ring",100,&mut stamina).unwrap();
        let saved=ActiveRelics::decode(&active.encode()).unwrap();
        assert_eq!(saved,active);
        assert!(matches!(active.activate(&items,"ring",120,&mut stamina),
            Err(ActivateError::Cooldown(45))));
        assert_eq!(active.bonuses(&items,113),RelicEffects::default());
        assert!(active.activate(&items,"ring",165,&mut stamina).is_ok());
    }
    #[test] fn unequipping_disables_the_buff_and_invalid_requests_are_free() {
        let mut stamina=Stamina::new(100.0,0.0,0);
        let before=stamina.current;
        let mut active=ActiveRelics::default();
        assert_eq!(active.activate(&Inventory::default(),"ring",10,&mut stamina),
            Err(ActivateError::NotEquipped));
        assert_eq!(stamina.current,before);
        let items=wearing(0);
        active.activate(&items,"ring",10,&mut stamina).unwrap();
        assert_eq!(active.bonuses(&Inventory::default(),11),RelicEffects::default());
    }
    #[test] fn corrupted_saved_state_fails_closed() {
        assert!(ActiveRelics::decode("not a state").is_none());
        assert!(ActiveRelics::decode("0;1").is_none());
    }
}
