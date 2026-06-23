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

/// Spawns a background thread running Tokio to serve the org.freedesktop.Notifications DBus daemon.
pub fn spawn_dbus_listener(tx: glib::Sender<NotificationMsg>) {
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
