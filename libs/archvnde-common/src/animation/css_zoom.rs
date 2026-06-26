use gtk4::prelude::*;

/// macOS-style zoom-in: adds the `.zoom-visible` CSS class to the widget.
/// The actual animation is driven by CSS transitions on `transform` and `opacity`.
///
/// The widget's CSS should define a default hidden state (`scale(0.85)` + `opacity(0)`)
/// and a `.zoom-visible` state (`scale(1)` + `opacity(1)`) with appropriate transitions.
pub fn css_zoom_in(widget: &gtk4::Widget) {
    widget.add_css_class("zoom-visible");
}

/// macOS-style zoom-out: removes the `.zoom-visible` CSS class and waits
/// for the CSS transition to finish before invoking the `on_complete` callback.
///
/// `duration_ms` should match the CSS transition duration so the callback fires
/// after the visual transition is complete.
pub fn css_zoom_out_cb<F>(widget: &gtk4::Widget, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    widget.remove_css_class("zoom-visible");
    glib::timeout_add_local_once(
        std::time::Duration::from_millis(duration_ms),
        move || {
            on_complete();
        },
    );
}
