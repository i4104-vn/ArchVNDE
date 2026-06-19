pub const VERTEX_SHADER: &str = r#"#version 300 es
in vec2 position;
in vec2 texcoord;
out vec2 v_texcoord;

void main() {
    v_texcoord = texcoord;
    gl_Position = vec4(position, 0.0, 1.0);
}
"#;

pub const DOWNSAMPLE_FRAG_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform vec2 u_halfpixel;

void main() {
    vec4 sum = texture(u_texture, v_texcoord) * 4.0;
    sum += texture(u_texture, v_texcoord - u_halfpixel.xy);
    sum += texture(u_texture, v_texcoord + u_halfpixel.xy);
    sum += texture(u_texture, v_texcoord + vec2(u_halfpixel.x, -u_halfpixel.y));
    sum += texture(u_texture, v_texcoord + vec2(-u_halfpixel.x, u_halfpixel.y));
    fragColor = sum / 8.0;
}
"#;

pub const UPSAMPLE_FRAG_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform vec2 u_halfpixel;
uniform float u_offset;

void main() {
    vec2 offset = u_halfpixel * u_offset;
    
    vec4 sum = vec4(0.0);
    sum += texture(u_texture, v_texcoord + vec2(-offset.x * 2.0, 0.0));
    sum += texture(u_texture, v_texcoord + vec2(-offset.x, offset.y)) * 2.0;
    sum += texture(u_texture, v_texcoord + vec2(0.0, offset.y * 2.0));
    sum += texture(u_texture, v_texcoord + vec2(offset.x, offset.y)) * 2.0;
    sum += texture(u_texture, v_texcoord + vec2(offset.x * 2.0, 0.0));
    sum += texture(u_texture, v_texcoord + vec2(offset.x, -offset.y)) * 2.0;
    sum += texture(u_texture, v_texcoord + vec2(0.0, -offset.y * 2.0));
    sum += texture(u_texture, v_texcoord + vec2(-offset.x, -offset.y)) * 2.0;
    
    fragColor = sum / 12.0;
}
"#;

pub const GLASS_COMPOSITION_FRAG_SHADER: &str = r#"#version 300 es
precision mediump float;
in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_blurred_background;
uniform sampler2D u_window_content;
uniform vec4 u_tint_color;
uniform vec2 u_resolution;

bool is_border(vec2 coord, vec2 res) {
    float border_width = 1.0;
    return coord.x < border_width || coord.x > (res.x - border_width) ||
           coord.y < border_width || coord.y > (res.y - border_width);
}

void main() {
    vec4 blurred = texture(u_blurred_background, v_texcoord);
    vec4 client_content = texture(u_window_content, v_texcoord);
    
    vec4 glass_base = mix(blurred, u_tint_color, u_tint_color.a);
    
    if (is_border(gl_FragCoord.xy, u_resolution)) {
        vec4 border_color = vec4(1.0, 1.0, 1.0, 0.08); // 8% white border highlight
        glass_base = mix(glass_base, border_color, border_color.a);
    }
    
    fragColor = mix(glass_base, client_content, client_content.a);
}
"#;
