use smithay::desktop::{Space, Window};
use smithay::reexports::wayland_server::backend::ObjectId;
use smithay::utils::{Logical, Rectangle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Z-order layer for a surface, matching the wlr-layer-shell protocol layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

/// Per-window metadata used by the render pipeline.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WindowInfo {
    /// Position and size in logical coordinates.
    pub rect: Rectangle<i32, Logical>,
    /// Higher values are drawn on top.
    pub z_index: i32,
    pub is_focused: bool,
    /// Whether the blur-glass effect should be applied to this window.
    pub blur_enabled: bool,
    pub layer: Layer,
}

/// Cached blur textures keyed by surface id.
///
/// Each entry contains the blur [`glow::Texture`] handle and a generation
/// counter used to invalidate stale cache entries.
#[allow(dead_code)]
pub struct BlurCache {
    pub map: HashMap<ObjectId, (glow::Texture, u64)>,
}

impl BlurCache {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }
}

/// Manages window layout, z-ordering, and the GPU blur cache.
pub struct WindowState {
    /// Smithay desktop space — used for surface mapping and hit-testing.
    pub space: Space<Window>,
    /// Metadata for every mapped surface.
    pub windows: HashMap<ObjectId, WindowInfo>,
    /// Monotonically increasing z-index counter.
    pub next_z: i32,
    /// Shared blur-texture cache accessed from the render thread.
    #[allow(dead_code)]
    pub blur_cache: Arc<Mutex<BlurCache>>,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            space: Space::default(),
            windows: HashMap::new(),
            next_z: 0,
            blur_cache: Arc::new(Mutex::new(BlurCache::new())),
        }
    }

    /// Inserts a new window entry with an auto-incremented z-index.
    pub fn add_window(
        &mut self,
        id: ObjectId,
        rect: Rectangle<i32, Logical>,
        blur: bool,
        layer: Layer,
    ) {
        self.windows.insert(
            id,
            WindowInfo {
                rect,
                z_index: self.next_z,
                is_focused: false,
                blur_enabled: blur,
                layer,
            },
        );
        self.next_z += 1;
    }

    /// Returns all windows sorted by ascending z-index (bottom → top).
    #[allow(dead_code)]
    pub fn sorted_windows(&self) -> Vec<(&ObjectId, &WindowInfo)> {
        let mut sorted: Vec<_> = self.windows.iter().collect();
        sorted.sort_by_key(|(_, info)| info.z_index);
        sorted
    }
}
