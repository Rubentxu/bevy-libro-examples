// Capítulo 5. Queries y Resources — English identifiers

use bevy::prelude::*;

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component, Debug, PartialEq)]
pub struct Player;

#[derive(Component, Debug, PartialEq)]
pub struct Enemy;

/// Marks an entity as dead (for Without<Dead> filters)
#[derive(Component, Debug, PartialEq)]
pub struct Dead;

/// Health component (snippet 4, §5.1)
#[derive(Component, Debug, PartialEq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

/// Velocity with rotation (used by tuple query example, §5.2ter)
#[derive(Component, Debug, PartialEq)]
pub struct AngularVelocity {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}

/// Alert flag (snippet 3, §5.1)
#[derive(Component, Debug, PartialEq)]
pub struct Alert(pub bool);

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource, Debug, PartialEq, Default)]
pub struct GlobalScore {
    pub points: u32,
    pub record: u32,
}

#[derive(Resource, Debug)]
pub struct GameConfig {
    pub volume: f32,
    pub music: bool,
    pub effects: bool,
}

#[derive(Resource, Debug, PartialEq)]
pub struct BestScore {
    pub value: u32,
}

impl FromWorld for BestScore {
    fn from_world(_world: &mut World) -> Self {
        let loaded_from_disk: Option<u32> = None;
        Self {
            value: loaded_from_disk.unwrap_or(0),
        }
    }
}

// ============================================================================
// APP BUILDERS
// ============================================================================

pub fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app
}

pub fn build_combat_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.world_mut().spawn((
        Player,
        Health {
            current: 100,
            max: 100,
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
    app.world_mut().spawn((
        Enemy,
        Health {
            current: 50,
            max: 50,
        },
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
    ));
    app.world_mut().spawn((
        Enemy,
        Health {
            current: 30,
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

    #[test]
    fn with_filter_returns_only_matching() {
        let mut app = build_combat_app();
        let mut q = app.world_mut().query_filtered::<&Transform, With<Player>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn without_filter_excludes_matching() {
        let mut app = build_combat_app();
        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, Without<Player>>();
        assert_eq!(q.iter(app.world()).count(), 2);
    }

    #[test]
    fn with_without_combined() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.world_mut().spawn((Player, Transform::default()));
        app.world_mut().spawn((Player, Enemy, Transform::default()));
        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, (With<Player>, Without<Enemy>)>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn added_filter_detects_new_components() {
        let mut app = build_combat_app();
        app.update();
        app.world_mut()
            .spawn((Enemy, Health { current: 1, max: 1 }));
        let mut q = app.world_mut().query_filtered::<Entity, Added<Enemy>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn changed_filter_detects_mutations() {
        let mut app = build_combat_app();
        app.update();
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().get_mut::<Health>(enemy).unwrap().current -= 10;
        let mut q = app.world_mut().query_filtered::<&Health, Changed<Health>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn ref_detects_changes() {
        let mut app = build_combat_app();
        app.update();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .next()
            .unwrap();
        {
            app.world_mut()
                .get_mut::<Transform>(player)
                .unwrap()
                .translation
                .x = 42.0;
        }
        let mut q = app.world_mut().query::<Ref<Transform>>();
        assert_eq!(q.iter(app.world()).filter(|t| t.is_changed()).count(), 1);
    }

    #[test]
    fn or_filter_combines_conditions() {
        let mut app = build_combat_app();
        app.update();
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().get_mut::<Health>(enemy).unwrap().current -= 10;
        app.world_mut()
            .spawn((Enemy, Health { current: 1, max: 1 }));
        let mut q = app
            .world_mut()
            .query_filtered::<&Health, Or<(Added<Enemy>, Changed<Health>)>>();
        assert_eq!(q.iter(app.world()).count(), 2);
    }

    #[test]
    fn tuple_query_multiple_components() {
        let mut app = build_combat_app();
        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().entity_mut(player).insert(AngularVelocity {
            x: 10.0,
            y: 0.0,
            rotation: 0.5,
        });
        let mut q = app.world_mut().query::<(&Transform, &AngularVelocity)>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    #[test]
    fn option_in_query_returns_none_when_missing() {
        let mut app = build_combat_app();
        let mut q = app
            .world_mut()
            .query::<(&Transform, Option<&AngularVelocity>)>();
        let mut with = 0;
        let mut without = 0;
        for (_, vel) in q.iter(app.world()) {
            if vel.is_some() {
                with += 1;
            } else {
                without += 1;
            }
        }
        assert_eq!(with, 0);
        assert_eq!(without, 3);
    }

    #[test]
    fn entity_in_query_returns_id() {
        let mut app = build_combat_app();
        app.add_systems(
            Update,
            |mut commands: Commands, q: Query<(Entity, &Health)>| {
                for (e, h) in &q {
                    if h.current <= 0 {
                        commands.entity(e).despawn();
                    }
                }
            },
        );
        let enemy = app
            .world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .next()
            .unwrap();
        app.world_mut().get_mut::<Health>(enemy).unwrap().current = 0;
        app.update();
        let mut enemies = app.world_mut().query_filtered::<(), With<Enemy>>();
        assert_eq!(enemies.iter(app.world()).count(), 1);
    }

    #[test]
    fn resources_read_and_write() {
        let mut app = App::new();
        app.insert_resource(GlobalScore {
            points: 100,
            record: 500,
        });
        app.insert_resource(GameConfig {
            volume: 0.8,
            music: true,
            effects: false,
        });
        assert_eq!(app.world().resource::<GlobalScore>().points, 100);
        assert_eq!(app.world().resource::<GameConfig>().volume, 0.8);
        app.world_mut().resource_mut::<GlobalScore>().points += 50;
        assert_eq!(app.world().resource::<GlobalScore>().points, 150);
    }

    #[test]
    fn init_resource_uses_default() {
        let mut app = App::new();
        app.init_resource::<GlobalScore>();
        assert_eq!(app.world().resource::<GlobalScore>().points, 0);
    }

    #[test]
    fn from_world_initializes_resource() {
        let mut app = App::new();
        app.init_resource::<BestScore>();
        assert_eq!(app.world().resource::<BestScore>().value, 0);
    }

    #[test]
    fn local_persists_across_frames() {
        #[derive(Resource, Default)]
        struct FrameCounter(u32);

        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(FrameCounter::default());
        app.add_systems(
            Update,
            |mut counter: Local<u32>, mut global: ResMut<FrameCounter>| {
                *counter += 1;
                global.0 = *counter;
            },
        );
        app.update();
        app.update();
        app.update();
        assert_eq!(app.world().resource::<FrameCounter>().0, 3);
    }
}
