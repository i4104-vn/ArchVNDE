use gtk4::prelude::*;
use super::easing;

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
        let eased = easing::ease_out_cubic(t);

        w.set_opacity(eased.min(1.0).max(0.0));
        glib::ControlFlow::Continue
    });
}

pub fn fade_out_cb<F>(
    widget: &gtk4::Widget,
    duration_ms: u64,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    let start_opacity = widget.opacity();
    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(0.0);
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

        w.set_opacity((start_opacity * (1.0 - eased)).min(1.0).max(0.0));
        glib::ControlFlow::Continue
    });
}
