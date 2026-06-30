use gtk4::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Clone)]
pub enum Drawing {
    Stroke {
        points: Vec<(f64, f64)>,
        color: (f64, f64, f64),
        width: f64,
    },
    Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        color: (f64, f64, f64),
        width: f64,
    },
    Blur {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Select,
    Pen,
    Rect,
    Blur,
    Eraser,
}

pub struct EditorState {
    pub bg_pixbuf: gdk_pixbuf::Pixbuf,
    
    // Crop selection coordinates (remains fixed after crop is done)
    pub crop_x: f64,
    pub crop_y: f64,
    pub crop_w: f64,
    pub crop_h: f64,
    pub has_selection: bool,
    
    // Current drag coordinates (for the active gesture)
    pub drag_start_x: f64,
    pub drag_start_y: f64,
    pub is_selecting: bool, // true when dragging to crop
    
    // Active drawing states
    pub current_tool: Tool,
    pub current_color: (f64, f64, f64), // RGB
    pub drawings: Vec<Drawing>,
    pub active_stroke: Option<Vec<(f64, f64)>>,
    pub active_rect: Option<(f64, f64, f64, f64)>,
}

impl EditorState {
    pub fn new(pixbuf: gdk_pixbuf::Pixbuf) -> Self {
        Self {
            bg_pixbuf: pixbuf,
            crop_x: 0.0,
            crop_y: 0.0,
            crop_w: 0.0,
            crop_h: 0.0,
            has_selection: false,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            is_selecting: false,
            current_tool: Tool::Select,
            current_color: (0.93, 0.15, 0.15), // Red by default
            drawings: Vec::new(),
            active_stroke: None,
            active_rect: None,
        }
    }
}

pub fn draw_pixelated_rect(cr: &cairo::Context, bg_pixbuf: &gdk_pixbuf::Pixbuf, x: f64, y: f64, w: f64, h: f64) {
    if w <= 5.0 || h <= 5.0 {
        return;
    }
    
    cr.save().unwrap();
    cr.rectangle(x, y, w, h);
    cr.clip();
    
    // Downscale and upscale to create a pixelated mosaic effect
    let scale = 0.08; // 8% of original size
    let sw = (w * scale).max(2.0) as i32;
    let sh = (h * scale).max(2.0) as i32;
    
    let sub_pb = bg_pixbuf.new_subpixbuf(x as i32, y as i32, w as i32, h as i32);
    if let Some(scaled_pb) = sub_pb.scale_simple(sw, sh, gdk_pixbuf::InterpType::Hyper) {
        cr.scale(1.0 / scale, 1.0 / scale);
        cr.set_source_pixbuf(&scaled_pb, x * scale, y * scale);
        cr.source().set_filter(cairo::Filter::Nearest);
        cr.paint().unwrap();
    }
    
    cr.restore().unwrap();
}

pub fn get_screenshot_save_path() -> PathBuf {
    let pictures_dir = dirs::picture_dir().unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join("Pictures")
    });
    let screenshots_dir = pictures_dir.join("Screenshots");
    let _ = std::fs::create_dir_all(&screenshots_dir);
    
    // Generate YYYY-MM-DD_HH-MM-SS format
    let datetime = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    screenshots_dir.join(format!("Screenshot_{}.png", datetime))
}

pub fn capture_screen_to_temp() -> Option<String> {
    let temp_path = "/tmp/archvnde-screenshot-temp.png";
    let _ = std::fs::remove_file(temp_path);

    // Capture all outputs. The editor will crop to the active monitor
    // using GTK4's native GDK monitor API after the window is realized.
    let status = std::process::Command::new("grim")
        .arg(temp_path)
        .status();

    match status {
        Ok(s) if s.success() => Some(temp_path.to_string()),
        _ => {
            eprintln!("Failed to capture screen using 'grim'. Please make sure it is installed.");
            None
        }
    }
}

pub fn save_cropped_surface(state: &EditorState) -> Option<cairo::ImageSurface> {
    if !state.has_selection || state.crop_w <= 5.0 || state.crop_h <= 5.0 {
        return None;
    }
    let rx = state.crop_x;
    let ry = state.crop_y;
    let rw = state.crop_w;
    let rh = state.crop_h;
    
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, rw as i32, rh as i32).ok()?;
    let cr = cairo::Context::new(&surface).ok()?;

    // Translate context to capture the cropped area
    cr.translate(-rx, -ry);

    // 1. Draw Background Image
    cr.set_source_pixbuf(&state.bg_pixbuf, 0.0, 0.0);
    cr.paint().unwrap();

    // 2. Draw Annotations
    for drawing in &state.drawings {
        match drawing {
            Drawing::Blur { x, y, w, h } => {
                draw_pixelated_rect(&cr, &state.bg_pixbuf, *x, *y, *w, *h);
            }
            Drawing::Stroke { points, color, width } => {
                if points.len() < 2 { continue; }
                cr.set_source_rgb(color.0, color.1, color.2);
                cr.set_line_width(*width);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                cr.move_to(points[0].0, points[0].1);
                for p in &points[1..] {
                    cr.line_to(p.0, p.1);
                }
                cr.stroke().unwrap();
            }
            Drawing::Rect { x, y, w, h, color, width } => {
                cr.set_source_rgb(color.0, color.1, color.2);
                cr.set_line_width(*width);
                cr.rectangle(*x, *y, *w, *h);
                cr.stroke().unwrap();
            }
        }
    }

    Some(surface)
}

pub fn trigger_save(state: &EditorState) -> bool {
    if let Some(surface) = save_cropped_surface(state) {
        let save_path = get_screenshot_save_path();
        if let Ok(mut file) = std::fs::File::create(&save_path) {
            if surface.write_to_png(&mut file).is_ok() {
                println!("Screenshot saved to: {:?}", save_path);
                // Trigger desktop notification
                let _ = std::process::Command::new("notify-send")
                    .args(&["-i", "image-x-generic", "Đã chụp ảnh màn hình", &format!("Đã lưu tại {:?}", save_path)])
                    .spawn();
                return true;
            }
        }
    }
    false
}

pub fn trigger_copy(state: &EditorState, window: &gtk4::ApplicationWindow) -> bool {
    if let Some(surface) = save_cropped_surface(state) {
        let temp_copy_path = "/tmp/archvnde-screenshot-copy.png";
        
        // Write the cropped surface to a temp PNG file
        if let Ok(mut file) = std::fs::File::create(temp_copy_path) {
            if surface.write_to_png(&mut file).is_ok() {
                // Pipe the file to wl-copy (standard Wayland clipboard tool)
                if let Ok(file_in) = std::fs::File::open(temp_copy_path) {
                    let status = std::process::Command::new("wl-copy")
                        .args(&["-t", "image/png"])
                        .stdin(file_in)
                        .status();
                    
                    if let Ok(s) = status {
                        if s.success() {
                            println!("Screenshot copied to clipboard via wl-copy.");
                            let _ = std::process::Command::new("notify-send")
                                .args(&["-i", "edit-paste", "Đã sao chép ảnh", "Ảnh chụp đã được lưu vào clipboard."])
                                .spawn();
                            return true;
                        }
                    }
                }
            }
        }

        // Fallback to GTK4 clipboard if wl-copy is not available
        let w = surface.width();
        let h = surface.height();
        let stride = surface.stride();
        if let Ok(data) = surface.data() {
            let pixbuf = gdk_pixbuf::Pixbuf::from_mut_slice(
                data.to_vec(),
                gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                w,
                h,
                stride,
            );

            let texture = gtk4::gdk::Texture::for_pixbuf(&pixbuf);
            let clipboard = window.upcast_ref::<gtk4::Widget>().display().clipboard();
            clipboard.set_texture(&texture);

            println!("Screenshot copied to clipboard via GTK fallback.");
            let _ = std::process::Command::new("notify-send")
                .args(&["-i", "edit-paste", "Đã sao chép ảnh", "Ảnh chụp đã được lưu vào clipboard."])
                .spawn();
            return true;
        }
    }
    false
}
