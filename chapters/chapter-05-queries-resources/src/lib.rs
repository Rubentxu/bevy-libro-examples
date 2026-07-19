// Capítulo 5. Queries y Resources
// All public types + systems shared across examples.

use bevy::prelude::*;

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component, Debug, PartialEq)]
pub struct Player;

#[derive(Component, Debug, PartialEq)]
pub struct Enemy;

/// Marks an entity as dead (for `Without<Dead>` filters)
#[derive(Component, Debug, PartialEq)]
pub struct Muerto;

/// Health component (snippet 4, §5.1)
#[derive(Component, Debug, PartialEq)]
pub struct Health {
    pub actual: i32,
    pub max: i32,
}

/// Alternative Vida component (used by Or filter example, §5.2bis)
#[derive(Component, Debug, PartialEq)]
pub struct Vida {
    pub actual: i32,
    pub max: i32,
}

/// Velocity with rotation (used by tuple query example, §5.2ter)
#[derive(Component, Debug, PartialEq)]
pub struct Velocidad {
    pub x: f32,
    pub y: f32,
    pub giro: f32,
}

/// Alert flag (snippet 3, §5.1)
#[derive(Component, Debug, PartialEq)]
pub struct Alerta(pub bool);

// ============================================================================
// RESOURCES
// ============================================================================

/// Global score resource (snippet 15, §5.5)
#[derive(Resource, Debug, PartialEq, Default)]
pub struct PuntuacionGlobal {
    pub puntos: u32,
    pub record: u32,
}

/// Game configuration resource (snippet 15, §5.5)
#[derive(Resource, Debug)]
pub struct ConfiguracionJuego {
    pub volumen: f32,
    pub musica: bool,
    pub efectos: bool,
}

/// Best score resource initialized via FromWorld (snippet 18, §5.6)
#[derive(Resource, Debug, PartialEq)]
pub struct MejorPuntuacion {
    pub valor: u32,
}

impl FromWorld for MejorPuntuacion {
    fn from_world(_world: &mut World) -> Self {
        // In a real game: read from disk, from another resource, etc.
        let cargada_de_disco: Option<u32> = None; // leer_de_disco()
        Self {
            valor: cargada_de_disco.unwrap_or(0),
        }
    }
}

// ============================================================================
// APP BUILDERS
// ============================================================================

/// Headless app for query/filter demos
pub fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app
}

/// App populated with player + 2 enemies for query examples
pub fn build_combat_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.world_mut().spawn((
        Player,
        Health {
            actual: 100,
            max: 100,
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
    app.world_mut().spawn((
        Enemy,
        Health {
            actual: 50,
            max: 50,
        },
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
    ));
    app.world_mut().spawn((
        Enemy,
        Health {
            actual: 30,
            max: 30,
        },
        Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)),
    ));
    app
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 5.1 With / Without filters ---

    #[test]
    fn with_filter_returns_only_matching() {
        let mut app = build_combat_app();
        let mut q = app.world_mut().query_filtered::<&Transform, With<Player>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "only one Player");
    }

    #[test]
    fn without_filter_excludes_matching() {
        let mut app = build_combat_app();
        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, Without<Player>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 2, "two non-player entities (enemies)");
    }

    #[test]
    fn with_without_combined() {
        // snippet 2c: With<Player> AND Without<Enemy>
        let mut app = App::new();
        app.init_resource::<Time>();
        app.world_mut().spawn((Player, Transform::default()));
        app.world_mut().spawn((Player, Enemy, Transform::default())); // ambiguous
        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, (With<Player>, Without<Enemy>)>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "only Player without Enemy");
    }

    // --- 5.1 Added filter ---

    #[test]
    fn added_filter_detects_new_components() {
        let mut app = build_combat_app();
        // Add a new enemy in frame 1
        app.update();
        app.world_mut().spawn((Enemy, Health { actual: 1, max: 1 }));

        // Now query Added<Enemy> should see the new one
        let mut q = app.world_mut().query_filtered::<Entity, Added<Enemy>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "one enemy was added this frame");
    }

    // --- 5.2 Changed filter ---

    #[test]
    fn changed_filter_detects_mutations() {
        let mut app = build_combat_app();
        app.update();
        // Mutate one enemy's Health
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        let mut health = app.world_mut().get_mut::<Health>(enemy).unwrap();
        health.actual -= 10;

        let mut q = app.world_mut().query_filtered::<&Health, Changed<Health>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "one health was mutated");
    }

    // --- 5.2 Ref<T> change detection ---

    #[test]
    fn ref_detects_changes() {
        let mut app = build_combat_app();
        app.update();
        // Mutate player's Transform
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .next()
            .unwrap();
        {
            let mut tf = app.world_mut().get_mut::<Transform>(player).unwrap();
            tf.translation.x = 42.0;
        }

        let mut q = app.world_mut().query::<Ref<Transform>>();
        let changed_count = q.iter(app.world()).filter(|t| t.is_changed()).count();
        assert_eq!(
            changed_count, 1,
            "only player's transform should be changed"
        );
    }

    // --- 5.2bis Or filter ---

    #[test]
    fn or_filter_combines_conditions() {
        let mut app = build_combat_app();
        app.update();
        // Mutate one enemy's Health
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        let mut health = app.world_mut().get_mut::<Health>(enemy).unwrap();
        health.actual -= 10;

        // Now Added<Enemy> should fire on new entities, Changed<Health> on mutated
        // Add a new enemy
        app.world_mut().spawn((Enemy, Health { actual: 1, max: 1 }));

        let mut q = app
            .world_mut()
            .query_filtered::<&Health, Or<(Added<Enemy>, Changed<Health>)>>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 2, "one added + one changed");
    }

    // --- 5.2ter Tuple queries ---

    #[test]
    fn tuple_query_multiple_components() {
        let mut app = build_combat_app();
        // Add Velocidad to player using entity_mut (returns EntityWorldMut)
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().entity_mut(player).insert(Velocidad {
            x: 10.0,
            y: 0.0,
            giro: 0.5,
        });

        let mut q = app.world_mut().query::<(&Transform, &Velocidad)>();
        let count = q.iter(app.world()).count();
        assert_eq!(count, 1, "only player has Velocidad");
    }

    #[test]
    fn option_in_query_returns_none_when_missing() {
        let mut app = build_combat_app();
        let mut q = app.world_mut().query::<(&Transform, Option<&Velocidad>)>();
        let mut with_vel = 0;
        let mut without_vel = 0;
        for (_tf, vel) in q.iter(app.world()) {
            if vel.is_some() {
                with_vel += 1;
            } else {
                without_vel += 1;
            }
        }
        assert_eq!(with_vel, 0, "no entity has Velocidad");
        assert_eq!(without_vel, 3, "all three entities have Transform");
    }

    // --- 5.4 Entity id ---

    #[test]
    fn entity_in_query_returns_id() {
        let mut app = build_combat_app();
        // Despawn dead entities (none yet)
        let dead: Vec<Entity> = app
            .world_mut()
            .query::<(Entity, &Health)>()
            .iter(app.world())
            .filter(|(_, h)| h.actual <= 0)
            .map(|(e, _)| e)
            .collect();
        assert_eq!(dead.len(), 0, "no dead entities initially");

        // Kill one and despawn via Commands
        app.add_systems(
            Update,
            |mut commands: Commands, q: Query<(Entity, &Health)>| {
                for (e, h) in &q {
                    if h.actual <= 0 {
                        commands.entity(e).despawn();
                    }
                }
            },
        );
        // Make an enemy dead
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().get_mut::<Health>(enemy).unwrap().actual = 0;
        app.update();

        let mut enemies = app.world_mut().query_filtered::<(), With<Enemy>>();
        let count = enemies.iter(app.world()).count();
        assert_eq!(count, 1, "one enemy despawned, one remains");
    }

    // --- 5.5 Resources ---

    #[test]
    fn resources_read_and_write() {
        let mut app = App::new();
        app.insert_resource(PuntuacionGlobal {
            puntos: 100,
            record: 500,
        });
        app.insert_resource(ConfiguracionJuego {
            volumen: 0.8,
            musica: true,
            efectos: false,
        });

        let score = app.world().resource::<PuntuacionGlobal>();
        assert_eq!(score.puntos, 100);
        assert_eq!(score.record, 500);

        let config = app.world().resource::<ConfiguracionJuego>();
        assert_eq!(config.volumen, 0.8);
        assert!(config.musica);
        assert!(!config.efectos);

        // ResMut
        let mut score = app.world_mut().resource_mut::<PuntuacionGlobal>();
        score.puntos += 50;
        assert_eq!(score.puntos, 150);
    }

    #[test]
    fn init_resource_uses_default() {
        let mut app = App::new();
        app.init_resource::<PuntuacionGlobal>();
        let score = app.world().resource::<PuntuacionGlobal>();
        assert_eq!(score.puntos, 0);
        assert_eq!(score.record, 0);
    }

    // --- 5.6 FromWorld ---

    #[test]
    fn from_world_initializes_resource() {
        let mut app = App::new();
        app.init_resource::<MejorPuntuacion>();
        let best = app.world().resource::<MejorPuntuacion>();
        // FromWorld returns 0 since leer_de_disco() returns None
        assert_eq!(best.valor, 0);
    }

    // --- 5.8 Local<T> ---

    #[test]
    fn local_persists_across_frames() {
        #[derive(Resource, Default)]
        struct FrameCounter(u32);

        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(FrameCounter::default());

        // Single system that uses Local to count frames AND mirrors to a resource
        app.add_systems(
            Update,
            |mut contador: Local<u32>, mut global: ResMut<FrameCounter>| {
                *contador += 1;
                global.0 = *contador;
            },
        );

        app.update();
        app.update();
        app.update();

        let counter = app.world().resource::<FrameCounter>();
        assert_eq!(counter.0, 3, "Local should have counted 3 frames");
    }
}
