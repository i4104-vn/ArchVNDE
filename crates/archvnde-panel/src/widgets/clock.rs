use gtk4::prelude::*;

/// Creates and returns a dynamic clock label widget that updates every second.
pub fn create_clock_widget() -> gtk4::Label {
    let clock_label = gtk4::Label::new(None);
    clock_label.add_css_class("panel-clock");
    clock_label.set_hexpand(true);

    let update_clock = {
        let clock_label = clock_label.clone();
        move || {
            let now = chrono::Local::now();
            let time_str = format!("26°C  |  {}", now.format("%a %b %d  |  %I:%M %p").to_string().to_uppercase());
            clock_label.set_text(&time_str);
            glib::ControlFlow::Continue
        }
    };
    update_clock(); // Run initially
    glib::timeout_add_local(std::time::Duration::from_secs(1), update_clock);

    clock_label
}
