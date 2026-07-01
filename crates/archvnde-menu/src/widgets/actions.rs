use gtk4::prelude::*;
use std::process::Command;

pub fn execute_terminal() {
    let _ = Command::new("foot").spawn().or_else(|_| Command::new("alacritty").spawn());
}

pub fn execute_file_manager() {
    let _ = Command::new("pcmanfm").spawn().or_else(|_| Command::new("thunar").spawn());
}

pub fn execute_change_wallpaper(window: &gtk4::ApplicationWindow, menu_box: &gtk4::Box) {
    let dialog = gtk4::FileDialog::new();
    dialog.set_title("Select Wallpaper Image");
    
    let filter = gtk4::FileFilter::new();
    filter.set_name(Some("Images"));
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    dialog.set_default_filter(Some(&filter));

    let win = window.clone();
    let mb = menu_box.clone();
    dialog.open(Some(&win), None::<&gio::Cancellable>, move |res| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                println!("Setting wallpaper to: {:?}", path);
                if let Err(e) = archvnde_wallpaper::set_wallpaper(&path) {
                    eprintln!("Error setting wallpaper: {}", e);
                }
            }
        }
    });
}

pub fn execute_reconfigure_shell() {
    let _ = Command::new("labwc").arg("--reconfigure").spawn();
}

pub fn execute_exit_shell() {
    let _ = Command::new("labwc").arg("--exit").spawn();
}
