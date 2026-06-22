use crate::state::State;
use std::collections::HashMap;

/// Maps `(modifier_mask, keycode)` pairs to compositor actions.
pub struct KeyBindings {
    bindings: HashMap<(u32, u32), Box<dyn Fn(&mut State) + Send + Sync>>,
}

impl KeyBindings {
    /// Registers the built-in default bindings.
    ///
    /// | Key | Action |
    /// |-----|--------|
    /// | Super + Return | Launch terminal from `config.shortcut.launch_terminal` |
    pub fn new() -> Self {
        let mut bindings: HashMap<(u32, u32), Box<dyn Fn(&mut State) + Send + Sync>> =
            HashMap::new();

        // Mod4 (Super) + Return (keycode 36)
        bindings.insert(
            (8, 36),
            Box::new(|state: &mut State| {
                let config = state.config.load();
                let _ = std::process::Command::new(&*config.shortcut.launch_terminal).spawn();
            }),
        );

        Self { bindings }
    }

    /// Returns `true` and executes the action if a binding matches.
    pub fn handle_key(&self, modifiers: u32, keycode: u32, state: &mut State) -> bool {
        if let Some(action) = self.bindings.get(&(modifiers, keycode)) {
            action(state);
            true
        } else {
            false
        }
    }
}
