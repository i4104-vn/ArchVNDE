use gtk4::prelude::*;
use super::easing;

pub fn island_zoom_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(1.0);
    widget.set_visible(true);
    widget.set_size_request(target_height, target_height);

    if let Some(ref child) = widget.first_child() {
        child.set_opacity(0.0);
    }

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(target_width, target_height);
            w.set_opacity(1.0);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(1.0);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        w.set_opacity(1.0);

        let eased_w = easing::ease_out_cubic(t);
        let current_w = target_height + ((target_width - target_height) as f64 * eased_w) as i32;
        w.set_size_request(current_w, target_height);

        if let Some(ref child) = w.first_child() {
            let child_t = (t - 0.5) * 2.0;
            let child_opacity = child_t.max(0.0).min(1.0);
            child.set_opacity(child_opacity);
        }

        glib::ControlFlow::Continue
    });
}

pub fn island_zoom_out(widget: &gtk4::Widget, target_width: i32, duration_ms: u64, hide_after: bool) {
    let start_h = widget.height().max(22);

    if let Some(ref child) = widget.first_child() {
        child.set_opacity(1.0);
    }

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(0, 0);
            w.set_opacity(0.0);
            if let Some(ref child) = w.first_child() {
                child.set_opacity(0.0);
            }
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let capsule_opacity = (1.0 - (t - 0.5) * 2.0).max(0.0).min(1.0);
        w.set_opacity(capsule_opacity);

        let eased_w = easing::ease_out_cubic(t);
        let current_w = start_h + ((target_width - start_h) as f64 * (1.0 - eased_w)) as i32;
        w.set_size_request(current_w, start_h);

        if let Some(ref child) = w.first_child() {
            let child_opacity = (1.0 - t * 2.0).max(0.0).min(1.0);
            child.set_opacity(child_opacity);
        }

        glib::ControlFlow::Continue
    });
}
