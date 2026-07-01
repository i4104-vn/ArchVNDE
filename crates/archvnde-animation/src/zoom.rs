use gtk4::prelude::*;
use super::easing;

/// Zoom a widget into view by animating its width request,
/// keeping its height constant at target_height, combined with a fade-in.
/// At the end of the animation, the size request is reset to -1.
pub fn zoom_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);
    widget.set_size_request(0, target_height);

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(1.0);
            w.set_size_request(-1, -1);
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        w.set_opacity(eased.min(1.0).max(0.0));

        let current_w = (target_width as f64 * eased) as i32;
        w.set_size_request(current_w, target_height);

        glib::ControlFlow::Continue
    });
}

/// Zoom a widget out of view by animating its width request down to 0,
/// keeping its height constant at the starting height, combined with a fade-out.
/// Optionally hides the widget at the end.
pub fn zoom_out(widget: &gtk4::Widget, duration_ms: u64, hide_after: bool) {
    let start_opacity = widget.opacity();
    let start_w = widget.width();
    let start_h = widget.height();

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(0.0);
            w.set_size_request(0, 0);
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        w.set_opacity((start_opacity * (1.0 - eased)).min(1.0).max(0.0));

        let current_w = (start_w as f64 * (1.0 - eased)) as i32;
        w.set_size_request(current_w, start_h);

        glib::ControlFlow::Continue
    });
}
