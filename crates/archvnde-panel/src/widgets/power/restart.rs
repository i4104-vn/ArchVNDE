use std::process::Command;

/// Triggers a system reboot.
pub fn trigger_restart() {
    println!("Reboot requested...");
    let _ = Command::new("systemctl").arg("reboot").spawn();
}
