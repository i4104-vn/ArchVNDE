use crate::state::State;
use super::{RenderLayer, RenderContext};

/// Renders system UI elements (top bar, dock) above all application windows.
pub struct SystemUiLayer;

impl SystemUiLayer {
    pub fn new() -> Self {
        Self
    }
}

impl RenderLayer for SystemUiLayer {
    fn draw(&mut self, _ctx: &RenderContext, _state: &State) {
        // TODO: render panel and dock with glassmorphism
    }
}
