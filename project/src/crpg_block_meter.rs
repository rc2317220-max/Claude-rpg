//! Real block-breaking progress from Starling block_break_progress_update.
//! Sent only to the player breaking the block, never extrapolated from tool speed.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct BlockProgressNotice {
    pub dimension:String,
    pub pos:[i32;3],
    pub stage:i32,
}
impl BlockProgressNotice {
    pub fn new(dimension:&str,pos:[i32;3],stage:i32)->Option<Self>{
        if dimension.len()<3 || dimension.len()>96 || dimension.matches(':').count()!=1 ||
           !dimension.bytes().all(|c|c.is_ascii_lowercase() || c.is_ascii_digit() ||
               matches!(c,b'_'|b'.'|b'-'|b'/'|b':')) ||
           !(-1..=10).contains(&stage) ||
           pos.iter().any(|n|(*n as i64).abs()>30_000_000) {return None;}
        Some(Self{dimension:dimension.to_string(),pos,stage})
    }
    pub fn encode(&self)->Vec<u8>{
        format!("1|{}|{}|{}|{}|{}",self.dimension,self.pos[0],self.pos[1],self.pos[2],self.stage).into_bytes()
    }
    pub fn decode(raw:&[u8])->Option<Self>{
        if raw.len()<12 || raw.len()>140 || !raw.is_ascii(){return None;}
        let raw=std::str::from_utf8(raw).ok()?;
        let mut parts=raw.split('|');
        if parts.next()? != "1" {return None;}
        let dimension=parts.next()?;
        fn num(s:&str)->Option<i32>{
            if s.len()>12 || s.is_empty() || !s.bytes().enumerate().all(|(i,c)|
                c.is_ascii_digit() || (i==0 && c==b'-')) {return None;}
            s.parse::<i32>().ok()
        }
        let p=[num(parts.next()?)?,num(parts.next()?)?,num(parts.next()?)?];
        let stage=num(parts.next()?)?;
        if parts.next().is_some(){return None;}
        Self::new(dimension,p,stage)
    }
}
use std::time::{Duration,Instant};

/// Only the server updates `stage`. GUI frames interpolate between confirmed
/// stages, so a ten-percent server step is presented in smooth one-percent
/// increments over the observed stage interval instead of jumping at once.
#[derive(Clone,Debug)]
pub struct MiningHud {
    pub pos:[i32;3],
    pub stage:i32,
    pub seen:u32,
    from:f32,
    to:f32,
    start:Instant,
    duration:Duration,
    last_notice:Instant,
}
impl MiningHud {
    pub fn new(pos:[i32;3],stage:i32,seen:u32,now:Instant)->Self {
        let to=((stage+1).clamp(0,10) as f32)/10.0;
        Self {pos,stage,seen,from:(to-0.10).max(0.0),to,
            start:now,duration:Duration::from_millis(170),last_notice:now}
    }
    pub fn progress(&self)->f32 {self.to}
    pub fn update(&mut self,stage:i32,seen:u32,now:Instant) {
        if stage==self.stage {self.seen=seen;return;}
        if stage<self.stage {self.from=0.0;} else {self.from=self.visual_progress(now);}
        let ms=now.saturating_duration_since(self.last_notice).as_millis();
        self.duration=Duration::from_millis((ms as u64).clamp(70,360));
        self.stage=stage;
        self.to=((stage+1).clamp(0,10) as f32)/10.0;
        self.seen=seen; self.start=now;self.last_notice=now;
    }
    pub fn visual_progress(&self,now:Instant)->f32 {
        let elapsed=now.saturating_duration_since(self.start).as_secs_f32();
        let span=self.duration.as_secs_f32().max(0.001);
        (self.from+(self.to-self.from)*(elapsed/span).clamp(0.0,1.0)).clamp(0.0,1.0)
    }
    pub fn remaining(&self,now:Instant)->f32 {1.0-self.visual_progress(now)}
    pub fn visible(&self,now:u32,pos:[i32;3])->bool {
        self.pos==pos && (0..=9).contains(&self.stage) && now.wrapping_sub(self.seen)<=25
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn round_trip_and_limits(){
        let v=BlockProgressNotice::new("minecraft:overworld",[12,64,-21],5).unwrap();
        assert_eq!(BlockProgressNotice::decode(&v.encode()),Some(v));
        assert!(BlockProgressNotice::new("bad",[0,0,0],2).is_none());
        assert!(BlockProgressNotice::decode(b"1|minecraft:overworld|0|1|2|abc").is_none());
    }
    #[test]fn interpolates_without_fabricating_stages(){
        let start=Instant::now();
        let mut v=MiningHud::new([12,64,-21],0,30,start);
        assert!((v.visual_progress(start)-0.0).abs()<0.001);
        assert!((v.visual_progress(start+Duration::from_millis(170))-0.1).abs()<0.001);
        v.update(1,34,start+Duration::from_millis(170));
        let mid=v.visual_progress(start+Duration::from_millis(255));
        assert!(mid>0.1 && mid<0.2);
        assert!((v.remaining(start+Duration::from_millis(340))-0.8).abs()<0.001);
        assert!(v.visible(34,[12,64,-21]));
        assert!(!v.visible(51,[12,64,-21]));
    }
}
