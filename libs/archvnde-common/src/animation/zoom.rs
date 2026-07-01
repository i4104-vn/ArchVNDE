use gtk4::prelude::*;
use super::easing;

/// Zoom a widget into view by animating its width request,
/// keeping its height constant at target_height, combined with a fade-in of its children.
pub fn zoom_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(1.0); // Keep container background solid
    widget.set_visible(true);
    widget.set_size_request(0, 0); // Start at 0, 0

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

        // Easing for height (expands faster, within first 60%)
        let h_t = (t / 0.6).min(1.0);
        let eased_h = easing::ease_out_cubic(h_t);
        let current_h = (target_height as f64 * eased_h) as i32;

        // Easing for width (starts after 20% of duration)
        let w_t = if t < 0.2 {
            0.0
        } else {
            ((t - 0.2) / 0.8).min(1.0)
        };
        let eased_w = easing::ease_out_cubic(w_t);
        let current_w = (target_width as f64 * eased_w) as i32;

        w.set_size_request(current_w, current_h);

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

        // Width factor: shrinks to 0 faster (e.g. within 75% of duration)
        let w_t = (t / 0.75).min(1.0);
        let eased_w = easing::ease_out_cubic(w_t);
        let current_w = (target_width as f64 * (1.0 - eased_w)) as i32;

        // Height factor: starts shrinking after 25% of duration
        let h_t = if t < 0.25 {
            0.0
        } else {
            ((t - 0.25) / 0.75).min(1.0)
        };
        let eased_h = easing::ease_out_cubic(h_t);
        let current_h = (start_h as f64 * (1.0 - eased_h)) as i32;

        w.set_size_request(current_w, current_h);

        // Fade out child elements in the first half of the animation
        if let Some(ref child) = w.first_child() {
            let child_opacity = (1.0 - t * 2.0).max(0.0).min(1.0);
            child.set_opacity(child_opacity);
        }

        glib::ControlFlow::Continue
    });
}
