use smithay::backend::winit::{self, WinitGraphicsBackend, WinitEventLoop, WinitEvent};
use smithay::backend::renderer::glow::GlowRenderer;
use smithay::backend::renderer::{Renderer, Frame};
use smithay::utils::{Size, Transform};
use std::sync::Arc;

use super::{CompositorBackend, BackendEvent};

/// [`CompositorBackend`] implementation that renders into a Winit window.
///
/// Useful for development and CI: the compositor runs nested inside an existing
/// desktop without requiring DRM/KMS access.
pub struct WinitBackend {
    backend: WinitGraphicsBackend<GlowRenderer>,
    input: WinitEventLoop,
}

impl WinitBackend {
    /// Initialises Winit, creates an EGL surface, and extracts the GL context.
    ///
    /// Returns the backend together with a [`glow::Context`] that must be
    /// passed to [`crate::render::GlassRenderer::new`].
    pub fn init() -> Result<(Self, Arc<glow::Context>), Box<dyn std::error::Error>> {
        let (mut backend, input) = winit::init::<GlowRenderer>()?;
        let gl = backend.renderer().with_context(|gl| gl.clone())?;
        Ok((Self { backend, input }, gl))
    }

    /// Returns the current window size in physical pixels.
    pub fn current_size(&self) -> (u32, u32) {
        let s = self.backend.window_size();
        (s.w as u32, s.h as u32)
    }

    /// Binds the EGL surface, executes `render_fn`, then finalises the frame.
    ///
    /// The Frame guard is held for the duration of `render_fn` so all GL calls
    /// inside it target the correct framebuffer.
    pub fn render_with<F: FnOnce()>(&mut self, width: i32, height: i32, render_fn: F) {
        if let Ok((glow_renderer, mut target)) = self.backend.bind() {
            let size = Size::<i32, smithay::utils::Physical>::from((width, height));
            if let Ok(frame) = glow_renderer.render(&mut target, size, Transform::Normal) {
                render_fn();
                let _ = frame.finish();
            }
        }
    }
}

impl CompositorBackend for WinitBackend {
    fn poll_events<F: FnMut(BackendEvent)>(&mut self, mut handler: F) {
        let _ = self.input.dispatch_new_events(|event| match event {
            WinitEvent::Resized { size, .. } => handler(BackendEvent::Resized {
                width: size.w as u32,
                height: size.h as u32,
            }),
            WinitEvent::Redraw => handler(BackendEvent::Redraw),
            WinitEvent::CloseRequested => handler(BackendEvent::CloseRequested),
            _ => {}
        });
    }

    fn submit(&mut self) {
        self.backend.submit(None).ok();
    }
}
