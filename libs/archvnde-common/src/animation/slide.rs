use gtk4::prelude::*;
use super::easing;

/// Direction for slide animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    /// Slide downward (moves Down).
    Down,
    /// Slide upward (moves Up).
    Up,
    /// Slide leftward (moves Left).
    Left,
    /// Slide rightward (moves Right).
    Right,
}

/// Slide a widget into view by animating its margin,
/// combined with a fade-in for a polished entrance.
///
/// `distance_px` — how many pixels the widget travels during the slide.
pub fn slide_in(widget: &gtk4::Widget, direction: SlideDirection, distance_px: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);

    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_left = widget.margin_left();
    let original_margin_right = widget.margin_right();

    // Offset the widget before animation starts
    match direction {
        SlideDirection::Down => widget.set_margin_top(original_margin_top - distance_px),
        SlideDirection::Up => widget.set_margin_bottom(original_margin_bottom - distance_px),
        SlideDirection::Right => widget.set_margin_left(original_margin_left - distance_px),
        SlideDirection::Left => widget.set_margin_right(original_margin_right - distance_px),
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
                SlideDirection::Right => w.set_margin_left(original_margin_left),
                SlideDirection::Left => w.set_margin_right(original_margin_right),
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_back(t);

        w.set_opacity(eased.min(1.0).max(0.0));
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
            SlideDirection::Right => {
                let offset = original_margin_left - distance_px;
                let current = offset + (distance_px as f64 * eased) as i32;
                w.set_margin_left(current);
            }
            SlideDirection::Left => {
                let offset = original_margin_right - distance_px;
                let current = offset + (distance_px as f64 * eased) as i32;
                w.set_margin_right(current);
            }
        }
        glib::ControlFlow::Continue
    });
}

/// Slide a widget out of view by animating its margin,
/// combined with a fade-out. Optionally hides the widget at the end.
pub fn slide_out(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    hide_after: bool,
) {
    let start_opacity = widget.opacity();
    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_left = widget.margin_left();
    let original_margin_right = widget.margin_right();

    let start = std::time::Instant::now();
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(0.0);
            match direction {
                SlideDirection::Down => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Up => w.set_margin_top(original_margin_top),
                SlideDirection::Right => w.set_margin_right(original_margin_right),
                SlideDirection::Left => w.set_margin_left(original_margin_left),
            }
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t); // Smooth deceleration on exit

        w.set_opacity((start_opacity * (1.0 - eased)).min(1.0).max(0.0));
        match direction {
            SlideDirection::Down => {
                let current = original_margin_bottom - (distance_px as f64 * eased) as i32;
                w.set_margin_bottom(current);
            }
            SlideDirection::Up => {
                let current = original_margin_top - (distance_px as f64 * eased) as i32;
                w.set_margin_top(current);
            }
            SlideDirection::Right => {
                let current = original_margin_right - (distance_px as f64 * eased) as i32;
                w.set_margin_right(current);
            }
            SlideDirection::Left => {
                let current = original_margin_left - (distance_px as f64 * eased) as i32;
                w.set_margin_left(current);
            }
        }
        glib::ControlFlow::Continue
    });
}
