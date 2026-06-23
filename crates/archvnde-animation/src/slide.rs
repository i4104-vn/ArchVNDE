use gtk4::prelude::*;
use crate::easing;

/// Direction for slide animations.
pub enum SlideDirection {
    /// Slide downward (from above).
    Down,
    /// Slide upward (from below).
    Up,
}

/// Slide a widget into view by animating its top or bottom margin,
/// combined with a fade-in for a polished entrance.
///
/// `distance_px` — how many pixels the widget travels during the slide.
pub fn slide_in(widget: &gtk4::Widget, direction: SlideDirection, distance_px: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);

    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();

    // Offset the widget before animation starts
    match direction {
        SlideDirection::Down => widget.set_margin_top(original_margin_top - distance_px),
        SlideDirection::Up => widget.set_margin_bottom(original_margin_bottom - distance_px),
    }

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(1.0);
            match direction {
                SlideDirection::Down => w.set_margin_top(original_margin_top),
                SlideDirection::Up => w.set_margin_bottom(original_margin_bottom),
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_quart(t);

        w.set_opacity(eased);
        match direction {
            SlideDirection::Down => {
                let offset = original_margin_top - distance_px;
                let current = offset + (distance_px as f64 * eased) as i32;
                w.set_margin_top(current);
            }
            SlideDirection::Up => {
                let offset = original_margin_bottom - distance_px;
                let current = offset + (distance_px as f64 * eased) as i32;
                w.set_margin_bottom(current);
            }
        }
        glib::ControlFlow::Continue
    });
}
