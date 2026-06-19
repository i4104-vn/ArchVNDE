pub mod pipeline;
pub mod shaders;
pub mod wallpaper;
pub mod window;

use std::sync::Arc;
use glow::HasContext;
use crate::render::pipeline::BlurPipeline;
use crate::render::wallpaper::WallpaperRenderer;
use crate::render::window::WindowRenderer;
use crate::State;
use smithay::utils::{Rectangle, Logical};

pub struct GlassRenderer {
    gl: Arc<glow::Context>,
    blur_pipeline: Option<BlurPipeline>,
    wallpaper_renderer: WallpaperRenderer,
    window_renderer: WindowRenderer,
}

impl GlassRenderer {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        let wallpaper_renderer = WallpaperRenderer::new(gl.clone());
        let window_renderer = WindowRenderer::new(gl.clone());

        Self {
            gl,
            blur_pipeline: None,
            wallpaper_renderer,
            window_renderer,
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.blur_pipeline = Some(BlurPipeline::new(self.gl.clone(), width, height, 4));
    }

    pub fn render_frame(&mut self, state: &mut State, output_rect: Rectangle<i32, Logical>) {
        unsafe {
            let gl = &self.gl;

            // 1. Clear Screen với màu nền tối
            gl.clear_color(0.04, 0.08, 0.16, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // Bật alpha blending
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            // 2. Vẽ Wallpaper
            self.wallpaper_renderer.draw(output_rect);

            // 3. Render các cửa sổ theo thứ tự Z-index
            let mut sorted_windows: Vec<_> = state.windows.iter().collect();
            sorted_windows.sort_by_key(|(_, info)| info.z_index);

            for (id, info) in sorted_windows {
                if info.blur_enabled {
                    if let Some(ref pipeline) = self.blur_pipeline {
                        // Trích xuất texture nền đã được vẽ đằng sau cửa sổ
                        let background_texture = self.capture_frame_rect(info.rect);

                        // Chạy blur shader trên GPU
                        let blurred_tex = pipeline.compute_blur(
                            background_texture,
                            info.rect.loc.x,
                            info.rect.loc.y,
                            info.rect.size.w,
                            info.rect.size.h,
                        );

                        // Phủ kính mờ (mix màu tint + viền + client buffer)
                        self.composite_glass_window(
                            blurred_tex,
                            id.clone(),
                            info.rect,
                            state,
                        );

                        gl.delete_texture(background_texture);
                    }
                } else {
                    self.window_renderer.draw_normal(id.clone(), info.rect, state);
                }
            }

            // 4. Vẽ UI Hệ thống lớp trên cùng (Top Bar / Dock)
            self.window_renderer.draw_system_ui(state);
        }
    }

    unsafe fn capture_frame_rect(&self, rect: Rectangle<i32, Logical>) -> glow::Texture {
        let gl = &self.gl;
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        
        gl.copy_tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA,
            rect.loc.x,
            rect.loc.y,
            rect.size.w,
            rect.size.h,
            0,
        );
        
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        
        texture
    }

    unsafe fn composite_glass_window(
        &self,
        blurred_bg: glow::Texture,
        _id: smithay::reexports::wayland_server::backend::ObjectId,
        rect: Rectangle<i32, Logical>,
        state: &State,
    ) {
        let gl = &self.gl;
        if let Some(ref pipeline) = self.blur_pipeline {
            gl.use_program(Some(pipeline.composite_program));

            let config = state.config.load();
            let tint_rgba = parse_hex_color(&config.theme.blur_tint, config.theme.blur_opacity);

            let tint_loc = gl.get_uniform_location(pipeline.composite_program, "u_tint_color");
            if let Some(loc) = tint_loc {
                gl.uniform_4_f32(Some(&loc), tint_rgba[0], tint_rgba[1], tint_rgba[2], tint_rgba[3]);
            }

            let res_loc = gl.get_uniform_location(pipeline.composite_program, "u_resolution");
            if let Some(loc) = res_loc {
                gl.uniform_2_f32(Some(&loc), rect.size.w as f32, rect.size.h as f32);
            }

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(blurred_bg));
            let bg_loc = gl.get_uniform_location(pipeline.composite_program, "u_blurred_background");
            if let Some(loc) = bg_loc {
                gl.uniform_1_i32(Some(&loc), 0);
            }

            gl.viewport(rect.loc.x, rect.loc.y, rect.size.w, rect.size.h);
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }
    }
}

fn parse_hex_color(hex: &str, opacity: f32) -> [f32; 4] {
    let clean_hex = hex.trim_start_matches('#');
    if clean_hex.len() == 6 {
        let r = u8::from_str_radix(&clean_hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&clean_hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&clean_hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
        [r, g, b, opacity]
    } else {
        [0.1, 0.15, 0.2, opacity]
    }
}
