use gtk4::prelude::*;
use super::easing;

/// Animates a widget growing from a tiny point to its target size using frame-synced FrameClock timing.
pub fn genie_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);
    widget.set_size_request(20, 20);

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    widget.add_tick_callback(move |w, clock| {
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_size_request(target_width, target_height);
            w.set_opacity(1.0);
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = easing::ease_out_cubic(t);

        let current_w = (20.0 + (target_width - 20) as f64 * eased) as i32;
        let current_h = (20.0 + (target_height - 20) as f64 * eased) as i32;
        w.set_size_request(current_w, current_h);
        w.set_opacity(t);

        glib::ControlFlow::Continue
    });
}

/// Animates a widget shrinking to a tiny point then calls completion callback. Uses FrameClock timing.
pub fn genie_out<F>(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;

    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, clock| {
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.set_size_request(0, 0);
            w.set_opacity(0.0);
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = easing::ease_out_cubic(t);

        let current_w = (target_width as f64 * (1.0 - eased)).max(20.0) as i32;
        let current_h = (target_height as f64 * (1.0 - eased)).max(20.0) as i32;
        w.set_size_request(current_w, current_h);
        w.set_opacity(1.0 - t);

        glib::ControlFlow::Continue
    });
}
