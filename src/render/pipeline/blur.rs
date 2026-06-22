use std::sync::Arc;
use glow::HasContext;
use crate::render::shaders::*;

/// Dual-Kawase blur pipeline using ping-pong framebuffers.
///
/// On each [`BlurPipeline::compute_blur`] call the pipeline:
/// 1. **Downsample** the source texture `passes` times at half resolution each step.
/// 2. **Upsample** back to full size with a configurable spread offset.
///
/// GPU resources are freed automatically when the pipeline is dropped.
pub struct BlurPipeline {
    gl: Arc<glow::Context>,
    pub(crate) downsample_program: glow::Program,
    pub(crate) upsample_program: glow::Program,
    /// Composite program exposed to [`crate::render::layers::windows::WindowsLayer`].
    pub composite_program: glow::Program,
    ping_pong_fbos: Vec<glow::Framebuffer>,
    ping_pong_textures: Vec<glow::Texture>,
}

impl BlurPipeline {
    /// Allocates GPU framebuffers and compiles all shader programs.
    ///
    /// `passes` controls blur quality (4 is a good default). Each pass halves
    /// the working resolution, so `passes` is capped when width or height
    /// reaches zero.
    pub fn new(gl: Arc<glow::Context>, width: i32, height: i32, passes: usize) -> Self {
        unsafe {
            let prev_fbo = save_fbo(&gl);

            let downsample_program = compile_shader_program(&gl, VERTEX_SHADER, DOWNSAMPLE_FRAG_SHADER);
            let upsample_program = compile_shader_program(&gl, VERTEX_SHADER, UPSAMPLE_FRAG_SHADER);
            let composite_program = compile_shader_program(&gl, VERTEX_SHADER, GLASS_COMPOSITION_FRAG_SHADER);

            let mut ping_pong_fbos = Vec::new();
            let mut ping_pong_textures = Vec::new();

            let mut w = width;
            let mut h = height;

            for _ in 0..passes {
                w /= 2;
                h /= 2;
                if w == 0 || h == 0 {
                    break;
                }

                let texture = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    w,
                    h,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(None),
                );
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
                gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);

                let fbo = gl.create_framebuffer().unwrap();
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
                gl.framebuffer_texture_2d(
                    glow::FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::TEXTURE_2D,
                    Some(texture),
                    0,
                );

                ping_pong_textures.push(texture);
                ping_pong_fbos.push(fbo);
            }

            restore_fbo(&gl, prev_fbo);

            BlurPipeline {
                gl,
                downsample_program,
                upsample_program,
                composite_program,
                ping_pong_fbos,
                ping_pong_textures,
            }
        }
    }

    /// Runs dual-Kawase blur on `source_texture` and returns the blurred result.
    ///
    /// # Safety
    /// Caller must ensure a valid OpenGL context is current.
    pub unsafe fn compute_blur(
        &self,
        source_texture: glow::Texture,
        w: i32,
        h: i32,
    ) -> glow::Texture {
        let gl = &self.gl;
        let prev_fbo = save_fbo(gl);

        gl.use_program(Some(self.downsample_program));
        let halfpixel_loc = gl.get_uniform_location(self.downsample_program, "u_halfpixel");

        let mut current_tex = source_texture;
        let mut current_w = w;
        let mut current_h = h;

        for (i, &fbo) in self.ping_pong_fbos.iter().enumerate() {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.viewport(0, 0, current_w / 2, current_h / 2);
            gl.bind_texture(glow::TEXTURE_2D, Some(current_tex));

            if let Some(loc) = halfpixel_loc {
                gl.uniform_2_f32(Some(&loc), 0.5 / current_w as f32, 0.5 / current_h as f32);
            }

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            current_tex = self.ping_pong_textures[i];
            current_w /= 2;
            current_h /= 2;
        }

        gl.use_program(Some(self.upsample_program));
        let up_halfpixel_loc = gl.get_uniform_location(self.upsample_program, "u_halfpixel");
        let offset_loc = gl.get_uniform_location(self.upsample_program, "u_offset");

        if let Some(loc) = offset_loc {
            gl.uniform_1_f32(Some(&loc), 2.0);
        }

        for i in (0..self.ping_pong_fbos.len().saturating_sub(1)).rev() {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ping_pong_fbos[i]));
            gl.viewport(0, 0, current_w * 2, current_h * 2);
            gl.bind_texture(glow::TEXTURE_2D, Some(current_tex));

            if let Some(loc) = up_halfpixel_loc {
                gl.uniform_2_f32(Some(&loc), 0.5 / current_w as f32, 0.5 / current_h as f32);
            }

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            current_tex = self.ping_pong_textures[i];
            current_w *= 2;
            current_h *= 2;
        }

        restore_fbo(gl, prev_fbo);
        current_tex
    }
}

impl Drop for BlurPipeline {
    fn drop(&mut self) {
        unsafe {
            let gl = &self.gl;
            gl.delete_program(self.downsample_program);
            gl.delete_program(self.upsample_program);
            gl.delete_program(self.composite_program);
            for &fbo in &self.ping_pong_fbos {
                gl.delete_framebuffer(fbo);
            }
            for &tex in &self.ping_pong_textures {
                gl.delete_texture(tex);
            }
        }
    }
}

// ── GL helpers ────────────────────────────────────────────────────────────────

/// Reads and returns the currently bound `GL_FRAMEBUFFER`.
pub(crate) unsafe fn save_fbo(gl: &glow::Context) -> Option<glow::NativeFramebuffer> {
    let raw = gl.get_parameter_i32(glow::FRAMEBUFFER_BINDING);
    std::num::NonZeroU32::new(raw as u32).map(glow::NativeFramebuffer)
}

/// Restores a previously saved framebuffer binding.
pub(crate) unsafe fn restore_fbo(gl: &glow::Context, fbo: Option<glow::NativeFramebuffer>) {
    gl.bind_framebuffer(glow::FRAMEBUFFER, fbo);
}

/// Compiles a GLSL vertex + fragment pair and links them into a program.
///
/// # Panics
/// Panics if shader compilation or program linking fails, printing the info log.
pub(crate) unsafe fn compile_shader_program(
    gl: &glow::Context,
    vert_src: &str,
    frag_src: &str,
) -> glow::Program {
    let program = gl.create_program().unwrap();

    let vert = gl.create_shader(glow::VERTEX_SHADER).unwrap();
    gl.shader_source(vert, vert_src);
    gl.compile_shader(vert);
    if !gl.get_shader_compile_status(vert) {
        panic!("Vertex shader compilation failed:\n{}", gl.get_shader_info_log(vert));
    }

    let frag = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
    gl.shader_source(frag, frag_src);
    gl.compile_shader(frag);
    if !gl.get_shader_compile_status(frag) {
        panic!("Fragment shader compilation failed:\n{}", gl.get_shader_info_log(frag));
    }

    gl.attach_shader(program, vert);
    gl.attach_shader(program, frag);
    gl.link_program(program);

    if !gl.get_program_link_status(program) {
        panic!("Shader program linking failed:\n{}", gl.get_program_info_log(program));
    }

    gl.delete_shader(vert);
    gl.delete_shader(frag);

    program
}
