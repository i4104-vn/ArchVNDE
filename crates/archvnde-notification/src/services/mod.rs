use std::collections::HashMap;
use std::thread;
use zbus::interface;

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
        println!("Received Notification via D-Bus from {}: [{}] {}", app_name, summary, body);
        
<<<<<<< HEAD:crates/archvnde-notification/src/services/mod.rs
        let _ = self.sender.send(NotificationMsg::New {
            summary: summary.to_string(),
            body: body.to_string(),
            icon: app_icon.to_string(),
=======
        let mut icon = app_icon.to_string();
        if icon.is_empty() {
            // Try searching desktop apps cache for a matching app
            let lower_name = app_name.to_lowercase();
            let apps = archvnde_common::desktop::find_desktop_apps();
            for app in apps {
                if app.name.to_lowercase() == lower_name {
                    if let Some(app_icon) = app.icon {
                        icon = app_icon;
                    }
                    break;
                }
            }
        }
        if icon.is_empty() {
            icon = app_name.to_lowercase();
        }
        
        let _ = self.sender.send(NotificationMsg::New {
            summary: summary.to_string(),
            body: body.to_string(),
            icon,
            app_name: app_name.to_string(),
>>>>>>> c5c198c (perf: optimize desktop caching, tray polling, alt-tab switching and notification behaviors):libs/archvnde-island/src/widgets/notification.rs
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
<<<<<<< HEAD:crates/archvnde-notification/src/services/mod.rs
=======

pub fn close_notification_popup() {
    // No-op: Notifications are managed directly inside the Dynamic Island notch
}

pub fn show_notification_popup(summary: &str, body: &str, icon_name: &str, app_name: &str, _timeout_ms: i32) {
    // Save to historical notifications list
    let notif = ActiveNotification {
        title: summary.to_string(),
        body: body.to_string(),
        icon: icon_name.to_string(),
        app_name: app_name.to_string(),
        timestamp: std::time::Instant::now(),
    };

    SHARED_NOTIFICATION.with(|sn| {
        *sn.borrow_mut() = Some(notif.clone());
    });

    HISTORICAL_NOTIFICATIONS.with(|list| {
        let mut list_borrow = list.borrow_mut();
        list_borrow.push(notif);
        if list_borrow.len() > 50 {
            list_borrow.remove(0); // Cap at 50 notifications
        }
    });
}
>>>>>>> c5c198c (perf: optimize desktop caching, tray polling, alt-tab switching and notification behaviors):libs/archvnde-island/src/widgets/notification.rs
