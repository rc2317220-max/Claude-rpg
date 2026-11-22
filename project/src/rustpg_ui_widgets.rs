// Small deterministic widgets. The Horizon adapter turns these into GuiDraw.

use super::rustpg_ui_layout::Rect;
#[derive(Clone, Debug, PartialEq)]
pub enum Widget {
    Panel(Rect, u32),
    Bar {
        rect: Rect,
        ratio: f32,
        fill: u32,
    },
    Label {
        text: String,
        x: f32,
        y: f32,
        color: u32,
    },
    Button {
        rect: Rect,
        label: String,
        selected: bool,
    },
}

pub fn panel(rect: Rect, color: u32) -> Widget {
    Widget::Panel(rect, color)
}
pub fn bar(rect: Rect, ratio: f32, fill: u32) -> Widget {
    Widget::Bar {
        rect,
        ratio: ratio.clamp(0.0, 1.0),
        fill,
    }
}
pub fn label(text: impl Into<String>, x: f32, y: f32, color: u32) -> Widget {
    Widget::Label {
        text: text.into(),
        x,
        y,
        color,
    }
}
pub fn button(rect: Rect, label: impl Into<String>, selected: bool) -> Widget {
    Widget::Button {
        rect,
        label: label.into(),
        selected,
    }
}
