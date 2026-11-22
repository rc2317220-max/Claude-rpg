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
