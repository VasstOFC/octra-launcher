use std::f32::consts::PI;

pub const PAGE_TRANSITION_MS: f32 = 200.0;
pub const PROGRESS_LERP: f32 = 0.12;

pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

pub fn pulse(time: f64, speed: f64) -> f32 {
    ((time * speed * 2.0 * PI as f64).sin() * 0.5 + 0.5) as f32
}

pub fn advance_t(current: f32, dt_sec: f32, duration_ms: f32) -> f32 {
    if duration_ms <= 0.0 {
        return 1.0;
    }
    (current + dt_sec * 1000.0 / duration_ms).min(1.0)
}

pub fn page_offset(t: f32) -> f32 {
    (1.0 - ease_out_cubic(t)) * 12.0
}

pub fn page_alpha(t: f32) -> f32 {
    ease_out_cubic(t)
}
