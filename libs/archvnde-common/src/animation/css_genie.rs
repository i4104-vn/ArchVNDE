use gtk4::prelude::*;
use super::easing;

/// Triggers a Rust genie-in animation by animating size request and opacity.
/// This bypasses CSS transitions completely for a smooth 120Hz rendering loop.
pub fn css_genie_in(widget: &gtk4::Widget) {
    widget.set_overflow(gtk4::Overflow::Hidden);
    
    let (_, natural_size) = widget.preferred_size();
    let target_width = if natural_size.width > 20 { natural_size.width } else { 360 };
    let target_height = if natural_size.height > 20 { natural_size.height } else { 360 };

    widget.set_opacity(0.0);
    widget.set_visible(true);
    widget.set_size_request(20, 20);

    let start_time = std::cell::Cell::new(0i64);
    let duration_ms = 400u64;
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

/// Triggers a Rust genie-out animation by animating size request and opacity to 0.
pub fn css_genie_out<F>(widget: &gtk4::Widget, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    widget.set_overflow(gtk4::Overflow::Hidden);
    
    let w = widget.width().max(20);
    let h = widget.height().max(20);

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;
    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w_widget, clock| {
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w_widget.set_size_request(0, 0);
            w_widget.set_opacity(0.0);
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = easing::ease_out_cubic(t);

        let current_w = (w as f64 * (1.0 - eased)).max(20.0) as i32;
        let current_h = (h as f64 * (1.0 - eased)).max(20.0) as i32;
        w_widget.set_size_request(current_w, current_h);
        w_widget.set_opacity(1.0 - t);

        glib::ControlFlow::Continue
    });
}
