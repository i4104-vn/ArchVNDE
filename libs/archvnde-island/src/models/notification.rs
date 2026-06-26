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
