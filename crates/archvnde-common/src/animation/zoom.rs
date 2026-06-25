use gtk4::prelude::*;
use super::easing;

/// Zoom a widget into view by animating its width request,
/// keeping its height constant at target_height, combined with a fade-in of its children.
pub fn zoom_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(1.0); // Keep container background solid
    widget.set_visible(true);
    widget.set_size_request(0, target_height);

    // Fade out inner child immediately
    if let Some(ref child) = widget.first_child() {
        child.set_opacity(0.0);
    }

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(target_width, target_height);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(1.0);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        // Animate width request
        let current_w = (target_width as f64 * eased) as i32;
        w.set_size_request(current_w, target_height);

        // Fade in child elements during the second half of the animation
        if let Some(ref child) = w.first_child() {
            let child_t = (t - 0.5) * 2.0; // Map [0.5, 1.0] to [0.0, 1.0]
            let child_opacity = child_t.max(0.0).min(1.0);
            child.set_opacity(child_opacity);
        }

        glib::ControlFlow::Continue
    });
}

/// Zoom a widget out of view by animating its width request down to 0,
/// keeping its height constant, combined with a quick fade-out of its children.
pub fn zoom_out(widget: &gtk4::Widget, target_width: i32, duration_ms: u64, hide_after: bool) {
    let start_h = widget.height().max(22); // Target height fallback

    // Immediately start fading out inner child
    if let Some(ref child) = widget.first_child() {
        child.set_opacity(1.0);
    }

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(0, 0);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(0.0);
            }
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        // Animate width request down to 0
        let current_w = (target_width as f64 * (1.0 - eased)) as i32;
        w.set_size_request(current_w, start_h);

        // Fade out child elements in the first half of the animation
        if let Some(ref child) = w.first_child() {
            let child_opacity = (1.0 - t * 2.0).max(0.0).min(1.0);
            child.set_opacity(child_opacity);
        }

        glib::ControlFlow::Continue
    });
}
}
