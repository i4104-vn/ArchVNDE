use gtk4::prelude::*;
use std::os::unix::net::UnixStream;
use std::io::Write;

mod history;
mod apps;
mod widgets;
mod render;

use apps::get_running_apps;

fn handle_single_instance() -> bool {
    let socket_path = "/tmp/archvnde-switcher.socket";
    
    if let Ok(mut stream) = UnixStream::connect(socket_path) {
        let _ = stream.write_all(b"next");
        return false;
    }
    
    let _ = std::fs::remove_file(socket_path);
    true
}

fn main() {
    if !handle_single_instance() {
        return;
    }

    // Check if there are running apps. If not, exit immediately.
    let apps = get_running_apps();
    if apps.is_empty() {
        return;
    }

    println!("Starting ArchVNDE Alt-Tab Switcher...");

    let application = gtk4::Application::new(
        Some("org.archvnde.switcher"),
        Default::default(),
    );

    let apps_clone = apps.clone();
    application.connect_activate(move |app| {
        let apps = apps_clone.clone();
        render::build_switcher_ui(app, apps);
    });

    application.run();

    // Clean up Unix socket file on exit
    std::fs::remove_file("/tmp/archvnde-switcher.socket").ok();
}
