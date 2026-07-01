use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

/// Standard window layer shell initialization helper to configure widgets as desktop shell layers.
pub fn init_layer_window(
    window: &gtk4::ApplicationWindow,
    layer: Layer,
    kbd_mode: KeyboardMode,
    exclusive_zone: i32,
    anchors: &[(Edge, bool)],
    margin_bottom: i32,
) {
    window.init_layer_shell();
    window.set_layer(layer);
    window.set_keyboard_mode(kbd_mode);
    window.set_exclusive_zone(exclusive_zone);
    for &(edge, anchor) in anchors {
        window.set_anchor(edge, anchor);
    }
    if margin_bottom > 0 {
        window.set_margin(Edge::Bottom, margin_bottom);
    }
}
