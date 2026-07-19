// Capítulo 20. Game Feel — Screen shake, hit pause, juice
// Pure math/logic, fully testable.

/// Screen shake: decaying offset applied to camera
#[derive(Clone, Debug)]
pub struct ScreenShake {
    pub trauma: f32,
    pub max_offset: f32,
    pub max_angle: f32,
    pub decay_rate: f32,
    seed: f32,
}

impl ScreenShake {
    pub fn new(max_offset: f32, max_angle: f32) -> Self {
        Self {
            trauma: 0.0,
            max_offset,
            max_angle,
            decay_rate: 1.5,
            seed: 0.0,
        }
    }

    pub fn add_trauma(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }

    pub fn update(&mut self, dt: f32, time: f32) -> (f32, f32, f32) {
        if self.trauma > 0.0 {
            self.trauma = (self.trauma - self.decay_rate * dt).max(0.0);
        }

        // Shake = trauma^2 (so small trauma feels subtle)
        let shake = self.trauma * self.trauma;

        // Pseudo-noise based on time and seed
        let angle = shake * self.max_angle * noise(time * 37.0 + self.seed);
        let offset_x = shake * self.max_offset * noise(time * 53.0 + self.seed);
        let offset_y = shake * self.max_offset * noise(time * 71.0 + self.seed);

        (offset_x, offset_y, angle)
    }

    pub fn is_shaking(&self) -> bool {
        self.trauma > 0.01
    }
}

/// Simple pseudo-noise function (sin-based, deterministic)
fn noise(x: f32) -> f32 {
    (x.sin() * 43758.5453).fract() * 2.0 - 1.0
}

/// Hit pause: freeze the game for a few frames on impact
pub struct HitPause {
    pub freeze_timer: f32,
    pub default_duration: f32,
}

impl HitPause {
    pub fn new(duration: f32) -> Self {
        Self {
            freeze_timer: 0.0,
            default_duration: duration,
        }
    }

    pub fn trigger(&mut self) {
        self.freeze_timer = self.default_duration;
    }

    pub fn trigger_custom(&mut self, duration: f32) {
        self.freeze_timer = duration;
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if self.freeze_timer > 0.0 {
            self.freeze_timer -= dt;
            if self.freeze_timer <= 0.0 {
                self.freeze_timer = 0.0;
                return false; // No longer frozen
            }
            return true; // Still frozen
        }
        false
    }

    pub fn is_frozen(&self) -> bool {
        self.freeze_timer > 0.0
    }
}

/// Knockback: apply impulse away from damage source
pub fn calculate_knockback(
    source: (f32, f32),
    target: (f32, f32),
    force: f32,
) -> (f32, f32) {
    let dx = target.0 - source.0;
    let dy = target.1 - source.1;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < 0.001 {
        return (0.0, force); // Default: knock up
    }

    let nx = dx / dist;
    let ny = dy / dist;

    (nx * force, ny * force)
}

/// Cooldown timer for abilities, attacks, etc.
pub struct Cooldown {
    pub remaining: f32,
    pub duration: f32,
}

impl Cooldown {
    pub fn new(duration: f32) -> Self {
        Self { remaining: 0.0, duration }
    }

    pub fn start(&mut self) {
        self.remaining = self.duration;
    }

    pub fn update(&mut self, dt: f32) {
        if self.remaining > 0.0 {
            self.remaining = (self.remaining - dt).max(0.0);
        }
    }

    pub fn is_ready(&self) -> bool {
        self.remaining <= 0.0
    }

    pub fn pct_remaining(&self) -> f32 {
        if self.duration > 0.0 {
            self.remaining / self.duration
        } else {
            0.0
        }
    }
}

/// Smooth damp (lerp towards target with framerate-independent smoothing)
pub fn smooth_damp(current: f32, target: f32, smooth_time: f32, dt: f32) -> f32 {
    let t = 1.0 - (-dt / smooth_time).exp();
    current + (target - current) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_shake_starts_idle() {
        let shake = ScreenShake::new(10.0, 5.0);
        assert!(!shake.is_shaking());
    }

    #[test]
    fn screen_shake_activates() {
        let mut shake = ScreenShake::new(10.0, 5.0);
        shake.add_trauma(0.5);
        assert!(shake.is_shaking());
    }

    #[test]
    fn screen_shake_trauma_clamped_to_1() {
        let mut shake = ScreenShake::new(10.0, 5.0);
        shake.add_trauma(2.0);
        assert!((shake.trauma - 1.0).abs() < 0.001);
    }

    #[test]
    fn screen_shake_decays_over_time() {
        let mut shake = ScreenShake::new(10.0, 5.0);
        shake.add_trauma(1.0);
        shake.update(1.0, 0.0);

        assert!(shake.trauma < 1.0, "Trauma should decay");
    }

    #[test]
    fn screen_shake_stops_eventually() {
        let mut shake = ScreenShake::new(10.0, 5.0);
        shake.add_trauma(0.3);
        // 1 second at decay_rate=1.5 should clear 0.3 trauma
        shake.update(1.0, 0.0);
        assert!(!shake.is_shaking());
    }

    #[test]
    fn hit_pause_triggers_freeze() {
        let mut pause = HitPause::new(0.08);
        assert!(!pause.is_frozen());

        pause.trigger();
        assert!(pause.is_frozen());
    }

    #[test]
    fn hit_pause_expires() {
        let mut pause = HitPause::new(0.05);
        pause.trigger();

        // After 0.1s, freeze should be over
        pause.update(0.1);
        assert!(!pause.is_frozen());
    }

    #[test]
    fn knockback_away_from_source() {
        let source = (0.0, 0.0);
        let target = (10.0, 0.0);
        let (kx, ky) = calculate_knockback(source, target, 100.0);

        assert!(kx > 0.0, "Knockback should push target away from source (positive X)");
        assert!(ky.abs() < 0.001, "No Y component for horizontal alignment");
    }

    #[test]
    fn knockback_diagonal() {
        let source = (0.0, 0.0);
        let target = (10.0, 10.0);
        let (kx, ky) = calculate_knockback(source, target, 100.0);

        // Both components should be positive (pushing away diagonally)
        assert!(kx > 0.0);
        assert!(ky > 0.0);

        // Should be normalized: kx^2 + ky^2 = force^2
        let magnitude = (kx * kx + ky * ky).sqrt();
        assert!((magnitude - 100.0).abs() < 0.1, "Magnitude should equal force");
    }

    #[test]
    fn cooldown_starts_not_ready() {
        let mut cd = Cooldown::new(1.0);
        cd.start();
        assert!(!cd.is_ready());
    }

    #[test]
    fn cooldown_becomes_ready() {
        let mut cd = Cooldown::new(0.5);
        cd.start();
        cd.update(0.6);
        assert!(cd.is_ready());
    }

    #[test]
    fn cooldown_pct_decreases() {
        let mut cd = Cooldown::new(2.0);
        cd.start();
        assert!((cd.pct_remaining() - 1.0).abs() < 0.001);

        cd.update(1.0);
        assert!((cd.pct_remaining() - 0.5).abs() < 0.001);
    }

    #[test]
    fn smooth_damp_approaches_target() {
        let current = 0.0;
        let target = 100.0;
        let result = smooth_damp(current, target, 0.1, 0.016);

        assert!(result > 0.0 && result < target, "Should move toward target but not reach it");
    }

    #[test]
    fn smooth_damp_converges() {
        let mut value = 0.0;
        let target = 100.0;
        for _ in 0..1000 {
            value = smooth_damp(value, target, 0.1, 0.016);
        }
        assert!((value - target).abs() < 1.0, "Should converge to target");
    }
}
