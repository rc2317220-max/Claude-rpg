// Touch and pointer actions normalized for the Presentation Engine.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiAction {
    Tap([f32; 2]),
    Drag { from: [f32; 2], to: [f32; 2] },
    Scroll(f32),
    Pinch(f32),
    Back,
}
pub fn drag_delta(from: [f32; 2], to: [f32; 2]) -> [f32; 2] {
    [to[0] - from[0], to[1] - from[1]]
}
