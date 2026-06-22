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

    fn focus_changed(&mut self, _seat: &Seat<Self>, focused: Option<&WlSurface>) {
        for window in self.windows.space.elements() {
            if let Some(toplevel) = window.toplevel() {
                let wl_surf = toplevel.wl_surface();
                let is_active = focused.map(|f| f == wl_surf).unwrap_or(false);
                toplevel.with_pending_state(|state| {
                    if is_active {
                        state.states.set(smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Activated);
                    } else {
                        state.states.unset(smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State::Activated);
                    }
                });
                toplevel.send_configure();
            }
        }
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {}
}

delegate_seat!(State);
