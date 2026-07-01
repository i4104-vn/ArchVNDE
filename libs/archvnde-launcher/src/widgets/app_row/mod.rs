use crate::models::DesktopApp;
use gtk4::prelude::*;
use std::process::Command;

mod render;

pub fn create_grid_app_widget(
    app: &DesktopApp,
    window: &gtk4::ApplicationWindow,
) -> gtk4::Button {
    let (btn, _, _) = render::build_grid_app_ui(app);

    // Click behavior (Launch)
    let exec_cmd = app.exec.clone();
    let win_to_close = window.clone();
    btn.connect_clicked(move |_| {
        println!("Launching from Grid: {}", exec_cmd);
        let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let program = parts[0];
            let args = &parts[1..];
            if let Err(e) = Command::new(program).args(args).spawn() {
                eprintln!("Failed to spawn command {}: {}", exec_cmd, e);
            }
        }

        win_to_close.close();
    });

    btn
}

pub fn create_list_app_widget(
    app: &DesktopApp,
    window: &gtk4::ApplicationWindow,
) -> gtk4::Button {
    let (btn, _, _) = render::build_list_app_ui(app);

    let exec_cmd = app.exec.clone();
    let win_to_close = window.clone();
    btn.connect_clicked(move |_| {
        println!("Launching from List: {}", exec_cmd);
        let parts: Vec<&str> = exec_cmd.split_whitespace().collect();
        if !parts.is_empty() {
            let program = parts[0];
            let args = &parts[1..];
            if let Err(e) = Command::new(program).args(args).spawn() {
                eprintln!("Failed to spawn command {}: {}", exec_cmd, e);
            }
        }

        win_to_close.close();
    });

    btn
}
