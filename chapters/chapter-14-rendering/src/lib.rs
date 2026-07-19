// Capítulo 14. Rendering — Camera math, layers, z-ordering
// Math concepts that don't require GPU, fully testable.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera2D {
    pub position: (f32, f32),
    pub zoom: f32,
    pub rotation: f32,
}

impl Camera2D {
    pub fn new() -> Self {
        Self { position: (0.0, 0.0), zoom: 1.0, rotation: 0.0 }
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world: (f32, f32), screen_size: (f32, f32)) -> (f32, f32) {
        let dx = world.0 - self.position.0;
        let dy = world.1 - self.position.1;

        // Apply rotation
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rx = dx * cos - dy * sin;
        let ry = dx * sin + dy * cos;

        // Apply zoom and center on screen
        let sx = rx * self.zoom + screen_size.0 * 0.5;
        let sy = ry * self.zoom + screen_size.1 * 0.5;
        (sx, sy)
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen: (f32, f32), screen_size: (f32, f32)) -> (f32, f32) {
        let dx = (screen.0 - screen_size.0 * 0.5) / self.zoom;
        let dy = (screen.1 - screen_size.1 * 0.5) / self.zoom;

        let cos = (-self.rotation).cos();
        let sin = (-self.rotation).sin();
        let rx = dx * cos - dy * sin;
        let ry = dx * sin + dy * cos;

        (rx + self.position.0, ry + self.position.1)
    }

    pub fn visible_bounds(&self, screen_size: (f32, f32)) -> (f32, f32, f32, f32) {
        let half_w = screen_size.0 * 0.5 / self.zoom;
        let half_h = screen_size.1 * 0.5 / self.zoom;
        (
            self.position.0 - half_w,
            self.position.1 - half_h,
            self.position.0 + half_w,
            self.position.1 + half_h,
        )
    }
}

impl Default for Camera2D {
    fn default() -> Self { Self::new() }
}

/// Z-layer ordering for 2D rendering
pub const Z_BACKGROUND: f32 = -10.0;
pub const Z_TILEMAP: f32 = -5.0;
pub const Z_ENTITIES: f32 = 0.0;
pub const Z_PARTICLES: f32 = 5.0;
pub const Z_UI: f32 = 10.0;

pub fn layer_z(layer: RenderLayer) -> f32 {
    match layer {
        RenderLayer::Background => Z_BACKGROUND,
        RenderLayer::Tilemap => Z_TILEMAP,
        RenderLayer::Entities => Z_ENTITIES,
        RenderLayer::Particles => Z_PARTICLES,
        RenderLayer::UI => Z_UI,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderLayer {
    Background,
    Tilemap,
    Entities,
    Particles,
    UI,
}

/// Lerp between two colors (linear interpolation)
pub fn lerp_color(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), t: f32) -> (f32, f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
        a.3 + (b.3 - a.3) * t,
    )
}

/// Convert sRGB to linear color space
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert linear to sRGB color space
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_world_to_screen_centered() {
        let cam = Camera2D::new();
        let screen = cam.world_to_screen((0.0, 0.0), (800.0, 600.0));
        assert!((screen.0 - 400.0).abs() < 0.001);
        assert!((screen.1 - 300.0).abs() < 0.001);
    }

    #[test]
    fn camera_world_to_screen_offset() {
        let cam = Camera2D::new();
        let screen = cam.world_to_screen((100.0, 50.0), (800.0, 600.0));
        assert!((screen.0 - 500.0).abs() < 0.001);
        assert!((screen.1 - 350.0).abs() < 0.001);
    }

    #[test]
    fn camera_screen_to_world_roundtrip() {
        let cam = Camera2D { position: (50.0, 50.0), zoom: 2.0, rotation: 0.0 };
        let world = (123.0, 456.0);
        let screen = cam.world_to_screen(world, (800.0, 600.0));
        let back = cam.screen_to_world(screen, (800.0, 600.0));
        assert!((back.0 - world.0).abs() < 0.01);
        assert!((back.1 - world.1).abs() < 0.01);
    }

    #[test]
    fn camera_zoom_affects_bounds() {
        let cam = Camera2D { zoom: 2.0, ..Camera2D::new() };
        let bounds = cam.visible_bounds((800.0, 600.0));
        let width = bounds.2 - bounds.0;
        assert!((width - 400.0).abs() < 0.001, "Zoom 2x should show half the width");
    }

    #[test]
    fn render_layer_ordering() {
        assert!(layer_z(RenderLayer::Background) < layer_z(RenderLayer::Tilemap));
        assert!(layer_z(RenderLayer::Tilemap) < layer_z(RenderLayer::Entities));
        assert!(layer_z(RenderLayer::Entities) < layer_z(RenderLayer::UI));
    }

    #[test]
    fn color_lerp_midpoint() {
        let black = (0.0, 0.0, 0.0, 1.0);
        let white = (1.0, 1.0, 1.0, 1.0);
        let mid = lerp_color(black, white, 0.5);
        assert!((mid.0 - 0.5).abs() < 0.001);
        assert!((mid.1 - 0.5).abs() < 0.001);
    }

    #[test]
    fn color_lerp_clamped() {
        let a = (0.0, 0.0, 0.0, 1.0);
        let b = (1.0, 0.0, 0.0, 1.0);
        let over = lerp_color(a, b, 2.0); // Should clamp to 1.0
        assert!((over.0 - 1.0).abs() < 0.001);
    }

    #[test]
    fn srgb_linear_roundtrip() {
        for c in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let linear = srgb_to_linear(c);
            let back = linear_to_srgb(linear);
            assert!((back - c).abs() < 0.001, "Roundtrip failed for {}", c);
        }
    }

    #[test]
    fn srgb_zero_is_zero() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
    }

    #[test]
    fn srgb_one_is_one() {
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 0.001);
    }
}
