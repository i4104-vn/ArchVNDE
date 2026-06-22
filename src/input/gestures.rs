use crate::state::State;

/// Handles multi-finger swipe gestures.
///
/// Currently recognises 3-finger left/right swipes for workspace switching.
pub fn handle_swipe(_state: &mut State, finger_count: u32, dx: f64, _dy: f64) {
    if finger_count == 3 {
        if dx > 10.0 {
            // TODO: switch to right workspace
        } else if dx < -10.0 {
            // TODO: switch to left workspace
        }
    }
}
