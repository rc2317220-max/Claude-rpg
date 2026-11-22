// Regras de ação do RPG Engine, separadas da fórmula de dano já existente.
use super::crpg_engine_attributes::EngineAttributes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Sprint,
    HeavyAttack,
    Dodge,
    Skill,
}

pub fn cost(action: Action, attrs: EngineAttributes) -> f64 {
    match action {
        Action::Sprint => attrs.sprint_cost,
        Action::HeavyAttack => attrs.heavy_attack_cost,
        Action::Dodge => attrs.dodge_cost,
        Action::Skill => attrs.skill_cost,
    }
}

pub fn is_heavy_weapon(item: &str) -> bool {
    let item = item.to_ascii_lowercase();
    item.contains("mace") || item.contains("axe") || item.contains("hammer")
}
