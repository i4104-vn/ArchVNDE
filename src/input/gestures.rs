use crate::state::State;

pub fn handle_swipe(_state: &mut State, finger_count: u32, dx: f64, _dy: f64) {
    if finger_count == 3 {
        if dx > 10.0 {
            tracing::info!("Gesture: Swipe phải với 3 ngón -> Chuyển sang workspace bên phải");
            // Thực hiện chuyển đổi layout / workspace
        } else if dx < -10.0 {
            tracing::info!("Gesture: Swipe trái với 3 ngón -> Chuyển sang workspace bên trái");
            // Thực hiện chuyển đổi layout / workspace
        }
    }
}
