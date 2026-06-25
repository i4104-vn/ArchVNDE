use gtk4::prelude::*;
use super::easing;

/// Smoothly scale/genie a widget into view by animating both width and height requests
/// from a tiny size (e.g. 20x20) to target size, combined with a fade-in.
pub fn genie_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);
    widget.set_size_request(20, 20);

    let start_cell = std::cell::Cell::new(None);
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        if start_cell.get().is_none() {
            start_cell.set(Some(std::time::Instant::now()));
        }
        let start = start_cell.get().unwrap();
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(target_width, target_height);
            w.set_opacity(1.0);
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        // Animate width and height requests
        let current_w = (20.0 + (target_width - 20) as f64 * eased) as i32;
        let current_h = (20.0 + (target_height - 20) as f64 * eased) as i32;
        w.set_size_request(current_w, current_h);
        w.set_opacity(t);

        glib::ControlFlow::Continue
    });
}

/// Smoothly scale/genie a widget out of view by animating both width and height requests
/// from target size down to a tiny size (e.g. 20x20), combined with a fade-out.
/// When finished, it invokes the on_complete callback.
pub fn genie_out<F>(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    let start_cell = std::cell::Cell::new(None);
    let dur = std::time::Duration::from_millis(duration_ms);

    let mut on_complete_opt = Some(on_complete);

    widget.add_tick_callback(move |w, _clock| {
        if start_cell.get().is_none() {
            start_cell.set(Some(std::time::Instant::now()));
        }
        let start = start_cell.get().unwrap();
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(0, 0);
            w.set_opacity(0.0);
            if let Some(cb) = on_complete_opt.take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        // Animate width and height requests down
        let current_w = (target_width as f64 * (1.0 - eased)).max(20.0) as i32;
        let current_h = (target_height as f64 * (1.0 - eased)).max(20.0) as i32;
        w.set_size_request(current_w, current_h);
        w.set_opacity(1.0 - t);

        glib::ControlFlow::Continue
    });
}
