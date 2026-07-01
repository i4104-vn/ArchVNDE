use gtk4::prelude::*;

pub fn css_genie_in(widget: &gtk4::Widget) {
    widget.add_tick_callback(|w, _clock| {
        w.add_css_class("genie-visible");
        glib::ControlFlow::Break
    });
}

pub fn css_genie_out<F>(widget: &gtk4::Widget, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    widget.remove_css_class("genie-visible");
    glib::timeout_add_local_once(
        std::time::Duration::from_millis(duration_ms),
        move || {
            on_complete();
        },
    );
}
