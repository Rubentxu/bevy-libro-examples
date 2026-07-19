// Capítulo 14F. Postprocessing — Color grading, LUT, effects
/// Color grading: apply tonemapping and color adjustments

/// Tonemapping operators
pub fn tonemap_reinhard(color: f32) -> f32 {
    color / (1.0 + color)
}

pub fn tonemap_reinhard_rgb(c: (f32, f32, f32)) -> (f32, f32, f32) {
    (
        tonemap_reinhard(c.0),
        tonemap_reinhard(c.1),
        tonemap_reinhard(c.2),
    )
}

pub fn tonemap_aces(color: f32) -> f32 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    ((color * (a * color + b)) / (color * (c * color + d) + e)).clamp(0.0, 1.0)
}

pub fn tonemap_aces_rgb(c: (f32, f32, f32)) -> (f32, f32, f32) {
    (tonemap_aces(c.0), tonemap_aces(c.1), tonemap_aces(c.2))
}

/// Brightness/contrast/gamma adjustment
pub fn adjust_brightness(color: f32, brightness: f32) -> f32 {
    (color + brightness).clamp(0.0, 1.0)
}

pub fn adjust_contrast(color: f32, contrast: f32) -> f32 {
    ((color - 0.5) * contrast + 0.5).clamp(0.0, 1.0)
}

pub fn adjust_gamma(color: f32, gamma: f32) -> f32 {
    if gamma <= 0.0 || color <= 0.0 {
        return 0.0;
    }
    color.powf(1.0 / gamma)
}

/// Color grading pipeline
pub struct ColorGrade {
    pub exposure: f32,
    pub temperature: f32,  // -1 (cool) to 1 (warm)
    pub tint: f32,        // -1 (green) to 1 (magenta)
    pub saturation: f32,   // 0 (grayscale) to 2 (oversaturated)
    pub brightness: f32,
    pub contrast: f32,
}

impl Default for ColorGrade {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            temperature: 0.0,
            tint: 0.0,
            saturation: 1.0,
            brightness: 0.0,
            contrast: 1.0,
        }
    }
}

impl ColorGrade {
    pub fn apply(&self, color: (f32, f32, f32)) -> (f32, f32, f32) {
        // Exposure
        let exposure_mult = 2.0_f32.powf(self.exposure);
        let mut r = color.0 * exposure_mult;
        let mut g = color.1 * exposure_mult;
        let mut b = color.2 * exposure_mult;

        // Temperature (warm/cool)
        r += self.temperature * 0.1;
        b -= self.temperature * 0.1;

        // Tint (green/magenta)
        g -= self.tint * 0.1;
        r += self.tint * 0.05;
        b += self.tint * 0.05;

        // Saturation (convert to grayscale, then lerp back)
        let gray = r * 0.299 + g * 0.587 + b * 0.114;
        r = gray + (r - gray) * self.saturation;
        g = gray + (g - gray) * self.saturation;
        b = gray + (b - gray) * self.saturation;

        // Brightness
        r = adjust_brightness(r, self.brightness);
        g = adjust_brightness(g, self.brightness);
        b = adjust_brightness(b, self.brightness);

        // Contrast
        r = adjust_contrast(r, self.contrast);
        g = adjust_contrast(g, self.contrast);
        b = adjust_contrast(b, self.contrast);

        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }
}

/// Vignette effect: darken edges of the screen
pub fn vignette(uv: (f32, f32), intensity: f32, smoothness: f32) -> f32 {
    let dx = uv.0 - 0.5;
    let dy = uv.1 - 0.5;
    let dist = (dx * dx + dy * dy).sqrt();
    let vignette = 1.0 - intensity * (dist / smoothness).min(1.0);
    vignette.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tonemap_reinhard_zero() {
        assert!((tonemap_reinhard(0.0)).abs() < 0.001);
    }

    #[test]
    fn tonemap_reinhard_bright() {
        let result = tonemap_reinhard(10.0);
        assert!((result - 0.909).abs() < 0.01, "Bright values should compress toward 1");
    }

    #[test]
    fn tonemap_aces_clamped() {
        let result = tonemap_aces(100.0);
        assert!(result <= 1.0 && result >= 0.0);
    }

    #[test]
    fn adjust_brightness_adds() {
        assert!((adjust_brightness(0.5, 0.2) - 0.7).abs() < 0.001);
    }

    #[test]
    fn adjust_brightness_clamped() {
        assert_eq!(adjust_brightness(0.9, 0.5), 1.0);
    }

    #[test]
    fn adjust_contrast_midpoint() {
        assert!((adjust_contrast(0.5, 2.0) - 0.5).abs() < 0.001);
    }

    #[test]
    fn adjust_contrast_stretches() {
        let result = adjust_contrast(0.6, 2.0);
        assert!(result > 0.6, "Contrast > 1 should push values away from 0.5");
    }

    #[test]
    fn gamma_correction() {
        let result = adjust_gamma(0.5, 2.2);
        assert!(result > 0.5, "Gamma 2.2 brightens midtones");
        assert!((result - 0.735).abs() < 0.01);
    }

    #[test]
    fn color_grade_exposure() {
        let grade = ColorGrade { exposure: 1.0, ..Default::default() };
        let result = grade.apply((0.5, 0.5, 0.5));
        assert!(result.0 > 0.5, "Positive exposure should brighten");
    }

    #[test]
    fn color_grade_saturation_grayscale() {
        let grade = ColorGrade { saturation: 0.0, ..Default::default() };
        let result = grade.apply((1.0, 0.0, 0.0));
        let gray = 1.0 * 0.299;
        assert!((result.0 - gray).abs() < 0.01, "Red channel should become grayscale weight");
    }

    #[test]
    fn color_grade_clamped() {
        let grade = ColorGrade { exposure: 10.0, ..Default::default() };
        let result = grade.apply((0.5, 0.5, 0.5));
        assert!(result.0 <= 1.0 && result.0 >= 0.0);
    }

    #[test]
    fn vignette_center_bright() {
        let v = vignette((0.5, 0.5), 1.0, 0.5);
        assert!((v - 1.0).abs() < 0.001, "Center should be full brightness");
    }

    #[test]
    fn vignette_edge_dark() {
        let v = vignette((0.0, 0.0), 1.0, 0.5);
        assert!(v < 1.0, "Corner should be darkened");
    }

    #[test]
    fn vignette_zero_intensity() {
        let v = vignette((0.0, 0.0), 0.0, 0.5);
        assert!((v - 1.0).abs() < 0.001, "Zero intensity = no vignette");
    }
}
