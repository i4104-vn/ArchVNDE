use gtk4::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, Edge, LayerShell};

fn has_battery() -> bool {
    let power_dir = std::path::Path::new("/sys/class/power_supply");
    if !power_dir.exists() { return false; }
    if let Ok(entries) = std::fs::read_dir(power_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(kind) = std::fs::read_to_string(path.join("type")) {
                if kind.trim() == "Battery" {
                    return true;
                }
            }
        }
    }
    false
}

pub fn build_status_indicators_ui() -> (gtk4::Box, gtk4::Button, gtk4::Label) {
    let status_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    status_box.add_css_class("status-indicators-box");

    let status_button = gtk4::Button::new();
    status_button.add_css_class("panel-status-btn");

    let inner_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let net_icon = archvnde_common::icon::get_icon("wifi", 14);
    net_icon.add_css_class("status-icon");
    
    let vol_icon = archvnde_common::icon::get_icon("volume", 14);
    vol_icon.add_css_class("status-icon");
    
    inner_layout.append(&net_icon);
    inner_layout.append(&vol_icon);

    if has_battery() {
        let bat_icon = archvnde_common::icon::get_icon("battery", 14);
        bat_icon.add_css_class("status-icon");
        inner_layout.append(&bat_icon);
    }

    status_button.set_child(Some(&inner_layout));

    let separator = gtk4::Label::new(Some("│"));
    separator.add_css_class("capsule-separator");

    // We will build the clock widget button externally and pass/append it, but let's return a second button reference if needed
    // Actually, let's return a dummy/placeholder clock button or let mod.rs create it and append it.
    // Wait, let's return status_box, status_button, separator.
    (status_box, status_button, separator)
}

pub fn build_control_center_window_ui(
    app: &gtk4::Application,
) -> (gtk4::ApplicationWindow, gtk4::Box) {
    let q_win = gtk4::ApplicationWindow::new(app);
    archvnde_common::apply_theme_class(&q_win);
    q_win.init_layer_shell();
    q_win.set_layer(Layer::Overlay);
    q_win.set_keyboard_mode(KeyboardMode::OnDemand);

    // Anchor to all 4 edges to cover the entire screen transparently
    q_win.set_anchor(Edge::Top, true);
    q_win.set_anchor(Edge::Bottom, true);
    q_win.set_anchor(Edge::Left, true);
    q_win.set_anchor(Edge::Right, true);
    q_win.add_css_class("control-center-window");

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 14);
    main_box.add_css_class("control-center-box");
    main_box.set_halign(gtk4::Align::End);
    main_box.set_valign(gtk4::Align::Start);
    main_box.set_size_request(360, 480);
    main_box.set_margin_top(6);
    main_box.set_margin_end(12);

    q_win.set_child(Some(&main_box));

    (q_win, main_box)
}
