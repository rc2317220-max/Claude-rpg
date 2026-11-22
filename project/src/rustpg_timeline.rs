//! Keyframe interpolation: an original, renderer-neutral Rust timeline.
//! Based on animation concepts examined in GeckoLib and Punchy, rather than
//! copied engine classes or animation assets. The GUI samples these at render.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Key {pub frame:f32,pub value:f32}
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Ease {Linear, Smoothstep, OutCubic}
fn ease(t:f32,kind:Ease)->f32 {let t=t.clamp(0.0,1.0);match kind{
    Ease::Linear=>t,
    Ease::Smoothstep=>t*t*(3.0-2.0*t),
    Ease::OutCubic=>1.0-(1.0-t).powi(3),
}}
/// Input keys must be in increasing frame order; bad/NaN keys are rejected.
pub fn sample(keys:&[Key],age:f32,kind:Ease)->Option<f32>{
    if keys.is_empty() || !age.is_finite(){return None}
    if keys.iter().any(|k| !k.frame.is_finite() || !k.value.is_finite()) ||
        keys.windows(2).any(|w|w[0].frame>=w[1].frame){return None}
    if age<=keys[0].frame{return Some(keys[0].value)}
    for pair in keys.windows(2) {
        let a=pair[0];let b=pair[1];
        if age<=b.frame {
            let t=ease((age-a.frame)/(b.frame-a.frame),kind);
            return Some(a.value+(b.value-a.value)*t);
        }
    }
    keys.last().map(|k|k.value)
}
pub const SPARK_KEYS:[Key;4]=[
    Key{frame:0.0,value:0.0},Key{frame:3.0,value:1.0},
    Key{frame:12.0,value:0.65},Key{frame:28.0,value:0.0},
];
pub fn spark(age:u32)->f32 {sample(&SPARK_KEYS,age as f32,Ease::OutCubic).unwrap_or(0.0)}
pub const PULSE_KEYS:[Key;3]=[
    Key{frame:0.0,value:0.1},Key{frame:14.0,value:1.0},Key{frame:28.0,value:0.1},
];
pub fn boss_pulse(tick:u32)->f32 {
    sample(&PULSE_KEYS,(tick%28) as f32,Ease::Smoothstep).unwrap_or(0.0)
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn starts_fades_and_rejects_unordered(){
        assert_eq!(spark(0),0.0);assert!(spark(3)>0.99);assert_eq!(spark(40),0.0);
        assert!(boss_pulse(14)>boss_pulse(0));
        assert_eq!(sample(&[Key{frame:1.0,value:1.0},Key{frame:0.0,value:0.0}],1.0,Ease::Linear),None);
        assert!(sample(&SPARK_KEYS,f32::NAN,Ease::Linear).is_none());
    }
}
