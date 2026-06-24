use gtk4::prelude::*;

/// Creates and returns a workspace switcher container containing 4 workspace buttons.
pub fn create_workspace_switcher() -> gtk4::Box {
    let workspace_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    workspace_box.add_css_class("workspace-box");

    let mut workspace_buttons = Vec::new();
    for i in 1..=4 {
        let btn = gtk4::Button::new();
        btn.add_css_class("workspace-dot");
        btn.set_valign(gtk4::Align::Center);
        btn.set_halign(gtk4::Align::Center);
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

    workspace_box
}
