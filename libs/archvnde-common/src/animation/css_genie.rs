use gtk4::prelude::*;

/// Triggers a CSS genie-in animation by adding the `genie-visible` class after 2 render frames.
/// Waiting 2 frames ensures the initial hidden state (scale 0, opacity 0) is fully composited
/// before the CSS transition begins, preventing the animation from being skipped or stuttering.
pub fn css_genie_in(widget: &gtk4::Widget) {
    let frame_count = std::cell::Cell::new(0u32);
    widget.add_tick_callback(move |w, _clock| {
        let count = frame_count.get() + 1;
        frame_count.set(count);
        if count >= 2 {
            w.add_css_class("genie-visible");
            return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });
}

/// Triggers a CSS genie-out animation by removing the `genie-visible` class and scheduling
/// a delayed callback for cleanup after the CSS transition completes.
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
