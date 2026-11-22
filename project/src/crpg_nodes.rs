// The skill-tree node bank, generated from the attribute catalogue: every attribute
// gets a chain of three nodes (Iniciante → Especialista → Mestre) in its own tree,
// plus unique keystone nodes. Each node changes exactly one attribute.
use super::crpg_attributes::{self as attributes, Unit};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Formula {
    /// value = per_rank × rank, no ceiling but the node's max rank.
    Linear { per_rank: f64 },
    /// value = cap × rank / (rank + half): approaches `cap` (used for reductions).
    Diminishing { cap: f64, half: f64 },
    /// A switch: 1.0 once bought.
    Flag,
}

impl Formula {
    pub fn value(&self, rank: u64) -> f64 {
        let r = rank as f64;
        match *self {
            Formula::Linear { per_rank } => super::crpg_num::finite(per_rank * r),
            Formula::Diminishing { cap, half } => super::crpg_num::diminishing(r, half, cap),
            Formula::Flag => {
                if rank > 0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
    pub fn describe(&self) -> String {
        match *self {
            Formula::Linear { per_rank } => format!("{per_rank} × rank"),
            Formula::Diminishing { cap, half } => format!("{cap} × rank / (rank + {half})"),
            Formula::Flag => "ativa ao comprar".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// An item id drawn as the node's icon.
    pub icon: &'static str,
    pub tree: String,
    pub category: &'static str,
    /// None: no gameplay ceiling (bounded only by u64).
    pub max_rank: Option<u64>,
    pub cost_base: u64,
    /// One more point per `cost_step` ranks already bought (0: flat cost).
    pub cost_step: u64,
    pub req_level: u32,
    /// Required level of the tree's proficiency.
    pub req_prof: u32,
    pub prereqs: Vec<(String, u64)>,
    /// Mutually exclusive nodes, enforced on the server at purchase time.
    pub exclusions: Vec<String>,
    pub attribute: String,
    pub formula: Formula,
}

impl NodeDef {
    /// Points to buy the rank after `rank`.
    pub fn cost_at(&self, rank: u64) -> u64 {
        let extra = if self.cost_step == 0 {
            0
        } else {
            rank / self.cost_step
        };
        self.cost_base.saturating_add(extra)
    }
    pub fn max_label(&self) -> String {
        self.max_rank.map_or("∞".into(), |m| m.to_string())
    }
}

/// The proficiency whose points and level a tree uses.
pub fn tree_proficiency(tree: &str) -> &str {
    if let Some(reference)=tree.strip_prefix("reference.") {
        return match reference{"mining"=>"mining","combat"=>"survival",_=>"general"};
    }
    if let Some(w) = tree.strip_prefix("weapon.") {
        return w;
    }
    if let Some(rest) = tree.strip_prefix("luck.") {
        return match rest {
            "mining" => "mining",
            "fishing" => "fishing",
            "farming" => "farming",
            "mobs" | "bosses" => "looting",
            "chests" | "exploration" => "exploration",
            "crafting" => "crafting",
            _ => "general",
        };
    }
    match tree {
        "defense" => "survival",
        "inventory" => "general",
        t => t,
    }
}

fn icon_for(tree: &str) -> &'static str {
    match tree {
        "weapon.sword" => "minecraft:iron_sword",
        "weapon.axe" => "minecraft:iron_axe",
        "weapon.bow" => "minecraft:bow",
        "weapon.crossbow" => "minecraft:crossbow",
        "weapon.mace" => "minecraft:mace",
        "weapon.trident" => "minecraft:trident",
        "weapon.spear" => "minecraft:iron_spear",
        "weapon.unarmed" => "minecraft:leather",
        "defense" => "minecraft:shield",
        "mining" => "minecraft:iron_pickaxe",
        "farming" => "minecraft:wheat",
        "fishing" => "minecraft:fishing_rod",
        "woodcutting" => "minecraft:oak_log",
        "excavation" => "minecraft:iron_shovel",
        "husbandry" => "minecraft:wheat_seeds",
        "hunting" => "minecraft:rabbit_hide",
        "exploration" => "minecraft:compass",
        "looting" => "minecraft:chest",
        "enchanting" => "minecraft:enchanted_book",
        "smithing" => "minecraft:anvil",
        "crafting" => "minecraft:crafting_table",
        "alchemy" => "minecraft:brewing_stand",
        "cooking" => "minecraft:furnace",
        "trading" => "minecraft:emerald",
        "taming" => "minecraft:bone",
        "survival" => "minecraft:golden_apple",
        "inventory" => "minecraft:chest",
        t if t.starts_with("luck.") => "minecraft:rabbit_foot",
        _ => "minecraft:experience_bottle",
    }
}

fn tree_of(attr: &str) -> String {
    let parts: Vec<&str> = attr.split('.').collect();
    match parts[0] {
        "weapon" | "luck" => format!("{}.{}", parts[0], parts[1]),
        p => p.to_string(),
    }
}

fn per_rank(unit: Unit, attr: &str) -> Formula {
    if attr.starts_with("inventory.stack_") {
        return Formula::Linear { per_rank: 1.0 };
    }
    if attr.ends_with("move_speed") {
        return Formula::Linear { per_rank: 0.001 };
    }
    match unit {
        Unit::Flat if attr.contains("health") => Formula::Linear { per_rank: 1.0 },
        Unit::Flat if attr.contains("armor") || attr.contains("toughness") => {
            Formula::Linear { per_rank: 0.5 }
        }
        Unit::Flat => Formula::Linear { per_rank: 0.25 },
        Unit::Percent => Formula::Linear { per_rank: 0.01 },
        Unit::Chance => Formula::Linear { per_rank: 0.002 },
        Unit::Count => Formula::Linear { per_rank: 0.02 },
        Unit::Reduction => Formula::Diminishing {
            cap: 0.9,
            half: 200.0,
        },
    }
}

const TIERS: &[(&str, Option<u64>, u64, u64, u32, u32, f64)] = &[
    // (title, max rank, cost base, cost step, general level, proficiency level, effect scale)
    ("Iniciante", Some(100), 1, 0, 1, 0, 1.0),
    ("Especialista", Some(1_000), 2, 0, 15, 10, 1.5),
    ("Mestre", None, 3, 500, 40, 30, 2.0),
];

fn build() -> Vec<NodeDef> {
    let mut out = Vec::new();
    for a in attributes::all() {
        let tree = tree_of(&a.id);
        let base = per_rank(a.unit, &a.id);
        let mut prev: Option<String> = None;
        for (i, (title, max, cost, step, lvl, prof, scale)) in TIERS.iter().enumerate() {
            // Player movement uses a 50% cap so the 10x 5% imported tree
            // remains traversable; NPC scaling retains its 10% separate cap.
            if (a.id.ends_with("move_speed") || a.id.starts_with("inventory.stack_")) && i > 0 {
                break;
            }
            let formula = match base {
                Formula::Linear { per_rank } => Formula::Linear {
                    per_rank: per_rank * scale,
                },
                f => f,
            };
            let max = if a.id.ends_with("move_speed") {
                Some(100)
            } else if a.id.starts_with("inventory.stack_") {
                Some(12)
            } else {
                *max
            };
            let id = format!("{}#{}", a.id, i + 1);
            let unit_text = match a.unit {
                Unit::Flat => "pontos",
                Unit::Percent => "(fração; 0,01 = 1%)",
                Unit::Chance => "de chance (fração)",
                Unit::Count => "(contagem esperada)",
                Unit::Reduction => "de redução",
            };
            out.push(NodeDef {
                id: id.clone(),
                name: format!("{} — {}", a.name, title),
                description: format!(
                    "Aumenta somente \"{}\" em {} {} por rank. Não altera nenhum outro atributo.",
                    a.name,
                    formula.describe(),
                    unit_text
                ),
                icon: icon_for(&tree),
                tree: tree.clone(),
                category: a.category,
                max_rank: max,
                cost_base: *cost,
                cost_step: *step,
                req_level: *lvl,
                req_prof: *prof,
                prereqs: prev
                    .iter()
                    .map(|p| (p.clone(), if i == 1 { 25 } else { 100 }))
                    .collect(),
                exclusions:Vec::new(),
                attribute: a.id.clone(),
                formula,
            });
            prev = Some(id);
        }
    }
    // New V4 cross-attribute synergy paths, without modifying legacy node IDs.
    // Each bridge unlocks only ONE explicitly named attribute; two independent
    // training paths are prerequisite, creating real branches like graph-based
    // RPG skill engines while keeping historical saves compatible.
    let mut groups=std::collections::BTreeMap::<String,Vec<NodeDef>>::new();
    for node in &out {
        if node.id.ends_with("#1") {groups.entry(node.tree.clone()).or_default().push(node.clone());}
    }
    for (tree,nodes) in groups {
        // Inventory stack caps are deliberately 12-rank tracks (16→64 and
        // 4→16). Generic bridges demand rank 20 and must not create an
        // impossible prerequisite for these finite-cap tracks.
        if tree=="inventory" {continue;}
        for (i,pair) in nodes.chunks_exact(2).enumerate() {
            let left=&pair[0];let right=&pair[1];
            let formula=match left.formula {
                Formula::Linear{per_rank}=>Formula::Linear{per_rank:per_rank*0.50},
                Formula::Diminishing{cap,half}=>Formula::Diminishing{cap:cap*0.20,half:half*2.0},
                Formula::Flag=>continue,
            };
            out.push(NodeDef {
                id:format!("bridge.{tree}.{i}"),
                name:format!("Convergência — {}",left.name),
                description:format!(
                    "Sinergia entre {} e {}. Aumenta exclusivamente {} em {} por rank; outras estatísticas não mudam.",
                    left.name,right.name,left.name,formula.describe()),
                icon:icon_for(&tree),
                tree:tree.clone(),category:left.category,max_rank:Some(100),
                cost_base:3,cost_step:25,req_level:20,req_prof:12,
                prereqs:vec![(left.id.clone(),20),(right.id.clone(),20)],
                exclusions:Vec::new(),
                attribute:left.attribute.clone(),formula,
            });
        }
    }
    // Keystones: unique switches, each one attribute.
    out.push(NodeDef {
        id: "enchanting.transcendence".into(),
        name: "Transcendência Arcana".into(),
        description: "Remove o teto de nível dos encantamentos aplicados por livros nível I."
            .into(),
        icon: "minecraft:enchanted_book",
        tree: "enchanting".into(),
        category: "profession",
        max_rank: Some(1),
        cost_base: 50,
        cost_step: 0,
        req_level: 60,
        req_prof: 50,
        prereqs: vec![("enchanting.mastery#3".into(), 100)],
        exclusions:Vec::new(),
        attribute: "enchanting.transcendence".into(),
        formula: Formula::Flag,
    });
    // Active skills: independently purchased, exactly one unlock per node.
    // They are backed by the server-authoritative crpg_powers module.
    for (id,name,desc,level,cost,icon) in [
        ("power.fury","Ímpeto","Dano corpo a corpo +25% por 4 segundos; recarga 12 segundos",5,8,"minecraft:iron_sword"),
        ("power.barrier","Égide","Redução de 28% do dano por 5 segundos; recarga 14 segundos",10,12,"minecraft:shield"),
        ("power.siphon","Sifão","Cura 9% do dano corpo a corpo durante 5 segundos",15,16,"minecraft:ghast_tear")]
    {
        out.push(NodeDef{id:id.into(),name:name.into(),description:desc.into(),icon,
            tree:"powers".into(),category:"combat",max_rank:Some(1),cost_base:cost,
            cost_step:0,req_level:level,req_prof:0,prereqs:Vec::new(),
            exclusions:Vec::new(),attribute:id.into(),formula:Formula::Flag});
    }
    crate::crpg_puffish_graph::add_nodes(&mut out);
    out
}

pub fn all() -> &'static [NodeDef] {
    static ALL: OnceLock<Vec<NodeDef>> = OnceLock::new();
    ALL.get_or_init(build)
}

/// Conservative runtime availability: a node must feed an attribute the
/// current engine explicitly marks as applied. Preserve legacy node IDs.
/// This is a declaration audit, not proof of an observed gameplay effect.
pub fn runtime_supported(node: &NodeDef) -> bool {
    node.attribute.starts_with("power.") ||
        attributes::get(&node.attribute).is_some_and(|attribute| attribute.applied)
}

pub fn get(id: &str) -> Option<&'static NodeDef> {
    // Built once; the index makes lookups O(log n) over 1000+ nodes.
    static INDEX: OnceLock<std::collections::BTreeMap<&'static str, usize>> = OnceLock::new();
    let index = INDEX.get_or_init(|| {
        all()
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect()
    });
    index.get(id).map(|&i| &all()[i])
}

/// Trees, in display order.
pub fn trees() -> Vec<String> {
    let mut seen = Vec::<String>::new();
    for n in all() {
        if !seen.contains(&n.tree) {
            seen.push(n.tree.clone());
        }
    }
    seen
}

pub fn in_tree(tree: &str) -> Vec<&'static NodeDef> {
    all().iter().filter(|n| n.tree == tree).collect()
}

#[cfg(test)]
mod graph_integrity_tests {
    use super::*;
    #[test] fn all_links_point_to_real_nodes_in_same_tree() {
        let mut ids=std::collections::HashSet::new();
        for node in all() { assert!(ids.insert(node.id.as_str()),"duplicate {}",node.id); }
        for node in all() {
            for exclusion in &node.exclusions {
                let other=get(exclusion).expect("exclusive node must exist");
                assert_eq!(other.tree,node.tree);
                assert!(other.exclusions.contains(&node.id));
            }
            for (parent,need) in &node.prereqs {
                let p=get(parent).unwrap_or_else(||panic!("missing prereq {parent}"));
                assert_eq!(p.tree,node.tree);
                assert!(*need>0);
                assert!(p.max_rank.is_none_or(|m|m>=*need));
                assert_ne!(p.id,node.id);
            }
        }
    }
    #[test] fn every_node_has_a_defined_and_finite_increment() {
        for node in all() {
            let low=node.formula.value(0);
            let high=node.formula.value(1);
            assert!(low.is_finite() && high.is_finite(),"{}",node.id);
            assert!(high>low,"{} has no first-rank effect",node.id);
            if runtime_supported(node) {
                assert!(attributes::get(&node.attribute).is_some()
                    || node.attribute=="enchanting.transcendence"
                    || node.attribute.starts_with("power."));
            }
        }
    }
    #[test] fn unfinished_enchantment_stacking_cannot_consume_player_points() {
        let node=get("enchanting.transcendence").expect("keystone remains in tree");
        assert!(!runtime_supported(node));
        assert!(matches!(crate::crpg_profile::Profile::new().check_buy(node),
            Err(crate::crpg_profile::BuyError::UnimplementedEffect)));
    }
    #[test] fn healing_and_jump_are_purchasable_only_with_native_hooks() {
        for (id,cap) in [("general.healing_bonus",3.0),("general.jump_bonus",1.5)] {
            let attr=attributes::get(id).expect("native attribute registered");
            assert!(attr.applied,"{id} must have a real runtime hook");
            assert_eq!(attr.cap,Some(cap));
            let node=get(&format!("{id}#1")).expect("first skill-tree node");
            assert!(runtime_supported(node));
        }
    }
    #[test] fn stack_capacity_is_a_real_twelve_rank_general_tree() {
        for id in ["inventory.stack_64_rank#1","inventory.stack_16_rank#1"] {
            let node=get(id).expect("stack capacity node");
            assert!(runtime_supported(node));
            assert_eq!(node.tree,"inventory");
            assert_eq!(node.max_rank,Some(12));
            assert_eq!(node.formula.value(12),12.0);
        }
        assert!(get("inventory.stack_64_rank#2").is_none());
    }
    #[test] fn bridges_are_real_and_preserve_original_tiers() {
        assert!(all().iter().any(|n|n.id.starts_with("bridge.") && n.prereqs.len()==2));
        assert!(all().iter().any(|n|n.id.ends_with("#1")));
        assert!(all().iter().any(|n|n.id.ends_with("#2")));
        assert!(all().iter().any(|n|n.id.ends_with("#3")));
    }
}
