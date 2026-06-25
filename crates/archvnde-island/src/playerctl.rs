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

use std::cell::RefCell;
use std::collections::HashSet;

pub fn load_album_art(art_url: &str, size: i32) -> Option<gtk4::Image> {
    if art_url.is_empty() {
        return None;
    }

    let local_path = if let Some(path_str) = art_url.strip_prefix("file://") {
        decode_uri(path_str)
    } else if art_url.starts_with("http://") || art_url.starts_with("https://") {
        let sanitized: String = art_url.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let cache_path = format!("/tmp/archvnde_art_cache/{}.png", sanitized);
        
        if std::path::Path::new(&cache_path).exists() {
            cache_path
        } else {
            thread_local! {
                static ACTIVE_DOWNLOADS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
            }
            ACTIVE_DOWNLOADS.with(|downloads| {
                let mut d = downloads.borrow_mut();
                if !d.contains(art_url) {
                    d.insert(art_url.to_string());
                    let _ = std::fs::create_dir_all("/tmp/archvnde_art_cache");
                    let url_clone = art_url.to_string();
                    let cache_path_clone = cache_path.clone();
                    std::thread::spawn(move || {
                        let status = std::process::Command::new("curl")
                            .args(&["-s", "-L", "-o", &cache_path_clone, &url_clone])
                            .status();
                        if let Ok(stat) = status {
                            if !stat.success() {
                                let _ = std::fs::File::create(&cache_path_clone);
                            }
                        } else {
                            let _ = std::fs::File::create(&cache_path_clone);
                        }
                    });
                }
            });
            return None;
        }
    } else if art_url.starts_with('/') {
        art_url.to_string()
    } else {
        return None;
    };

    let pb = gdk_pixbuf::Pixbuf::from_file_at_scale(
        &local_path,
        size,
        size,
        true,
    ).ok()?;
    
    let texture = gdk4::Texture::for_pixbuf(&pb);
    Some(gtk4::Image::from_paintable(Some(&texture)))
}
