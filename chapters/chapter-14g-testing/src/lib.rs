// Capítulo 14G. Testing — Test patterns, fixtures, assertions
// This crate demonstrates testing strategies for Bevy games.

use bevy::prelude::*;

/// Minimal app for unit testing systems
pub fn minimal_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app
}

/// Full app with all needed plugins for integration testing
pub fn test_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_systems(Update, default_system);
    app
}

fn default_system() {}

/// Test fixture: spawn a test entity with known state
pub fn spawn_test_entity(world: &mut World, health: i32) -> Entity {
    world.spawn(TestHealth { current: health, max: 100 }).id()
}

#[derive(Component, Debug)]
pub struct TestHealth {
    pub current: i32,
    pub max: i32,
}

/// System that halves all health values (for testing system execution)
pub fn halve_health(mut query: Query<&mut TestHealth>) {
    for mut health in &mut query {
        health.current /= 2;
    }
}

/// System that adds to health
pub fn heal(amount: i32) -> impl FnMut(Query<&mut TestHealth>) + Clone {
    move |mut query: Query<&mut TestHealth>| {
        for mut health in &mut query {
            health.current = (health.current + amount).min(health.max);
        }
    }
}

/// Assertion helper: check entity health
pub fn assert_health(world: &World, entity: Entity, expected: i32) {
    let health = world
        .get::<TestHealth>(entity)
        .expect("Entity should have TestHealth");
    assert_eq!(
        health.current, expected,
        "Entity {:?} health should be {}, got {}",
        entity, expected, health.current
    );
}

/// Benchmark-like helper: measure system execution time
pub fn time_system(app: &mut App, iterations: usize) -> f64 {
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        app.update();
    }
    let elapsed = start.elapsed();
    elapsed.as_secs_f64() * 1000.0 / iterations as f64
}

/// Parameterized test data generator
pub fn test_health_values() -> Vec<(i32, i32)> {
    vec![
        (0, 0),      // Zero health
        (50, 25),    // Even number
        (51, 25),    // Odd number (integer division)
        (100, 50),   // Full health
        (-10, -5),   // Negative
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_app_has_time() {
        let app = minimal_app();
        assert!(app.world().contains_resource::<Time>());
    }

    #[test]
    fn spawn_test_entity_has_health() {
        let mut world = World::new();
        let entity = spawn_test_entity(&mut world, 75);
        let health = world.get::<TestHealth>(entity).unwrap();
        assert_eq!(health.current, 75);
    }

    #[test]
    fn halve_health_system_works() {
        let mut app = minimal_app();
        let entity = spawn_test_entity(app.world_mut(), 80);
        app.add_systems(Update, halve_health);
        app.update();

        assert_health(app.world(), entity, 40);
    }

    #[test]
    fn halve_health_odd_number() {
        let mut app = minimal_app();
        let entity = spawn_test_entity(app.world_mut(), 81);
        app.add_systems(Update, halve_health);
        app.update();

        assert_health(app.world(), entity, 40); // 81 / 2 = 40 (integer division)
    }

    #[test]
    fn heal_system_clamps_to_max() {
        let mut app = minimal_app();
        let entity = spawn_test_entity(app.world_mut(), 90);

        let heal_amount = 50;
        app.add_systems(Update, heal(heal_amount));
        app.update();

        assert_health(app.world(), entity, 100); // Clamped to max
    }

    #[test]
    fn multiple_entities_processed() {
        let mut app = minimal_app();
        let e1 = spawn_test_entity(app.world_mut(), 50);
        let e2 = spawn_test_entity(app.world_mut(), 100);
        let e3 = spawn_test_entity(app.world_mut(), 20);

        app.add_systems(Update, halve_health);
        app.update();

        assert_health(app.world(), e1, 25);
        assert_health(app.world(), e2, 50);
        assert_health(app.world(), e3, 10);
    }

    #[test]
    fn system_runs_multiple_times() {
        let mut app = minimal_app();
        let entity = spawn_test_entity(app.world_mut(), 100);

        app.add_systems(Update, halve_health);
        app.update(); // 100 -> 50
        app.update(); // 50 -> 25
        app.update(); // 25 -> 12

        assert_health(app.world(), entity, 12);
    }

    #[test]
    fn test_health_values_parameterized() {
        for (input, expected) in test_health_values() {
            let mut world = World::new();
            let mut query = world.query::<&mut TestHealth>();
            let entity = world.spawn(TestHealth { current: input, max: 100 }).id();

            // Simulate halve
            let mut health = query.get_mut(&mut world, entity).unwrap();
            health.current /= 2;

            let result = world.get::<TestHealth>(entity).unwrap().current;
            assert_eq!(result, expected, "halve({}) should be {}", input, expected);
        }
    }

    #[test]
    fn time_system_measures() {
        let mut app = minimal_app();
        let avg_ms = time_system(&mut app, 10);
        assert!(avg_ms >= 0.0, "Time should be non-negative");
    }

    #[test]
    fn assert_health_panics_on_wrong_value() {
        let mut world = World::new();
        let entity = spawn_test_entity(&mut world, 50);

        // This should NOT panic
        assert_health(&world, entity, 50);

        // This SHOULD panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_health(&world, entity, 999);
        }));
        assert!(result.is_err(), "Should panic on wrong health value");
    }
}
