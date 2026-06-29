use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

use crate::capture::{
    EditorState, Tool, Drawing,
    draw_pixelated_rect, trigger_save, trigger_copy
};

pub fn build_editor_ui(app: &gtk4::Application, temp_path: &str) -> gtk4::ApplicationWindow {
    let pixbuf = match gdk_pixbuf::Pixbuf::from_file(temp_path) {
        Ok(pb) => pb,
        Err(e) => {
            eprintln!("Failed to load temporary screenshot file: {}", e);
            return gtk4::ApplicationWindow::new(app);
        }
    };

    let state = Rc::new(RefCell::new(EditorState::new(pixbuf)));

    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    // Stretch across the entire screen
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.add_css_class("screenshot-window");

    let overlay = gtk4::Overlay::new();
    window.set_child(Some(&overlay));

    // Drawing Canvas
    let drawing_area = gtk4::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    overlay.set_child(Some(&drawing_area));

    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_, cr, width, height| {
        let s = state_draw.borrow();
        
        // 1. Draw Background Screenshot
        cr.set_source_pixbuf(&s.bg_pixbuf, 0.0, 0.0);
        cr.paint().unwrap();

        // 2. Draw Dark Overlay
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.45);
        
        if s.has_selection && s.crop_w > 5.0 && s.crop_h > 5.0 {
            let rx = s.crop_x;
            let ry = s.crop_y;
            let rw = s.crop_w;
            let rh = s.crop_h;
            
            // Clip out the selection area so it remains bright
            cr.save().unwrap();
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            cr.rectangle(rx, ry + rh, rw, -rh); // Hole
            cr.set_fill_rule(cairo::FillRule::EvenOdd);
            cr.fill().unwrap();
            cr.restore().unwrap();

            // Draw Selection Border
            cr.set_source_rgba(0.23, 0.51, 0.96, 0.85); // Blue
            cr.set_line_width(2.0);
            cr.rectangle(rx, ry, rw, rh);
            cr.stroke().unwrap();
        } else {
            cr.paint().unwrap();
        }

        // Clip drawings to the crop selection so they don't draw over the dark overlay
        let has_clip = s.has_selection && s.crop_w > 5.0 && s.crop_h > 5.0;
        if has_clip {
            cr.save().unwrap();
            cr.rectangle(s.crop_x, s.crop_y, s.crop_w, s.crop_h);
            cr.clip();
        }

        // 3. Draw All Completed Annotations
        for drawing in &s.drawings {
            match drawing {
                Drawing::Blur { x, y, w, h } => {
                    draw_pixelated_rect(cr, &s.bg_pixbuf, *x, *y, *w, *h);
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

        // 4. Draw Active/Temporary Drawing
        if let Some(points) = &s.active_stroke {
            if points.len() >= 2 {
                cr.set_source_rgb(s.current_color.0, s.current_color.1, s.current_color.2);
                cr.set_line_width(3.5);
                cr.set_line_cap(cairo::LineCap::Round);
                cr.set_line_join(cairo::LineJoin::Round);
                cr.move_to(points[0].0, points[0].1);
                for p in &points[1..] {
                    cr.line_to(p.0, p.1);
                }
                cr.stroke().unwrap();
            }
        }

        if let Some((x, y, w, h)) = s.active_rect {
            if s.current_tool == Tool::Rect {
                cr.set_source_rgb(s.current_color.0, s.current_color.1, s.current_color.2);
                cr.set_line_width(3.0);
                cr.rectangle(x, y, w, h);
                cr.stroke().unwrap();
            } else if s.current_tool == Tool::Blur {
                draw_pixelated_rect(cr, &s.bg_pixbuf, x, y, w, h);
            }
        }

        if has_clip {
            cr.restore().unwrap();
        }
    });

    // Floating macOS-style Glassmorphic Toolbar at the bottom-center
    let toolbar_wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    toolbar_wrapper.set_halign(gtk4::Align::Center);
    toolbar_wrapper.set_valign(gtk4::Align::End);
    toolbar_wrapper.set_margin_bottom(30);
    toolbar_wrapper.set_visible(false); // Hidden initially

    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    toolbar.add_css_class("screenshot-toolbar"); // Compact glassmorphic styling
    toolbar.set_margin_start(16);
    toolbar.set_margin_end(16);
    toolbar.set_margin_top(8);
    toolbar.set_margin_bottom(8);

    // Tool buttons
    let btn_reset = gtk4::Button::from_icon_name("view-refresh-symbolic");
    btn_reset.set_tooltip_text(Some("Bỏ chụp và làm lại (Xóa hết nét vẽ)"));
    btn_reset.add_css_class("screenshot-toolbar-btn");

    let btn_pen = gtk4::Button::from_icon_name("document-edit-symbolic");
    btn_pen.set_tooltip_text(Some("Bút vẽ"));
    btn_pen.add_css_class("screenshot-toolbar-btn");

    let btn_rect = gtk4::Button::from_icon_name("media-record-symbolic");
    btn_rect.set_tooltip_text(Some("Vẽ hình chữ nhật"));
    btn_rect.add_css_class("screenshot-toolbar-btn");

    let btn_blur = gtk4::Button::from_icon_name("view-grid-symbolic");
    btn_blur.set_tooltip_text(Some("Làm mờ thông tin"));
    btn_blur.add_css_class("screenshot-toolbar-btn");

    let btn_eraser = gtk4::Button::from_icon_name("edit-clear-all-symbolic");
    btn_eraser.set_tooltip_text(Some("Xóa hình vẽ"));
    btn_eraser.add_css_class("screenshot-toolbar-btn");

    // Color selection buttons
    let color_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    color_box.set_valign(gtk4::Align::Center);
    
    let colors = vec![
        ("red", (0.93, 0.15, 0.15)),
        ("green", (0.06, 0.63, 0.31)),
        ("blue", (0.15, 0.45, 0.93)),
        ("yellow", (0.93, 0.70, 0.15)),
    ];

    let active_color_btn: Rc<RefCell<Option<gtk4::Button>>> = Rc::new(RefCell::new(None));

    for (name, rgb) in colors {
        let color_btn = gtk4::Button::new();
        color_btn.add_css_class("color-dot-btn");
        color_btn.add_css_class(&format!("color-dot-{}", name));
        color_btn.set_size_request(16, 16);

        let state_color = state.clone();
        let color_btn_clone = color_btn.clone();
        let active_color_clone = active_color_btn.clone();
        color_btn.connect_clicked(move |_| {
            state_color.borrow_mut().current_color = rgb;
            
            // Visual indicator for active color
            if let Some(prev) = active_color_clone.borrow_mut().take() {
                prev.remove_css_class("color-active");
            }
            color_btn_clone.add_css_class("color-active");
            *active_color_clone.borrow_mut() = Some(color_btn_clone.clone());
        });
        
        if name == "red" {
            color_btn.add_css_class("color-active");
            *active_color_btn.borrow_mut() = Some(color_btn.clone());
        }

        color_box.append(&color_btn);
    }

    // Reset button click event
    let state_reset = state.clone();
    let toolbar_wrapper_reset = toolbar_wrapper.clone();
    let canvas_reset = drawing_area.clone();
    btn_reset.connect_clicked(move |_| {
        let mut s = state_reset.borrow_mut();
        s.has_selection = false;
        s.crop_x = 0.0;
        s.crop_y = 0.0;
        s.crop_w = 0.0;
        s.crop_h = 0.0;
        s.drawings.clear();
        s.active_stroke = None;
        s.active_rect = None;
        s.current_tool = Tool::Select;
        toolbar_wrapper_reset.set_visible(false);
        canvas_reset.queue_draw();
    });

    // Tool buttons click events
    let tools = vec![
        (btn_pen.clone(), Tool::Pen),
        (btn_rect.clone(), Tool::Rect),
        (btn_blur.clone(), Tool::Blur),
        (btn_eraser.clone(), Tool::Eraser),
    ];

    let tools_list = Rc::new(tools.clone());

    for (btn, tool) in tools {
        let state_tool = state.clone();
        let btn_clone = btn.clone();
        let tools_clone = tools_list.clone();
        btn.connect_clicked(move |_| {
            state_tool.borrow_mut().current_tool = tool;
            
            // Update selected button style
            for (t_btn, _) in tools_clone.iter() {
                t_btn.remove_css_class("selected");
            }
            btn_clone.add_css_class("selected");
        });
    }

    // Action buttons
    let btn_copy = gtk4::Button::from_icon_name("edit-copy-symbolic");
    btn_copy.set_tooltip_text(Some("Sao chép vào Clipboard (Enter)"));
    btn_copy.add_css_class("screenshot-toolbar-btn");
    
    let state_copy = state.clone();
    let win_copy = window.clone();
    btn_copy.connect_clicked(move |_| {
        if trigger_copy(&state_copy.borrow(), &win_copy) {
            win_copy.close();
        }
    });

    let btn_save = gtk4::Button::from_icon_name("document-save-symbolic");
    btn_save.set_tooltip_text(Some("Lưu ảnh chụp (Ctrl+S)"));
    btn_save.add_css_class("screenshot-toolbar-btn");
    
    let state_save = state.clone();
    let win_save = window.clone();
    btn_save.connect_clicked(move |_| {
        if trigger_save(&state_save.borrow()) {
            win_save.close();
        }
    });

    let btn_cancel = gtk4::Button::from_icon_name("window-close-symbolic");
    btn_cancel.set_tooltip_text(Some("Hủy (Escape)"));
    btn_cancel.add_css_class("screenshot-toolbar-btn");
    
    let win_cancel = window.clone();
    btn_cancel.connect_clicked(move |_| {
        win_cancel.close();
    });

    // Assemble toolbar
    toolbar.append(&btn_reset);
    
    let sep0 = gtk4::Label::new(Some("│"));
    sep0.add_css_class("capsule-separator");
    toolbar.append(&sep0);
    
    toolbar.append(&btn_pen);
    toolbar.append(&btn_rect);
    toolbar.append(&btn_blur);
    toolbar.append(&btn_eraser);
    
    let sep1 = gtk4::Label::new(Some("│"));
    sep1.add_css_class("capsule-separator");
    toolbar.append(&sep1);
    toolbar.append(&color_box);

    let sep2 = gtk4::Label::new(Some("│"));
    sep2.add_css_class("capsule-separator");
    toolbar.append(&sep2);

    toolbar.append(&btn_copy);
    toolbar.append(&btn_save);
    toolbar.append(&btn_cancel);

    toolbar_wrapper.append(&toolbar);
    overlay.add_overlay(&toolbar_wrapper);

    // Mouse gestures for selection & drawing
    let drag_gesture = gtk4::GestureDrag::new();
    let state_mouse = state.clone();
    let canvas_mouse = drawing_area.clone();
    
    let toolbar_wrapper_begin = toolbar_wrapper.clone();
    drag_gesture.connect_drag_begin(move |_, start_x, start_y| {
        let mut s_mut = state_mouse.borrow_mut();
        let s = &mut *s_mut;
        
        // If there's no selection yet, force the tool to be Select
        if !s.has_selection {
            s.current_tool = Tool::Select;
        }
        
        // Prevent drawing from starting outside the crop box
        if s.has_selection && s.current_tool != Tool::Select {
            let inside_crop = start_x >= s.crop_x 
                && start_x <= s.crop_x + s.crop_w 
                && start_y >= s.crop_y 
                && start_y <= s.crop_y + s.crop_h;
            if !inside_crop {
                return; // Ignore drawing start outside crop box
            }
        }

        // Set the start coordinates for the active drag gesture
        s.drag_start_x = start_x;
        s.drag_start_y = start_y;
        
        match s.current_tool {
            Tool::Select => {
                s.is_selecting = true;
                s.has_selection = true;
                s.crop_x = start_x;
                s.crop_y = start_y;
                s.crop_w = 0.0;
                s.crop_h = 0.0;
                toolbar_wrapper_begin.set_visible(false); // Hide toolbar while selecting/re-selecting
            }
            Tool::Pen => {
                s.active_stroke = Some(vec![(start_x, start_y)]);
            }
            Tool::Rect | Tool::Blur => {
                s.active_rect = Some((start_x, start_y, 0.0, 0.0));
            }
            Tool::Eraser => {
                // Find and remove the drawing under click
                let click_p = (start_x, start_y);
                s.drawings.retain(|d| {
                    match d {
                        Drawing::Stroke { points, .. } => {
                            // If click is close to any point in the stroke
                            !points.iter().any(|p| ((p.0 - click_p.0).powi(2) + (p.1 - click_p.1).powi(2)).sqrt() < 10.0)
                        }
                        Drawing::Rect { x, y, w, h, .. } | Drawing::Blur { x, y, w, h } => {
                            // If click is inside the rect
                            !(click_p.0 >= *x && click_p.0 <= x + w && click_p.1 >= *y && click_p.1 <= y + h)
                        }
                    }
                });
            }
        }
        canvas_mouse.queue_draw();
    });

    let state_mouse_update = state.clone();
    let canvas_mouse_update = drawing_area.clone();
    drag_gesture.connect_drag_update(move |_, offset_x, offset_y| {
        let mut s_mut = state_mouse_update.borrow_mut();
        let s = &mut *s_mut;
        match s.current_tool {
            Tool::Select => {
                if s.is_selecting {
                    let rx = s.drag_start_x.min(s.drag_start_x + offset_x);
                    let ry = s.drag_start_y.min(s.drag_start_y + offset_y);
                    let rw = offset_x.abs();
                    let rh = offset_y.abs();
                    s.crop_x = rx;
                    s.crop_y = ry;
                    s.crop_w = rw;
                    s.crop_h = rh;
                }
            }
            Tool::Pen => {
                let start_x = s.drag_start_x;
                let start_y = s.drag_start_y;
                if let Some(points) = &mut s.active_stroke {
                    let last = points.last().copied().unwrap_or((0.0, 0.0));
                    let next = (start_x + offset_x, start_y + offset_y);
                    // Only add point if moved enough to avoid overhead
                    if ((last.0 - next.0).powi(2) + (last.1 - next.1).powi(2)).sqrt() > 2.0 {
                        points.push(next);
                    }
                }
            }
            Tool::Rect | Tool::Blur => {
                let rx = s.drag_start_x.min(s.drag_start_x + offset_x);
                let ry = s.drag_start_y.min(s.drag_start_y + offset_y);
                let rw = offset_x.abs();
                let rh = offset_y.abs();
                s.active_rect = Some((rx, ry, rw, rh));
            }
            _ => {}
        }
        canvas_mouse_update.queue_draw();
    });

    let state_mouse_end = state.clone();
    let toolbar_wrapper_end = toolbar_wrapper.clone();
    let canvas_mouse_end = drawing_area.clone();
    let btn_pen_end = btn_pen.clone();
    drag_gesture.connect_drag_end(move |_, _, _| {
        let mut s_mut = state_mouse_end.borrow_mut();
        let s = &mut *s_mut;
        match s.current_tool {
            Tool::Select => {
                s.is_selecting = false;
                // Validate selection: if too small, discard it
                if s.crop_w > 5.0 && s.crop_h > 5.0 {
                    // Automatically switch tool to Pen so the user can draw immediately
                    s.current_tool = Tool::Pen;
                    btn_pen_end.add_css_class("selected");
                } else {
                    s.has_selection = false;
                    s.crop_x = 0.0;
                    s.crop_y = 0.0;
                    s.crop_w = 0.0;
                    s.crop_h = 0.0;
                }
            }
            Tool::Pen => {
                let color = s.current_color;
                if let Some(points) = s.active_stroke.take() {
                    if points.len() >= 2 {
                        s.drawings.push(Drawing::Stroke {
                            points,
                            color,
                            width: 3.5,
                        });
                    }
                }
            }
            Tool::Rect => {
                let color = s.current_color;
                if let Some((x, y, w, h)) = s.active_rect.take() {
                    if w > 5.0 && h > 5.0 {
                        s.drawings.push(Drawing::Rect {
                            x,
                            y,
                            w,
                            h,
                            color,
                            width: 3.0,
                        });
                    }
                }
            }
            Tool::Blur => {
                if let Some((x, y, w, h)) = s.active_rect.take() {
                    if w > 5.0 && h > 5.0 {
                        s.drawings.push(Drawing::Blur { x, y, w, h });
                    }
                }
            }
            _ => {}
        }
        
        // Show toolbar if we have a valid selection, otherwise hide it
        if s.has_selection {
            toolbar_wrapper_end.set_visible(true);
        } else {
            toolbar_wrapper_end.set_visible(false);
        }
        
        canvas_mouse_end.queue_draw();
    });

    drawing_area.add_controller(drag_gesture);

    // Keyboard event controller for shortcuts
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    
    let state_key = state.clone();
    let win_key = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, state_flags| {
        match key {
            gtk4::gdk::Key::Escape => {
                win_key.close();
                gtk4::glib::Propagation::Stop
            }
            gtk4::gdk::Key::Return => {
                if trigger_copy(&state_key.borrow(), &win_key) {
                    win_key.close();
                }
                gtk4::glib::Propagation::Stop
            }
            gtk4::gdk::Key::s | gtk4::gdk::Key::S => {
                // Check if Ctrl is pressed
                if state_flags.contains(gtk4::gdk::ModifierType::CONTROL_MASK) {
                    if trigger_save(&state_key.borrow()) {
                        win_key.close();
                    }
                    gtk4::glib::Propagation::Stop
                } else {
                    gtk4::glib::Propagation::Proceed
                }
            }
            _ => gtk4::glib::Propagation::Proceed,
        }
    });

    window.add_controller(key_controller);
    
    window
}
