use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

pub fn create_visualizer() -> (gtk4::Box, Vec<gtk4::Box>) {
    let visualizer_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
    visualizer_box.add_css_class("notch-visualizer");
    visualizer_box.set_valign(gtk4::Align::Center);

    let mut bars = Vec::new();
    for _ in 0..4 {
        let bar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        bar.add_css_class("visualizer-bar");
        bar.set_size_request(2, 2);
        bar.set_valign(gtk4::Align::End);
        visualizer_box.append(&bar);
        bars.push(bar);
    }
    (visualizer_box, bars)
}

pub fn start_visualizer_animation(bars: Vec<gtk4::Box>, is_playing: Rc<Cell<bool>>) {
    if bars.is_empty() {
        return;
    }
    let start_time = std::cell::Cell::new(0i64);
    bars[0].add_tick_callback(move |_w, clock| {
        if is_playing.get() {
            let now = clock.frame_time();
            if start_time.get() == 0 {
                start_time.set(now);
            }
            let elapsed_sec = (now - start_time.get()) as f64 / 1_000_000.0;
            for (i, bar) in bars.iter().enumerate() {
                let speed = 15.0;
                let phase = i as f64 * 1.5;
                let val = (((elapsed_sec * speed + phase)).sin() * 5.0 + 7.0) as i32;
                bar.set_size_request(2, val.max(2).min(12));
            }
        } else {
            start_time.set(0);
            for bar in &bars {
                bar.set_size_request(2, 2);
            }
        }
        glib::ControlFlow::Continue
    });
}
