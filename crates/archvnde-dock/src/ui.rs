use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::process::Command;

fn create_dock_button(icon_name: &str, tooltip: &str, command: &str, args: &[&str]) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("dock-item-btn");
    btn.set_tooltip_text(Some(tooltip));

    // macOS style dock has larger icons (36px is clean and looks premium)
    let icon = archvnde_common::icon::get_icon_colored(icon_name, 36, "#ffffff");
    btn.set_child(Some(&icon));

    let cmd_str = command.to_string();
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    btn.connect_clicked(move |_| {
        let mut cmd = Command::new(&cmd_str);
        for arg in &args_vec {
            cmd.arg(arg);
        }
        let _ = cmd.spawn();
    });

    btn
}

pub fn build_dock_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();

    // Assign to Top layer so it floats above normal windows
    window.set_layer(Layer::Top);

    // Set an exclusive zone of 64px so other windows respect the dock space at the bottom
    window.set_exclusive_zone(64);

    // Anchor ONLY to the bottom so it dynamically centers horizontally!
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Top, false);

    // Add margin to make it float above the bottom screen edge elegantly (macOS style)
    window.set_margin(Edge::Bottom, 10);

    window.add_css_class("dock-window");

    let dock_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    dock_box.add_css_class("dock-box");

    // 1. App Launcher (logo icon)
    let launcher_btn = create_dock_button("logo", "Application Launcher", "archvnde-launcher", &[]);
    dock_box.append(&launcher_btn);

    // Separator
    let sep1 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep1.add_css_class("dock-separator");
    dock_box.append(&sep1);

    // 2. Terminal
    let term_btn = create_dock_button("terminal", "Terminal", "foot", &[]);
    // Fallback: if foot fails, it can try alacritty. For now, simple command is good
    dock_box.append(&term_btn);

    // 3. File Manager
    let files_btn = create_dock_button("folder", "Files", "pcmanfm", &[]);
    dock_box.append(&files_btn);

    // 4. Web Browser
    let browser_btn = create_dock_button("search", "Web Browser", "firefox", &[]);
    dock_box.append(&browser_btn);

    // 5. Music
    let music_btn = create_dock_button("music", "Music Player", "amberol", &[]);
    dock_box.append(&music_btn);

    // 6. Settings
    let settings_btn = create_dock_button("settings", "System Settings", "gnome-control-center", &[]);
    dock_box.append(&settings_btn);

    // Separator
    let sep2 = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sep2.add_css_class("dock-separator");
    dock_box.append(&sep2);

    // 7. Trash
    let trash_btn = create_dock_button("trash", "Trash Bin", "pcmanfm", &["trash:///"]);
    dock_box.append(&trash_btn);

    window.set_child(Some(&dock_box));

    window
}
