// Capítulo 22B. Particles — Emitter config, lifetime, gradients
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: (f32, f32, f32, f32),
    pub alive: bool,
}

impl Particle {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32, life: f32) -> Self {
        Self {
            x, y, vx, vy,
            life,
            max_life: life,
            size: 4.0,
            color: (1.0, 1.0, 1.0, 1.0),
            alive: true,
        }
    }

    pub fn update(&mut self, dt: f32, gravity: f32) {
        if !self.alive { return; }

        self.vy += gravity * dt;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.life -= dt;

        if self.life <= 0.0 {
            self.alive = false;
        }
    }

    pub fn life_pct(&self) -> f32 {
        if self.max_life > 0.0 {
            (self.life / self.max_life).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Emitter configuration
#[derive(Clone, Debug)]
pub struct EmitterConfig {
    pub rate: f32,           // particles per second
    pub lifetime: (f32, f32), // min, max
    pub speed: (f32, f32),    // min, max
    pub angle: (f32, f32),    // min, max (radians)
    pub size: (f32, f32),     // min, max
    pub gravity: f32,
    pub max_particles: usize,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            rate: 50.0,
            lifetime: (0.5, 1.5),
            speed: (50.0, 150.0),
            angle: (0.0, std::f32::consts::TAU),
            size: (2.0, 6.0),
            gravity: 100.0,
            max_particles: 500,
        }
    }
}

/// Particle emitter with pooling
pub struct ParticleEmitter {
    pub config: EmitterConfig,
    pub x: f32,
    pub y: f32,
    particles: VecDeque<Particle>,
    emit_accumulator: f32,
}

impl ParticleEmitter {
    pub fn new(x: f32, y: f32, config: EmitterConfig) -> Self {
        let max_particles = config.max_particles;
        Self {
            config,
            x, y,
            particles: VecDeque::with_capacity(max_particles),
            emit_accumulator: 0.0,
        }
    }

    fn spawn_particle(&mut self, rng_value: f32) {
        let life = self.config.lifetime.0
            + rng_value * (self.config.lifetime.1 - self.config.lifetime.0);

        let speed = self.config.speed.0
            + ((rng_value * 7919.0) % 1.0) * (self.config.speed.1 - self.config.speed.0);

        let angle = self.config.angle.0
            + ((rng_value * 3457.0) % 1.0) * (self.config.angle.1 - self.config.angle.0);

        let size = self.config.size.0
            + ((rng_value * 2389.0) % 1.0) * (self.config.size.1 - self.config.size.0);

        let mut p = Particle::new(
            self.x,
            self.y,
            angle.cos() * speed,
            angle.sin() * speed,
            life,
        );
        p.size = size;
        self.particles.push_back(p);
    }

    pub fn update(&mut self, dt: f32) {
        // Emit new particles
        self.emit_accumulator += self.config.rate * dt;
        while self.emit_accumulator >= 1.0 {
            self.emit_accumulator -= 1.0;
            if self.particles.len() < self.config.max_particles {
                // Remove dead particles from front first
                while let Some(front) = self.particles.front() {
                    if !front.alive {
                        self.particles.pop_front();
                    } else {
                        break;
                    }
                }
                self.spawn_particle(self.emit_accumulator.fract());
            }
        }

        // Update existing particles
        for p in &mut self.particles {
            p.update(dt, self.config.gravity);
        }
    }

    pub fn active_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    pub fn total_count(&self) -> usize {
        self.particles.len()
    }
}

/// Color gradient: interpolate along a color ramp
pub struct ColorGradient {
    pub stops: Vec<(f32, (f32, f32, f32, f32))>, // (position 0-1, RGBA)
}

impl ColorGradient {
    pub fn new(stops: Vec<(f32, (f32, f32, f32, f32))>) -> Self {
        Self { stops }
    }

    pub fn sample(&self, t: f32) -> (f32, f32, f32, f32) {
        let t = t.clamp(0.0, 1.0);
        if self.stops.is_empty() {
            return (1.0, 1.0, 1.0, 1.0);
        }
        if self.stops.len() == 1 {
            return self.stops[0].1;
        }

        for i in 0..self.stops.len() - 1 {
            let (pos_a, color_a) = self.stops[i];
            let (pos_b, color_b) = self.stops[i + 1];

            if t >= pos_a && t <= pos_b {
                let local_t = if pos_b > pos_a {
                    (t - pos_a) / (pos_b - pos_a)
                } else {
                    0.0
                };
                return lerp_rgba(color_a, color_b, local_t);
            }
        }

        self.stops.last().unwrap().1
    }
}

fn lerp_rgba(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32), t: f32) -> (f32, f32, f32, f32) {
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
        a.3 + (b.3 - a.3) * t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_starts_alive() {
        let p = Particle::new(0.0, 0.0, 10.0, 0.0, 1.0);
        assert!(p.alive);
        assert!((p.life_pct() - 1.0).abs() < 0.001);
    }

    #[test]
    fn particle_dies_after_lifetime() {
        let mut p = Particle::new(0.0, 0.0, 0.0, 0.0, 0.5);
        p.update(0.6, 0.0);
        assert!(!p.alive);
    }

    #[test]
    fn particle_applies_gravity() {
        let mut p = Particle::new(0.0, 0.0, 0.0, 0.0, 1.0);
        let initial_vy = p.vy;
        p.update(0.1, 200.0);
        assert!(p.vy > initial_vy, "Gravity should increase vy");
    }

    #[test]
    fn particle_life_pct_decreases() {
        let mut p = Particle::new(0.0, 0.0, 0.0, 0.0, 2.0);
        p.update(1.0, 0.0);
        assert!((p.life_pct() - 0.5).abs() < 0.01);
    }

    #[test]
    fn emitter_spawns_particles() {
        let config = EmitterConfig { rate: 100.0, max_particles: 10, ..Default::default() };
        let mut emitter = ParticleEmitter::new(0.0, 0.0, config);

        emitter.update(0.1); // Should spawn ~10 particles

        assert!(emitter.total_count() > 0, "Should have spawned particles");
        assert!(emitter.total_count() <= 10, "Should not exceed max");
    }

    #[test]
    fn emitter_respects_max() {
        let config = EmitterConfig { rate: 1000.0, max_particles: 5, ..Default::default() };
        let mut emitter = ParticleEmitter::new(0.0, 0.0, config);

        emitter.update(1.0);

        assert!(emitter.total_count() <= 5, "Should not exceed max_particles");
    }

    #[test]
    fn emitter_particles_die_over_time() {
        let config = EmitterConfig {
            rate: 0.0, // No new spawns
            lifetime: (0.1, 0.1),
            ..Default::default()
        };
        let mut emitter = ParticleEmitter::new(0.0, 0.0, config);
        emitter.particles.push_back(Particle::new(0.0, 0.0, 0.0, 0.0, 0.1));

        emitter.update(0.2);

        assert_eq!(emitter.active_count(), 0, "Particle should have died");
    }

    #[test]
    fn color_gradient_single_color() {
        let grad = ColorGradient::new(vec![(0.0, (1.0, 0.0, 0.0, 1.0))]);
        let c = grad.sample(0.5);
        assert!((c.0 - 1.0).abs() < 0.001 && (c.1 - 0.0).abs() < 0.001);
    }

    #[test]
    fn color_gradient_interpolates() {
        let grad = ColorGradient::new(vec![
            (0.0, (0.0, 0.0, 0.0, 1.0)),  // Black
            (1.0, (1.0, 1.0, 1.0, 1.0)),  // White
        ]);

        let mid = grad.sample(0.5);
        assert!((mid.0 - 0.5).abs() < 0.01, "Should be gray at midpoint");
    }

    #[test]
    fn color_gradient_three_stops() {
        let grad = ColorGradient::new(vec![
            (0.0, (1.0, 0.0, 0.0, 1.0)),  // Red
            (0.5, (0.0, 1.0, 0.0, 1.0)),  // Green
            (1.0, (0.0, 0.0, 1.0, 1.0)),  // Blue
        ]);

        let c = grad.sample(0.25);
        assert!(c.0 > 0.0 && c.1 > 0.0, "At 25% should be between red and green");

        let c = grad.sample(0.75);
        assert!(c.1 > 0.0 && c.2 > 0.0, "At 75% should be between green and blue");
    }

    #[test]
    fn color_gradient_clamped() {
        let grad = ColorGradient::new(vec![
            (0.0, (0.0, 0.0, 0.0, 1.0)),
            (1.0, (1.0, 1.0, 1.0, 1.0)),
        ]);

        let before = grad.sample(-0.5);
        assert!((before.0 - 0.0).abs() < 0.001, "Should clamp to first stop");

        let after = grad.sample(2.0);
        assert!((after.0 - 1.0).abs() < 0.001, "Should clamp to last stop");
    }
}
