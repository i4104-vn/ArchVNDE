use glow::HasContext;
use crate::state::State;
use super::{RenderLayer, RenderContext};

/// Renders the desktop background (wallpaper / gradient) below all windows.
pub struct BackgroundLayer;

impl BackgroundLayer {
    pub fn new() -> Self {
        Self
    }
}

impl RenderLayer for BackgroundLayer {
    fn draw(&mut self, ctx: &RenderContext, _state: &State) {
        let rect = ctx.output_rect();
        unsafe {
            ctx.gl.viewport(rect.loc.x, rect.loc.y, rect.size.w, rect.size.h);
            // TODO: blit wallpaper texture or render gradient from config
        }
    }
}
