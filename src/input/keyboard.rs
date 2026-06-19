use crate::state::State;
use crate::input::keybindings::KeyBindings;

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

    pub fn process_key(&mut self, keycode: u32, pressed: bool, state: &mut State) {
        if pressed {
            // Kiểm tra phím tắt trước khi chuyển tiếp cho client
            let handled = self.keybindings.handle_key(self.active_modifiers, keycode, state);
            if handled {
                return;
            }
        }
        
        // Chuyển tiếp sự kiện phím sang Seat đang active để gửi tới client Wayland
        state.seat.get_keyboard().map(|_kbd| {
            // _kbd.input(state, keycode, pressed_state, serial, time);
        });
    }

    pub fn update_modifiers(&mut self, modifiers: u32) {
        self.active_modifiers = modifiers;
    }
}
