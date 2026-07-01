use gtk4::prelude::*;
use crate::easing;

/// Fade a widget in from fully transparent to fully opaque.
///
/// Uses `Widget::add_tick_callback` for frame-accurate interpolation
/// with an ease-out-cubic curve.
pub fn fade_in(widget: &gtk4::Widget, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(1.0);
            return glib::ControlFlow::Break;
        }
        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        w.set_opacity(easing::ease_out_cubic(t));
        glib::ControlFlow::Continue
    });
}

/// Fade a widget out from fully opaque to fully transparent,
/// then optionally hide it.
pub fn fade_out(widget: &gtk4::Widget, duration_ms: u64, hide_after: bool) {
    let start_opacity = widget.opacity();
    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(0.0);
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }
        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);
        w.set_opacity(start_opacity * (1.0 - eased));
        glib::ControlFlow::Continue
    });
}
