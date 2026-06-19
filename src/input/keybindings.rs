use crate::state::State;
use std::collections::HashMap;

pub struct KeyBindings {
    bindings: HashMap<(u32, u32), Box<dyn Fn(&mut State) + Send + Sync>>,
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // Mod4 (Super) + Enter (keycode 36) -> Mở terminal được cấu hình trong config.toml
        bindings.insert(
            (8, 36),
            Box::new(|state: &mut State| {
                let config = state.config.load();
                let term = &config.shortcut.launch_terminal;
                tracing::info!("Shortcut triggered: Launching {}", term);
                let _ = std::process::Command::new(term).spawn();
            }) as Box<dyn Fn(&mut State) + Send + Sync>
        );

        Self { bindings }
    }

    pub fn handle_key(&self, modifiers: u32, keycode: u32, state: &mut State) -> bool {
        if let Some(action) = self.bindings.get(&(modifiers, keycode)) {
            action(state);
            true
        } else {
            false
        }
    }
}
