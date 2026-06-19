use smithay::desktop::{Space, Window};
use smithay::input::{Seat, SeatState};
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::backend::ObjectId;
use smithay::utils::{Logical, Rectangle};
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shell::wlr_layer::WlrLayerShellState;
use smithay::wayland::shm::ShmState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use crate::config::Config;

pub struct ClientState {
    pub compositor_state: smithay::wayland::compositor::CompositorClientState,
}

impl smithay::reexports::wayland_server::backend::ClientData for ClientState {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub rect: Rectangle<i32, Logical>,
    pub z_index: i32,
    pub is_focused: bool,
    pub blur_enabled: bool,
    pub layer: Layer,
}

pub struct State {
    pub display_handle: DisplayHandle,
    pub space: Space<Window>,
    pub loop_handle: smithay::reexports::calloop::LoopHandle<'static, State>,
    
    // Core Handlers
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState,
    pub shm_state: ShmState,
    
    // Input
    pub seat: Seat<State>,
    pub seat_state: SeatState<State>,
    
    // Custom logic states
    pub windows: HashMap<ObjectId, WindowInfo>,
    pub next_z: i32,
    pub config: Arc<ArcSwap<Config>>,
    pub blur_cache: Arc<Mutex<BlurCache>>,
}

pub struct BlurCache {
    pub map: HashMap<ObjectId, (glow::Texture, u64)>, // Texture ID and Generation timestamp/count
}

impl BlurCache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl State {
    pub fn new(display_handle: DisplayHandle, loop_handle: smithay::reexports::calloop::LoopHandle<'static, State>) -> Self {
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&display_handle, "glass-seat-0");
        
        State {
            display_handle: display_handle.clone(),
            space: Space::default(),
            loop_handle,
            compositor_state: CompositorState::new::<State>(&display_handle),
            xdg_shell_state: XdgShellState::new::<State>(&display_handle),
            layer_shell_state: WlrLayerShellState::new::<State>(&display_handle),
            shm_state: ShmState::new::<State>(&display_handle, vec![]),
            seat,
            seat_state,
            windows: HashMap::new(),
            next_z: 0,
            config: Arc::new(ArcSwap::new(Arc::new(Config::default()))),
            blur_cache: Arc::new(Mutex::new(BlurCache::new())),
        }
    }

    pub fn add_window(&mut self, id: ObjectId, rect: Rectangle<i32, Logical>, blur: bool, layer: Layer) {
        let info = WindowInfo {
            rect,
            z_index: self.next_z,
            is_focused: false,
            blur_enabled: blur,
            layer,
        };
        self.windows.insert(id, info);
        self.next_z += 1;
    }
}

