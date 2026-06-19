use crate::state::State;

pub fn handle_motion(_state: &mut State, _dx: f64, _dy: f64) {
    // 1. Cập nhật vị trí con trỏ chuột tương đối
    // 2. Tìm surface nằm ngay dưới vị trí chuột trong Space
    // 3. Gửi sự kiện hover/motion đến client
    _state.space.elements().next().map(|_window| {
        // Gửi tọa độ di chuyển chuột tương ứng sang cho Wayland client
    });
}

pub fn handle_button_press(_state: &mut State, button: u32, pressed: bool) {
    if pressed {
        tracing::info!("Mouse click: button {}", button);
        // Thay đổi tiêu điểm (Focus) cửa sổ đang được click
    }
}
