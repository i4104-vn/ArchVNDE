pub mod pipeline;
pub mod layers;
pub mod shaders;

use std::sync::Arc;
use glow::HasContext;
use smithay::utils::{Rectangle, Logical};

use crate::state::State;
use self::layers::{RenderLayer, RenderContext};
use self::layers::background::BackgroundLayer;
use self::layers::windows::WindowsLayer;
use self::layers::ui::SystemUiLayer;

/// Drives the compositor's render pipeline by dispatching to an ordered list of [`RenderLayer`]s.
///
/// To add a visual layer, implement [`RenderLayer`] and push it in [`GlassRenderer::new`].
/// The default layer order is:
/// 1. [`BackgroundLayer`] — desktop wallpaper
/// 2. [`WindowsLayer`] — client surfaces with optional glass-blur
/// 3. [`SystemUiLayer`] — top bar and dock
pub struct GlassRenderer {
    ctx: RenderContext,
    layers: Vec<Box<dyn RenderLayer>>,
}

impl GlassRenderer {
    /// Creates the renderer with the default layer stack.
    ///
    /// `gl` must be the context extracted from the active backend before the
    /// first render call (see [`crate::backend::winit::WinitBackend::init`]).
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let ctx = RenderContext::new(gl, 1024, 768);
        let layers: Vec<Box<dyn RenderLayer>> = vec![
            Box::new(BackgroundLayer::new()),
            Box::new(WindowsLayer::new()),
            Box::new(SystemUiLayer::new()),
        ];
        Self { ctx, layers }
    }

    /// Updates the output dimensions and notifies every layer.
    pub fn resize(&mut self, width: i32, height: i32) {
        self.ctx.width = width;
        self.ctx.height = height;
        for layer in &mut self.layers {
            layer.resize(&self.ctx);
        }
    }

    /// Runs the preparation pass for all layers (runs with the renderer).
    pub fn prepare(
        &mut self,
        state: &State,
        renderer: &mut smithay::backend::renderer::glow::GlowRenderer,
    ) {
        for layer in &mut self.layers {
            layer.prepare(&self.ctx, state, renderer);
        }
    }

    /// Clears the framebuffer and runs all layers in order.
    ///
    /// `_output_rect` is currently unused but kept for future sub-region rendering.
    pub fn render_frame(
        &mut self,
        state: &mut State,
        _output_rect: Rectangle<i32, Logical>,
        frame: &mut smithay::backend::renderer::glow::GlowFrame<'_, '_>,
    ) {
        unsafe {
            let gl = &self.ctx.gl;
            gl.clear_color(0.04, 0.08, 0.16, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        for layer in &mut self.layers {
            layer.draw(&self.ctx, state, frame);
        }
    }
}
