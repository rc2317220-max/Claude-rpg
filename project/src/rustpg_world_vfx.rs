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
    let Some(&impact)=IMPACT.get() else{return};
    // Every hit retains a physical contact burst. Critical adds a second,
    // wider arcane ring instead of replacing the impact, so it reads as a
    // stronger event without inventing any gameplay state.
    let base_count=if critical{6}else{4};
    let options=ParticleOptions::simple(impact);
    for i in 0..base_count {
        let a=i as f64*std::f64::consts::TAU/(base_count as f64);
        let dy=(i as f64*0.23).fract()*0.42-0.10;
        let vel=[a.cos()*0.075,0.05+dy,a.sin()*0.075];
        let _=host.add_particle(&options,at,vel);
    }
    if critical {
        let Some(&arcane)=ARCANE.get() else{return};
        let options=ParticleOptions::simple(arcane);
        for i in 0..8 {
            let a=i as f64*std::f64::consts::TAU/8.0+0.3926991;
            let rise=0.035+(i%2) as f64*0.035;
            let vel=[a.cos()*0.13,rise,a.sin()*0.13];
            let _=host.add_particle(&options,at,vel);
        }
    }
}
