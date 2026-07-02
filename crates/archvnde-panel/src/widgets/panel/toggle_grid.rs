use gtk4::prelude::*;
use std::rc::Rc;
use tokio::sync::mpsc;

fn create_tile_row(
    icon_name: &str,
    title: &str,
    subtitle: &str,
    is_active: bool,
    active_class: &str,
    on_click: Option<impl Fn() + 'static>,
) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-tile-row");
    if is_active {
        btn.add_css_class(active_class);
    }
    btn.set_hexpand(true);
    btn.set_vexpand(true);
    btn.set_valign(gtk4::Align::Fill);

    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_box.set_valign(gtk4::Align::Center);

    // Icon Circle wrapper
    let circle = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    circle.add_css_class("control-icon-circle");
    if is_active {
        circle.add_css_class("active");
    }
    
    let initial_color = if is_active { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
    let icon_widget = archvnde_common::icon::get_icon_colored(icon_name, 14, initial_color);
    circle.append(&icon_widget);
    main_box.append(&circle);

    // Text box
    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
    let title_label = gtk4::Label::new(Some(title));
    title_label.set_xalign(0.0);
    title_label.add_css_class("tile-title");

    let sub_label = gtk4::Label::new(Some(subtitle));
    sub_label.set_xalign(0.0);
    sub_label.add_css_class("tile-subtitle");

    text_box.append(&title_label);
    text_box.append(&sub_label);
    main_box.append(&text_box);

    btn.set_child(Some(&main_box));

    let act_class = active_class.to_string();
    let icon_name_str = icon_name.to_string();
    let circle_clone = circle.clone();
    let icon_widget_clone = icon_widget.clone();

    let on_click_opt = std::rc::Rc::new(std::cell::RefCell::new(on_click));

    btn.connect_clicked(move |b| {
        let is_now_active = if b.has_css_class(&act_class) {
            b.remove_css_class(&act_class);
            circle_clone.remove_css_class("active");
            false
        } else {
            b.add_css_class(&act_class);
            circle_clone.add_css_class("active");
            true
        };

        let color = if is_now_active { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
        let new_img = archvnde_common::icon::get_icon_colored(&icon_name_str, 14, color);
        if let Some(paintable) = new_img.paintable() {
            icon_widget_clone.set_paintable(Some(&paintable));
        }

        if let Some(ref cb) = *on_click_opt.borrow() {
            cb();
        }
    });

    btn
}

pub fn create_small_theme_toggle_tile() -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-square-tile");
    btn.set_hexpand(true);
    btn.set_valign(gtk4::Align::Fill);
    btn.set_vexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.set_valign(gtk4::Align::Center);
    main_box.set_halign(gtk4::Align::Center);

    let is_dark_init = gtk4::Settings::default()
        .map(|s| s.is_gtk_application_prefer_dark_theme())
        .unwrap_or(true);

    if is_dark_init {
        btn.add_css_class("active");
    }

    // Use an icon container Box so we can swap the entire widget on toggle
    let icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    icon_container.set_halign(gtk4::Align::Center);

    let initial_icon_name = if is_dark_init { "dark-mode" } else { "brightness" };
    let initial_color = if is_dark_init { "#ffffff" } else { "rgba(255, 255, 255, 0.8)" };
    let icon_widget = archvnde_common::icon::get_icon_colored(initial_icon_name, 16, initial_color);
    icon_container.append(&icon_widget);

    let label = gtk4::Label::new(Some(&archvnde_common::i18n::t("control.dark_mode")));
    label.add_css_class("control-square-label");
    label.set_halign(gtk4::Align::Center);

    main_box.append(&icon_container);
    main_box.append(&label);
    btn.set_child(Some(&main_box));

    btn.connect_clicked(move |b| {
        let settings = gtk4::Settings::default();
        let current_dark = settings.as_ref()
            .map(|s| s.is_gtk_application_prefer_dark_theme())
            .unwrap_or(true);
        let new_dark = !current_dark;

        // Write gsettings synchronously FIRST so the value is settled before the
        // gtk_application_prefer_dark_theme_notify signal fires and reloads CSS.
        // Also ensures separate binaries (screenshot, switcher, etc.) launched
        // afterwards will read the correct color-scheme via init_theme().
        let scheme = if new_dark { "prefer-dark" } else { "prefer-light" };
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.desktop.interface", "color-scheme", scheme])
            .output(); // .output() blocks until done — gsettings is fast (~5ms)

        // Now trigger the signal; is_dark_mode() reads the GTK in-process setting
        // which is about to be updated to new_dark.
        if let Some(ref s) = settings {
            s.set_gtk_application_prefer_dark_theme(new_dark);
        }

        // Swap icon widget inside the container
        if let Some(old) = icon_container.first_child() {
            icon_container.remove(&old);
        }
        if new_dark {
            b.add_css_class("active");
            let new_img = archvnde_common::icon::get_icon_colored("dark-mode", 16, "#ffffff");
            icon_container.append(&new_img);
        } else {
            b.remove_css_class("active");
            let new_img = archvnde_common::icon::get_icon_colored("brightness", 16, "rgba(255, 255, 255, 0.8)");
            icon_container.append(&new_img);
        }
    });

    btn
}

fn get_wifi_state() -> (bool, String) {
    let dev_output = std::process::Command::new("iwctl")
        .args(&["device", "list"])
        .output();
    let is_powered = if let Ok(out) = dev_output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut powered = false;
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[0] == "wlan0" {
                powered = parts[2] == "on";
                break;
            }
        }
        powered
    } else {
        false
    };

    if !is_powered {
        return (false, "Off".to_string());
    }

    let station_output = std::process::Command::new("iwctl")
        .args(&["station", "wlan0", "show"])
        .output();
    if let Ok(out) = station_output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut state = "Disconnected".to_string();
        let mut connected_network = None;

        for line in stdout.lines() {
            if line.contains("State") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    state = parts[parts.len() - 1].to_string();
                }
            } else if line.contains("Connected network") {
                if let Some(pos) = line.find("Connected network") {
                    let val = &line[pos + "Connected network".len()..];
                    let trimmed = val.trim();
                    if !trimmed.is_empty() {
                        connected_network = Some(trimmed.to_string());
                    }
                }
            }
        }

        if state == "connected" {
            if let Some(net) = connected_network {
                (true, net)
            } else {
                (true, "Connected".to_string())
            }
        } else if state == "connecting" {
            (true, "Connecting...".to_string())
        } else if state == "authenticating" {
            (true, "Authenticating...".to_string())
        } else {
            (true, "Disconnected".to_string())
        }
    } else {
        (true, "Disconnected".to_string())
    }
}

fn known_networks() -> Vec<String> {
    let mut ssids = Vec::new();
    let output = std::process::Command::new("iwctl")
        .args(&["known-networks", "list"])
        .output();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut start_parsing = false;
        for line in stdout.lines() {
            if line.contains("----") {
                start_parsing = true;
                continue;
            }
            if start_parsing {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Some(idx) = parts.iter().position(|&x| x == "psk" || x == "open" || x == "8021x") {
                        let ssid = parts[0..idx].join(" ");
                        ssids.push(ssid);
                    }
                }
            }
        }
    }
    ssids
}

fn scan_networks() -> Vec<(String, String, String, bool)> {
    let _ = std::process::Command::new("iwctl")
        .args(&["station", "wlan0", "scan"])
        .output();
        
    std::thread::sleep(std::time::Duration::from_millis(150));

    let mut networks = Vec::new();
    let output = std::process::Command::new("iwctl")
        .args(&["station", "wlan0", "get-networks"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut start_parsing = false;
        for line in stdout.lines() {
            if line.contains("----") {
                start_parsing = true;
                continue;
            }
            if start_parsing {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let is_connected = line.starts_with("  >");
                
                let clean_line = if is_connected {
                    line.replacen('>', "", 1)
                } else {
                    line.to_string()
                };

                let parts: Vec<&str> = clean_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Some(idx) = parts.iter().position(|&x| x == "psk" || x == "open" || x == "8021x") {
                        let ssid = parts[0..idx].join(" ");
                        let security = parts[idx].to_string();
                        let signal = if idx + 1 < parts.len() {
                            parts[idx + 1].to_string()
                        } else {
                            "****".to_string()
                        };
                        networks.push((ssid, security, signal, is_connected));
                    }
                }
            }
        }
    }
    networks
}

pub fn create_wifi_tile(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    container.add_css_class("control-tile-container");
    container.set_hexpand(true);
    
    let left_btn = gtk4::Button::new();
    left_btn.add_css_class("control-tile-left-btn");
    left_btn.set_hexpand(true);
    
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_box.set_valign(gtk4::Align::Center);
    
    let circle = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    circle.add_css_class("control-icon-circle");
    
    let (is_active, ssid) = get_wifi_state();
    if is_active {
        left_btn.add_css_class("active");
        circle.add_css_class("active");
    }
    
    let color = if is_active { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
    let icon_widget = archvnde_common::icon::get_icon_colored("wifi", 14, color);
    circle.append(&icon_widget);
    main_box.append(&circle);
    
    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
    let title_label = gtk4::Label::new(Some(&archvnde_common::i18n::t("control.network")));
    title_label.set_xalign(0.0);
    title_label.add_css_class("tile-title");
    
    let sub_label = gtk4::Label::new(Some(&ssid));
    sub_label.set_xalign(0.0);
    sub_label.add_css_class("tile-subtitle");
    
    text_box.append(&title_label);
    text_box.append(&sub_label);
    main_box.append(&text_box);
    left_btn.set_child(Some(&main_box));
    
    let right_btn = gtk4::Button::new();
    right_btn.add_css_class("control-tile-right-btn");
    let arrow_icon = archvnde_common::icon::get_icon_colored("go-next-symbolic", 12, "rgba(255, 255, 255, 0.7)");
    right_btn.set_child(Some(&arrow_icon));
    
    let popover = gtk4::Popover::new();
    popover.add_css_class("taskbar-popover");
    popover.set_parent(&right_btn);
    popover.set_position(gtk4::PositionType::Right);
    popover.set_has_arrow(true);
    
    setup_wifi_popover(&popover, sub_label.clone(), left_btn.clone(), circle.clone(), icon_widget.clone());
    
    let on_popover_toggled_c = on_popover_toggled.clone();
    let popover_c1 = popover.clone();
    right_btn.connect_clicked(move |_| {
        popover_c1.popup();
        if let Some(ref cb) = on_popover_toggled_c {
            cb(true);
        }
    });

    if let Some(ref cb) = on_popover_toggled {
        let cb_clone = cb.clone();
        popover.connect_closed(move |_| {
            cb_clone(false);
        });
    }
    
    let circle_c = circle.clone();
    let icon_widget_c = icon_widget.clone();
    let sub_label_c = sub_label.clone();
    left_btn.connect_clicked(move |b| {
        let current_active = b.has_css_class("active");
        let new_active = !current_active;
        if new_active {
            b.add_css_class("active");
            circle_c.add_css_class("active");
            let _ = std::process::Command::new("iwctl")
                .args(&["device", "wlan0", "set-property", "Powered", "on"])
                .spawn();
            sub_label_c.set_text("Scanning...");
        } else {
            b.remove_css_class("active");
            circle_c.remove_css_class("active");
            let _ = std::process::Command::new("iwctl")
                .args(&["device", "wlan0", "set-property", "Powered", "off"])
                .spawn();
            sub_label_c.set_text("Off");
        }
        let color = if new_active { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
        let new_img = archvnde_common::icon::get_icon_colored("wifi", 14, color);
        if let Some(paintable) = new_img.paintable() {
            icon_widget_c.set_paintable(Some(&paintable));
        }
    });
    
    container.append(&left_btn);
    container.append(&right_btn);
    container
}

fn setup_wifi_popover(
    popover: &gtk4::Popover,
    sub_label: gtk4::Label,
    left_btn: gtk4::Button,
    circle: gtk4::Box,
    icon_widget: gtk4::Image,
) {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    main_box.add_css_class("wifi-popover-box");
    popover.set_child(Some(&main_box));

    let popover_clone = popover.clone();
    let sub_label_clone = sub_label.clone();
    let left_btn_clone = left_btn.clone();
    let circle_clone = circle.clone();
    let icon_widget_clone = icon_widget.clone();

    popover.connect_map(move |_| {
        refresh_wifi_popover_list(
            &main_box,
            sub_label_clone.clone(),
            left_btn_clone.clone(),
            circle_clone.clone(),
            icon_widget_clone.clone(),
            popover_clone.clone(),
        );
    });
}

fn refresh_wifi_popover_list(
    main_box: &gtk4::Box,
    sub_label: gtk4::Label,
    left_btn: gtk4::Button,
    circle: gtk4::Box,
    icon_widget: gtk4::Image,
    popover: gtk4::Popover,
) {
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }

    let title = gtk4::Label::new(Some("Wi-Fi Networks"));
    title.add_css_class("wifi-popover-title");
    title.set_xalign(0.0);
    main_box.append(&title);

    let scanning_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    scanning_box.set_halign(gtk4::Align::Center);
    scanning_box.set_margin_top(15);
    scanning_box.set_margin_bottom(15);

    let spinner = gtk4::Spinner::new();
    spinner.start();
    scanning_box.append(&spinner);

    let scan_label = gtk4::Label::new(Some("Scanning for networks..."));
    scanning_box.append(&scan_label);
    main_box.append(&scanning_box);

    let main_box_clone = main_box.clone();
    let sub_label_clone = sub_label.clone();
    let left_btn_clone = left_btn.clone();
    let circle_clone = circle.clone();
    let icon_widget_clone = icon_widget.clone();
    let popover_clone = popover.clone();

    let (tx, mut rx) = mpsc::unbounded_channel::<Option<(Vec<(String, String, String, bool)>, Vec<String>)>>();

    std::thread::spawn(move || {
        let nets = scan_networks();
        let known = known_networks();
        let _ = tx.send(Some((nets, known)));
    });

    glib::spawn_future_local(async move {
        if let Some(Some((nets, known))) = rx.recv().await {
            build_wifi_list_ui(
                &main_box_clone,
                nets,
                known,
                sub_label_clone.clone(),
                left_btn_clone.clone(),
                circle_clone.clone(),
                icon_widget_clone.clone(),
                popover_clone.clone(),
            );
        }
    });
}

fn build_wifi_list_ui(
    main_box: &gtk4::Box,
    networks: Vec<(String, String, String, bool)>,
    known: Vec<String>,
    sub_label: gtk4::Label,
    left_btn: gtk4::Button,
    circle: gtk4::Box,
    icon_widget: gtk4::Image,
    popover: gtk4::Popover,
) {
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }

    let title = gtk4::Label::new(Some("Wi-Fi Networks"));
    title.add_css_class("wifi-popover-title");
    title.set_xalign(0.0);
    main_box.append(&title);

    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);

    for (ssid, security, signal, is_connected) in networks {
        let row_btn = gtk4::Button::new();
        row_btn.add_css_class("wifi-network-item");
        if is_connected {
            row_btn.add_css_class("active");
        }

        let item_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        item_box.set_valign(gtk4::Align::Center);

        let wifi_icon = archvnde_common::icon::get_icon("wifi", 12);
        item_box.append(&wifi_icon);

        let name_label = gtk4::Label::new(Some(&ssid));
        name_label.set_hexpand(true);
        name_label.set_xalign(0.0);
        item_box.append(&name_label);

        let is_secured = security != "open";
        if is_secured {
            let lock_icon = archvnde_common::icon::get_icon("lock", 10);
            lock_icon.set_opacity(0.6);
            item_box.append(&lock_icon);
        }

        let sig_label = gtk4::Label::new(Some(&signal));
        sig_label.set_opacity(0.6);
        sig_label.set_xalign(1.0);
        item_box.append(&sig_label);

        row_btn.set_child(Some(&item_box));

        let ssid_clone = ssid.clone();
        let security_clone = security.clone();
        let is_saved = known.contains(&ssid);
        
        let main_box_c = main_box.clone();
        let sub_label_c = sub_label.clone();
        let left_btn_c = left_btn.clone();
        let circle_c = circle.clone();
        let icon_widget_c = icon_widget.clone();
        let popover_c = popover.clone();

        row_btn.connect_clicked(move |_| {
            if is_connected {
                return;
            }

            if is_saved || security_clone == "open" {
                show_connecting_state(&main_box_c, &ssid_clone);
                connect_wifi_async(
                    &ssid_clone,
                    None,
                    None,
                    sub_label_c.clone(),
                    left_btn_c.clone(),
                    circle_c.clone(),
                    icon_widget_c.clone(),
                    popover_c.clone(),
                );
            } else {
                show_credentials_form(
                    &main_box_c,
                    &ssid_clone,
                    &security_clone,
                    sub_label_c.clone(),
                    left_btn_c.clone(),
                    circle_c.clone(),
                    icon_widget_c.clone(),
                    popover_c.clone(),
                );
            }
        });

        list_box.append(&row_btn);
    }

    let scroll = gtk4::ScrolledWindow::new();
    scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scroll.set_max_content_height(200);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&list_box));

    main_box.append(&scroll);
}

fn show_connecting_state(main_box: &gtk4::Box, ssid: &str) {
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }
    
    let label = gtk4::Label::new(Some(&format!("Connecting to {}...", ssid)));
    label.add_css_class("wifi-popover-title");
    label.set_margin_bottom(10);
    main_box.append(&label);

    let spinner = gtk4::Spinner::new();
    spinner.start();
    spinner.set_halign(gtk4::Align::Center);
    main_box.append(&spinner);
}

fn show_credentials_form(
    main_box: &gtk4::Box,
    ssid: &str,
    security: &str,
    sub_label: gtk4::Label,
    left_btn: gtk4::Button,
    circle: gtk4::Box,
    icon_widget: gtk4::Image,
    popover: gtk4::Popover,
) {
    while let Some(child) = main_box.first_child() {
        main_box.remove(&child);
    }

    let title = gtk4::Label::new(Some(&format!("Connect to {}", ssid)));
    title.add_css_class("wifi-popover-title");
    title.set_xalign(0.0);
    main_box.append(&title);

    let form_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);

    let username_entry = if security == "8021x" {
        let entry = gtk4::Entry::new();
        entry.set_placeholder_text(Some("Username"));
        entry.add_css_class("wifi-input-field");
        form_box.append(&entry);
        Some(entry)
    } else {
        None
    };

    let password_entry = gtk4::Entry::new();
    password_entry.set_placeholder_text(Some("Password"));
    password_entry.set_visibility(false);
    password_entry.add_css_class("wifi-input-field");
    form_box.append(&password_entry);

    let button_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    button_box.add_css_class("wifi-button-row");
    button_box.set_homogeneous(true);

    let cancel_btn = gtk4::Button::new();
    cancel_btn.set_label("Cancel");
    cancel_btn.add_css_class("wifi-btn-secondary");

    let connect_btn = gtk4::Button::new();
    connect_btn.set_label("Connect");
    connect_btn.add_css_class("wifi-btn-primary");

    button_box.append(&cancel_btn);
    button_box.append(&connect_btn);
    form_box.append(&button_box);
    main_box.append(&form_box);

    let main_box_c = main_box.clone();
    let sub_label_c = sub_label.clone();
    let left_btn_c = left_btn.clone();
    let circle_c = circle.clone();
    let icon_widget_c = icon_widget.clone();
    let popover_c = popover.clone();
    cancel_btn.connect_clicked(move |_| {
        refresh_wifi_popover_list(
            &main_box_c,
            sub_label_c.clone(),
            left_btn_c.clone(),
            circle_c.clone(),
            icon_widget_c.clone(),
            popover_c.clone(),
        );
    });

    let ssid_str = ssid.to_string();
    let main_box_c2 = main_box.clone();
    let sub_label_c2 = sub_label.clone();
    let left_btn_c2 = left_btn.clone();
    let circle_c2 = circle.clone();
    let icon_widget_c2 = icon_widget.clone();
    let popover_c2 = popover.clone();

    connect_btn.connect_clicked(move |_| {
        let password = password_entry.text().to_string();
        let username = username_entry.as_ref().map(|e| e.text().to_string());

        show_connecting_state(&main_box_c2, &ssid_str);

        connect_wifi_async(
            &ssid_str,
            username,
            Some(password),
            sub_label_c2.clone(),
            left_btn_c2.clone(),
            circle_c2.clone(),
            icon_widget_c2.clone(),
            popover_c2.clone(),
        );
    });
}

fn connect_wifi_async(
    ssid: &str,
    username: Option<String>,
    password: Option<String>,
    sub_label: gtk4::Label,
    left_btn: gtk4::Button,
    circle: gtk4::Box,
    icon_widget: gtk4::Image,
    popover: gtk4::Popover,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<bool>();

    let ssid_str = ssid.to_string();
    std::thread::spawn(move || {
        let mut cmd = std::process::Command::new("iwctl");
        
        let mut is_enterprise = false;
        if let Some(user) = username {
            cmd.arg("--username").arg(user);
            is_enterprise = true;
        }
        if let Some(pass) = password {
            if is_enterprise {
                cmd.arg("--password").arg(pass);
            } else {
                cmd.arg("--passphrase").arg(pass);
            }
        }
        
        cmd.args(&["station", "wlan0", "connect", &ssid_str]);
        
        let status = cmd.status();
        let success = status.map(|s| s.success()).unwrap_or(false);
        let _ = tx.send(success);
    });

    let sub_label_c = sub_label.clone();
    let left_btn_c = left_btn.clone();
    let circle_c = circle.clone();
    let icon_widget_c = icon_widget.clone();
    let popover_c = popover.clone();
    let ssid_str2 = ssid.to_string();

    glib::spawn_future_local(async move {
        if let Some(success) = rx.recv().await {
            if success {
                sub_label_c.set_text(&ssid_str2);
                left_btn_c.add_css_class("active");
                circle_c.add_css_class("active");
                let new_img = archvnde_common::icon::get_icon_colored("wifi", 14, "#ffffff");
                if let Some(paintable) = new_img.paintable() {
                    icon_widget_c.set_paintable(Some(&paintable));
                }
                popover_c.popdown();
            } else {
                sub_label_c.set_text("Failed");
                popover_c.popdown();
            }
        }
    });
}

pub fn create_left_box_toggles(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    container.add_css_class("control-left-toggles-box");
    container.set_valign(gtk4::Align::Fill);
    container.set_vexpand(true);

    let wifi_tile = create_wifi_tile(on_popover_toggled);
    let bt_btn = create_tile_row(
        "bluetooth",
        &archvnde_common::i18n::t("control.bluetooth"),
        &archvnde_common::i18n::t("control.not_connected"),
        false,
        "active",
        None::<fn()>,
    );

    container.append(&wifi_tile);
    container.append(&bt_btn);
    container
}

fn is_dnd_active() -> bool {
    // Check dunst
    if let Ok(output) = std::process::Command::new("dunstctl").arg("is-paused").output() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout == "true" {
            return true;
        }
    }
    // Check mako
    if let Ok(output) = std::process::Command::new("makoctl").arg("mode").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("dnd") {
            return true;
        }
    }
    false
}

pub fn create_dnd_tile() -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-dnd-tile");
    btn.set_hexpand(true);
    btn.set_valign(gtk4::Align::Fill);
    btn.set_vexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    main_box.set_valign(gtk4::Align::Center);

    let circle = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    circle.add_css_class("control-icon-circle");

    let is_active_init = is_dnd_active();
    if is_active_init {
        btn.add_css_class("active");
        circle.add_css_class("active");
    }

    let initial_icon = if is_active_init { "bell-off" } else { "bell" };
    let initial_color = if is_active_init { "#ffffff" } else { "rgba(255, 255, 255, 0.7)" };
    let icon_widget = archvnde_common::icon::get_icon_colored(initial_icon, 14, initial_color);
    circle.append(&icon_widget);
    main_box.append(&circle);

    let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
    let title_label = gtk4::Label::new(Some(&archvnde_common::i18n::t("control.dnd")));
    title_label.set_xalign(0.0);
    title_label.add_css_class("tile-title");

    let initial_status = if is_active_init { "control.on" } else { "control.off" };
    let sub_label = gtk4::Label::new(Some(&archvnde_common::i18n::t(initial_status)));
    sub_label.set_xalign(0.0);
    sub_label.add_css_class("tile-subtitle");

    text_box.append(&title_label);
    text_box.append(&sub_label);
    main_box.append(&text_box);

    btn.set_child(Some(&main_box));

    let circle_clone = circle.clone();
    let icon_widget_clone = icon_widget.clone();
    let sub_label_clone = sub_label.clone();

    btn.connect_clicked(move |b| {
        if b.has_css_class("active") {
            b.remove_css_class("active");
            circle_clone.remove_css_class("active");
            sub_label_clone.set_text(&archvnde_common::i18n::t("control.off"));
            let new_img = archvnde_common::icon::get_icon_colored("bell", 14, "rgba(255, 255, 255, 0.7)");
            if let Some(paintable) = new_img.paintable() {
                icon_widget_clone.set_paintable(Some(&paintable));
            }
            let _ = std::process::Command::new("dunstctl").args(&["set-paused", "false"]).spawn();
            let _ = std::process::Command::new("makoctl").args(&["mode", "-r", "dnd"]).spawn();
        } else {
            b.add_css_class("active");
            circle_clone.add_css_class("active");
            sub_label_clone.set_text(&archvnde_common::i18n::t("control.on"));
            let new_img = archvnde_common::icon::get_icon_colored("bell-off", 14, "#ffffff");
            if let Some(paintable) = new_img.paintable() {
                icon_widget_clone.set_paintable(Some(&paintable));
            }
            let _ = std::process::Command::new("dunstctl").args(&["set-paused", "true"]).spawn();
            let _ = std::process::Command::new("makoctl").args(&["mode", "-a", "dnd"]).spawn();
        }
    });

    btn
}

fn is_process_running(name: &str) -> bool {
    if let Ok(output) = std::process::Command::new("pgrep").arg(name).output() {
        return !output.stdout.is_empty();
    }
    false
}

pub fn create_night_light_tile() -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("control-square-tile");
    btn.set_hexpand(true);
    btn.set_valign(gtk4::Align::Fill);
    btn.set_vexpand(true);

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.set_valign(gtk4::Align::Center);
    main_box.set_halign(gtk4::Align::Center);

    let is_active_init = is_process_running("gammastep") || is_process_running("wlsunset");
    if is_active_init {
        btn.add_css_class("active");
    }

    // Use an icon container Box so we can swap the widget on toggle
    let icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    icon_container.set_halign(gtk4::Align::Center);

    let initial_color = if is_active_init { "#ffffff" } else { "rgba(255, 255, 255, 0.8)" };
    let icon_widget = archvnde_common::icon::get_icon_colored("night-light", 16, initial_color);
    icon_container.append(&icon_widget);

    let label = gtk4::Label::new(Some(&archvnde_common::i18n::t("control.night_light")));
    label.add_css_class("control-square-label");
    label.set_halign(gtk4::Align::Center);

    main_box.append(&icon_container);
    main_box.append(&label);
    btn.set_child(Some(&main_box));

    btn.connect_clicked(move |b| {
        if b.has_css_class("active") {
            b.remove_css_class("active");
            if let Some(old) = icon_container.first_child() { icon_container.remove(&old); }
            let new_img = archvnde_common::icon::get_icon_colored("night-light", 16, "rgba(255, 255, 255, 0.8)");
            icon_container.append(&new_img);
            let _ = std::process::Command::new("pkill").arg("-x").arg("gammastep").spawn();
            let _ = std::process::Command::new("pkill").arg("-x").arg("wlsunset").spawn();
        } else {
            b.add_css_class("active");
            if let Some(old) = icon_container.first_child() { icon_container.remove(&old); }
            let new_img = archvnde_common::icon::get_icon_colored("night-light", 16, "#ffffff");
            icon_container.append(&new_img);
            let _ = std::process::Command::new("gammastep")
                .args(&["-O", "4000", "-l", "0:0"])
                .spawn();
            let _ = std::process::Command::new("wlsunset")
                .args(&["-t", "4000", "-T", "6500"])
                .spawn();
        }
    });

    btn
}


pub fn create_control_center_grid(on_popover_toggled: Option<Rc<dyn Fn(bool) + 'static>>) -> gtk4::Grid {
    let grid = gtk4::Grid::new();
    grid.set_row_spacing(10);
    grid.set_column_spacing(10);
    grid.set_row_homogeneous(true);
    grid.set_column_homogeneous(true);

    let left_box = create_left_box_toggles(on_popover_toggled);
    grid.attach(&left_box, 0, 0, 1, 2);

    let dnd_btn = create_dnd_tile();
    grid.attach(&dnd_btn, 1, 0, 1, 1);

    let small_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    small_box.set_homogeneous(true);
    small_box.set_valign(gtk4::Align::Fill);
    small_box.set_vexpand(true);

    let theme_btn = create_small_theme_toggle_tile();
    let night_btn = create_night_light_tile();

    small_box.append(&theme_btn);
    small_box.append(&night_btn);
    grid.attach(&small_box, 1, 1, 1, 1);

    grid
}
