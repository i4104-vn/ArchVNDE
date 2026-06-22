use crate::state::State;
use crate::input::keybindings::KeyBindings;

/// Processes raw keyboard events and routes them to keybindings or the focused client.
pub struct KeyboardManager {
    keybindings: KeyBindings,
    active_modifiers: u32,
}

impl KeyboardManager {
    pub fn new() -> Self {
        Self {
            keybindings: KeyBindings::new(),
            active_modifiers: 0,
        }
    }

    /// Handles a key press/release. Returns early if a keybinding consumed the event.
    pub fn process_key(&mut self, keycode: u32, pressed: bool, state: &mut State) {
        if pressed && self.keybindings.handle_key(self.active_modifiers, keycode, state) {
            return;
        }
        state.input.seat.get_keyboard().map(|_kbd| {
            // TODO: forward event to focused Wayland client
            // _kbd.input(state, keycode, pressed_state, serial, time);
        });
    }

    /// Updates the modifier mask (Shift, Ctrl, Super, …) from a modifier event.
    pub fn update_modifiers(&mut self, modifiers: u32) {
        self.active_modifiers = modifiers;
    }
}
