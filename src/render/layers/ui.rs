use glow::HasContext;
use crate::state::State;
use crate::render::pipeline::BlurPipeline;
use super::{RenderLayer, RenderContext};
use smithay::backend::renderer::Frame;
use smithay::backend::renderer::glow::GlowFrame;
use smithay::utils::{Rectangle, Physical};

/// Renders system UI elements (top bar, dock) above all application windows.
pub struct SystemUiLayer {
    blur_pipeline: Option<BlurPipeline>,
}

impl SystemUiLayer {
    pub fn new() -> Self {
        Self { blur_pipeline: None }
    }

    /// Copies the screen region behind a UI element into a fresh RGBA texture.
    unsafe fn capture_background(gl: &glow::Context, screen_h: i32, x: i32, y: i32, w: i32, h: i32) -> glow::Texture {
        let tex = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        let gl_y = screen_h - (y + h);
        gl.copy_tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA, x, gl_y, w, h, 0);
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

impl RenderLayer for SystemUiLayer {
    fn resize(&mut self, ctx: &RenderContext) {
        self.blur_pipeline = Some(BlurPipeline::new(ctx.gl.clone(), ctx.width, ctx.height, 4));
    }

    fn draw(
        &mut self,
        ctx: &RenderContext,
        state: &State,
        frame: &mut GlowFrame<'_, '_>,
    ) {
        let config = state.config.load();
        let tint = parse_hex_color(&config.theme.blur_tint, config.theme.blur_opacity);

        let screen_w = ctx.width;
        let screen_h = ctx.height;

        unsafe {
            let gl = &ctx.gl;

            // 1. Render Top Bar (Height: 32px)
            let top_bar_rect = Rectangle::<i32, Physical>::new((0, 0).into(), (screen_w, 32).into());
            let intersection = Rectangle::<i32, Physical>::new((0, 0).into(), (screen_w, screen_h).into()).intersection(top_bar_rect);
            if let Some(clamped) = intersection {
                let cx = clamped.loc.x;
                let cy = clamped.loc.y;
                let cw = clamped.size.w;
                let ch = clamped.size.h;

                if cw > 0 && ch > 0 {
                    let gl_y = screen_h - (cy + ch);
                    let bg_tex = Self::capture_background(gl, screen_h, cx, cy, cw, ch);
                    if let Some(ref pipeline) = self.blur_pipeline {
                        Self::composite_glass(gl, pipeline, bg_tex, cx, gl_y, cw, ch, tint);
                    }
                    gl.delete_texture(bg_tex);
                }
            }

            // Draw Top Bar bottom border
            let top_bar_border_color = parse_hex_color(&config.window.border_color, 0.25).into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((0, 31).into(), (screen_w, 1).into()),
                &[Rectangle::<i32, Physical>::from_size((screen_w, 1).into())],
                top_bar_border_color,
            );

            // 2. Render Dock (Width: 400px, Height: 48px, 12px from bottom)
            let dock_w = 400;
            let dock_h = 48;
            let dock_x = (screen_w - dock_w) / 2;
            let dock_y = screen_h - 60;
            let dock_rect = Rectangle::<i32, Physical>::new((dock_x, dock_y).into(), (dock_w, dock_h).into());

            let intersection_dock = Rectangle::<i32, Physical>::new((0, 0).into(), (screen_w, screen_h).into()).intersection(dock_rect);
            if let Some(clamped) = intersection_dock {
                let cx = clamped.loc.x;
                let cy = clamped.loc.y;
                let cw = clamped.size.w;
                let ch = clamped.size.h;

                if cw > 0 && ch > 0 {
                    let gl_y = screen_h - (cy + ch);
                    let bg_tex = Self::capture_background(gl, screen_h, cx, cy, cw, ch);
                    if let Some(ref pipeline) = self.blur_pipeline {
                        Self::composite_glass(gl, pipeline, bg_tex, cx, gl_y, cw, ch, tint);
                    }
                    gl.delete_texture(bg_tex);
                }
            }

            // Draw Dock border highlight
            let dock_border_color = parse_hex_color(&config.window.border_color, 0.35).into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x, dock_y).into(), (dock_w, 1).into()),
                &[Rectangle::<i32, Physical>::from_size((dock_w, 1).into())],
                dock_border_color,
            );
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x, dock_y + dock_h - 1).into(), (dock_w, 1).into()),
                &[Rectangle::<i32, Physical>::from_size((dock_w, 1).into())],
                dock_border_color,
            );
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x, dock_y).into(), (1, dock_h).into()),
                &[Rectangle::<i32, Physical>::from_size((1, dock_h).into())],
                dock_border_color,
            );
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x + dock_w - 1, dock_y).into(), (1, dock_h).into()),
                &[Rectangle::<i32, Physical>::from_size((1, dock_h).into())],
                dock_border_color,
            );

            // 3. Render Workspace Indicators (Top-Left)
            // Draw 3 workspace dots (active dot is white, inactive are semi-transparent)
            let active_dot_color = [1.0, 1.0, 1.0, 1.0].into();
            let inactive_dot_color = [1.0, 1.0, 1.0, 0.4].into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((16, 12).into(), (8, 8).into()),
                &[Rectangle::<i32, Physical>::from_size((8, 8).into())],
                active_dot_color,
            );
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((30, 12).into(), (8, 8).into()),
                &[Rectangle::<i32, Physical>::from_size((8, 8).into())],
                inactive_dot_color,
            );
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((44, 12).into(), (8, 8).into()),
                &[Rectangle::<i32, Physical>::from_size((8, 8).into())],
                inactive_dot_color,
            );

            // 4. Render Simple Indicators (Top-Right)
            // Clock placeholder (a simple rounded block at the top right)
            let indicator_bg_color = [1.0, 1.0, 1.0, 0.15].into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((screen_w - 120, 8).into(), (80, 16).into()),
                &[Rectangle::<i32, Physical>::from_size((80, 16).into())],
                indicator_bg_color,
            );
            // Battery block
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((screen_w - 30, 8).into(), (18, 16).into()),
                &[Rectangle::<i32, Physical>::from_size((18, 16).into())],
                indicator_bg_color,
            );

            // 5. Render Dock Application Icons
            // Icon 1: Terminal (slate blue)
            let icon_color_term = [0.2, 0.4, 0.6, 0.85].into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x + 40, dock_y + 8).into(), (32, 32).into()),
                &[Rectangle::<i32, Physical>::from_size((32, 32).into())],
                icon_color_term,
            );
            // Icon 2: Browser (teal)
            let icon_color_web = [0.1, 0.6, 0.6, 0.85].into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x + 130, dock_y + 8).into(), (32, 32).into()),
                &[Rectangle::<i32, Physical>::from_size((32, 32).into())],
                icon_color_web,
            );
            // Icon 3: Files (gold)
            let icon_color_files = [0.7, 0.5, 0.2, 0.85].into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x + 220, dock_y + 8).into(), (32, 32).into()),
                &[Rectangle::<i32, Physical>::from_size((32, 32).into())],
                icon_color_files,
            );
            // Icon 4: Settings (purple)
            let icon_color_settings = [0.5, 0.3, 0.6, 0.85].into();
            let _ = frame.draw_solid(
                Rectangle::<i32, Physical>::new((dock_x + 310, dock_y + 8).into(), (32, 32).into()),
                &[Rectangle::<i32, Physical>::from_size((32, 32).into())],
                icon_color_settings,
            );
        }
    }
}

/// Parses a `#RRGGBB` hex string into an `[r, g, b, a]` linear colour.
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
