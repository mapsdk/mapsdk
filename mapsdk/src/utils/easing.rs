pub fn ease_out_cubic(x: f64) -> f64 {
    1.0 - (1.0 - x.clamp(0.0, 1.0)).powi(3)
}
