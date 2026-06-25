mod widgets;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;
use widgets::panel::create_status_indicators;
use widgets::workspace::create_workspace_switcher;

fn main() {
    println!("Starting ArchVNDE Panel...");

    let application = gtk4::Application::new(
        Some("org.archvnde.panel"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);

        // Define shared window states for mutual exclusivity
        let quick_settings_window = Rc::new(RefCell::new(None));
        let calendar_window = Rc::new(RefCell::new(None));

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Assign to the Top layer so it renders above normal windows
        window.set_layer(Layer::Top);

        // Set exclusive zone so other maximized windows don't overlap it
        window.set_exclusive_zone(36);

        // Anchor it to the top, left, and right edges of the screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Set default height of the panel
        window.set_default_size(0, 36);

        // Add styling class
        window.add_css_class("panel-window");

        // Layout container
        let box_layout = gtk4::CenterBox::new();
        box_layout.add_css_class("panel-box");

        // 1. Logo Button (launches launcher)
        let logo_btn = gtk4::Button::new();
        logo_btn.add_css_class("panel-logo-btn");
        let logo_icon = archvnde_common::icon::get_icon("logo", 16);
        logo_btn.set_child(Some(&logo_icon));
        logo_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("archvnde-launcher").spawn();
        });

        // 2. Workspace Switcher
        let workspace_box = create_workspace_switcher();

        // Create a separator to visual separate logo and dots inside the same capsule
        let separator = gtk4::Label::new(Some("│"));
        separator.add_css_class("capsule-separator");

        workspace_box.prepend(&separator);
        workspace_box.prepend(&logo_btn);

        // 3. Unified Status and Clock Capsule
        let status_indicators = create_status_indicators(
            app,
            quick_settings_window.clone(),
            calendar_window.clone(),
        );

        // Left-aligned section: Workspaces capsule (now containing logo + separator + dots)
        let left_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        left_box.set_hexpand(true);
        left_box.set_halign(gtk4::Align::Start);
        left_box.set_valign(gtk4::Align::Center);
        left_box.append(&workspace_box);
        // Center-aligned section: Clean placeholder center space with interactive notch
        let center_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        center_box.set_hexpand(true);
        center_box.set_halign(gtk4::Align::Center);
        center_box.set_valign(gtk4::Align::Start);

        // Notch Capsule (Apple macOS style dropdown notch)
        let notch_capsule = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        notch_capsule.add_css_class("panel-notch");
        notch_capsule.set_valign(gtk4::Align::Start);
        notch_capsule.set_halign(gtk4::Align::Center);

        // Notch content box (so we can transition opacity of contents)
        let notch_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        notch_content.add_css_class("notch-content");
        notch_content.set_valign(gtk4::Align::Center);
        notch_content.set_halign(gtk4::Align::Center);

        // --- 1. Music Player section ---
        let player_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        player_box.set_valign(gtk4::Align::Center);
        
        let play_btn = gtk4::Button::new();
        play_btn.add_css_class("notch-btn");
        let play_icon = archvnde_common::icon::get_icon_colored("music", 12, "#3b82f6");
        play_btn.set_child(Some(&play_icon));
        
        let is_playing = Rc::new(std::cell::Cell::new(true));
        let is_playing_clone = is_playing.clone();
        let play_btn_clone = play_btn.clone();
        play_btn.connect_clicked(move |_| {
            let playing = !is_playing_clone.get();
            is_playing_clone.set(playing);
            if playing {
                play_btn_clone.set_child(Some(&archvnde_common::icon::get_icon_colored("music", 12, "#3b82f6")));
            } else {
                play_btn_clone.set_child(Some(&archvnde_common::icon::get_icon_colored("music", 12, "#94a3b8")));
            }
        });

        let track_label = gtk4::Label::new(Some("Track 01"));
        track_label.add_css_class("notch-player-text");

        player_box.append(&play_btn);
        player_box.append(&track_label);

        // --- 2. Separator line ---
        let notch_sep = gtk4::Label::new(Some("│"));
        notch_sep.add_css_class("notch-separator");

        // --- 3. File Shelf / Drop zone ---
        let shelf_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        shelf_box.set_valign(gtk4::Align::Center);

        let drop_zone = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        drop_zone.add_css_class("shelf-dropzone");
        let drop_icon = archvnde_common::icon::get_icon("folder", 12);
        let drop_label = gtk4::Label::new(Some("Shelf"));
        drop_label.add_css_class("shelf-dropzone-text");
        drop_zone.append(&drop_icon);
        drop_zone.append(&drop_label);
        
        shelf_box.append(&drop_zone);

        let drop_target = gtk4::DropTarget::new(
            gtk4::gdk::FileList::static_type(),
            gtk4::gdk::DragAction::COPY,
        );

        let shelf_box_clone = shelf_box.clone();
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(file_list) = value.get::<gtk4::gdk::FileList>() {
                for file in file_list.files() {
                    let filename = file.basename()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "File".to_string());
                    
                    let path = file.path();
                    let is_dir = path.as_ref().map(|p| p.is_dir()).unwrap_or(false);
                    let icon_name = if is_dir { "folder" } else { "text" };

                    let item_btn = gtk4::Button::new();
                    item_btn.add_css_class("shelf-item");
                    
                    let item_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
                    let item_icon = archvnde_common::icon::get_icon(icon_name, 12);
                    
                    let display_name = if filename.len() > 10 {
                        format!("{}..", &filename[..8])
                    } else {
                        filename.clone()
                    };
                    let item_label = gtk4::Label::new(Some(&display_name));
                    item_label.add_css_class("shelf-item-text");

                    item_content.append(&item_icon);
                    item_content.append(&item_label);
                    item_btn.set_child(Some(&item_content));

                    // DragSource to allow dragging it out
                    let file_clone = file.clone();
                    let drag_source = gtk4::DragSource::builder()
                        .actions(gtk4::gdk::DragAction::COPY)
                        .build();
                    drag_source.connect_prepare(move |_, _, _| {
                        let fl = gtk4::gdk::FileList::new(&[file_clone.clone()]);
                        Some(gtk4::gdk::ContentProvider::for_value(&fl.to_value()))
                    });
                    item_btn.add_controller(drag_source);

                    // Right click to remove
                    let click_gesture = gtk4::GestureClick::builder().button(3).build();
                    let item_btn_clone = item_btn.clone();
                    let shelf_box_inner = shelf_box_clone.clone();
                    click_gesture.connect_released(move |_, _, _, _| {
                        shelf_box_inner.remove(&item_btn_clone);
                    });
                    item_btn.add_controller(click_gesture);

                    shelf_box_clone.append(&item_btn);
                }
                return true;
            }
            false
        });

        drop_zone.add_controller(drop_target);

        // Append components
        notch_content.append(&player_box);
        notch_content.append(&notch_sep);
        notch_content.append(&shelf_box);

        notch_capsule.append(&notch_content);
        center_box.append(&notch_capsule);

        // Right-aligned section: Status & Clock capsule
        let right_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        right_box.set_hexpand(true);
        right_box.set_halign(gtk4::Align::End);
        right_box.set_valign(gtk4::Align::Center);
        right_box.append(&status_indicators);

        // Assemble columns into the main panel box using CenterBox
        box_layout.set_start_widget(Some(&left_box));
        box_layout.set_center_widget(Some(&center_box));
        box_layout.set_end_widget(Some(&right_box));

        window.set_child(Some(&box_layout));

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
