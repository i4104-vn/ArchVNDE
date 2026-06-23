use std::process::Command;

/// Triggers a system poweroff.
pub fn trigger_shutdown() {
    println!("Power Off requested...");
    let _ = Command::new("systemctl").arg("poweroff").spawn();
}
