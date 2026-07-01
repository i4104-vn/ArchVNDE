use gtk4::prelude::*;

pub fn run_playerctl(args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("playerctl")
        .args(args)
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !stdout.is_empty() {
            return Some(stdout);
        }
    }
    None
}

fn decode_uri(uri: &str) -> String {
    let mut decoded = String::new();
    let mut chars = uri.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                if let Some(hex) = u8::from_str_radix(&format!("{}{}", h1, h2), 16).ok() {
                    decoded.push(hex as char);
                    continue;
                }
            }
        }
        decoded.push(c);
    }
    decoded
}

pub fn load_album_art(art_url: &str, size: i32) -> Option<gtk4::Image> {
    if art_url.is_empty() {
        return None;
    }

    let local_path = if let Some(path_str) = art_url.strip_prefix("file://") {
        decode_uri(path_str)
    } else if art_url.starts_with('/') {
        art_url.to_string()
    } else {
        return None;
    };

    let pb = gdk_pixbuf::Pixbuf::from_file_at_scale(
        &local_path,
        size,
        size,
        false,
    ).ok()?;
    
    let texture = gdk4::Texture::for_pixbuf(&pb);
    let img = gtk4::Image::from_paintable(Some(&texture));
    img.set_pixel_size(size);
    Some(img)
}
