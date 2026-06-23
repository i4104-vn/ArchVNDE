mod dbus;
mod window;

use dbus::{spawn_dbus_listener, NotificationMsg};
use gtk4::prelude::*;
use std::rc::Rc;
use window::NotificationWindow;

fn main() {
    println!("Starting ArchVNDE Notification Daemon...");

    // Create a thread-safe GLib channel to send messages from D-Bus thread to GTK thread
    let (tx, rx) = glib::MainContext::channel::<NotificationMsg>(glib::Priority::default());

    // Spawn the D-Bus daemon thread (from dbus module)
    spawn_dbus_listener(tx);

    let application = gtk4::Application::new(
        Some("org.archvnde.notification"),
        Default::default(),
    );

    application.connect_activate(move |app| {
        // Initialize style provider
        archvnde_common::init_theme();

        // Create the notification window wrapper (from window module)
        let notif_window = Rc::new(NotificationWindow::new(app));

        // Connect the message receiver from DBus
        let nw_clone = notif_window.clone();
        rx.attach(None, move |msg| {
            match msg {
                NotificationMsg::New { summary, body, icon, timeout } => {
                    // Update card layout
                    nw_clone.update(&summary, &body, &icon);
                    
                    // Display overlay card
                    nw_clone.show();

                    // Cancel existing timer if any (debounce multiple notifications)
                    if let Some(src_id) = nw_clone.active_timer.borrow_mut().take() {
                        src_id.remove();
                    }

                    // Calculate timeout duration (default to 5000ms if not specified or negative)
                    let duration_ms = if timeout > 0 { timeout as u64 } else { 5000 };

                    // Start a new hide timer
                    let nw_hide = nw_clone.clone();
                    let new_src_id = glib::timeout_add_local(
                        std::time::Duration::from_millis(duration_ms),
                        move || {
                            nw_hide.hide();
                            *nw_hide.active_timer.borrow_mut() = None;
                            glib::ControlFlow::Break
                        }
                    );
                    *nw_clone.active_timer.borrow_mut() = Some(new_src_id);
                }
                NotificationMsg::Close => {
                    nw_clone.hide();
                    if let Some(src_id) = nw_clone.active_timer.borrow_mut().take() {
                        src_id.remove();
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    });

    application.run();
}
