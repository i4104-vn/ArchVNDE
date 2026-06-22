use glow::HasContext;
use crate::state::State;
use crate::render::pipeline::BlurPipeline;
use super::{RenderLayer, RenderContext};

use smithay::backend::renderer::element::{AsRenderElements, RenderElement, Element};
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::Frame;
use smithay::backend::renderer::glow::{GlowRenderer, GlowFrame};
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Rectangle, Logical, Physical, Scale};

/// Renders all mapped client windows, applying the glass-blur effect where enabled.
pub struct WindowsLayer {
    blur_pipeline: Option<BlurPipeline>,
    rendered_elements: Vec<(
        Rectangle<i32, Logical>,
        bool,
        Vec<WaylandSurfaceRenderElement<GlowRenderer>>,
    )>,
}

impl WindowsLayer {
    pub fn new() -> Self {
        Self {
            blur_pipeline: None,
            rendered_elements: Vec::new(),
        }
    }

    /// Copies the screen region behind a window into a fresh RGBA texture.
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

impl RenderLayer for WindowsLayer {
    fn resize(&mut self, ctx: &RenderContext) {
        self.blur_pipeline = Some(BlurPipeline::new(ctx.gl.clone(), ctx.width, ctx.height, 4));
    }

    fn prepare(
        &mut self,
        _ctx: &RenderContext,
        state: &State,
        renderer: &mut GlowRenderer,
    ) {
        self.rendered_elements.clear();
        for window in state.windows.space.elements() {
            let rect = state.windows.space.element_bbox(window).unwrap_or_default();
            let x = rect.loc.x;
            let y = rect.loc.y;

            let id = window.toplevel().map(|t| t.wl_surface().id());
            let blur_enabled = id
                .and_then(|id| state.windows.windows.get(&id).map(|info| info.blur_enabled))
                .unwrap_or(true);

            let elements: Vec<WaylandSurfaceRenderElement<GlowRenderer>> = window.render_elements(
                renderer,
                (x, y).into(),
                Scale::from(1.0),
                1.0,
            );

            self.rendered_elements.push((rect, blur_enabled, elements));
        }
    }

    fn draw(
        &mut self,
        ctx: &RenderContext,
        state: &State,
        frame: &mut GlowFrame<'_, '_>,
    ) {
        let config = state.config.load();
        let tint = parse_hex_color(&config.theme.blur_tint, config.theme.blur_opacity);

        unsafe {
            let gl = &ctx.gl;
            let screen_w = ctx.width;
            let screen_h = ctx.height;

            for &(rect, blur_enabled, ref elements) in &self.rendered_elements {
                let x = rect.loc.x;
                let y = rect.loc.y;
                let w = rect.size.w;
                let h = rect.size.h;

                if blur_enabled {
                    let intersection = Rectangle::new((0, 0).into(), (screen_w, screen_h).into()).intersection(rect);
                    if let Some(clamped) = intersection {
                        let cx = clamped.loc.x;
                        let cy = clamped.loc.y;
                        let cw = clamped.size.w;
                        let ch = clamped.size.h;

                        if cw > 0 && ch > 0 {
                            let gl_y = screen_h - (cy + ch);
                            let bg_tex = Self::capture_background(gl, screen_h, cx, cy, cw, ch);
                            if let Some(ref pipeline) = self.blur_pipeline {
                                let blurred_bg = pipeline.compute_blur(bg_tex, cw, ch);
                                Self::composite_glass(gl, pipeline, blurred_bg, cx, gl_y, cw, ch, tint);
                                gl.delete_texture(blurred_bg);
                            }
                            gl.delete_texture(bg_tex);
                        }
                    }
                }

                // Draw window border highlight
                let border_color = parse_hex_color(&config.window.border_color, 0.4).into();
                let border_width = config.window.border_width as i32;

                if border_width > 0 {
                    let border_top = Rectangle::<i32, Physical>::new((x - border_width, y - border_width).into(), (w + 2 * border_width, border_width).into());
                    let _ = frame.draw_solid(border_top, &[border_top], border_color);

                    let border_bottom = Rectangle::<i32, Physical>::new((x - border_width, y + h).into(), (w + 2 * border_width, border_width).into());
                    let _ = frame.draw_solid(border_bottom, &[border_bottom], border_color);

                    let border_left = Rectangle::<i32, Physical>::new((x - border_width, y).into(), (border_width, h).into());
                    let _ = frame.draw_solid(border_left, &[border_left], border_color);

                    let border_right = Rectangle::<i32, Physical>::new((x + w, y).into(), (border_width, h).into());
                    let _ = frame.draw_solid(border_right, &[border_right], border_color);
                }

                for element in elements {
                    let src = element.src();
                    let dst = element.geometry(Scale::from(1.0));
                    let damage = &[Rectangle::from_size(dst.size)];
                    let opaque = &[];
                    let _ = element.draw(frame, src, dst, damage, opaque);
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
