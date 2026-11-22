// Atributos efetivos do RPG Engine. Não substitui o catálogo de nodes: é a
// fronteira que transforma Stats persistidos em números usados pelos sistemas.
use super::crpg_num::finite;
use super::crpg_profile::Stats;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineAttributes {
    pub max_health: f64,
    pub health_regen: f64,
    pub stamina_max: f64,
    pub stamina_regen: f64,
    pub sprint_cost: f64,
    pub heavy_attack_cost: f64,
    pub dodge_cost: f64,
    pub skill_cost: f64,
}

impl Default for EngineAttributes {
    fn default() -> Self {
        Self {
            max_health: 20.0,
            health_regen: 0.0,
            stamina_max: 100.0,
            stamina_regen: 6.0,
            sprint_cost: 1.0,
            heavy_attack_cost: 12.0,
            dodge_cost: 18.0,
            skill_cost: 10.0,
        }
    }
}

impl EngineAttributes {
    pub fn from_stats(s: &Stats) -> Self {
        let d = Self::default();
        let max_health = (d.max_health + s.get("defense.max_health_flat"))
            * (1.0 + s.get("defense.max_health_pct")).max(0.0);
        let exploration_stamina=s.get("exploration.stamina").clamp(0.0,10.0);
        let stamina_max = (d.stamina_max + s.get("general.stamina_max_flat"))
            * (1.0 + s.get("general.stamina_max_pct") + exploration_stamina).max(0.0);
        Self {
            max_health: finite(max_health.max(1.0)),
            health_regen: finite((d.health_regen + s.get("defense.regen")).max(0.0)),
            stamina_max: finite(stamina_max.max(1.0)),
            stamina_regen: finite(
                (d.stamina_regen * (1.0 + s.get("general.stamina_regen_pct"))
                    + s.get("general.stamina_regen_flat"))
                .max(0.0),
            ),
            sprint_cost: finite(
                (d.sprint_cost * (1.0 - s.get("general.stamina_efficiency"))).max(0.05),
            ),
            heavy_attack_cost: finite(
                (d.heavy_attack_cost * (1.0 - s.get("general.heavy_attack_cost_reduction")))
                    .max(0.5),
            ),
            dodge_cost: finite(
                (d.dodge_cost * (1.0 - s.get("general.dodge_cost_reduction"))).max(0.5),
            ),
            skill_cost: finite(
                (d.skill_cost * (1.0 - s.get("general.skill_cost_reduction"))).max(0.5),
            ),
        }
    }
}

#[cfg(test)] mod exploration_stamina_tests {
    use super::*;
    #[test]fn exploration_stamina_only_increases_max_pool(){
        let mut s=Stats::default();s.add("exploration.stamina",0.25);
        let a=EngineAttributes::from_stats(&s);
        assert!((a.stamina_max-125.0).abs()<0.001);
        assert_eq!(a.stamina_regen,EngineAttributes::default().stamina_regen);
    }
}
