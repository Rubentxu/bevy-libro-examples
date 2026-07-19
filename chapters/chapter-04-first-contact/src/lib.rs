use bevy::prelude::*;

// ============================================================================
// COMPONENTS (snippets 1-4)
// ============================================================================

/// Position with named fields — matches 2D convention (snippet 1, 4.2)
#[derive(Component, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Position with Default — defaults to (0, 0) (snippet 2, 4.2)
#[derive(Component, Debug, PartialEq, Default)]
pub struct PositionDefault {
    pub x: f32,
    pub y: f32,
}

/// Velocity with named fields (snippet 1, 4.2)
#[derive(Component, Debug, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// Tag component: marks a Player entity (snippet 3, 4.2)
#[derive(Component, Debug, PartialEq)]
pub struct Player;

/// Tag component: marks an Enemy entity (snippet 3, 4.2)
#[derive(Component, Debug, PartialEq)]
pub struct Enemy;

/// State component using enum (snippet 4, 4.2)
#[derive(Component, Debug, PartialEq)]
pub enum Estado {
    Idle,
    Caminando,
    Saltando,
    Muerto,
}

/// Velocidad2D for Transform-based movement (snippet 12, 4.8)
#[derive(Component, Debug, PartialEq)]
pub struct Velocidad2D {
    pub x: f32,
    pub y: f32,
}

/// Marker for cube entities used in time-based movement (snippet 10, 4.7)
#[derive(Component, Debug, PartialEq)]
pub struct Cube;

// ============================================================================
// RESOURCES (snippets 5-6, 4.3)
// ============================================================================

/// Game configuration resource (snippet 5, 4.3)
#[derive(Resource, Debug)]
pub struct GameConfig {
    pub volumen: f32,
    pub dificultad: f32,
    pub fullscreen: bool,
}

/// Global score resource (snippet 5, 4.3)
#[derive(Resource, Debug, PartialEq)]
pub struct PuntuacionGlobal(pub u32);

// ============================================================================
// EVENTS (snippet 7, 4.4)
// ============================================================================

/// Event triggered when player collects a coin (snippet 7, 4.4)
#[derive(Message, Debug)]
pub struct MonedaRecogida {
    pub cantidad: u32,
}

// ============================================================================
// APP BUILDER (snippet 11, 4.7) — headless variant
// ============================================================================

pub fn build_app() -> App {
    let mut app = App::new();
    app.add_systems(Startup, spawn_player)
        .add_systems(Update, move_entities);
    app
}

/// Headless app with Time resource for deterministic time-based tests (snippet 11, 4.7)
pub fn build_app_with_time() -> App {
    let mut app = App::new();
    app.add_systems(Startup, spawn_cube)
        .add_systems(Update, mover_cubo);
    app
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 2.0, y: -1.0 }));
}

fn move_entities(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

/// mover_cubo system — moves entities with Posicion + Velocidad using Time (snippet 10, 4.7)
/// Uses Posicion(f32, f32) tuple struct to match book exactly.
#[derive(Component, Debug, PartialEq)]
pub struct Posicion(pub f32, pub f32);

#[derive(Component, Debug, PartialEq)]
pub struct Velocidad(pub f32);

fn mover_cubo(time: Res<Time>, mut query: Query<(&mut Posicion, &Velocidad)>) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_secs();
    }
}

/// Spawn system for cube entities used with Time-based movement
fn spawn_cube(mut commands: Commands) {
    commands.spawn((Posicion(0.0, 0.0), Velocidad(50.0), Cube));
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn update_moves_the_spawned_entity() {
        let mut app = build_app();
        app.update();

        let mut positions = app.world_mut().query::<&Position>();
        let position = positions.single(app.world()).expect("one player");
        assert_eq!(position, &Position { x: 2.0, y: -1.0 });
    }

    #[test]
    fn position_default_is_zero() {
        let pos = PositionDefault::default();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn player_and_enemy_are_distinct_tags() {
        let mut app = App::new();
        app.world_mut().spawn((Player, Position { x: 0.0, y: 0.0 }));
        app.world_mut().spawn((Enemy, Position { x: 1.0, y: 1.0 }));

        let mut players = app.world_mut().query::<&Player>();
        let mut enemies = app.world_mut().query::<&Enemy>();
        assert_eq!(players.iter(app.world()).count(), 1);
        assert_eq!(enemies.iter(app.world()).count(), 1);
    }

    #[test]
    fn estado_enum_variants() {
        let idle = Estado::Idle;
        let walking = Estado::Caminando;
        assert!(matches!(idle, Estado::Idle));
        assert!(matches!(walking, Estado::Caminando));
    }

    #[test]
    fn game_config_resource() {
        let mut app = App::new();
        app.insert_resource(GameConfig {
            volumen: 0.8,
            dificultad: 1.5,
            fullscreen: false,
        });

        let config = app.world().resource::<GameConfig>();
        assert_eq!(config.volumen, 0.8);
        assert_eq!(config.dificultad, 1.5);
        assert!(!config.fullscreen);
    }

    #[test]
    fn puntuacion_global_mutable() {
        let mut app = App::new();
        app.insert_resource(PuntuacionGlobal(42));

        let mut score = app.world_mut().resource_mut::<PuntuacionGlobal>();
        score.0 += 8;
        assert_eq!(score.0, 50);
    }

    #[test]
    fn moneda_recogida_event_roundtrip() {
        // snippet 7 — Message round-trip using Messages<T> resource
        let mut app = App::new();
        app.add_message::<MonedaRecogida>();

        // Write a message directly via the Messages resource
        app.world_mut()
            .resource_mut::<Messages<MonedaRecogida>>()
            .write(MonedaRecogida { cantidad: 10 });

        // First update: swap buffer so cursor can see the message
        app.world_mut()
            .resource_mut::<Messages<MonedaRecogida>>()
            .update();

        // Read back using cursor
        let messages = app.world_mut().resource_mut::<Messages<MonedaRecogida>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].cantidad, 10);
    }

    #[test]
    fn entity_world_mut_inserts_component() {
        // snippet 9 — EntityWorldMut + .insert()
        // Demonstrates that commands.spawn() returns EntityWorldMut,
        // which lets us .insert() a component immediately (synchronously).
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let entity = commands.spawn(Position { x: 0.0, y: 0.0 }).id();
            commands.entity(entity).insert(Velocity { x: 10.0, y: 0.0 });
        });
        app.update(); // flush command buffer

        let (entity, pos) = app
            .world_mut()
            .query::<(Entity, &Position)>()
            .iter(app.world())
            .next()
            .unwrap();
        let vel = app.world().get::<Velocity>(entity).unwrap();
        assert_eq!(pos.x, 0.0);
        assert_eq!(vel.x, 10.0);
    }

    #[test]
    fn mover_cubo_with_time() {
        // snippet 10 — mover_cubo with Res<Time>
        let mut app = build_app_with_time();
        app.init_resource::<Time>(); // Time is not added by default
        // Advance time by one frame (1/60 second)
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();

        // velocity 50 px/s * (1/60) s ≈ 0.833 px
        let mut positions = app.world_mut().query::<&Posicion>();
        let pos = positions.single(app.world()).expect("one cube");
        assert!(pos.0 > 0.0, "cube should have moved forward");
    }

    #[test]
    fn transform_velocity_moves_entity() {
        // snippet 12 — Velocidad2D + Transform
        let mut app = App::new();
        app.init_resource::<Time>(); // Time is not added by default
        app.world_mut()
            .spawn((Velocidad2D { x: 30.0, y: 0.0 }, Transform::default()));

        // Advance time by 1/60s and run the system
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));

        let dt = app.world().resource::<Time>().delta_secs();
        let mut query = app.world_mut().query::<(&Velocidad2D, &mut Transform)>();
        for (vel, mut tf) in query.iter_mut(app.world_mut()) {
            tf.translation.x += vel.x * dt;
            tf.translation.y += vel.y * dt;
        }

        let tf = app
            .world_mut()
            .query::<&Transform>()
            .single(app.world())
            .expect("one transform");
        assert!(tf.translation.x > 0.0, "entity should have moved in X");
    }
}
