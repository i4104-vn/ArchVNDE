use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Write, Read};

mod history;
mod apps;
mod widgets;

use apps::{get_running_apps, activate_app};
use history::save_history;
use widgets::list::build_apps_list;

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
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        // Center vertically, stretch horizontally across the screen
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.add_css_class("switcher-window");

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.add_css_class("switcher-box");
        main_box.set_valign(gtk4::Align::Center);
        main_box.set_halign(gtk4::Align::Fill);

        let (icons_row, item_buttons) = build_apps_list(&apps);
        main_box.append(&icons_row);
        window.set_child(Some(&main_box));

        let current_index = Rc::new(RefCell::new(0));

        let update_selection = {
            let current_index = current_index.clone();
            let item_buttons = item_buttons.clone();

            move |new_idx: usize| {
                let mut idx = new_idx;
                if idx >= item_buttons.len() {
                    idx = 0;
                }
                *current_index.borrow_mut() = idx;

                for (i, btn) in item_buttons.iter().enumerate() {
                    if i == idx {
                        btn.add_css_class("selected");
                    } else {
                        btn.remove_css_class("selected");
                    }
                }
            }
        };

        let update_selection_rc = Rc::new(update_selection);
        let initial_idx = if apps.len() > 1 { 1 } else { 0 };
        update_selection_rc(initial_idx);

        for (i, btn) in item_buttons.iter().enumerate() {
            let update_sel = update_selection_rc.clone();
            let window_close = window.clone();
            let apps_click = apps.clone();
            btn.connect_clicked(move |_| {
                update_sel(i);
                let app_item = &apps_click[i];
                save_history(&app_item.name);
                activate_app(app_item);
                window_close.close();
            });
        }

        // Unix Socket Listener to handle subsequent Alt-Tab signals
        let (sender, receiver) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            let socket_path = "/tmp/archvnde-switcher.socket";
            if let Ok(listener) = UnixListener::bind(socket_path) {
                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        let mut buf = [0; 4];
                        if let Ok(_) = stream.read(&mut buf) {
                            if &buf[0..4] == b"next" {
                                let _ = sender.send(());
                            }
                        }
                    }
                }
            }
        });

        let update_sel_socket = update_selection_rc.clone();
        let current_idx_socket = current_index.clone();
        let apps_len = apps.len();
        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            while let Ok(_) = receiver.try_recv() {
                let idx = *current_idx_socket.borrow();
                let next = (idx + 1) % apps_len;
                update_sel_socket(next);
            }
            gtk4::glib::ControlFlow::Continue
        });

        // Keyboard navigation
        let key_controller = gtk4::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let current_idx_key = current_index.clone();
        let update_sel_key = update_selection_rc.clone();
        let window_close = window.clone();
        let apps_key = apps.clone();
        
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let idx = *current_idx_key.borrow();
            match key {
                gtk4::gdk::Key::Tab | gtk4::gdk::Key::Right => {
                    let next = (idx + 1) % apps_key.len();
                    update_sel_key(next);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::ISO_Left_Tab | gtk4::gdk::Key::Left => {
                    let prev = if idx == 0 { apps_key.len() - 1 } else { idx - 1 };
                    update_sel_key(prev);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Return | gtk4::gdk::Key::space => {
                    let app_item = &apps_key[idx];
                    println!("Selected App: {}", app_item.name);
                    save_history(&app_item.name);
                    activate_app(app_item);
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Escape => {
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });

        // Activate and close on Alt release
        let current_idx_release = current_index.clone();
        let apps_release = apps.clone();
        let window_release = window.clone();
        key_controller.connect_key_released(move |_, key, _, _| {
            match key {
                gtk4::gdk::Key::Alt_L | gtk4::gdk::Key::Alt_R => {
                    let idx = *current_idx_release.borrow();
                    if idx < apps_release.len() {
                        let app_item = &apps_release[idx];
                        println!("Alt released. Activating: {}", app_item.name);
                        save_history(&app_item.name);
                        activate_app(app_item);
                    }
                    window_release.close();
                }
                _ => {}
            }
        });

        window.add_controller(key_controller);
        window.present();

        if !item_buttons.is_empty() {
            item_buttons[0].grab_focus();
        }
    });

    application.run();

    // Clean up Unix socket file on exit
    std::fs::remove_file("/tmp/archvnde-switcher.socket").ok();
}
