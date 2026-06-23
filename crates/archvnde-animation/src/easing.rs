/// Common easing functions for smooth animations.

/// Cubic ease-out: fast start, smooth deceleration.
pub fn ease_out_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Cubic ease-in: slow start, fast finish.
pub fn ease_in_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t.powi(3)
}

/// Cubic ease-in-out: smooth start and finish.
pub fn ease_in_out_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t.powi(3)
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// Quart ease-out: slightly snappier than cubic.
pub fn ease_out_quart(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(4)
}

/// Linear (no easing).
pub fn linear(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}
