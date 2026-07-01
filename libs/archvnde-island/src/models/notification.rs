#[derive(Clone, Debug)]
pub struct ActiveNotification {
    pub title: String,
    pub body: String,
    pub icon: String,
    pub app_name: String,
    pub timestamp: std::time::Instant,
}

#[derive(Debug)]
pub enum NotificationMsg {
    New {
        summary: String,
        body: String,
        icon: String,
        app_name: String,
        timeout: i32,
    },
    Close,
}
