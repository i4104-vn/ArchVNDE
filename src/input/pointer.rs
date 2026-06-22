use crate::state::State;

/// Handles pointer motion and button events from the backend.
pub fn handle_motion(_state: &mut State, _dx: f64, _dy: f64) {
    // TODO: hit-test Space, update pointer focus, forward motion to client
    _state.windows.space.elements().next().map(|_window| {});
}

/// Handles a pointer button press or release.
pub fn handle_button_press(_state: &mut State, _button: u32, _pressed: bool) {
    // TODO: update focus, send button event to focused surface
}
