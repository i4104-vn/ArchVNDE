use smithay::input::{Seat, SeatState};
use smithay::reexports::wayland_server::DisplayHandle;

/// Holds the Smithay seat and seat-state for the compositor's single input seat.
///
/// Uses the concrete [`crate::state::State`] type instead of a generic parameter
/// to avoid propagating complex `SeatHandler` + `WaylandFocus` bounds everywhere.
pub struct InputState {
    pub seat: Seat<crate::state::State>,
    pub seat_state: SeatState<crate::state::State>,
}

impl InputState {
    /// Creates the compositor's single Wayland seat named `"glass-seat-0"`.
    pub fn new(display_handle: &DisplayHandle) -> Self {
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(display_handle, "glass-seat-0");

        let _keyboard = seat.add_keyboard(Default::default(), 200, 25)
            .expect("Failed to initialize seat keyboard");
        let _pointer = seat.add_pointer();

        Self { seat, seat_state }
    }
}
