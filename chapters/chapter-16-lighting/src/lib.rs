// Capítulo 16. Lighting — Attenuation, color temperature, visibility

/// Point light with attenuation
#[derive(Clone, Debug)]
pub struct PointLight2D {
    pub position: (f32, f32),
    pub intensity: f32,
    pub radius: f32,
    pub color: (f32, f32, f32),
}

impl PointLight2D {
    /// Calculate light intensity at a given distance
    /// Uses inverse-square law with smooth falloff
    pub fn intensity_at_distance(&self, distance: f32) -> f32 {
        if distance >= self.radius {
            return 0.0;
        }
        // Smooth attenuation: (1 - d/r)^2
        let t = 1.0 - distance / self.radius;
        self.intensity * t * t
    }

    /// Check if a point is within the light's radius
    pub fn illuminates(&self, point: (f32, f32)) -> bool {
        let dx = point.0 - self.position.0;
        let dy = point.1 - self.position.1;
        let dist = (dx * dx + dy * dy).sqrt();
        dist < self.radius
    }

    /// Distance from light to a point
    pub fn distance_to(&self, point: (f32, f32)) -> f32 {
        let dx = point.0 - self.position.0;
        let dy = point.1 - self.position.1;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Color temperature in Kelvin
pub struct ColorTemperature;

impl ColorTemperature {
    /// Convert Kelvin temperature to RGB color
    /// Based on Tanner Helland's algorithm
    pub fn kelvin_to_rgb(kelvin: f32) -> (f32, f32, f32) {
        let temp = (kelvin / 100.0).clamp(1.0, 400.0);
        let temp = temp.clamp(1000.0 / 100.0, 40000.0 / 100.0);

        let r;
        let g;
        let b;

        // Red
        if temp <= 66.0 {
            r = 1.0;
        } else {
            r = (329.698727446 * (temp - 60.0).powf(-0.1332047592)) / 255.0;
        }

        // Green
        if temp <= 66.0 {
            g = (99.4708025861 * temp.ln() - 161.1195681661) / 255.0;
        } else {
            g = (288.1221695283 * (temp - 60.0).powf(-0.0755148492)) / 255.0;
        }

        // Blue
        if temp >= 66.0 {
            b = 1.0;
        } else if temp <= 19.0 {
            b = 0.0;
        } else {
            b = (138.5177312231 * (temp - 10.0).ln() - 305.0447927307) / 255.0;
        }

        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }
}

/// Ambient light: base illumination level
#[derive(Clone, Debug)]
pub struct AmbientLight {
    pub brightness: f32,
    pub color: (f32, f32, f32),
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self { brightness: 0.1, color: (1.0, 1.0, 1.0) }
    }
}

/// Calculate effective brightness at a point considering ambient + point lights
pub fn effective_brightness(
    point: (f32, f32),
    ambient: &AmbientLight,
    lights: &[PointLight2D],
) -> f32 {
    let mut total = ambient.brightness;

    for light in lights {
        let dist = light.distance_to(point);
        total += light.intensity_at_distance(dist);
    }

    total.min(1.0) // Clamp to max brightness
}

/// Fog of war: reveal tiles within light radius
pub fn visible_tiles(center: (i32, i32), radius: i32) -> Vec<(i32, i32)> {
    let mut tiles = Vec::new();
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                tiles.push((center.0 + dx, center.1 + dy));
            }
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_intensity_at_center() {
        let light = PointLight2D {
            position: (0.0, 0.0),
            intensity: 1.0,
            radius: 100.0,
            color: (1.0, 1.0, 1.0),
        };
        assert!((light.intensity_at_distance(0.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn light_intensity_at_edge() {
        let light = PointLight2D {
            position: (0.0, 0.0),
            intensity: 1.0,
            radius: 100.0,
            color: (1.0, 1.0, 1.0),
        };
        assert!(light.intensity_at_distance(100.0) < 0.01);
    }

    #[test]
    fn light_intensity_beyond_radius() {
        let light = PointLight2D {
            position: (0.0, 0.0),
            intensity: 1.0,
            radius: 100.0,
            color: (1.0, 1.0, 1.0),
        };
        assert_eq!(light.intensity_at_distance(200.0), 0.0);
    }

    #[test]
    fn light_illuminates_nearby() {
        let light = PointLight2D {
            position: (0.0, 0.0),
            intensity: 1.0,
            radius: 50.0,
            color: (1.0, 1.0, 1.0),
        };
        assert!(light.illuminates((30.0, 0.0)));
        assert!(!light.illuminates((100.0, 0.0)));
    }

    #[test]
    fn effective_brightness_clamped() {
        let ambient = AmbientLight { brightness: 0.5, color: (1.0, 1.0, 1.0) };
        let lights = vec![
            PointLight2D {
                position: (0.0, 0.0),
                intensity: 1.0,
                radius: 100.0,
                color: (1.0, 1.0, 1.0),
            },
        ];

        let brightness = effective_brightness((0.0, 0.0), &ambient, &lights);
        assert!((brightness - 1.0).abs() < 0.001, "Should be clamped to 1.0");
    }

    #[test]
    fn effective_brightness_ambient_only() {
        let ambient = AmbientLight { brightness: 0.3, color: (1.0, 1.0, 1.0) };
        let brightness = effective_brightness((0.0, 0.0), &ambient, &[]);
        assert!((brightness - 0.3).abs() < 0.001);
    }

    #[test]
    fn kelvin_warm_light() {
        let (r, g, b) = ColorTemperature::kelvin_to_rgb(2700.0);
        assert!(r > b, "Warm light should have more red than blue");
    }

    #[test]
    fn kelvin_cool_light() {
        let (r, g, b) = ColorTemperature::kelvin_to_rgb(6500.0);
        assert!(b >= r * 0.9, "Cool light should have comparable or more blue");
    }

    #[test]
    fn kelvin_daylight() {
        let (_, _, b) = ColorTemperature::kelvin_to_rgb(10000.0);
        assert!(b > 0.5, "Very cool light should be bluish");
    }

    #[test]
    fn visible_tiles_in_radius() {
        let tiles = visible_tiles((0, 0), 2);
        // Circle of radius 2 should contain center + ring
        assert!(tiles.contains(&(0, 0)));
        assert!(tiles.contains(&(1, 0)));
        assert!(tiles.contains(&(0, 1)));
        assert!(!tiles.contains(&(3, 0)), "Should not include tiles outside radius");
    }

    #[test]
    fn visible_tiles_count() {
        let tiles = visible_tiles((0, 0), 1);
        // Radius 1 = center + 4 neighbors = 5 tiles (3x3 minus corners)
        assert_eq!(tiles.len(), 5);
    }

    #[test]
    fn light_attenuation_midpoint() {
        let light = PointLight2D {
            position: (0.0, 0.0),
            intensity: 1.0,
            radius: 100.0,
            color: (1.0, 1.0, 1.0),
        };
        // At half radius: (1 - 0.5)^2 = 0.25
        let intensity = light.intensity_at_distance(50.0);
        assert!((intensity - 0.25).abs() < 0.01);
    }
}
