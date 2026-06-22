pub mod background;
pub mod windows;
pub mod ui;

use std::sync::Arc;
use smithay::utils::{Rectangle, Logical};
use crate::state::State;

/// Shared GPU context passed to every [`RenderLayer`] on each frame.
pub struct RenderContext {
    pub gl: Arc<glow::Context>,
    pub width: i32,
    pub height: i32,
}

impl RenderContext {
    pub fn new(gl: Arc<glow::Context>, width: i32, height: i32) -> Self {
        Self { gl, width, height }
    }

    /// Full-screen output rectangle in logical coordinates.
    pub fn output_rect(&self) -> Rectangle<i32, Logical> {
        Rectangle::new((0, 0).into(), (self.width, self.height).into())
    }
}

/// A single rendering pass in the compositor's frame pipeline.
///
/// Layers are drawn bottom-to-top by [`crate::render::GlassRenderer`].
/// Implement this trait for each visual concern (background, windows, UI, …).
///
/// Default implementations are no-ops so a layer only overrides what it needs.
pub trait RenderLayer {
    /// Called once per frame before [`draw`](RenderLayer::draw).
    ///
    /// Use this to update GPU resources that must be ready before drawing
    /// (e.g. uploading dirty textures, computing animations).
    fn prepare(
        &mut self,
        _ctx: &RenderContext,
        _state: &State,
        _renderer: &mut smithay::backend::renderer::glow::GlowRenderer,
    ) {}

    /// Emits draw calls for this layer.
    fn draw(
        &mut self,
        ctx: &RenderContext,
        state: &State,
        frame: &mut smithay::backend::renderer::glow::GlowFrame<'_, '_>,
    );

    /// Called when the output dimensions change.
    ///
    /// Use this to reallocate framebuffers or update projection matrices.
    fn resize(&mut self, _ctx: &RenderContext) {}
}
