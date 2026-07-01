use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::Cell;
use std::rc::Rc;
use std::process::Command;

pub fn build_menu_ui(app: &gtk4::Application) -> gtk4::ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    // Exclusive keyboard mode so Escape can close it instantly
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    // Anchor to all edges to make the window fullscreen and cover the screen
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);

    // Add a class name for custom CSS styling (transparent background)
    window.add_css_class("menu-fullscreen");

    let fixed_layout = gtk4::Fixed::new();
    window.set_child(Some(&fixed_layout));

    // Create the container for the context menu
    let menu_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    menu_box.add_css_class("menu-box");
    menu_box.set_width_request(220);

    // We will position menu_box on the first enter or motion event
    let is_positioned = Rc::new(Cell::new(false));

    // Event handler for motion/enter to place the menu under the cursor
    let motion_controller = gtk4::EventControllerMotion::new();
    let menu_box_clone = menu_box.clone();
    let fixed_layout_clone = fixed_layout.clone();
    let window_clone = window.clone();
    let is_positioned_clone = is_positioned.clone();

    let position_menu = move |x: f64, y: f64| {
        if is_positioned_clone.get() {
            return;
        }
        is_positioned_clone.set(true);

        // Get window size
        let mut win_width = window_clone.width() as f64;
        let mut win_height = window_clone.height() as f64;

        // Fallback to monitor geometry if window has not been allocated size yet (<= 1.0)
        if win_width <= 1.0 || win_height <= 1.0 {
            if let Some(display) = gtk4::gdk::Display::default() {
                if let Some(monitor) = display.monitor_at_point(x as i32, y as i32) {
                    let geometry = monitor.geometry();
                    win_width = geometry.width() as f64;
                    win_height = geometry.height() as f64;
                }
            }
        }

        // Estimated menu dimensions
        let menu_w = 220.0;
        let menu_h = 240.0; // Estimate height based on number of items

        // Adjust coordinates to keep the menu on screen
        let mut pos_x = x;
        let mut pos_y = y;

        if pos_x + menu_w > win_width {
            pos_x = (win_width - menu_w - 10.0).max(0.0);
        }
        if pos_y + menu_h > win_height {
            pos_y = (win_height - menu_h - 10.0).max(0.0);
        }

        fixed_layout_clone.put(&menu_box_clone, pos_x, pos_y);
        menu_box_clone.set_opacity(1.0);

        // Apply slide-in animation from top/down for polished entrance
        archvnde_common::animation::slide_in(
            menu_box_clone.upcast_ref(),
            archvnde_common::animation::SlideDirection::Down,
            10,
            180,
        );
    };

    let pos_menu_enter = position_menu.clone();
    motion_controller.connect_enter(move |_, x, y| {
        pos_menu_enter(x, y);
    });

    let pos_menu_motion = position_menu.clone();
    motion_controller.connect_motion(move |_, x, y| {
        pos_menu_motion(x, y);
    });

    window.add_controller(motion_controller);

    // Click gesture to detect clicks outside the menu box
    let click_gesture = gtk4::GestureClick::new();
    let menu_box_clone2 = menu_box.clone();
    let window_clone2 = window.clone();
    click_gesture.connect_pressed(move |_, _, x, y| {
        let picked = window_clone2.pick(x, y, gtk4::PickFlags::DEFAULT);
        let inside_menu = picked
            .map(|w| w.is_ancestor(&menu_box_clone2) || w == menu_box_clone2)
            .unwrap_or(false);

        if !inside_menu {
            window_clone2.close();
        }
    });
    window.add_controller(click_gesture);

    // Close on Escape key press
    let key_controller = gtk4::EventControllerKey::new();
    let window_clone3 = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            window_clone3.close();
            gtk4::glib::Propagation::Proceed
        } else {
            gtk4::glib::Propagation::Stop
        }
    });
    window.add_controller(key_controller);

    // Helper to add menu items
    let add_menu_item = {
        let menu_box = menu_box.clone();
        let window = window.clone();
        move |label_text: &str, icon_name: &str, action: Box<dyn Fn() + 'static>| {
            let btn = gtk4::Button::new();
            btn.add_css_class("menu-item");

            let item_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
            item_layout.set_halign(gtk4::Align::Start);
            item_layout.set_valign(gtk4::Align::Center);

            let icon = archvnde_common::icon::get_icon_colored(icon_name, 16, "#ffffff");
            let label = gtk4::Label::new(Some(label_text));
            label.set_halign(gtk4::Align::Start);

            item_layout.append(&icon);
            item_layout.append(&label);
            btn.set_child(Some(&item_layout));

            let win = window.clone();
            btn.connect_clicked(move |_| {
                action();
                win.close();
            });

            menu_box.append(&btn);
        }
    };

    // Populate context menu items
    add_menu_item(
        "Terminal",
        "terminal",
        Box::new(|| {
            let _ = Command::new("foot").spawn().or_else(|_| Command::new("alacritty").spawn());
        }),
    );

    add_menu_item(
        "File Manager",
        "folder",
        Box::new(|| {
            let _ = Command::new("pcmanfm").spawn().or_else(|_| Command::new("thunar").spawn());
        }),
    );

    // Select Wallpaper action with FileDialog
    let window_file_dialog = window.clone();
    add_menu_item(
        "Change Wallpaper",
        "display",
        Box::new(move || {
            let dialog = gtk4::FileDialog::new();
            dialog.set_title("Select Wallpaper Image");
            
            let filter = gtk4::FileFilter::new();
            filter.set_name(Some("Images"));
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/jpeg");
            dialog.set_default_filter(Some(&filter));

            let win = window_file_dialog.clone();
            dialog.open(Some(&win), None::<&gio::Cancellable>, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        println!("Setting wallpaper to: {:?}", path);
                        if let Err(e) = archvnde_wallpaper::set_wallpaper(&path) {
                            eprintln!("Error setting wallpaper: {}", e);
                        }
                    }
                }
            });
        }),
    );

    add_menu_item(
        "Reconfigure Shell",
        "restart",
        Box::new(|| {
            let _ = Command::new("labwc").arg("--reconfigure").spawn();
        }),
    );

    // Separator
    let sep = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    sep.add_css_class("menu-item-separator");
    menu_box.append(&sep);

    add_menu_item(
        "Exit Shell",
        "logout",
        Box::new(|| {
            let _ = Command::new("labwc").arg("--exit").spawn();
        }),
    );

    // Start off-screen or invisible, it will be positioned and animated on enter/motion
    menu_box.set_opacity(0.0);

    window
}
