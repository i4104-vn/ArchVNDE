use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

mod calendar_window;
mod notifications;

/// Creates and returns a clock button widget that updates every second and
/// spawns a centered, glassmorphic calendar popup dropdown when clicked.
pub fn create_clock_widget(
    app: &gtk4::Application,
    quick_settings_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    calendar_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
    launcher_window: Rc<RefCell<Option<gtk4::ApplicationWindow>>>,
) -> gtk4::Button {
    let clock_button = gtk4::Button::new();
    clock_button.add_css_class("panel-clock-btn");

    let clock_label = gtk4::Label::new(None);
    clock_label.add_css_class("panel-clock");
    clock_button.set_child(Some(&clock_label));

    let update_clock = {
        let clock_label = clock_label.clone();
        move || {
            let now = chrono::Local::now();
            let time_str = format!(
                "{}   {}",
                now.format("%d/%m").to_string(),
                now.format("%I:%M %p").to_string().to_uppercase()
            );
            clock_label.set_text(&time_str);
            glib::ControlFlow::Continue
        }
    };
    update_clock(); // Run initially
    glib::timeout_add_local(std::time::Duration::from_secs(1), update_clock);

    let cw_clone = calendar_window.clone();
    let qsw_clone = quick_settings_window.clone();
    let lw_clone = launcher_window.clone();
    let app_clone = app.clone();

    clock_button.connect_clicked(move |_| {
        // Close Quick Settings window if open
        let qs_win = qsw_clone.borrow().clone();
        if let Some(win) = qs_win {
            win.close();
        }

        // Close Launcher if open
        let launch_win = lw_clone.borrow().clone();
        if let Some(win) = launch_win {
            win.close();
        }

        let existing = cw_clone.borrow().clone();
        if let Some(existing_window) = existing {
            existing_window.close();
        } else {
            let window = calendar_window::show_calendar_window(&app_clone, cw_clone.clone());
            if let Ok(mut borrow) = cw_clone.try_borrow_mut() {
                *borrow = Some(window);
            }
        }
    });

    clock_button
}
