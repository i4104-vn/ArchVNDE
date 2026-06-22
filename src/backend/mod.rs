pub mod winit;

use smithay::backend::input::InputEvent;
use smithay::backend::winit::WinitInput;

/// Backend-agnostic event emitted by a [`CompositorBackend`] to the main loop.
#[derive(Debug)]
pub enum BackendEvent {
    /// The output surface was resized (in physical pixels).
    Resized { width: u32, height: u32 },
    /// The backend requests a new frame to be rendered and presented.
    Redraw,
    /// The user (or OS) requested that the compositor exits.
    CloseRequested,
    /// An input event occurred.
    Input(InputEvent<WinitInput>),
}

/// Abstraction over compositor backends (winit, udev/DRM, headless, …).
///
/// The main loop interacts exclusively with this trait, so swapping the
/// underlying windowing system requires only a different implementor.
pub trait CompositorBackend {
    /// Drains pending backend events, calling `handler` for each one.
    fn poll_events<F: FnMut(BackendEvent)>(&mut self, handler: F);

    /// Presents the rendered frame to the display.
    fn submit(&mut self);
}
