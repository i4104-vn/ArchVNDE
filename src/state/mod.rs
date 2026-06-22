pub mod compositor;
pub mod input;
pub mod window;

pub use window::Layer;

use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::backend::ObjectId;
use smithay::utils::{Logical, Rectangle};
use std::sync::Arc;
use arc_swap::ArcSwap;
use crate::config::Config;

use self::compositor::WaylandState;
use self::input::InputState;
use self::window::WindowState;

/// Per-client data stored by the Wayland server.
pub struct ClientState {
    pub compositor_state: smithay::wayland::compositor::CompositorClientState,
}

impl smithay::reexports::wayland_server::backend::ClientData for ClientState {}

/// Top-level compositor state — a thin container delegating to domain sub-states.
///
/// # Domain layout
/// - [`WaylandState`] — all Wayland protocol globals
/// - [`InputState`] — seat and input routing
/// - [`WindowState`] — window layout, z-ordering, blur cache
pub struct State {
    pub display_handle: DisplayHandle,
    #[allow(dead_code)]
    pub loop_handle: LoopHandle<'static, State>,
    pub wayland: WaylandState,
    pub input: InputState,
    pub windows: WindowState,
    pub config: Arc<ArcSwap<Config>>,
}

impl State {
    /// Initialises all sub-states and registers Wayland globals on the display.
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, State>,
    ) -> Self {
        State {
            wayland: WaylandState::new(&display_handle),
            input: InputState::new(&display_handle),
            windows: WindowState::new(),
            config: Arc::new(ArcSwap::new(Arc::new(Config::default()))),
            display_handle,
            loop_handle,
        }
    }

    /// Convenience forwarder so protocol handlers don't reach into `windows` directly.
    pub fn add_window(
        &mut self,
        id: ObjectId,
        rect: Rectangle<i32, Logical>,
        blur: bool,
        layer: Layer,
    ) {
        self.windows.add_window(id, rect, blur, layer);
    }
}
