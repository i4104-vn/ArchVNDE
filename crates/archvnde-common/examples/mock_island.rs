use std::thread;
use std::time::Duration;
use archvnde_common::{IslandState, update_island_state, clear_island_state};

fn main() {
    println!("Mocking Dynamic Island Notification...");

    // 1. Trigger notification
    let state = IslandState {
        active: true,
        title: "Download".to_string(),
        subtitle: "Completed".to_string(),
        icon: "download".to_string(),
    };
    println!("Updating state to: {:?}", state);
    if let Err(e) = update_island_state(&state) {
        eprintln!("Error: {}", e);
        return;
    }

    println!("Notification active for 5 seconds...");
    thread::sleep(Duration::from_secs(5));

    // 2. Clear state
    println!("Clearing notification...");
    let _ = clear_island_state();
    println!("Done!");
}
