// Capítulo 17. Física — AABB collision, velocity integration
// Pure math, no GPU dependency, fully testable.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn zero() -> Self { Self { x: 0.0, y: 0.0 } }

    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.dot(self)
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0001 {
            Vec2::new(self.x / len, self.y / len)
        } else {
            Vec2::zero()
        }
    }

    pub fn add(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }

    pub fn scale(&self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

/// Axis-Aligned Bounding Box for 2D collision detection
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    pub center: Vec2,
    pub half_extents: Vec2,
}

impl AABB {
    pub fn new(center: Vec2, half_extents: Vec2) -> Self {
        Self { center, half_extents }
    }

    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        let center = Vec2::new(
            (min.x + max.x) * 0.5,
            (min.y + max.y) * 0.5,
        );
        let half_extents = Vec2::new(
            (max.x - min.x) * 0.5,
            (max.y - min.y) * 0.5,
        );
        Self { center, half_extents }
    }

    pub fn min(&self) -> Vec2 {
        Vec2::new(
            self.center.x - self.half_extents.x,
            self.center.y - self.half_extents.y,
        )
    }

    pub fn max(&self) -> Vec2 {
        Vec2::new(
            self.center.x + self.half_extents.x,
            self.center.y + self.half_extents.y,
        )
    }

    /// AABB-AABB intersection test
    pub fn intersects(&self, other: &AABB) -> bool {
        let diff = self.center.sub(&other.center);
        let combined_half = self.half_extents.add(&other.half_extents);

        diff.x.abs() < combined_half.x && diff.y.abs() < combined_half.y
    }

    /// Returns the overlap vector (penetration depth) if overlapping
    pub fn penetration(&self, other: &AABB) -> Option<Vec2> {
        if !self.intersects(other) {
            return None;
        }

        let diff = self.center.sub(&other.center);
        let combined_half = self.half_extents.add(&other.half_extents);

        let overlap_x = combined_half.x - diff.x.abs();
        let overlap_y = combined_half.y - diff.y.abs();

        if overlap_x < overlap_y {
            let sign = if diff.x >= 0.0 { 1.0 } else { -1.0 };
            Some(Vec2::new(overlap_x * sign, 0.0))
        } else {
            let sign = if diff.y >= 0.0 { 1.0 } else { -1.0 };
            Some(Vec2::new(0.0, overlap_y * sign))
        }
    }
}

/// Circle collider
#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn intersects_circle(&self, other: &Circle) -> bool {
        let dist_sq = self.center.sub(&other.center).length_squared();
        let radius_sum = self.radius + other.radius;
        dist_sq < radius_sum * radius_sum
    }

    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        let closest_x = self.center.x.clamp(aabb.min().x, aabb.max().x);
        let closest_y = self.center.y.clamp(aabb.min().y, aabb.max().y);
        let dx = self.center.x - closest_x;
        let dy = self.center.y - closest_y;
        dx * dx + dy * dy < self.radius * self.radius
    }
}

/// Kinematic body for physics simulation
#[derive(Clone, Debug)]
pub struct KinematicBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub drag: f32,
}

impl KinematicBody {
    pub fn new(pos: Vec2) -> Self {
        Self {
            position: pos,
            velocity: Vec2::zero(),
            acceleration: Vec2::zero(),
            drag: 0.98,
        }
    }

    /// Semi-implicit Euler integration step
    pub fn integrate(&mut self, dt: f32) {
        // Apply acceleration to velocity
        self.velocity = self.velocity.add(&self.acceleration.scale(dt));
        // Apply drag
        self.velocity = self.velocity.scale(self.drag.powf(dt * 60.0));
        // Apply velocity to position
        self.position = self.position.add(&self.velocity.scale(dt));
        // Reset acceleration (forces are re-applied each frame)
        self.acceleration = Vec2::zero();
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.acceleration = self.acceleration.add(&force);
    }
}

/// Coyote time: allow jumping shortly after leaving a platform
pub struct CoyoteTimer {
    pub remaining: f32,
    pub max: f32,
}

impl CoyoteTimer {
    pub fn new(duration: f32) -> Self {
        Self { remaining: 0.0, max: duration }
    }

    pub fn reset(&mut self) {
        self.remaining = self.max;
    }

    pub fn tick(&mut self, dt: f32) {
        self.remaining = (self.remaining - dt).max(0.0);
    }

    pub fn can_jump(&self) -> bool {
        self.remaining > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabb_no_overlap() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let b = AABB::new(Vec2::new(5.0, 5.0), Vec2::new(1.0, 1.0));
        assert!(!a.intersects(&b));
    }

    #[test]
    fn aabb_overlap() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let b = AABB::new(Vec2::new(1.5, 0.0), Vec2::new(1.0, 1.0));
        assert!(a.intersects(&b));
    }

    #[test]
    fn aabb_touching_not_overlapping() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let b = AABB::new(Vec2::new(2.0, 0.0), Vec2::new(1.0, 1.0));
        assert!(!a.intersects(&b), "Touching edges should not count as overlap");
    }

    #[test]
    fn aabb_penetration_x_axis() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0));
        let b = AABB::new(Vec2::new(3.0, 0.0), Vec2::new(2.0, 2.0));
        let pen = a.penetration(&b).expect("Should penetrate");
        assert!(pen.x.abs() > 0.0 && pen.y == 0.0, "Penetration should be on X axis");
    }

    #[test]
    fn circle_circle_collision() {
        let a = Circle { center: Vec2::new(0.0, 0.0), radius: 5.0 };
        let b = Circle { center: Vec2::new(8.0, 0.0), radius: 5.0 };
        assert!(a.intersects_circle(&b));

        let c = Circle { center: Vec2::new(20.0, 0.0), radius: 5.0 };
        assert!(!a.intersects_circle(&c));
    }

    #[test]
    fn circle_aabb_collision() {
        let circle = Circle { center: Vec2::new(0.0, 0.0), radius: 5.0 };
        let aabb = AABB::new(Vec2::new(4.0, 0.0), Vec2::new(2.0, 2.0));
        assert!(circle.intersects_aabb(&aabb));

        let far_aabb = AABB::new(Vec2::new(20.0, 0.0), Vec2::new(2.0, 2.0));
        assert!(!circle.intersects_aabb(&far_aabb));
    }

    #[test]
    fn kinematic_body_gravity() {
        let mut body = KinematicBody::new(Vec2::new(0.0, 100.0));
        body.apply_force(Vec2::new(0.0, -200.0)); // Gravity
        body.integrate(0.5);

        // After 0.5s: velocity should be -100, position should be 100 + (-100)*0.5 = 50
        assert!(body.velocity.y < 0.0, "Should be falling");
        assert!(body.position.y < 100.0, "Should have moved down");
    }

    #[test]
    fn kinematic_body_drag_decelerates() {
        let mut body = KinematicBody::new(Vec2::zero());
        body.velocity = Vec2::new(100.0, 0.0);
        body.drag = 0.5;

        body.integrate(1.0 / 60.0);
        assert!(body.velocity.x < 100.0, "Drag should decelerate");
    }

    #[test]
    fn coyote_time_allows_jump() {
        let mut timer = CoyoteTimer::new(0.15);
        timer.reset();
        assert!(timer.can_jump());

        // After 0.1s, still can jump
        timer.tick(0.1);
        assert!(timer.can_jump());

        // After 0.2s total (0.15 + 0.05 past), cannot jump
        timer.tick(0.1);
        assert!(!timer.can_jump());
    }

    #[test]
    fn coyote_time_expires() {
        let mut timer = CoyoteTimer::new(0.1);
        timer.reset();
        timer.tick(0.2); // Past the window
        assert!(!timer.can_jump());
    }

    #[test]
    fn vec2_normalize() {
        let v = Vec2::new(3.0, 4.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 0.001, "Normalized vector should have length 1");
    }

    #[test]
    fn vec2_dot_product() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert_eq!(a.dot(&b), 0.0, "Perpendicular vectors have 0 dot product");

        let c = Vec2::new(1.0, 0.0);
        let d = Vec2::new(2.0, 0.0);
        assert_eq!(c.dot(&d), 2.0, "Parallel vectors dot product is product of lengths");
    }

    #[test]
    fn aabb_from_min_max() {
        let aabb = AABB::from_min_max(Vec2::new(0.0, 0.0), Vec2::new(10.0, 20.0));
        assert_eq!(aabb.center, Vec2::new(5.0, 10.0));
        assert_eq!(aabb.half_extents, Vec2::new(5.0, 10.0));
    }
}
