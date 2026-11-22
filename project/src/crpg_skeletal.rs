//! Original, renderer-independent bone animation sampler.
//! Based on the *concepts* examined in GeckoLib / Epic Fight / Punchy:
//! named bones, per-axis keyframes, separate clip time and blend weights.
//! No GeckoLib Java code or copyrighted animation data is embedded.
//! This core does not bind to Starling's entity renderer yet.
use std::collections::BTreeMap;

#[derive(Clone,Copy,Debug,Default,PartialEq)]
pub struct Vec3 {pub x:f32,pub y:f32,pub z:f32}
impl Vec3 {
    fn lerp(self,b:Self,t:f32)->Self {
        let t=t.clamp(0.0,1.0);
        Self{x:self.x+(b.x-self.x)*t,y:self.y+(b.y-self.y)*t,z:self.z+(b.z-self.z)*t}
    }
    fn finite(self)->bool{self.x.is_finite()&&self.y.is_finite()&&self.z.is_finite()}
}
#[derive(Clone,Copy,Debug,Default,PartialEq)]
pub struct Pose {pub pivot:Vec3,pub rotation:Vec3,pub scale:Vec3}
impl Pose {
    fn interpolate(self,b:Self,t:f32)->Self {
        Self{pivot:self.pivot.lerp(b.pivot,t),rotation:self.rotation.lerp(b.rotation,t),
            scale:self.scale.lerp(b.scale,t)}
    }
    pub fn identity()->Self {Self{pivot:Vec3::default(),rotation:Vec3::default(),
        scale:Vec3{x:1.0,y:1.0,z:1.0}}}
    pub fn finite(self)->bool{self.pivot.finite()&&self.rotation.finite()&&self.scale.finite()}
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Key {pub at:f32,pub pose:Pose}
#[derive(Clone,Debug,Default)]
pub struct Track {pub bone:String,pub keys:Vec<Key>}
impl Track {
    pub fn valid(&self)->bool{
        !self.bone.is_empty() && self.bone.len()<=128 && self.keys.len()<=256 &&
        !self.keys.is_empty() && self.keys.iter().all(|k|k.at.is_finite()&&k.at>=0.0&&k.pose.finite()) &&
        self.keys.windows(2).all(|k|k[1].at>k[0].at)
    }
    pub fn sample(&self,time:f32)->Option<Pose>{
        if !self.valid()||!time.is_finite(){return None;}
        let first=self.keys.first().copied()?;
        if time<=first.at {return Some(first.pose);}
        for pair in self.keys.windows(2){
            if time<=pair[1].at {
                let t=(time-pair[0].at)/(pair[1].at-pair[0].at);
                return Some(pair[0].pose.interpolate(pair[1].pose,t));
            }
        }
        self.keys.last().map(|k|k.pose)
    }
}
#[derive(Clone,Debug,Default)]
pub struct Clip {pub name:String,pub length:f32,pub looping:bool,pub tracks:Vec<Track>}
impl Clip {
    pub fn valid(&self)->bool{
        self.name.len()<=128&&!self.name.is_empty()&&self.length.is_finite()&&
        self.length>0.0&&self.tracks.len()<=128&&self.tracks.iter().all(Track::valid)&&
        self.tracks.iter().all(|t|t.keys.last().is_some_and(|k|k.at<=self.length))&&{
            let mut keys=std::collections::BTreeSet::new();
            self.tracks.iter().all(|track|keys.insert(track.bone.as_str()))
        }
    }
    pub fn sample(&self,time:f32)->Option<BTreeMap<String,Pose>> {
        if !self.valid()||!time.is_finite(){return None;}
        let now=if self.looping{time.max(0.0)%self.length}else{time.clamp(0.0,self.length)};
        let mut out=BTreeMap::new();
        for track in &self.tracks {out.insert(track.bone.clone(),track.sample(now)?);}
        Some(out)
    }
}
/// Blend only corresponding bones, preserving base bones when the overlay lacks them.
pub fn blend(base:&BTreeMap<String,Pose>,overlay:&BTreeMap<String,Pose>,weight:f32)
    ->BTreeMap<String,Pose>{
    let mut out=base.clone();
    let w=if weight.is_finite(){weight.clamp(0.0,1.0)}else{0.0};
    for (name,pose) in overlay{
        let old=out.get(name).copied().unwrap_or_else(Pose::identity);
        out.insert(name.clone(),old.interpolate(*pose,w));
    }
    out
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn clips_sample_loop_blend_and_reject_invalid(){
        let pose=|x|Pose{pivot:Vec3{x,y:0.0,z:0.0},..Pose::identity()};
        let clip=Clip{name:"swing".into(),length:10.0,looping:true,
            tracks:vec![Track{bone:"arm".into(),keys:vec![Key{at:0.0,pose:pose(0.0)},Key{at:10.0,pose:pose(10.0)}]}]};
        assert_eq!(clip.sample(12.0).unwrap()["arm"].pivot.x,2.0);
        let half=blend(&clip.sample(0.0).unwrap(),&clip.sample(10.0).unwrap(),0.5);
        // Looping 10.0 returns first key, so blend should remain at zero.
        assert_eq!(half["arm"].pivot.x,0.0);
        let mut bad=clip.clone();bad.tracks[0].keys[1].at=-1.0;assert!(!bad.valid());
    }
}
