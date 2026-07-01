use std::io::Write;

pub fn get_history() -> Vec<String> {
    let history_path = "/tmp/archvnde-switcher-history.txt";
    if let Ok(content) = std::fs::read_to_string(history_path) {
        content.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

pub fn save_history(active_name: &str) {
    let history_path = "/tmp/archvnde-switcher-history.txt";
    let mut history = get_history();
    
    // Remove if already exists
    history.retain(|x| x != active_name);
    
    // Insert at the beginning
    history.insert(0, active_name.to_string());
    
    // Keep only top 20
    history.truncate(20);
    
    // Write back
    if let Ok(mut file) = std::fs::File::create(history_path) {
        for name in history {
            let _ = writeln!(file, "{}", name);
        }
    }
}
