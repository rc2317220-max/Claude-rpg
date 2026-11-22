// Presentation animation clocks, independent from rendering backend.

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationClock {
    pub tick: u32,
}
impl AnimationClock {
    pub fn advance(&mut self) {
        self.tick = self.tick.saturating_add(1);
    }
    pub fn pulse(self, period: u32) -> f32 {
        if period == 0 {
            return 0.0;
        }
        let p = self.tick % period;
        (p as f32 / period as f32 * std::f32::consts::TAU)
            .sin()
            .abs()
    }
    pub fn fade(self, lifetime: u32) -> f32 {
        if lifetime == 0 {
            0.0
        } else {
            (1.0 - self.tick as f32 / lifetime as f32).clamp(0.0, 1.0)
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    pub age: u32,
    pub duration: u32,
}
impl Transition {
    pub fn progress(self) -> f32 {
        if self.duration == 0 {
            1.0
        } else {
            (self.age as f32 / self.duration as f32).clamp(0.0, 1.0)
        }
    }
}


/// Smooth 0..1 transition without overshoot; useful for selection/fill state.
pub fn smoothstep(t:f32)->f32 {
    let t=t.clamp(0.0,1.0);
    t*t*(3.0-2.0*t)
}
/// Fast response with a soft landing. UI movement should communicate state,
/// not make the player wait for decoration.
pub fn ease_out_cubic(t:f32)->f32 {
    let t=t.clamp(0.0,1.0);
    1.0-(1.0-t).powi(3)
}
/// Stable typewriter reveal count. Layout must be reserved before this count
/// is applied; this function only controls character visibility.
pub fn typewriter_chars(tick:u32,chars_per_tick:usize,lead:usize)->usize {
    (tick as usize).saturating_mul(chars_per_tick).saturating_add(lead)
}
/// Subtle state pulse in the 0.72..1 range. Reserved for actionable/selected
/// controls, never for passive decoration.
pub fn action_pulse(tick:u32,period:u32)->f32 {
    let p=AnimationClock{tick}.pulse(period.max(1));
    0.72+0.28*p
}

#[cfg(test)]
mod motion_tests {
    use super::*;
    #[test] fn easing_is_bounded(){
        for x in [-1.0,0.0,0.25,0.5,1.0,2.0] {
            assert!((0.0..=1.0).contains(&smoothstep(x)));
            assert!((0.0..=1.0).contains(&ease_out_cubic(x)));
        }
    }
    #[test] fn typewriter_is_monotonic(){
        assert!(typewriter_chars(10,3,4)>typewriter_chars(9,3,4));
    }
}
