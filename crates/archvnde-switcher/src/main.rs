use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
struct RunningApp {
    name: String,
    id: String,
    icon: String,
}

fn get_running_apps() -> Vec<RunningApp> {
    let mut apps = Vec::new();
    let mut detected = std::collections::HashSet::new();

    // Check /proc directory to scan running processes on Linux
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.chars().all(|c| c.is_ascii_digit()) {
                        let comm_path = entry.path().join("comm");
                        if let Ok(comm) = std::fs::read_to_string(comm_path) {
                            let process_name = comm.trim().to_lowercase();
                            let app_info = match process_name.as_str() {
                                "firefox" => Some(("Firefox Web Browser", "firefox", "firefox")),
                                "chrome" | "chromium" | "google-chrome" => Some(("Google Chrome", "chrome", "google-chrome")),
                                "foot" => Some(("Foot Terminal", "foot", "terminal")),
                                "alacritty" => Some(("Alacritty Terminal", "alacritty", "alacritty")),
                                "code" | "vscode" => Some(("VS Code Editor", "code", "com.visualstudio.code")),
                                "thunar" => Some(("Thunar Files", "thunar", "system-file-manager")),
                                "pcmanfm" => Some(("PCManFM Files", "pcmanfm", "system-file-manager")),
                                "spotify" => Some(("Spotify Music", "spotify", "spotify")),
                                "gimp" => Some(("GIMP Editor", "gimp", "gimp")),
                                "vlc" => Some(("VLC Media Player", "vlc", "vlc")),
                                _ => None,
                            };
                            if let Some((display_name, id, icon)) = app_info {
                                if !detected.contains(id) {
                                    detected.insert(id.to_string());
                                    apps.push(RunningApp {
                                        name: display_name.to_string(),
                                        id: id.to_string(),
                                        icon: icon.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback if no GUI apps detected, to ensure a premium visual demo
    if apps.is_empty() {
        apps = vec![
            RunningApp {
                name: "Foot Terminal".to_string(),
                id: "foot".to_string(),
                icon: "terminal".to_string(),
            },
            RunningApp {
                name: "Firefox Web Browser".to_string(),
                id: "firefox".to_string(),
                icon: "firefox".to_string(),
            },
            RunningApp {
                name: "VS Code Editor".to_string(),
                id: "code".to_string(),
                icon: "com.visualstudio.code".to_string(),
            },
            RunningApp {
                name: "Thunar Files".to_string(),
                id: "thunar".to_string(),
                icon: "system-file-manager".to_string(),
            },
            RunningApp {
                name: "Spotify Music".to_string(),
                id: "spotify".to_string(),
                icon: "spotify".to_string(),
            },
        ];
    }
    apps
}

fn main() {
    println!("Starting ArchVNDE Alt-Tab Switcher...");

    let application = gtk4::Application::new(
        Some("org.archvnde.switcher"),
        Default::default(),
    );

    application.connect_activate(|app| {
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        // Center on screen
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.add_css_class("switcher-window");

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        main_box.add_css_class("switcher-box");

        // 1. App Preview Stack
        let preview_stack = gtk4::Stack::new();
        preview_stack.add_css_class("preview-frame");
        preview_stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        preview_stack.set_transition_duration(150);

        let apps = get_running_apps();
        let apps_count = apps.len();

        for app_item in &apps {
            let p_widget = create_mock_preview(&app_item.id, &app_item.name, &app_item.icon);
            preview_stack.add_named(&p_widget, Some(&app_item.id));
        }
        main_box.append(&preview_stack);

        // 2. Selected App Details (Name + Subtitle/Process)
        let details_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        details_box.add_css_class("switcher-details-box");
        details_box.set_halign(gtk4::Align::Center);

        let app_title_lbl = gtk4::Label::new(None);
        app_title_lbl.add_css_class("switcher-app-title");
        app_title_lbl.set_halign(gtk4::Align::Center);

        let app_sub_lbl = gtk4::Label::new(None);
        app_sub_lbl.add_css_class("switcher-app-subtitle");
        app_sub_lbl.set_halign(gtk4::Align::Center);

        details_box.append(&app_title_lbl);
        details_box.append(&app_sub_lbl);
        main_box.append(&details_box);

        // 3. Horizontal Icons Row
        let icons_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        icons_row.add_css_class("switcher-list-row");
        icons_row.set_halign(gtk4::Align::Center);

        let mut item_buttons = Vec::new();

        for (idx, app_item) in apps.iter().enumerate() {
            let btn = gtk4::Button::new();
            btn.add_css_class("switcher-item-btn");
            
            let btn_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
            let icon_widget = archvnde_common::icon::get_system_or_file_icon(&app_item.icon, "application-x-executable");
            icon_widget.set_pixel_size(32);
            icon_widget.add_css_class("switcher-item-icon");

            btn_box.append(&icon_widget);
            btn.set_child(Some(&btn_box));
            
            icons_row.append(&btn);
            item_buttons.push(btn);
        }
        main_box.append(&icons_row);

        window.set_child(Some(&main_box));

        // State tracking
        let current_index = Rc::new(RefCell::new(0));

        let update_selection = {
            let current_index = current_index.clone();
            let apps = apps.clone();
            let preview_stack = preview_stack.clone();
            let app_title_lbl = app_title_lbl.clone();
            let app_sub_lbl = app_sub_lbl.clone();
            let item_buttons = item_buttons.clone();

            move |new_idx: usize| {
                let mut idx = new_idx;
                if idx >= apps.len() {
                    idx = 0;
                }
                *current_index.borrow_mut() = idx;

                let app_item = &apps[idx];
                app_title_lbl.set_text(&app_item.name);
                app_sub_lbl.set_text(&format!("process: {}", app_item.id));

                preview_stack.set_visible_child_name(&app_item.id);

                for (i, btn) in item_buttons.iter().enumerate() {
                    if i == idx {
                        btn.add_css_class("selected");
                    } else {
                        btn.remove_css_class("selected");
                    }
                }
            }
        };

        // Initial selection setup
        let update_selection_rc = Rc::new(update_selection);
        update_selection_rc(0);

        // Click handlers on buttons
        for (i, btn) in item_buttons.iter().enumerate() {
            let update_sel = update_selection_rc.clone();
            btn.connect_clicked(move |_| {
                update_sel(i);
            });
        }

        // Keyboard navigation
        let key_controller = gtk4::EventControllerKey::new();
        let current_idx_key = current_index.clone();
        let update_sel_key = update_selection_rc.clone();
        let window_close = window.clone();
        let apps_key = apps.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let idx = *current_idx_key.borrow();
            match key {
                gtk4::gdk::Key::Tab | gtk4::gdk::Key::Right => {
                    let next = (idx + 1) % apps_key.len();
                    update_sel_key(next);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Left => {
                    let prev = if idx == 0 { apps_key.len() - 1 } else { idx - 1 };
                    update_sel_key(prev);
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Return | gtk4::gdk::Key::space => {
                    let app_item = &apps_key[idx];
                    println!("Selected App: {}", app_item.name);
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                gtk4::gdk::Key::Escape => {
                    window_close.close();
                    gtk4::glib::Propagation::Stop
                }
                _ => gtk4::glib::Propagation::Proceed,
            }
        });
        window.add_controller(key_controller);

        window.present();
    });

    application.run();
}

fn create_mock_preview(id: &str, name: &str, icon: &str) -> gtk4::Widget {
    match id {
        "foot" | "alacritty" => create_terminal_preview(),
        "firefox" | "chrome" => create_browser_preview(),
        "code" => create_editor_preview(),
        "thunar" | "pcmanfm" => create_files_preview(),
        "spotify" => create_spotify_preview(),
        _ => create_generic_preview(name, icon),
    }
}

fn create_terminal_preview() -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.add_css_class("preview-terminal");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    let title_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    title_bar.add_css_class("terminal-titlebar");

    let r_dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0); r_dot.add_css_class("terminal-btn-red");
    let y_dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0); y_dot.add_css_class("terminal-btn-yellow");
    let g_dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0); g_dot.add_css_class("terminal-btn-green");
    title_bar.append(&r_dot);
    title_bar.append(&y_dot);
    title_bar.append(&g_dot);

    let title_lbl = gtk4::Label::new(Some("user@archvnde: ~"));
    title_lbl.set_halign(gtk4::Align::Center);
    title_lbl.set_hexpand(true);
    title_lbl.add_css_class("terminal-text");
    title_bar.append(&title_lbl);

    let body_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    body_box.set_margin_start(12);
    body_box.set_margin_end(12);
    body_box.set_margin_top(12);

    let prompt1 = gtk4::Label::new(Some("[user@archvnde ~]$ neofetch"));
    prompt1.set_halign(gtk4::Align::Start);
    prompt1.add_css_class("terminal-text");

    let neofetch_info = gtk4::Label::new(Some(
        "<b>OS</b>: Arch Linux x86_64\n<b>Kernel</b>: 6.9.1-archvnde\n<b>Shell</b>: bash 5.2.26\n<b>DE</b>: ArchVNDE (Labwc)\n<b>WM</b>: labwc\n<b>Theme</b>: Glassmorphism-Dark"
    ));
    neofetch_info.set_use_markup(true);
    neofetch_info.set_halign(gtk4::Align::Start);

    let prompt2 = gtk4::Label::new(Some("[user@archvnde ~]$ _"));
    prompt2.set_halign(gtk4::Align::Start);
    prompt2.add_css_class("terminal-text");

    body_box.append(&prompt1);
    body_box.append(&neofetch_info);
    body_box.append(&prompt2);

    main_box.append(&title_bar);
    main_box.append(&body_box);

    main_box.upcast::<gtk4::Widget>()
}

fn create_browser_preview() -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.add_css_class("preview-browser");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    header.add_css_class("browser-header");

    let tab = gtk4::Label::new(Some("Google"));
    tab.add_css_class("browser-tab");
    header.append(&tab);

    let address_bar = gtk4::Label::new(Some("https://www.google.com"));
    address_bar.add_css_class("browser-address-bar");
    address_bar.set_hexpand(true);
    address_bar.set_halign(gtk4::Align::Start);
    header.append(&address_bar);

    let content_grid = gtk4::Grid::new();
    content_grid.add_css_class("browser-grid");
    content_grid.set_column_spacing(16);
    content_grid.set_row_spacing(16);
    content_grid.set_halign(gtk4::Align::Center);
    content_grid.set_valign(gtk4::Align::Center);
    content_grid.set_vexpand(true);

    let search_logo = gtk4::Label::new(Some("<span font_weight='800' font_size='xx-large'>Google</span>"));
    search_logo.set_use_markup(true);
    search_logo.set_halign(gtk4::Align::Center);
    content_grid.attach(&search_logo, 0, 0, 4, 1);

    let search_bar = gtk4::Label::new(Some("Tìm kiếm trên Google hoặc nhập một URL"));
    search_bar.add_css_class("browser-address-bar");
    search_bar.set_size_request(320, -1);
    content_grid.attach(&search_bar, 0, 1, 4, 1);

    // Shortcuts
    let sites = [("GitHub", "github"), ("YouTube", "youtube"), ("Reddit", "reddit"), ("News", "info")];
    for (i, (name, icon)) in sites.iter().enumerate() {
        let sc_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        sc_box.add_css_class("browser-shortcut");
        sc_box.set_size_request(60, 60);

        let sc_icon = archvnde_common::icon::get_icon(icon, 20);
        sc_icon.set_halign(gtk4::Align::Center);
        let sc_lbl = gtk4::Label::new(Some(name));
        sc_lbl.set_halign(gtk4::Align::Center);
        sc_lbl.add_css_class("popup-time");

        sc_box.append(&sc_icon);
        sc_box.append(&sc_lbl);
        content_grid.attach(&sc_box, i as i32, 2, 1, 1);
    }

    main_box.append(&header);
    main_box.append(&content_grid);

    main_box.upcast::<gtk4::Widget>()
}

fn create_editor_preview() -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    main_box.add_css_class("preview-editor");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    sidebar.add_css_class("editor-sidebar");

    let sidebar_title = gtk4::Label::new(Some("EXPLORER: ARCHVNDE"));
    sidebar_title.add_css_class("editor-sidebar-title");
    sidebar_title.set_halign(gtk4::Align::Start);
    sidebar.append(&sidebar_title);

    let files = [("📁 src", "folder"), ("  📄 main.rs", "text-x-generic"), ("  📄 widgets.rs", "text-x-generic"), ("📄 Cargo.toml", "text-x-generic")];
    for (name, _) in files {
        let f_lbl = gtk4::Label::new(Some(name));
        f_lbl.add_css_class("editor-file-item");
        f_lbl.set_halign(gtk4::Align::Start);
        sidebar.append(&f_lbl);
    }

    let code_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    code_box.add_css_class("editor-content");
    code_box.set_hexpand(true);

    let line_nums = gtk4::Label::new(Some("1\n2\n3\n4\n5\n6\n7\n8\n9"));
    line_nums.add_css_class("editor-line-num");
    line_nums.set_halign(gtk4::Align::End);

    let code_lbl = gtk4::Label::new(Some(
        "<span color='#569cd6'>fn</span> <span color='#dcdcaa'>main</span>() {\n    <span color='#4ec9b0'>println!</span>(<span color='#ce9178'>\"Starting panel...\"</span>);\n    <span color='#569cd6'>let</span> app = Application::<span color='#dcdcaa'>new</span>();\n    app.<span color='#dcdcaa'>run</span>();\n}"
    ));
    code_lbl.set_use_markup(true);
    code_lbl.set_halign(gtk4::Align::Start);

    code_box.append(&line_nums);
    code_box.append(&code_lbl);

    main_box.append(&sidebar);
    main_box.append(&code_box);

    main_box.upcast::<gtk4::Widget>()
}

fn create_files_preview() -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    main_box.add_css_class("preview-files");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    sidebar.add_css_class("files-sidebar");

    let places = ["🏠 Home", "📁 Documents", "📥 Downloads", "🎵 Music", "🖼️ Pictures"];
    for (i, place) in places.iter().enumerate() {
        let item = gtk4::Label::new(Some(place));
        item.add_css_class("files-sidebar-item");
        if i == 0 {
            item.add_css_class("active");
        }
        item.set_halign(gtk4::Align::Start);
        sidebar.append(&item);
    }

    let grid = gtk4::Grid::new();
    grid.add_css_class("files-grid");
    grid.set_column_spacing(18);
    grid.set_row_spacing(18);

    let folders = [("Documents", "folder"), ("Downloads", "folder-download"), ("Music", "folder-music"), ("Pictures", "folder-pictures")];
    for (i, (name, icon)) in folders.iter().enumerate() {
        let f_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        f_box.add_css_class("files-grid-item");

        let f_icon = archvnde_common::icon::get_icon(icon, 36);
        f_icon.set_halign(gtk4::Align::Center);
        let f_lbl = gtk4::Label::new(Some(name));
        f_lbl.add_css_class("files-grid-label");
        f_lbl.set_halign(gtk4::Align::Center);

        f_box.append(&f_icon);
        f_box.append(&f_lbl);

        let row = (i / 3) as i32;
        let col = (i % 3) as i32;
        grid.attach(&f_box, col, row, 1, 1);
    }

    main_box.append(&sidebar);
    main_box.append(&grid);

    main_box.upcast::<gtk4::Widget>()
}

fn create_spotify_preview() -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 20);
    main_box.add_css_class("preview-spotify");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);
    main_box.set_valign(gtk4::Align::Center);

    let art = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    art.add_css_class("spotify-art");
    art.set_halign(gtk4::Align::Start);

    let music_icon = archvnde_common::icon::get_icon_colored("audio-x-generic", 48, "#ffffff");
    music_icon.set_halign(gtk4::Align::Center);
    music_icon.set_valign(gtk4::Align::Center);
    music_icon.set_vexpand(true);
    art.append(&music_icon);

    let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    info_box.set_valign(gtk4::Align::Center);

    let title = gtk4::Label::new(Some("Stairway to Heaven"));
    title.add_css_class("spotify-title");
    title.set_halign(gtk4::Align::Start);

    let artist = gtk4::Label::new(Some("Led Zeppelin"));
    artist.add_css_class("spotify-artist");
    artist.set_halign(gtk4::Align::Start);

    let playback_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    playback_row.set_valign(gtk4::Align::Center);

    let time_elapsed = gtk4::Label::new(Some("2:15"));
    time_elapsed.add_css_class("popup-time");

    let progress_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    progress_bar.add_css_class("spotify-bar");

    let progress_fill = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    progress_fill.add_css_class("spotify-progress");
    progress_fill.set_size_request(90, -1); // 2:15 out of 8:02
    progress_bar.append(&progress_fill);

    let time_total = gtk4::Label::new(Some("8:02"));
    time_total.add_css_class("popup-time");

    playback_row.append(&time_elapsed);
    playback_row.append(&progress_bar);
    playback_row.append(&time_total);

    info_box.append(&title);
    info_box.append(&artist);
    info_box.append(&playback_row);

    main_box.append(&art);
    main_box.append(&info_box);

    main_box.upcast::<gtk4::Widget>()
}

fn create_generic_preview(name: &str, icon: &str) -> gtk4::Widget {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.add_css_class("preview-generic");
    main_box.set_vexpand(true);
    main_box.set_hexpand(true);

    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    card.add_css_class("preview-generic-box");
    card.set_halign(gtk4::Align::Center);
    card.set_valign(gtk4::Align::Center);

    let icon_w = archvnde_common::icon::get_system_or_file_icon(icon, "application-x-executable");
    icon_w.set_pixel_size(64);
    icon_w.set_halign(gtk4::Align::Center);

    let name_lbl = gtk4::Label::new(Some(name));
    name_lbl.add_css_class("switcher-app-title");
    name_lbl.set_halign(gtk4::Align::Center);

    let desc_lbl = gtk4::Label::new(Some("Giao diện đang chạy"));
    desc_lbl.add_css_class("popup-time");
    desc_lbl.set_halign(gtk4::Align::Center);

    card.append(&icon_w);
    card.append(&name_lbl);
    card.append(&desc_lbl);

    main_box.append(&card);

    main_box.upcast::<gtk4::Widget>()
}
