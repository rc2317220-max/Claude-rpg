//! Starling/Horizon 0.9.0: catálogo de atributos nativos com estado autoritativo.
//!
//! O catálogo desta versão contém somente efeitos com consumidores reais na API
//! pública. Recursos que exigiam patches do motor foram removidos do RustPG.
//!
//! Conferido contra o código 0.9.0: AttributesRegistrar e EntityAttributesRegistrar,
//! living_attribute_value e living_replace_attribute_modifier.
use crate::{rustpg_attributes, rustpg_profile::Stats};
use mod_api::{Registrar, Sentiment};
use std::collections::BTreeMap;

/// Caracteres usados pelos IDs dos atributos e modifiers são válidos no
/// esquema de Identifier (pontos são válidos em caminhos namespaced).
pub fn attribute_id(path: &str) -> String { format!("rustpg:{path}") }
pub fn modifier_id(path: &str) -> String {
    format!("rustpg:skill_{}", path.replace('.', "_"))
}

/// Registra TODO o catálogo existente (inclusive nodes futuros). Os flags
/// applied/wired NÃO mudam a mecânica do jogo: controlam apenas sincronização.
/// Nenhum atributo do tipo é inserido nas criaturas por acidente: o perfil é
/// individual do jogador; o mob scaling já utiliza atributos vanilla próprios.
pub fn register(registrar: &mut dyn Registrar) {
    let mut player_attributes = Vec::new();
    for def in rustpg_attributes::all() {
        let max = def.cap.unwrap_or(1_000_000.0).max(1.0);
        // Base zero: o efeito calculado de skills/equipamento é um modificador
        // nomeado e temporário. Valores negativos de equipamentos são válidos.
        let _ = registrar.declare_attribute(
            &def.id, 0.0, -1_000_000.0, max, def.applied,
            Sentiment::Positive, Default::default()
        );
        player_attributes.push((attribute_id(&def.id), None));
    }
    // Keystone fora do catálogo de atributos comuns: o registro especial do
    // encantamento também deve existir na camada nativa.
    let _ = registrar.declare_attribute("enchanting.transcendence", 0.0, 0.0, 1.0,
        true, Sentiment::Positive, Default::default());
    player_attributes.push((attribute_id("enchanting.transcendence"),None));
    // Este tipo consta na tabela de atributos do jogo 0.9.0.
    let _ = registrar.modify_entity_attributes("minecraft:player", player_attributes);
    // O limite vanilla seria baixo para o RPG; ampliar é diferente de
    // sobrescrever bases e é compatível com outros mods.
    for (name, lo, hi) in [
        ("minecraft:max_health",1.0,1_000_000.0),
        ("minecraft:armor",0.0,2048.0),
        ("minecraft:armor_toughness",0.0,2048.0),
        ("minecraft:attack_speed",0.0,1024.0),
    ] {
        let _ = registrar.widen_attribute_bounds(name,lo,hi);
    }
}

/// Somente aplicar modificadores conhecidos e adquiridos; o registro vazio de
/// um atributo NÃO concede habilidade. Este adaptador não muta o Profile.
/// Retorno: valores aceitos, usados como cache para evitar escrita a cada tick.
pub fn reconcile(
    host: &mut dyn mod_api::ServerHost,
    living: mod_api::LivingRef,
    desired: &Stats,
    previously: &BTreeMap<String,f64>,
) -> BTreeMap<String,f64> {
    let mut accepted=BTreeMap::new();
    // Conjunto finito: IDs do catálogo que têm modificadores adquiridos agora
    // ou anteriormente. Se o jogador resetou a árvore, removemos o antigo.
    for key in desired.0.keys().chain(previously.keys()) {
        if accepted.contains_key(key) { continue; }
        let cap=match rustpg_attributes::get(key) {
            Some(def)=>def.cap,
            None if key=="enchanting.transcendence"=>Some(1.0),
            None=>continue,
        };
        let amount=desired.get(key);
        let Some(amount)=clean_value(amount,cap) else {continue;};
        let prev=previously.get(key).copied().unwrap_or(0.0);
        if amount==0.0 && prev==0.0 {continue;}
        let Ok(id)=mod_api::Identifier::parse(&modifier_id(key)) else {continue;};
        let attribute=attribute_id(key);
        // Se a entidade foi recriada (relogin/morte/dimensão), o chamador passa
        // um cache vazio e forçamos uma nova aplicação.
        if (amount-prev).abs()<0.000000001 {
            accepted.insert(key.clone(),amount);
            continue;
        }
        if amount.abs()<0.000000001 {
            let _=living.remove_attribute_modifier(host,&attribute,&id);
            continue;
        }
        let ok=living.replace_attribute_modifier(host,&attribute,
            mod_api::AttributeModifier::new(id,amount,mod_api::AttributeOperation::AddValue),
            mod_api::ModifierLifetime::Transient);
        if ok {accepted.insert(key.clone(),amount);}
    }
    accepted
}

/// Atributos que hooks existentes efetivamente consomem são lidos da entidade
/// (incluindo modificadores externos), depois da reconciliação do nosso bônus.
/// Outros permanecem no Profile até ganharem seus hooks reais.
pub fn effective_in_hooks(
    host: &dyn mod_api::ServerHost,
    living: mod_api::LivingRef,
    stats: &mut Stats,
) {
    for (key,value) in stats.0.iter_mut() {
        let enabled=rustpg_attributes::get(key).is_some_and(|def|def.applied)
            || key=="enchanting.transcendence";
        if !enabled {continue;}
        if let Some(actual)=living.attribute_value(host,&attribute_id(key)) {
            if actual.is_finite() {*value=actual;}
        }
    }
}

fn clean_value(value:f64, cap:Option<f64>) -> Option<f64> {
    if !value.is_finite() {return None;}
    let upper=cap.unwrap_or(1_000_000.0).max(1.0);
    Some(value.clamp(-1_000_000.0,upper))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ids_unique_with_paths() {
        let mut ids=std::collections::BTreeSet::new();
        assert!(ids.insert(modifier_id("enchanting.transcendence")));
        for a in rustpg_attributes::all() {
            assert!(ids.insert(modifier_id(&a.id)));
            assert!(a.id.bytes().all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || b"._/-".contains(&x)));
        }
    }
    #[test] fn bounded_modifiers_are_finite() {
        assert_eq!(clean_value(0.5,Some(0.1)),Some(0.5));
        assert_eq!(clean_value(f64::NAN,None),None);
        assert_eq!(clean_value(10.0,Some(1.0)),Some(1.0));
    }
}
