//! Real client particle emission for authoritative hit notices.
//! Uses Starling 0.9 ClientHost::add_particle and vanilla registry keys.
use std::sync::OnceLock;
use mod_api::{self,ClientHost,ParticleKey,ParticleOptions,Registrar};

static IMPACT:OnceLock<ParticleKey>=OnceLock::new();
static ARCANE:OnceLock<ParticleKey>=OnceLock::new();

pub fn register(r:&mut dyn Registrar){
    let _=IMPACT.set(r.particle_key("minecraft:crit"));
    let _=ARCANE.set(r.particle_key("minecraft:enchant"));
}
/// Bounded emission; the server sends one notice per real final hit.
pub fn impact(host:&mut dyn ClientHost,at:[f64;3],critical:bool){
    if at.iter().any(|x| !x.is_finite()) || host.particle_count()>2500{return}
    let Some(&key)= (if critical{ARCANE.get()}else{IMPACT.get()}) else{return};
    let count=if critical{10}else{5};
    let options=ParticleOptions::simple(key);
    for i in 0..count {
        let a=i as f64*std::f64::consts::TAU/(count as f64);
        let dy=(i as f64*0.19).fract()*0.55-0.16;
        let vel=[a.cos()*0.09,dy,a.sin()*0.09];
        let _=host.add_particle(&options,at,vel);
    }
}
