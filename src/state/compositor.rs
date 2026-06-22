use smithay::reexports::wayland_server::DisplayHandle;
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::wayland::shm::ShmState;

/// Aggregates all Wayland protocol handler states into one struct.
///
/// Keeps Wayland globals separate from input and window management concerns.
/// Constructed via [`crate::state::State`] which satisfies all required
/// `GlobalDispatch` and `Dispatch` bounds through the `delegate_*!` macros.
pub struct WaylandState {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState,
    pub shm_state: ShmState,
}

impl WaylandState {
    /// Registers all Wayland globals on the given display.
    ///
    /// Must be called after [`crate::state::State`] has implemented all
    /// `delegate_*!` macros so the required trait bounds are satisfied.
    pub fn new(display_handle: &DisplayHandle) -> Self {
        Self {
            compositor_state: CompositorState::new::<crate::state::State>(display_handle),
            xdg_shell_state: XdgShellState::new::<crate::state::State>(display_handle),
            layer_shell_state: WlrLayerShellState::new::<crate::state::State>(display_handle),
            shm_state: ShmState::new::<crate::state::State>(display_handle, vec![]),
        }
    }
}
