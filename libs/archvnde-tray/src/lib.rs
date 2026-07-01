use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use zbus::interface;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrayItem {
    pub service: String,
    pub icon_name: String,
    pub title: String,
}

static TRAY_ITEMS: OnceLock<Arc<Mutex<Vec<TrayItem>>>> = OnceLock::new();

pub fn get_tray_items() -> Vec<TrayItem> {
    let registry = TRAY_ITEMS.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    registry.lock().unwrap().clone()
}

#[zbus::proxy(
    interface = "org.kde.StatusNotifierItem",
    default_path = "/StatusNotifierItem"
)]
trait StatusNotifierItem {
    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn secondary_activate(&self, x: i32, y: i32) -> zbus::Result<()>;
    fn context_menu(&self, x: i32, y: i32) -> zbus::Result<()>;

    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;
}

pub struct StatusNotifierWatcher;

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcher {
    async fn register_status_notifier_item(
        &self,
        service: String,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) {
        let sender = header.sender().map(|s| s.to_string()).unwrap_or_default();
        let target_service = if service.starts_with('/') {
            // Some buggy apps register by passing object path instead of service name.
            // In that case, fall back to the message sender's unique connection name.
            sender.clone()
        } else {
            service.clone()
        };

        if target_service.is_empty() {
            return;
        }

        println!("RegisterStatusNotifierItem called for service: {}", target_service);

        let connection = match zbus::Connection::session().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to get session connection: {}", e);
                return;
            }
        };

        let bus_name = match zbus::names::BusName::try_from(target_service.clone()) {
            Ok(name) => name,
            Err(e) => {
                eprintln!("Invalid bus name: {}", e);
                return;
            }
        };

        let proxy = match StatusNotifierItemProxy::builder(&connection)
            .destination(bus_name)
            .unwrap()
            .path("/StatusNotifierItem")
            .unwrap()
            .build()
            .await
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to build proxy: {}", e);
                return;
            }
        };

        let icon_name = proxy.icon_name().await.unwrap_or_else(|_| "image-missing".to_string());
        let title = proxy.title().await.unwrap_or_else(|_| String::new());

        let item = TrayItem {
            service: target_service,
            icon_name,
            title,
        };

        let registry = TRAY_ITEMS.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
        let mut lock = registry.lock().unwrap();
        // Remove existing item with same service name to avoid duplicates
        lock.retain(|x| x.service != item.service);
        lock.push(item);
    }

    async fn register_status_notifier_host(&self, service: String) {
        println!("StatusNotifierHost registered: {}", service);
    }

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        get_tray_items().into_iter().map(|x| x.service).collect()
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }
}

/// Spawns the D-Bus StatusNotifierWatcher service in a background tokio thread.
/// Also starts a periodic health check loop to remove disconnected tray icons.
pub fn spawn_watcher_service() {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            println!("Starting D-Bus StatusNotifierWatcher...");
            let watcher = StatusNotifierWatcher;
            
            let conn = match zbus::connection::Builder::session()
                .unwrap()
                .name("org.kde.StatusNotifierWatcher")
                .unwrap()
                .serve_at("/StatusNotifierWatcher", watcher)
                .unwrap()
                .build()
                .await
            {
                Ok(c) => {
                    println!("StatusNotifierWatcher registered on D-Bus successfully.");
                    c
                }
                Err(e) => {
                    eprintln!("Failed to register StatusNotifierWatcher: {}", e);
                    return;
                }
            };

            // Garbage Collection Loop: Check connection owners every 5 seconds.
            // If an app crashed or quit, its unique bus name owner disappears.
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                let registry = TRAY_ITEMS.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
                let current_items = {
                    let lock = registry.lock().unwrap();
                    lock.clone()
                };

                let mut active_items = Vec::new();
                if let Ok(dbus_proxy) = zbus::fdo::DBusProxy::new(&conn).await {
                    for item in current_items {
                        if let Ok(name) = zbus::names::BusName::try_from(item.service.clone()) {
                            match dbus_proxy.name_has_owner(name).await {
                                Ok(true) => active_items.push(item),
                                _ => println!("Pruning disconnected tray item: {}", item.service),
                            }
                        }
                    }
                }

                let mut lock = registry.lock().unwrap();
                *lock = active_items;
            }
        });
    });
}

/// Sends an Activate or ContextMenu signal to the item's D-Bus service, letting the application open its menu or window.
pub fn activate_item(service: &str, x: i32, y: i32, is_right_click: bool) {
    let service_str = service.to_string();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Ok(conn) = zbus::Connection::session().await {
                let bus_name = match zbus::names::BusName::try_from(service_str.clone()) {
                    Ok(name) => name,
                    Err(e) => {
                        eprintln!("Invalid bus name for activation: {}", e);
                        return;
                    }
                };

                let proxy = match StatusNotifierItemProxy::builder(&conn)
                    .destination(bus_name)
                    .unwrap()
                    .path("/StatusNotifierItem")
                    .unwrap()
                    .build()
                    .await
                {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Failed to build proxy for activation: {}", e);
                        return;
                    }
                };

                if is_right_click {
                    println!("Sending D-Bus context_menu({}, {}) to {}", x, y, service_str);
                    if let Err(e) = proxy.context_menu(x, y).await {
                        eprintln!("D-Bus context_menu call failed for {}: {}", service_str, e);
                        // Fallback 1: Some apps use SecondaryActivate for menus
                        println!("Attempting fallback secondary_activate({}, {}) for {}", x, y, service_str);
                        if let Err(e2) = proxy.secondary_activate(x, y).await {
                            eprintln!("D-Bus secondary_activate call failed for {}: {}", service_str, e2);
                            // Fallback 2: Some apps use Activate as a catch-all
                            println!("Attempting fallback activate({}, {}) for {}", x, y, service_str);
                            let _ = proxy.activate(x, y).await;
                        }
                    }
                } else {
                    println!("Sending D-Bus activate({}, {}) to {}", x, y, service_str);
                    if let Err(e) = proxy.activate(x, y).await {
                        eprintln!("D-Bus activate call failed for {}: {}", service_str, e);
                    }
                }
            }
        });
    });
}
