//! Escolha conservadora de XP por estilo de combate. A identidade da arma
//! não é exposta pelo evento de morte: nunca deduzir machados e bestas.
//! Projetéis conhecidos usam a profissão compatível; demais ataques são
//! classificados como combate corpo a corpo por compatibilidade legada.
use super::rustpg_v4_core::{classify_living,CreatureClass};

pub fn kill_profession(victim_kind:&str, damage_type:&str)->&'static str {
    if classify_living(victim_kind)==CreatureClass::Passive {return "hunting";}
    let kind=damage_type.rsplit(':').next().unwrap_or(damage_type);
    match kind {
        "arrow"=>"bow",
        "indirect_magic"|"fireball"=>"survival",
        "trident"=>"trident",
        _=>"sword",
    }
}

pub fn kill_profession_xp(victim_kind:&str,base:u64)->u64 {
    match classify_living(victim_kind){
        CreatureClass::Passive=>4,
        CreatureClass::Hostile=>base,
        _=>0,
    }
}

#[cfg(test)]mod tests {
    use super::*;
    #[test] fn every_vanilla_animal_is_hunting() {
        assert_eq!(kill_profession("minecraft:cow","minecraft:player_attack"),"hunting");
        assert_eq!(kill_profession_xp("minecraft:chicken",0),4);
    }
    #[test] fn known_projectiles_are_not_mislabeled_as_swords(){
        assert_eq!(kill_profession("minecraft:zombie","minecraft:arrow"),"bow");
        assert_eq!(kill_profession("minecraft:skeleton","minecraft:trident"),"trident");
        assert_eq!(kill_profession("minecraft:zombie","minecraft:player_attack"),"sword");
    }
}
