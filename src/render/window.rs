use std::sync::Arc;
use smithay::reexports::wayland_server::backend::ObjectId;
use smithay::utils::{Rectangle, Logical};
use crate::state::State;

pub struct WindowRenderer {
    gl: Arc<glow::Context>,
}

impl WindowRenderer {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self { gl }
    }

    pub unsafe fn draw_normal(&self, _id: ObjectId, _rect: Rectangle<i32, Logical>, _state: &State) {
        // Render buffer client trực tiếp lên tọa độ màn hình mà không cần qua pipeline làm mờ
    }

    pub unsafe fn draw_system_ui(&self, _state: &State) {
        // Vẽ thanh Panel phía trên và Dock ứng dụng phía dưới lơ lửng
        // Áp dụng độ trong suốt và làm mờ tương tự phong cách Glassmorphism
    }
}
