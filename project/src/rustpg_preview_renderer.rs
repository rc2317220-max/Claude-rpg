// Renderer contract for the Bestiary viewport.
// A real implementation must submit WGPU geometry to a GUI-owned target. This module
// keeps the state and capability boundary explicit until that integration exists.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewCapability {
    WorldEntityLayer,
    GuiViewport,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewRequest {
    pub kind: String,
    pub rotation: f32,
    pub zoom: f32,
    pub idle_tick: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewStatus {
    pub capability: PreviewCapability,
    pub rendered: bool,
    pub reason: String,
}
pub fn unavailable_status() -> PreviewStatus {
    PreviewStatus { capability: PreviewCapability::WorldEntityLayer, rendered: false, reason: "A API exportada permite camadas no mundo, mas não fornece viewport GUI independente nem alvo de textura para modelo vanilla.".into() }
}
