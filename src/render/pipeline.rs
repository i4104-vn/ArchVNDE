use glow::HasContext;
use std::sync::Arc;
use crate::render::shaders::*;

pub struct BlurPipeline {
    gl: Arc<glow::Context>,
    downsample_program: glow::Program,
    upsample_program: glow::Program,
    pub composite_program: glow::Program,
    ping_pong_fbos: Vec<glow::Framebuffer>,
    ping_pong_textures: Vec<glow::Texture>,
    passes: usize,
    width: i32,
    height: i32,
}

impl BlurPipeline {
    pub fn new(gl: Arc<glow::Context>, width: i32, height: i32, passes: usize) -> Self {
        unsafe {
            // Compile Shaders
            let downsample_program = compile_shader_program(&gl, VERTEX_SHADER, DOWNSAMPLE_FRAG_SHADER);
            let upsample_program = compile_shader_program(&gl, VERTEX_SHADER, UPSAMPLE_FRAG_SHADER);
            let composite_program = compile_shader_program(&gl, VERTEX_SHADER, GLASS_COMPOSITION_FRAG_SHADER);

            let mut ping_pong_fbos = Vec::new();
            let mut ping_pong_textures = Vec::new();

            // Khởi tạo các FBOs tương ứng dán các texture giảm dần kích thước (width/2, height/2)
            let mut current_width = width;
            let mut current_height = height;

            for _ in 0..passes {
                current_width /= 2;
                current_height /= 2;
                if current_width == 0 || current_height == 0 {
                    break;
                }

                let texture = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_2D, Some(texture));
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as i32,
                    current_width,
                    current_height,
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

            // Bind default FB
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            BlurPipeline {
                gl,
                downsample_program,
                upsample_program,
                composite_program,
                ping_pong_fbos,
                ping_pong_textures,
                passes,
                width,
                height,
            }
        }
    }

    pub unsafe fn compute_blur(
        &self,
        source_texture: glow::Texture,
        _x: i32,
        _y: i32,
        w: i32,
        h: i32,
    ) -> glow::Texture {
        // Bước 1: Trích xuất (Copy) vùng màn hình đằng sau cửa sổ vào FBO đầu tiên
        // Bước 2: Thực hiện Downsample tuần tự
        // Bước 3: Thực hiện Upsample ngược lại kèm theo blur offset
        
        let gl = &self.gl;

        // Downsample
        gl.use_program(Some(self.downsample_program));
        let halfpixel_loc = gl.get_uniform_location(self.downsample_program, "u_halfpixel");
        
        let mut current_tex = source_texture;
        let mut current_w = w;
        let mut current_h = h;

        for i in 0..self.ping_pong_fbos.len() {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ping_pong_fbos[i]));
            gl.viewport(0, 0, current_w / 2, current_h / 2);
            gl.bind_texture(glow::TEXTURE_2D, Some(current_tex));

            if let Some(loc) = halfpixel_loc {
                gl.uniform_2_f32(Some(&loc), 0.5 / (current_w as f32), 0.5 / (current_h as f32));
            }

            // Vẽ màn fullscreen quad lấp đầy viewport
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            current_tex = self.ping_pong_textures[i];
            current_w /= 2;
            current_h /= 2;
        }

        // Upsample
        gl.use_program(Some(self.upsample_program));
        let up_halfpixel_loc = gl.get_uniform_location(self.upsample_program, "u_halfpixel");
        let offset_loc = gl.get_uniform_location(self.upsample_program, "u_offset");

        if let Some(loc) = offset_loc {
            gl.uniform_1_f32(Some(&loc), 2.0); // Blur radius/offset
        }

        for i in (0..self.ping_pong_fbos.len() - 1).rev() {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.ping_pong_fbos[i]));
            gl.viewport(0, 0, current_w * 2, current_h * 2);
            gl.bind_texture(glow::TEXTURE_2D, Some(current_tex));

            if let Some(loc) = up_halfpixel_loc {
                gl.uniform_2_f32(Some(&loc), 0.5 / (current_w as f32), 0.5 / (current_h as f32));
            }

            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            current_tex = self.ping_pong_textures[i];
            current_w *= 2;
            current_h *= 2;
        }

        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        current_tex
    }
}

unsafe fn compile_shader_program(gl: &glow::Context, vert_src: &str, frag_src: &str) -> glow::Program {
    let program = gl.create_program().unwrap();

    let vert_shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
    gl.shader_source(vert_shader, vert_src);
    gl.compile_shader(vert_shader);
    if !gl.get_shader_compile_status(vert_shader) {
        panic!("Vertex shader compilation failed: {}", gl.get_shader_info_log(vert_shader));
    }

    let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
    gl.shader_source(frag_shader, frag_src);
    gl.compile_shader(frag_shader);
    if !gl.get_shader_compile_status(frag_shader) {
        panic!("Fragment shader compilation failed: {}", gl.get_shader_info_log(frag_shader));
    }

    gl.attach_shader(program, vert_shader);
    gl.attach_shader(program, frag_shader);
    gl.link_program(program);

    if !gl.get_program_link_status(program) {
        panic!("Program linking failed: {}", gl.get_program_info_log(program));
    }

    gl.delete_shader(vert_shader);
    gl.delete_shader(frag_shader);

    program
}
impl Drop for BlurPipeline {
    fn drop(&mut self) {
        unsafe {
            let gl = &self.gl;
            gl.delete_program(self.downsample_program);
            gl.delete_program(self.upsample_program);
            gl.delete_program(self.composite_program);
            for fbo in &self.ping_pong_fbos {
                gl.delete_framebuffer(*fbo);
            }
            for tex in &self.ping_pong_textures {
                gl.delete_texture(*tex);
            }
        }
    }
}
