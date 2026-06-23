use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use zbus::interface;

#[derive(Debug)]
enum NotificationMsg {
    New {
        summary: String,
        body: String,
        icon: String,
        timeout: i32,
    },
    Close,
}

struct NotificationService {
    sender: glib::Sender<NotificationMsg>,
    current_id: std::sync::atomic::AtomicU32,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationService {
    async fn notify(
        &self,
        _app_name: &str,
        _replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        _actions: Vec<&str>,
        _hints: HashMap<&str, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let id = self.current_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("Received Notification via D-Bus: [{}] {}", summary, body);
        
        let _ = self.sender.send(NotificationMsg::New {
            summary: summary.to_string(),
            body: body.to_string(),
            icon: app_icon.to_string(),
            timeout: expire_timeout,
        });
        
        id
    }

    async fn close_notification(&self, _id: u32) {
        let _ = self.sender.send(NotificationMsg::Close);
    }

    async fn get_capabilities(&self) -> Vec<String> {
        vec!["body".to_string(), "icon-static".to_string()]
    }

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "archvnde-notification".to_string(),
            "ArchVNDE".to_string(),
            "0.1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

fn main() {
    println!("Starting ArchVNDE Notification Daemon...");

    // Create a thread-safe GLib channel to send messages from D-Bus thread to GTK thread
    let (tx, rx) = glib::MainContext::channel::<NotificationMsg>(glib::Priority::default());

    // Spawn D-Bus handler thread using Tokio
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = NotificationService {
                sender: tx_clone,
                current_id: std::sync::atomic::AtomicU32::new(1),
            };
            println!("Requesting org.freedesktop.Notifications DBus name...");
            match zbus::connection::Builder::session()
                .unwrap()
                .name("org.freedesktop.Notifications")
                .unwrap()
                .serve_at("/org/freedesktop/Notifications", service)
                .unwrap()
                .build()
                .await
            {
                Ok(_conn) => {
                    println!("Notification D-Bus daemon started successfully. Listening for incoming notifications.");
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to start DBus daemon: {}", e);
                }
            }
        });
    });

    let application = gtk4::Application::new(
        Some("org.archvnde.notification"),
        Default::default(),
    );

    application.connect_activate(move |app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);
        window.init_layer_shell();

        // Run in Overlay layer, no focus
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 15);
        window.set_margin(Edge::Right, 15);
        window.set_default_size(320, 80);

        window.add_css_class("notification-card");

        let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        box_layout.set_margin_all(12);

        // Icon display
        let icon_widget = gtk4::Image::from_icon_name("dialog-information");
        icon_widget.set_pixel_size(36);
        box_layout.append(&icon_widget);

        // Text layout box
        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        
        let title_label = gtk4::Label::new(Some("System Notification"));
        title_label.set_xalign(0.0);
        title_label.add_css_class("notification-title");

        let body_label = gtk4::Label::new(Some("Welcome to ArchVNDE."));
        body_label.set_xalign(0.0);
        body_label.add_css_class("notification-body");

        text_box.append(&title_label);
        text_box.append(&body_label);
        box_layout.append(&text_box);

        window.set_child(Some(&box_layout));

        // State variables to manage active auto-dismiss timer source
        let active_timer_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

        // Connect the message receiver from DBus
        let win_clone = window.clone();
        let title_clone = title_label.clone();
        let body_clone = body_label.clone();
        let icon_clone = icon_widget.clone();
        let timer_clone = active_timer_source.clone();

        rx.attach(None, move |msg| {
            match msg {
                NotificationMsg::New { summary, body, icon, timeout } => {
                    // Update content
                    title_clone.set_text(&summary);
                    body_clone.set_text(&body);

                    // Update icon if possible
                    if !icon.is_empty() {
                        if icon.starts_with('/') {
                            icon_clone.set_from_file(Some(&icon));
                        } else {
                            icon_clone.set_from_icon_name(Some(&icon));
                        }
                    } else {
                        icon_clone.set_from_icon_name(Some("dialog-information"));
                    }

                    // Show window
                    win_clone.present();

                    // Cancel existing timer if any to debounce
                    if let Some(src_id) = timer_clone.borrow_mut().take() {
                        src_id.remove();
                    }

                    // Calculate timeout duration (default to 5000ms if not specified or negative)
                    let duration_ms = if timeout > 0 { timeout as u64 } else { 5000 };

                    // Start a new hide timer
                    let win_hide = win_clone.clone();
                    let timer_inner = timer_clone.clone();
                    let new_src_id = glib::timeout_add_local(
                        std::time::Duration::from_millis(duration_ms),
                        move || {
                            win_hide.hide();
                            *timer_inner.borrow_mut() = None;
                            glib::ControlFlow::Break
                        }
                    );
                    *timer_clone.borrow_mut() = Some(new_src_id);
                }
                NotificationMsg::Close => {
                    win_clone.hide();
                    if let Some(src_id) = timer_clone.borrow_mut().take() {
                        src_id.remove();
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        // Hide window initially
        window.hide();
    });

    application.run();
}
