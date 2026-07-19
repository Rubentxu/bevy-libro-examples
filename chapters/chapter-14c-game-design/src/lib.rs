// Capítulo 14C. Game Design — Bullet patterns, object pooling, difficulty curves
use std::collections::VecDeque;

/// Bullet pattern: predefined trajectories for bullet hell games
#[derive(Clone, Debug)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub alive: bool,
}

impl Bullet {
    pub fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self { x, y, vx, vy, alive: true }
    }

    pub fn update(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

/// Ring pattern: spawn N bullets in a circle
pub fn ring_pattern(origin: (f32, f32), count: usize, speed: f32) -> Vec<Bullet> {
    (0..count)
        .map(|i| {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            Bullet::new(
                origin.0,
                origin.1,
                angle.cos() * speed,
                angle.sin() * speed,
            )
        })
        .collect()
}

/// Spread pattern: fan of bullets aimed at a direction
pub fn spread_pattern(
    origin: (f32, f32),
    direction: f32,
    spread: f32,
    count: usize,
    speed: f32,
) -> Vec<Bullet> {
    (0..count)
        .map(|i| {
            let t = if count > 1 {
                (i as f32 / (count - 1) as f32) - 0.5
            } else {
                0.0
            };
            let angle = direction + t * spread;
            Bullet::new(
                origin.0,
                origin.1,
                angle.cos() * speed,
                angle.sin() * speed,
            )
        })
        .collect()
}

/// Spiral pattern: bullets emitted over time with rotating angle
pub fn spiral_step(angle: &mut f32, angular_speed: f32, dt: f32) -> f32 {
    *angle += angular_speed * dt;
    *angle % std::f32::consts::TAU
}

/// Object pool: reuse bullet allocations to avoid GC pressure
pub struct BulletPool {
    bullets: VecDeque<Bullet>,
    max_size: usize,
}

impl BulletPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            bullets: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, vx: f32, vy: f32) {
        // Try to reuse a dead bullet
        if let Some(bullet) = self.bullets.iter_mut().find(|b| !b.alive) {
            bullet.x = x;
            bullet.y = y;
            bullet.vx = vx;
            bullet.vy = vy;
            bullet.alive = true;
            return;
        }

        // No dead bullets — create new one if under capacity
        if self.bullets.len() < self.max_size {
            self.bullets.push_back(Bullet::new(x, y, vx, vy));
        }
    }

    pub fn update_all(&mut self, dt: f32, bounds: (f32, f32, f32, f32)) {
        for bullet in &mut self.bullets {
            if bullet.alive {
                bullet.update(dt);
                // Kill bullets out of bounds
                if bullet.x < bounds.0 || bullet.x > bounds.2
                    || bullet.y < bounds.1 || bullet.y > bounds.3
                {
                    bullet.alive = false;
                }
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.bullets.iter().filter(|b| b.alive).count()
    }

    pub fn total_count(&self) -> usize {
        self.bullets.len()
    }
}

/// Difficulty curve: scales enemy stats based on level
pub fn difficulty_multiplier(level: u32, base: f32, growth_rate: f32) -> f32 {
    base * (1.0 + growth_rate).powf(level as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_pattern_generates_circle() {
        let bullets = ring_pattern((0.0, 0.0), 8, 100.0);
        assert_eq!(bullets.len(), 8);

        // Check bullets are evenly spaced
        let angles: Vec<f32> = bullets
            .iter()
            .map(|b| b.y.atan2(b.x))
            .collect();

        // First bullet should be at angle 0
        assert!(angles[0].abs() < 0.01);
    }

    #[test]
    fn spread_pattern_aimed() {
        let bullets = spread_pattern((0.0, 0.0), 0.0, 0.5, 5, 100.0);
        assert_eq!(bullets.len(), 5);

        // Middle bullet should go straight (direction = 0)
        let mid = &bullets[2];
        assert!(mid.vx > 0.0, "Middle bullet should move right");
        assert!(mid.vy.abs() < 1.0, "Middle bullet should have ~0 vertical velocity");
    }

    #[test]
    fn bullet_moves_with_velocity() {
        let mut bullet = Bullet::new(0.0, 0.0, 10.0, 0.0);
        bullet.update(1.0);
        assert!((bullet.x - 10.0).abs() < 0.001);
        assert!(bullet.y.abs() < 0.001);
    }

    #[test]
    fn bullet_pool_reuses_dead() {
        let mut pool = BulletPool::new(10);

        // Spawn 5 bullets
        for i in 0..5 {
            pool.spawn(i as f32, 0.0, 1.0, 0.0);
        }
        assert_eq!(pool.active_count(), 5);
        assert_eq!(pool.total_count(), 5);

        // Kill all by moving them out of bounds
        pool.update_all(1000.0, (-10.0, -10.0, 10.0, 10.0));
        assert_eq!(pool.active_count(), 0);

        // Spawn again — should reuse, not allocate
        pool.spawn(0.0, 0.0, 1.0, 0.0);
        assert_eq!(pool.total_count(), 5, "Should reuse dead bullet, not allocate new");
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn bullet_pool_respects_max() {
        let mut pool = BulletPool::new(3);
        for _ in 0..10 {
            pool.spawn(0.0, 0.0, 1.0, 0.0);
        }
        assert_eq!(pool.total_count(), 3, "Should not exceed max_size");
    }

    #[test]
    fn spiral_step_rotates() {
        let mut angle = 0.0;
        let result1 = spiral_step(&mut angle, 1.0, 0.5);
        assert!((result1 - 0.5).abs() < 0.001);

        let result2 = spiral_step(&mut angle, 1.0, 0.5);
        assert!((result2 - 1.0).abs() < 0.001);
    }

    #[test]
    fn spiral_wraps_around_tau() {
        let mut angle = std::f32::consts::TAU - 0.1;
        let result = spiral_step(&mut angle, 1.0, 0.2);
        assert!(result < 0.2, "Should wrap around TAU, got {}", result);
    }

    #[test]
    fn difficulty_grows_exponentially() {
        let level_0 = difficulty_multiplier(0, 1.0, 0.1);
        let level_10 = difficulty_multiplier(10, 1.0, 0.1);

        assert!((level_0 - 1.0).abs() < 0.001);
        assert!(level_10 > level_0 * 2.0, "Level 10 should be more than 2x harder");
    }

    #[test]
    fn difficulty_no_growth() {
        let result = difficulty_multiplier(100, 1.0, 0.0);
        assert!((result - 1.0).abs() < 0.001, "Zero growth rate = constant difficulty");
    }
}
