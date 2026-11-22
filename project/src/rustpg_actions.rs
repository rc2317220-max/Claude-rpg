//! Protocolo estrito das operações que alteram o perfil.
//! Apenas o servidor executa operações; o cliente envia identificadores, nunca saldos.
use super::rustpg_gear::SLOTS;
use super::rustpg_guard::Stance;

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Equip(u64),
    Unequip(String),
    Salvage(u64),
    Buy(String),
    Respec,
    Activate(String),
    Defend(Stance),
    Power(super::rustpg_powers::Power),
}

/// Formato v1: ASCII curto, sem UUIDs, estatísticas, valores monetários ou payloads aninhados.
pub fn parse(data: &[u8]) -> Option<Action> {
    if data.is_empty() || data.len() > 96 || !data.is_ascii() { return None; }
    let raw=std::str::from_utf8(data).ok()?;
    let (kind,arg)=raw.split_once(':')?;
    if arg.is_empty() || arg.contains(':') || arg.bytes().any(|b| b <= 32 || b >= 127) {return None;}
    match kind {
        "power" if arg=="fury" => Some(Action::Power(super::rustpg_powers::Power::Fury)),
        "power" if arg=="barrier" => Some(Action::Power(super::rustpg_powers::Power::Barrier)),
        "power" if arg=="siphon" => Some(Action::Power(super::rustpg_powers::Power::Siphon)),
        "defend" if arg=="parry" => Some(Action::Defend(Stance::Parry)),
        "defend" if arg=="dodge" => Some(Action::Defend(Stance::Dodge)),
        "respec" if arg=="confirm" => Some(Action::Respec),
        "activate" if SLOTS.iter().any(|(slot,_)|*slot==arg) => Some(Action::Activate(arg.to_owned())),
        "equip" => Some(Action::Equip(arg.parse::<u64>().ok().filter(|n|*n>0)?)),
        "salvage" => Some(Action::Salvage(arg.parse::<u64>().ok().filter(|n|*n>0)?)),
        "unequip" if SLOTS.iter().any(|(slot,_)| *slot==arg) => Some(Action::Unequip(arg.to_owned())),
        "buy" if arg.len()<=90 && arg.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'.'||b==b'-'||b==b'#')
            => Some(Action::Buy(arg.to_owned())),
        _=>None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_noncanonical_or_hostile_packets() {
        for bad in ["buy:","buy:no:extra","equip:0","equip:-1","unequip:hacker", "salvage:xx", "buy:../../saves", "defend:admin", "equip:18446744073709551616"] {
            assert_eq!(parse(bad.as_bytes()),None, "{bad}");
        }
        assert_eq!(parse(&vec![b'x';128]),None);
        assert_eq!(parse(&[0xff]),None);
    }
    #[test]
    fn accepts_only_whitelisted_actions() {
        assert_eq!(parse(b"defend:parry"),Some(Action::Defend(Stance::Parry)));
        assert_eq!(parse(b"defend:dodge"),Some(Action::Defend(Stance::Dodge)));
        assert_eq!(parse(b"respec:confirm"),Some(Action::Respec));
        assert_eq!(parse(b"activate:ring"),Some(Action::Activate("ring".into())));
        assert_eq!(parse(b"activate:invalid"),None);
        assert_eq!(parse(b"equip:42"),Some(Action::Equip(42)));
        assert_eq!(parse(b"unequip:ring"),Some(Action::Unequip("ring".into())));
        assert_eq!(parse(b"salvage:7"),Some(Action::Salvage(7)));
        assert_eq!(parse(b"buy:sword.power"),Some(Action::Buy("sword.power".into())));
        // Every original three-rank node contains '#': this was blocked in 4.0.5.
        assert_eq!(parse(b"buy:weapon.sword.damage_flat#1"),
            Some(Action::Buy("weapon.sword.damage_flat#1".into())));
    }
}
