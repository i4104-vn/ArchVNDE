use crate::state::State;
use smithay::input::{SeatHandler, SeatState, Seat};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::delegate_seat;

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {
        // Cập nhật cửa sổ active khi trỏ chuột click hoặc di chuyển tiêu điểm
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
        // Cập nhật đồ họa con trỏ chuột hệ thống
    }
}

delegate_seat!(State);
