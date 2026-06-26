use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::collections::HashMap;
use std::thread;
use zbus::interface;

#[derive(Clone, Debug)]
pub struct ActiveNotification {
    pub title: String,
    pub body: String,
    pub icon: String,
    pub timestamp: std::time::Instant,
}

#[derive(Debug)]
pub enum NotificationMsg {
    New {
        summary: String,
        body: String,
        icon: String,
        timeout: i32,
    },
    Close,
}

thread_local! {
    pub static SHARED_NOTIFICATION: RefCell<Option<ActiveNotification>> = RefCell::new(None);
    pub static HISTORICAL_NOTIFICATIONS: RefCell<Vec<ActiveNotification>> = RefCell::new(Vec::new());
    static ACTIVE_POPUP: RefCell<Option<gtk4::Window>> = RefCell::new(None);
    static ACTIVE_TIMER: RefCell<Option<glib::SourceId>> = RefCell::new(None);
}

pub struct NotificationService {
    sender: tokio::sync::mpsc::UnboundedSender<NotificationMsg>,
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

/// Spawns a background thread running Tokio to serve the org.freedesktop.Notifications DBus daemon.
pub fn spawn_dbus_listener(tx: tokio::sync::mpsc::UnboundedSender<NotificationMsg>) {
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let service = NotificationService {
                sender: tx,
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
}

pub fn close_notification_popup() {
    ACTIVE_TIMER.with(|t| {
        if let Some(src_id) = t.borrow_mut().take() {
            src_id.remove();
        }
    });
    ACTIVE_POPUP.with(|p| {
        if let Some(win) = p.borrow_mut().take() {
            win.close();
        }
    });
}

fn close_and_fade(window: &gtk4::Window, container_box: &gtk4::Box) {
    ACTIVE_TIMER.with(|t| {
        if let Some(src_id) = t.borrow_mut().take() {
            src_id.remove();
        }
    });

    let win = window.clone();
    archvnde_common::animation::css_genie_out(
        container_box.upcast_ref(),
        400,
        move || {
            win.close();
        }
    );
}

pub fn show_notification_popup(summary: &str, body: &str, icon_name: &str, timeout_ms: i32) {
    close_notification_popup();

    // Save to historical notifications list
    let notif = ActiveNotification {
        title: summary.to_string(),
        body: body.to_string(),
        icon: icon_name.to_string(),
        timestamp: std::time::Instant::now(),
    };
    HISTORICAL_NOTIFICATIONS.with(|list| {
        let mut list_borrow = list.borrow_mut();
        list_borrow.push(notif);
        if list_borrow.len() > 50 {
            list_borrow.remove(0); // Cap at 50 notifications
        }
    });

    let window = gtk4::Window::new();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);

    // Centered top position
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_margin(Edge::Top, 48);

    window.add_css_class("notification-popup-card");

    let container_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    container_box.add_css_class("notification-popup-box");
    container_box.set_size_request(380, 76);
    container_box.set_overflow(gtk4::Overflow::Hidden);

    // content row
    let content_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    content_row.set_valign(gtk4::Align::Center);

    let app_icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    app_icon_box.add_css_class("popup-app-icon-box");
    let icon_symbol = if icon_name.is_empty() { "message" } else { icon_name };
    let app_icon = archvnde_common::icon::get_icon_colored(icon_symbol, 24, "#ffffff");
    app_icon_box.append(&app_icon);

    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let title_label = gtk4::Label::new(Some(summary));
    title_label.add_css_class("popup-title");
    title_label.set_halign(gtk4::Align::Start);

    let time_label = gtk4::Label::new(Some("Vừa xong"));
    time_label.add_css_class("popup-time");
    time_label.set_halign(gtk4::Align::End);
    time_label.set_hexpand(true);

    header_box.append(&title_label);
    header_box.append(&time_label);

    let body_label = gtk4::Label::new(Some(body));
    body_label.add_css_class("popup-body");
    body_label.set_halign(gtk4::Align::Start);
    body_label.set_wrap(true);
    body_label.set_max_width_chars(32);

    text_box.append(&header_box);
    text_box.append(&body_label);

    let avatar_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    avatar_box.add_css_class("popup-avatar-box");
    let avatar_icon = archvnde_common::icon::get_icon_colored("user", 16, "#ffffff");
    avatar_box.append(&avatar_icon);

    content_row.append(&app_icon_box);
    content_row.append(&text_box);
    content_row.append(&avatar_box);

    container_box.append(&content_row);
    window.set_child(Some(&container_box));

    // Click gesture to dismiss the notification on click
    let click_gesture = gtk4::GestureClick::new();
    let win_c = window.clone();
    let box_c = container_box.clone();
    click_gesture.connect_released(move |_, _, _, _| {
        close_and_fade(&win_c, &box_c);
    });
    container_box.add_controller(click_gesture);

    window.present();

    ACTIVE_POPUP.with(|p| *p.borrow_mut() = Some(window.clone()));

    archvnde_common::animation::css_genie_in(
        container_box.upcast_ref(),
    );

    let win_timer = window.clone();
    let container_timer = container_box.clone();
    let duration = if timeout_ms > 0 { timeout_ms as u64 } else { 5000 };
    let timer_id = glib::timeout_add_local(
        std::time::Duration::from_millis(duration),
        move || {
            close_and_fade(&win_timer, &container_timer);
            glib::ControlFlow::Break
        }
    );
    ACTIVE_TIMER.with(|t| *t.borrow_mut() = Some(timer_id));
}
