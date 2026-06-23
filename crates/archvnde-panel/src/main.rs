use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn main() {
    // Set up logging or debug output
    println!("Starting ArchVNDE Panel...");

    let application = gtk4::Application::new(
        Some("org.archvnde.panel"),
        Default::default(),
    );

    application.connect_activate(|app| {
        // Initialize style provider
        archvnde_common::init_theme();

        let window = gtk4::ApplicationWindow::new(app);

        // Initialize layer shell properties on the window
        window.init_layer_shell();

        // Assign to the Top layer so it renders above normal windows
        window.set_layer(Layer::Top);

        // Set exclusive zone so other maximized windows don't overlap it
        window.set_exclusive_zone(40);

        // Anchor it to the top, left, and right edges of the screen
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);

        // Set default height of the panel
        window.set_default_size(0, 40);

        // Add styling class
        window.add_css_class("panel-window");

        // Add placeholder UI widget
        let box_layout = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        box_layout.set_margin_start(15);
        box_layout.set_margin_end(15);

        let title_label = gtk4::Label::new(Some("ArchVNDE"));
        title_label.add_css_class("panel-title");

        // Workspace Switcher Widget
        let workspace_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
        workspace_box.add_css_class("workspace-box");

        let mut workspace_buttons = Vec::new();
        for i in 1..=4 {
            let btn = gtk4::Button::with_label(&format!("WS {}", i));
            btn.add_css_class("workspace-button");
            if i == 1 {
                btn.add_css_class("active");
            }
            workspace_buttons.push(btn);
        }

        for (idx, btn) in workspace_buttons.iter().enumerate() {
            let buttons_clone = workspace_buttons.clone();
            let idx_val = idx + 1;
            btn.connect_clicked(move |_| {
                println!("Workspace Switcher clicked: Switch to WS {}", idx_val);
                for (j, b) in buttons_clone.iter().enumerate() {
                    if j == idx {
                        b.add_css_class("active");
                    } else {
                        b.remove_css_class("active");
                    }
                }
            });
            workspace_box.append(btn);
        }
        
        // Dynamic Clock Widget
        let clock_label = gtk4::Label::new(None);
        clock_label.add_css_class("panel-clock");
        clock_label.set_hexpand(true);

        let update_clock = {
            let clock_label = clock_label.clone();
            move || {
                let now = chrono::Local::now();
                let time_str = now.format("%a %b %d | %I:%M %p").to_string().to_uppercase();
                clock_label.set_text(&time_str);
                glib::ControlFlow::Continue
            }
        };
        update_clock(); // Run initially
        glib::timeout_add_local(std::time::Duration::from_secs(1), update_clock);

        let status_label = gtk4::Label::new(Some("Wi-Fi | 100%"));

        box_layout.append(&title_label);
        box_layout.append(&workspace_box);
        box_layout.append(&clock_label);
        box_layout.append(&status_label);

        window.set_child(Some(&box_layout));

        // Display the window on Wayland
        window.present();
    });

    // Run the GTK loop (this blocks until application exits)
    application.run();
}
