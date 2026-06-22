use glow::HasContext;
use crate::state::State;
use crate::render::pipeline::BlurPipeline;
use super::{RenderLayer, RenderContext};

/// Renders all mapped client windows, applying the glass-blur effect where enabled.
pub struct WindowsLayer {
    blur_pipeline: Option<BlurPipeline>,
}

impl WindowsLayer {
    pub fn new() -> Self {
        Self { blur_pipeline: None }
    }

    /// Copies the screen region behind a window into a fresh RGBA texture.
    unsafe fn capture_background(gl: &glow::Context, x: i32, y: i32, w: i32, h: i32) -> glow::Texture {
        let tex = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.copy_tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA, x, y, w, h, 0);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        tex
    }

    /// Composites a blurred background with a color tint to produce the glass effect.
    unsafe fn composite_glass(
        gl: &glow::Context,
        pipeline: &BlurPipeline,
        blurred_bg: glow::Texture,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        tint: [f32; 4],
    ) {
        gl.use_program(Some(pipeline.composite_program));

        if let Some(loc) = gl.get_uniform_location(pipeline.composite_program, "u_tint_color") {
            gl.uniform_4_f32(Some(&loc), tint[0], tint[1], tint[2], tint[3]);
        }
        if let Some(loc) = gl.get_uniform_location(pipeline.composite_program, "u_resolution") {
            gl.uniform_2_f32(Some(&loc), w as f32, h as f32);
        }

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(blurred_bg));
        if let Some(loc) = gl.get_uniform_location(pipeline.composite_program, "u_blurred_background") {
            gl.uniform_1_i32(Some(&loc), 0);
        }

        gl.viewport(x, y, w, h);
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    }
}

impl RenderLayer for WindowsLayer {
    fn resize(&mut self, ctx: &RenderContext) {
        self.blur_pipeline = Some(BlurPipeline::new(ctx.gl.clone(), ctx.width, ctx.height, 4));
    }

    fn draw(&mut self, ctx: &RenderContext, state: &State) {
        let config = state.config.load();
        let tint = parse_hex_color(&config.theme.blur_tint, config.theme.blur_opacity);

        unsafe {
            let gl = &ctx.gl;
            for (_id, info) in state.windows.sorted_windows() {
                let x = info.rect.loc.x;
                let y = info.rect.loc.y;
                let w = info.rect.size.w;
                let h = info.rect.size.h;

                if info.blur_enabled {
                    if let Some(ref pipeline) = self.blur_pipeline {
                        let bg_tex = Self::capture_background(gl, x, y, w, h);
                        let blurred = pipeline.compute_blur(bg_tex, w, h);
                        Self::composite_glass(gl, pipeline, blurred, x, y, w, h, tint);
                        gl.delete_texture(bg_tex);
                    }
                } else {
                    // TODO: blit wl_surface client buffer
                    gl.viewport(x, y, w, h);
                }
            }
        }
    }
}

/// Parses a `#RRGGBB` hex string into an `[r, g, b, a]` linear colour.
///
/// Returns a dark blue-grey fallback if the input is not a valid 6-digit hex.
fn parse_hex_color(hex: &str, opacity: f32) -> [f32; 4] {
    let s = hex.trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0) as f32 / 255.0;
        [r, g, b, opacity]
    } else {
        [0.1, 0.15, 0.2, opacity]
    }
}
