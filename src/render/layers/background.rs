use glow::HasContext;
use crate::state::State;
use super::{RenderLayer, RenderContext};

const BACKGROUND_FRAG_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
out vec4 fragColor;

void main() {
    // Beautiful premium dark gradient from deep slatey indigo to rich violet/purple
    vec3 color1 = vec3(0.06, 0.08, 0.16);
    vec3 color2 = vec3(0.12, 0.08, 0.22);
    vec3 finalColor = mix(color1, color2, v_texcoord.x + v_texcoord.y * 0.5);
    fragColor = vec4(finalColor, 1.0);
}
"#;

/// Renders the desktop background (wallpaper / gradient) below all windows.
pub struct BackgroundLayer {
    program: Option<glow::Program>,
}

impl BackgroundLayer {
    pub fn new() -> Self {
        Self { program: None }
    }
}

impl RenderLayer for BackgroundLayer {
    fn draw(
        &mut self,
        ctx: &RenderContext,
        _state: &State,
        _frame: &mut smithay::backend::renderer::glow::GlowFrame<'_, '_>,
    ) {
        let rect = ctx.output_rect();
        let gl = &ctx.gl;

        if self.program.is_none() {
            unsafe {
                let prog = crate::render::pipeline::blur::compile_shader_program(
                    gl,
                    crate::render::shaders::VERTEX_SHADER,
                    BACKGROUND_FRAG_SHADER,
                );
                self.program = Some(prog);
            }
        }

        if let Some(program) = self.program {
            unsafe {
                gl.use_program(Some(program));
                gl.viewport(rect.loc.x, rect.loc.y, rect.size.w, rect.size.h);
                gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            }
        }
    }
}
