use gtk4::prelude::*;
use super::easing;

/// Triggers a GPU-accelerated genie-in animation by animating CSS scale and opacity in Rust.
/// This bypasses CSS transitions completely for a smooth 120Hz rendering loop.
pub fn css_genie_in(widget: &gtk4::Widget) {
    let class_name = widget
        .css_classes()
        .first()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "animating-widget".to_string());

    let provider = gtk4::CssProvider::new();
    widget.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    widget.set_opacity(0.0);
    widget.set_visible(true);

    let start_time = std::cell::Cell::new(0i64);
    let duration_ms = 400u64; // Gentle 400ms duration
    let dur_us = duration_ms as i64 * 1000;

    let provider_clone = provider.clone();

    widget.add_tick_callback(move |w, clock| {
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            // Remove the provider to clean up styling when finished
            w.style_context().remove_provider(&provider_clone);
            // Ensure final style state is clean (scale 1, opacity 1)
            w.set_opacity(1.0);
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = easing::ease_out_cubic(t);

        let scale = eased;
        let opacity = t;

        let css = format!(
            ".{} {{ transform: scale({}); transform-origin: top center; opacity: {}; }}",
            class_name, scale, opacity
        );
        provider_clone.load_from_string(&css);

        glib::ControlFlow::Continue
    });
}

/// Triggers a GPU-accelerated genie-out animation by animating CSS scale and opacity to 0 in Rust.
pub fn css_genie_out<F>(widget: &gtk4::Widget, duration_ms: u64, on_complete: F)
where
    F: FnOnce() + 'static,
{
    let class_name = widget
        .css_classes()
        .first()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "animating-widget".to_string());

    let provider = gtk4::CssProvider::new();
    widget.style_context().add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let start_time = std::cell::Cell::new(0i64);
    let dur_us = duration_ms as i64 * 1000;
    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));
    let provider_clone = provider.clone();

    widget.add_tick_callback(move |w, clock| {
        let now = clock.frame_time();
        if start_time.get() == 0 {
            start_time.set(now);
        }
        let elapsed_us = now - start_time.get();
        if elapsed_us >= dur_us {
            w.style_context().remove_provider(&provider_clone);
            w.set_opacity(0.0);
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed_us as f64 / dur_us as f64;
        let eased = easing::ease_out_cubic(t);

        let scale = (1.0 - eased).max(0.0);
        let opacity = (1.0 - t).max(0.0);

        let css = format!(
            ".{} {{ transform: scale({}); transform-origin: top center; opacity: {}; }}",
            class_name, scale, opacity
        );
        provider_clone.load_from_string(&css);

        glib::ControlFlow::Continue
    });
}
