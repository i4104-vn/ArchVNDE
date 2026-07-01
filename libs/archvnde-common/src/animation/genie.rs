use gtk4::prelude::*;
use super::easing;

pub fn genie_in(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);
    widget.set_size_request(20, 20);

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(target_width, target_height);
            w.set_opacity(1.0);
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        let current_w = (20.0 + (target_width - 20) as f64 * eased) as i32;
        let current_h = (20.0 + (target_height - 20) as f64 * eased) as i32;
        w.set_size_request(current_w, current_h);
        w.set_opacity(t);

        glib::ControlFlow::Continue
    });
}

pub fn genie_out<F>(widget: &gtk4::Widget, target_width: i32, target_height: i32, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_size_request(0, 0);
            w.set_opacity(0.0);
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        let current_w = (target_width as f64 * (1.0 - eased)).max(20.0) as i32;
        let current_h = (target_height as f64 * (1.0 - eased)).max(20.0) as i32;
        w.set_size_request(current_w, current_h);
        w.set_opacity(1.0 - t);

        glib::ControlFlow::Continue
    });
}
