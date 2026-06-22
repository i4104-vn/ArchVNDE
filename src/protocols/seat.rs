use crate::state::State;
use smithay::input::{SeatHandler, SeatState, Seat};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::delegate_seat;

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.input.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {}
}

delegate_seat!(State);
