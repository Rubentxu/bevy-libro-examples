// Capítulo 14E. Shaders — UV math, color operations
// GPU-independent shader math concepts.

/// UV coordinate wrapping modes
pub fn wrap_repeat(uv: f32) -> f32 {
    uv.rem_euclid(1.0)
}

pub fn wrap_clamp(uv: f32) -> f32 {
    uv.clamp(0.0, 1.0)
}

pub fn wrap_mirror(uv: f32) -> f32 {
    let t = uv.rem_euclid(2.0);
    if t > 1.0 { 2.0 - t } else { t }
}

/// Blend modes for combining two colors
pub fn blend_normal(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let alpha = b.3;
    let inv = 1.0 - alpha;
    (
        a.0 * inv + b.0 * alpha,
        a.1 * inv + b.1 * alpha,
        a.2 * inv + b.2 * alpha,
        a.3,
    )
}

pub fn blend_multiply(a: (f32, f32, f32), b: (f32, f32, f32)) -> (f32, f32, f32) {
    (a.0 * b.0, a.1 * b.1, a.2 * b.2)
}

pub fn blend_screen(a: (f32, f32, f32), b: (f32, f32, f32)) -> (f32, f32, f32) {
    (
        1.0 - (1.0 - a.0) * (1.0 - b.0),
        1.0 - (1.0 - a.1) * (1.0 - b.1),
        1.0 - (1.0 - a.2) * (1.0 - b.2),
    )
}

pub fn blend_overlay(a: (f32, f32, f32), b: (f32, f32, f32)) -> (f32, f32, f32) {
    let overlay_channel = |a: f32, b: f32| {
        if a < 0.5 { 2.0 * a * b } else { 1.0 - 2.0 * (1.0 - a) * (1.0 - b) }
    };
    (overlay_channel(a.0, b.0), overlay_channel(a.1, b.1), overlay_channel(a.2, b.2))
}

/// HSV to RGB conversion
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h / 60.0) % 6.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (r1 + m, g1 + m, b1 + m)
}

/// RGB to HSV conversion
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max > 0.0 { delta / max } else { 0.0 };

    let h = if delta < 0.001 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

/// Noise function for procedural texture generation
pub fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let n = (x as u32).wrapping_add(seed.wrapping_mul(374761393))
        .wrapping_add((y as u32).wrapping_mul(668265263));
    let n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n as f32 / u32::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_repeat_cycles() {
        assert!((wrap_repeat(1.5) - 0.5).abs() < 0.001);
        assert!((wrap_repeat(2.25) - 0.25).abs() < 0.001);
    }

    #[test]
    fn wrap_clamp_limits() {
        assert_eq!(wrap_clamp(-0.5), 0.0);
        assert_eq!(wrap_clamp(1.5), 1.0);
    }

    #[test]
    fn wrap_mirror_reflects() {
        assert!((wrap_mirror(0.5) - 0.5).abs() < 0.001);
        assert!((wrap_mirror(1.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn blend_normal_alpha() {
        let a = (1.0, 0.0, 0.0, 1.0);
        let b = (0.0, 0.0, 1.0, 0.5);
        let result = blend_normal(a, b);
        assert!((result.0 - 0.5).abs() < 0.001, "Red should be halved");
        assert!((result.2 - 0.5).abs() < 0.001, "Blue should be half");
    }

    #[test]
    fn blend_multiply_darkens() {
        let a = (1.0, 0.5, 0.25);
        let b = (0.5, 0.5, 0.5);
        let result = blend_multiply(a, b);
        assert!((result.0 - 0.5).abs() < 0.001);
        assert!((result.1 - 0.25).abs() < 0.001);
    }

    #[test]
    fn hsv_to_rgb_red() {
        let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 0.001);
        assert!(g.abs() < 0.001);
        assert!(b.abs() < 0.001);
    }

    #[test]
    fn hsv_to_rgb_green() {
        let (r, g, b) = hsv_to_rgb(120.0, 1.0, 1.0);
        assert!(r.abs() < 0.001);
        assert!((g - 1.0).abs() < 0.001);
        assert!(b.abs() < 0.001);
    }

    #[test]
    fn hsv_rgb_roundtrip() {
        let (h, s, v) = (45.0, 0.8, 0.6);
        let (r, g, b) = hsv_to_rgb(h, s, v);
        let (rh, rs, rv) = rgb_to_hsv(r, g, b);
        assert!((rh - h).abs() < 1.0, "Hue should roundtrip");
        assert!((rs - s).abs() < 0.01);
        assert!((rv - v).abs() < 0.01);
    }

    #[test]
    fn value_noise_deterministic() {
        let n1 = value_noise(5.0, 10.0, 42);
        let n2 = value_noise(5.0, 10.0, 42);
        assert_eq!(n1, n2, "Same input = same output");
    }

    #[test]
    fn value_noise_different_seed() {
        let n1 = value_noise(5.0, 10.0, 42);
        let n2 = value_noise(5.0, 10.0, 100);
        assert_ne!(n1, n2, "Different seed = different output");
    }
}
