use gtk4::prelude::*;
use std::process::Command;

pub fn create_dock_button(icon_name: &str, tooltip: &str, command: &str, args: &[&str]) -> gtk4::Button {
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
