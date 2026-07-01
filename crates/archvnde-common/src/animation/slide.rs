use gtk4::prelude::*;
use super::easing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Down,
    Up,
    Left,
    Right,
}

pub fn slide_in(widget: &gtk4::Widget, direction: SlideDirection, distance_px: i32, duration_ms: u64) {
    widget.set_opacity(0.0);
    widget.set_visible(true);

    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_start = widget.margin_start();
    let original_margin_end = widget.margin_end();

    match direction {
        SlideDirection::Down => widget.set_margin_top(original_margin_top - distance_px),
        SlideDirection::Up => widget.set_margin_bottom(original_margin_bottom - distance_px),
        SlideDirection::Right => widget.set_margin_start(original_margin_start - distance_px),
        SlideDirection::Left => widget.set_margin_end(original_margin_end - distance_px),
    }

    let start_cell = std::cell::Cell::new(None);
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        if start_cell.get().is_none() {
            start_cell.set(Some(std::time::Instant::now()));
        }
        let start = start_cell.get().unwrap();
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(1.0);
            match direction {
                SlideDirection::Down => w.set_margin_top(original_margin_top),
                SlideDirection::Up => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Right => w.set_margin_start(original_margin_start),
                SlideDirection::Left => w.set_margin_end(original_margin_end),
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
                let offset = original_margin_start - distance_px;
                let current = offset + (distance_px as f64 * eased) as i32;
                w.set_margin_start(current);
            }
            SlideDirection::Left => {
                let offset = original_margin_end - distance_px;
                let current = offset + (distance_px as f64 * eased) as i32;
                w.set_margin_end(current);
            }
        }
        glib::ControlFlow::Continue
    });
}

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
    let original_margin_start = widget.margin_start();
    let original_margin_end = widget.margin_end();

    let start_cell = std::cell::Cell::new(None);
    let dur = std::time::Duration::from_millis(duration_ms);

    widget.add_tick_callback(move |w, _clock| {
        if start_cell.get().is_none() {
            start_cell.set(Some(std::time::Instant::now()));
        }
        let start = start_cell.get().unwrap();
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(0.0);
            match direction {
                SlideDirection::Down => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Up => w.set_margin_top(original_margin_top),
                SlideDirection::Right => w.set_margin_end(original_margin_end),
                SlideDirection::Left => w.set_margin_start(original_margin_start),
            }
            if hide_after {
                w.set_visible(false);
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

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
                let current = original_margin_end - (distance_px as f64 * eased) as i32;
                w.set_margin_end(current);
            }
            SlideDirection::Left => {
                let current = original_margin_start - (distance_px as f64 * eased) as i32;
                w.set_margin_start(current);
            }
        }
        glib::ControlFlow::Continue
    });
}

pub fn slide_out_cb<F>(
    widget: &gtk4::Widget,
    direction: SlideDirection,
    distance_px: i32,
    duration_ms: u64,
    hide_after: bool,
    on_complete: F,
) where
    F: FnOnce() + 'static,
{
    let start_opacity = widget.opacity();
    let original_margin_top = widget.margin_top();
    let original_margin_bottom = widget.margin_bottom();
    let original_margin_start = widget.margin_start();
    let original_margin_end = widget.margin_end();

    let start_cell = std::cell::Cell::new(None);
    let dur = std::time::Duration::from_millis(duration_ms);

    let on_complete_opt = std::cell::RefCell::new(Some(on_complete));

    widget.add_tick_callback(move |w, _clock| {
        if start_cell.get().is_none() {
            start_cell.set(Some(std::time::Instant::now()));
        }
        let start = start_cell.get().unwrap();
        let elapsed = start.elapsed();
        if elapsed >= dur {
            w.set_opacity(0.0);
            match direction {
                SlideDirection::Down => w.set_margin_bottom(original_margin_bottom),
                SlideDirection::Up => w.set_margin_top(original_margin_top),
                SlideDirection::Right => w.set_margin_end(original_margin_end),
                SlideDirection::Left => w.set_margin_start(original_margin_start),
            }
            if hide_after {
                w.set_visible(false);
            }
            if let Some(cb) = on_complete_opt.borrow_mut().take() {
                cb();
            }
            return glib::ControlFlow::Break;
        }

        let t = elapsed.as_secs_f64() / dur.as_secs_f64();
        let eased = easing::ease_out_cubic(t);

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
                let current = original_margin_end - (distance_px as f64 * eased) as i32;
                w.set_margin_end(current);
            }
            SlideDirection::Left => {
                let current = original_margin_start - (distance_px as f64 * eased) as i32;
                w.set_margin_start(current);
            }
        }
        glib::ControlFlow::Continue
    });
}

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

