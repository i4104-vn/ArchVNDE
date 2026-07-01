use gtk4::prelude::*;
use std::rc::Rc;
use std::cell::Cell;

#[derive(Clone, Debug)]
struct AudioDevice {
    name: String,
    description: String,
    is_default: bool,
}

fn is_muted() -> bool {
    if let Ok(output) = std::process::Command::new("wpctl")
        .args(&["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("[MUTED]");
    }
    if let Ok(output) = std::process::Command::new("pactl")
        .args(&["get-sink-mute", "@DEFAULT_SINK@"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("Mute: yes");
    }
    false
}

fn toggle_mute() {
    let _ = std::process::Command::new("wpctl")
        .args(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
        .spawn();
    let _ = std::process::Command::new("pactl")
        .args(&["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
        .spawn();
    let _ = std::process::Command::new("amixer")
        .args(&["set", "Master", "toggle"])
        .spawn();
}

fn get_audio_devices(is_source: bool) -> Vec<AudioDevice> {
    let mut devices = Vec::new();
    let output = std::process::Command::new("wpctl")
        .arg("status")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let mut in_sinks_section = false;
        let mut in_sources_section = false;

        for line in stdout.lines() {
            let line_trimmed = line.trim();
            
            // Detect sections
            if line_trimmed.contains("Sinks:") {
                in_sinks_section = true;
                in_sources_section = false;
                continue;
            } else if line_trimmed.contains("Sources:") {
                in_sinks_section = false;
                in_sources_section = true;
                continue;
            } else if line_trimmed.contains("Devices:") || line_trimmed.contains("Filters:") || line_trimmed.contains("Streams:") || line_trimmed.contains("Settings:") {
                in_sinks_section = false;
                in_sources_section = false;
                continue;
            }

            let active_section = if is_source { in_sources_section } else { in_sinks_section };
            if !active_section {
                continue;
            }

            // Clean tree/border characters from the line
            let clean_line = line.replace('│', "")
                                 .replace('├', "")
                                 .replace('└', "")
                                 .replace('─', "");
            let mut clean_trimmed = clean_line.trim().to_string();
            if clean_trimmed.is_empty() {
                continue;
            }

            let mut is_default = false;
            if clean_trimmed.starts_with('*') {
                is_default = true;
                clean_trimmed = clean_trimmed[1..].trim().to_string();
            }

            // Check if it starts with a number followed by '.'
            if let Some(dot_pos) = clean_trimmed.find('.') {
                let id_str = &clean_trimmed[..dot_pos];
                if id_str.chars().all(|c| c.is_ascii_digit()) {
                    let id = id_str.to_string();
                    let mut desc = clean_trimmed[dot_pos + 1..].trim().to_string();
                    
                    // Remove trailing [vol: ...] or [MUTED]
                    if let Some(bracket_pos) = desc.rfind('[') {
                        desc = desc[..bracket_pos].trim().to_string();
                    }
                    
                    if !id.is_empty() && !desc.is_empty() {
                        devices.push(AudioDevice {
                            name: id,
                            description: desc,
                            is_default,
                        });
                    }
                }
            }
        }
    }
    devices
}

fn populate_audio_menu(popover: &gtk4::Popover, update_mute_btn: Rc<dyn Fn()>) {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    container.add_css_class("audio-menu-popover");
    container.set_size_request(260, -1);

    // --- Output Devices (Sinks) ---
    let out_label = gtk4::Label::new(Some("Output Devices"));
    out_label.add_css_class("audio-menu-section-title");
    out_label.set_xalign(0.0);
    container.append(&out_label);

    let sinks = get_audio_devices(false);
    if sinks.is_empty() {
        let empty = gtk4::Label::new(Some("No output devices found"));
        empty.add_css_class("tile-subtitle");
        container.append(&empty);
    } else {
        for sink in sinks {
            let btn = gtk4::Button::new();
            btn.add_css_class("audio-menu-item-btn");
            if sink.is_default {
                btn.add_css_class("active");
            }
            
            let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            let icon = archvnde_common::icon::get_icon_colored(
                "volume", 14, 
                if sink.is_default { "#ffffff" } else { "rgba(255, 255, 255, 0.5)" }
            );
            let name_label = gtk4::Label::new(Some(&sink.description));
            name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            name_label.set_hexpand(true);
            name_label.set_halign(gtk4::Align::Start);
            
            btn_box.append(&icon);
            btn_box.append(&name_label);

            if sink.is_default {
                let check_label = gtk4::Label::new(Some("✓"));
                check_label.add_css_class("audio-menu-item-check");
                btn_box.append(&check_label);
            }

            btn.set_child(Some(&btn_box));

            let name = sink.name.clone();
            let pop_clone = popover.clone();
            let update_mute_clone = update_mute_btn.clone();
            btn.connect_clicked(move |_| {
                let _ = std::process::Command::new("wpctl")
                    .args(&["set-default", &name])
                    .status();
                populate_audio_menu(&pop_clone, update_mute_clone.clone());
            });
            container.append(&btn);
        }
    }

    // --- Separator ---
    let sep = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    sep.set_margin_top(4);
    sep.set_margin_bottom(4);
    container.append(&sep);

    // --- Input Devices (Sources) ---
    let in_label = gtk4::Label::new(Some("Input Devices"));
    in_label.add_css_class("audio-menu-section-title");
    in_label.set_xalign(0.0);
    container.append(&in_label);

    let sources = get_audio_devices(true);
    let mut input_added = false;
    for source in sources {
        if source.name.contains(".monitor") {
            continue;
        }
        input_added = true;
        let btn = gtk4::Button::new();
        btn.add_css_class("audio-menu-item-btn");
        if source.is_default {
            btn.add_css_class("active");
        }
        
        let btn_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let icon = archvnde_common::icon::get_icon_colored(
            "microphone", 14, 
            if source.is_default { "#ffffff" } else { "rgba(255, 255, 255, 0.5)" }
        );
        let name_label = gtk4::Label::new(Some(&source.description));
        name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name_label.set_hexpand(true);
        name_label.set_halign(gtk4::Align::Start);
        
        btn_box.append(&icon);
        btn_box.append(&name_label);

        if source.is_default {
            let check_label = gtk4::Label::new(Some("✓"));
            check_label.add_css_class("audio-menu-item-check");
            btn_box.append(&check_label);
        }

        btn.set_child(Some(&btn_box));

        let name = source.name.clone();
        let pop_clone = popover.clone();
        let update_mute_clone = update_mute_btn.clone();
        btn.connect_clicked(move |_| {
            let _ = std::process::Command::new("wpctl")
                .args(&["set-default", &name])
                .status();
            populate_audio_menu(&pop_clone, update_mute_clone.clone());
        });
        container.append(&btn);
    }

    if !input_added {
        let empty = gtk4::Label::new(Some("No input devices found"));
        empty.add_css_class("tile-subtitle");
        container.append(&empty);
    }

    popover.set_child(Some(&container));
}

pub fn create_slider_row(
    icon_name: &str,
    initial_val: f64,
    on_popover_toggled: Option<Rc<dyn Fn(bool)>>,
    on_changed: impl Fn(f64) + 'static,
) -> gtk4::Box {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    main_box.add_css_class("control-slider-card");

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    let title_text = match icon_name {
        "volume" => "Volume",
        "brightness" => "Brightness",
        _ => "Slider",
    };
    let title_label = gtk4::Label::new(Some(title_text));
    title_label.add_css_class("control-slider-title");
    title_label.set_xalign(0.0);
    title_label.set_hexpand(true);

    let value_label = gtk4::Label::new(Some(&format!("{:.0}%", initial_val)));
    value_label.add_css_class("control-slider-value");
    value_label.set_xalign(1.0);

    header_box.append(&title_label);
    header_box.append(&value_label);

    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

    // --- Left Icon Widget (Clickable for Volume to Mute/Unmute) ---
    let icon_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    icon_container.set_valign(gtk4::Align::Center);

    let update_mute_icon = {
        let icon_container = icon_container.clone();
        let icon_name = icon_name.to_string();
        Rc::new(move || {
            if icon_name == "volume" {
                if let Some(old) = icon_container.first_child() {
                    icon_container.remove(&old);
                }
                let icon_widget = if is_muted() {
                    archvnde_common::icon::get_system_or_file_icon("audio-volume-muted-symbolic", "image-missing")
                } else {
                    archvnde_common::icon::get_icon("volume", 16)
                };
                icon_widget.add_css_class("slider-icon");
                icon_container.append(&icon_widget);
            }
        })
    };

    if icon_name == "volume" {
        let mute_btn = gtk4::Button::new();
        mute_btn.add_css_class("slider-mute-btn");
        mute_btn.set_child(Some(&icon_container));
        
        let update_mute_icon_clone = update_mute_icon.clone();
        mute_btn.connect_clicked(move |_| {
            toggle_mute();
            update_mute_icon_clone();
        });
        
        row_box.append(&mute_btn);
    } else {
        let icon_widget = archvnde_common::icon::get_icon(icon_name, 16);
        icon_widget.add_css_class("slider-icon");
        icon_container.append(&icon_widget);
        row_box.append(&icon_container);
    }

    // Set initial icon
    update_mute_icon();

    // --- Slider Scale ---
    let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
    scale.set_value(initial_val);
    scale.set_hexpand(true);
    scale.set_draw_value(false);

    // Debounce: prevent spamming system commands on every pixel moved
    let last_source: Rc<Cell<Option<gtk4::glib::SourceId>>> = Rc::new(Cell::new(None));
    let on_changed = Rc::new(on_changed);

    let last_source_clone = last_source.clone();
    scale.connect_value_changed(move |s| {
        let val = s.value();
        value_label.set_text(&format!("{:.0}%", val));

        if let Some(id) = last_source_clone.take() {
            id.remove();
        }

        let on_changed = on_changed.clone();
        let last_source_inner = last_source_clone.clone();
        let new_id = gtk4::glib::timeout_add_local_once(
            std::time::Duration::from_millis(80),
            move || {
                last_source_inner.set(None);
                on_changed(val);
            }
        );
        last_source_clone.set(Some(new_id));
    });

    row_box.append(&scale);

    // --- Right Menu Button (Only for Volume to Switch Devices) ---
    if icon_name == "volume" {
        let menu_btn = gtk4::Button::new();
        menu_btn.add_css_class("slider-menu-btn");
        let menu_icon = archvnde_common::icon::get_system_or_file_icon("go-next-symbolic", "image-missing");
        menu_icon.set_pixel_size(12);
        menu_btn.set_child(Some(&menu_icon));

        let popover = gtk4::Popover::new();
        popover.add_css_class("taskbar-popover");
        popover.set_parent(&menu_btn);
        popover.set_position(gtk4::PositionType::Bottom);
        popover.set_has_arrow(true);

        let popover_clone = popover.clone();
        let update_mute_clone = update_mute_icon.clone();
        let on_popover_toggled_c = on_popover_toggled.clone();
        menu_btn.connect_clicked(move |_| {
            populate_audio_menu(&popover_clone, update_mute_clone.clone());
            popover_clone.popup();
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

        row_box.append(&menu_btn);
    }

    main_box.append(&header_box);
    main_box.append(&row_box);
    main_box
}
