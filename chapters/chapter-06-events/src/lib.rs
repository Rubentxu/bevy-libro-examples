// Capítulo 6. Events (Messages in Bevy 0.19) — English identifiers

use bevy::prelude::*;

// ============================================================================
// MESSAGE TYPES (§6.2)
// ============================================================================

#[derive(Message, Debug)]
pub struct DamageReceived {
    pub entity: Entity,
    pub amount: f32,
}

#[derive(Message, Debug)]
pub struct CoinCollected {
    pub amount: u32,
    pub position: (f32, f32),
}

#[derive(Message, Debug)]
pub struct LevelChange {
    pub new_level: usize,
}

// Combat messages (§6.7)
#[derive(Message, Debug)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
}

#[derive(Message, Debug)]
pub struct DeathMessage {
    pub victim: Entity,
    pub killer: Option<Entity>,
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Hitbox {
    pub should_take_damage: bool,
}

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct Score(pub u32);

// ============================================================================
// APP BUILDER
// ============================================================================

pub fn build_combat_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Score>();
    app.add_message::<DamageMessage>();
    app.add_message::<DeathMessage>();
    app.add_message::<DamageReceived>();
    app.add_message::<CoinCollected>();
    app.add_message::<LevelChange>();
    app.world_mut().spawn((
        Health {
            current: 100.0,
            max: 100.0,
        },
        Player,
    ));
    app.world_mut().spawn((
        Health {
            current: 50.0,
            max: 50.0,
        },
        Enemy,
    ));
    app.world_mut().spawn((
        Health {
            current: 30.0,
            max: 30.0,
        },
        Enemy,
    ));
    app
}

// ============================================================================
// COMBAT SYSTEMS (§6.7)
// ============================================================================

pub fn attack(enemies: Query<Entity, With<Enemy>>, mut writer: MessageWriter<DamageMessage>) {
    if let Some(target) = enemies.iter().next() {
        writer.write(DamageMessage {
            target,
            amount: 25.0,
        });
    }
}

pub fn process_damage(
    mut reader: MessageReader<DamageMessage>,
    mut query: Query<&mut Health>,
    mut death_writer: MessageWriter<DeathMessage>,
) {
    for event in reader.read() {
        if let Ok(mut health) = query.get_mut(event.target) {
            health.current -= event.amount;
            if health.current <= 0.0 {
                death_writer.write(DeathMessage {
                    victim: event.target,
                    killer: None,
                });
            }
        }
    }
}

pub fn react_to_death(
    mut reader: MessageReader<DeathMessage>,
    mut score: ResMut<Score>,
    mut commands: Commands,
) {
    for event in reader.read() {
        println!("Entity {:?} has died!", event.victim);
        score.0 += 100;
        commands.entity(event.victim).despawn();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_send_and_receive_roundtrip() {
        let mut app = App::new();
        app.add_message::<DamageReceived>();
        app.world_mut()
            .resource_mut::<Messages<DamageReceived>>()
            .write(DamageReceived {
                entity: Entity::PLACEHOLDER,
                amount: 25.0,
            });
        app.world_mut()
            .resource_mut::<Messages<DamageReceived>>()
            .update();
        let messages = app.world_mut().resource_mut::<Messages<DamageReceived>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].amount, 25.0);
    }

    #[test]
    fn multiple_messages_same_frame() {
        let mut app = App::new();
        app.add_message::<CoinCollected>();
        {
            let mut m = app.world_mut().resource_mut::<Messages<CoinCollected>>();
            m.write(CoinCollected {
                amount: 5,
                position: (10.0, 20.0),
            });
            m.write(CoinCollected {
                amount: 10,
                position: (30.0, 40.0),
            });
            m.write(CoinCollected {
                amount: 15,
                position: (50.0, 60.0),
            });
        }
        app.world_mut()
            .resource_mut::<Messages<CoinCollected>>()
            .update();
        let messages = app.world_mut().resource_mut::<Messages<CoinCollected>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn combat_damage_reduces_health() {
        let mut app = build_combat_app();
        app.add_systems(Update, (attack, process_damage, react_to_death).chain());
        for _ in 0..5 {
            app.update();
        }
        let enemy = app
            .world_mut()
            .query_filtered::<&Health, With<Enemy>>()
            .iter(app.world())
            .next();
        if let Some(h) = enemy {
            assert!(h.current < 50.0, "enemy should have taken damage");
        }
    }

    #[test]
    fn combat_death_triggers_on_zero_health() {
        let mut app = build_combat_app();
        app.add_systems(Update, (attack, process_damage, react_to_death).chain());
        for _ in 0..10 {
            app.update();
        }
        assert!(app.world().resource::<Score>().0 > 0);
    }

    #[test]
    fn double_buffer_persists_messages_across_frames() {
        let mut app = App::new();
        app.add_message::<LevelChange>();
        app.world_mut()
            .resource_mut::<Messages<LevelChange>>()
            .write(LevelChange { new_level: 2 });
        let mut cursor = {
            let mut m = app.world_mut().resource_mut::<Messages<LevelChange>>();
            let c = m.get_cursor();
            m.update();
            c
        };
        let messages = app.world().resource::<Messages<LevelChange>>();
        assert!(!cursor.read(messages).collect::<Vec<_>>().is_empty());
    }

    #[test]
    fn reader_read_consumes_events() {
        let mut app = App::new();
        app.add_message::<DamageReceived>();
        app.world_mut()
            .resource_mut::<Messages<DamageReceived>>()
            .write(DamageReceived {
                entity: Entity::PLACEHOLDER,
                amount: 10.0,
            });
        app.world_mut()
            .resource_mut::<Messages<DamageReceived>>()
            .update();
        let messages = app.world_mut().resource_mut::<Messages<DamageReceived>>();
        let mut cursor = messages.get_cursor();
        assert_eq!(cursor.read(&messages).collect::<Vec<_>>().len(), 1);
        assert_eq!(cursor.read(&messages).collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn multiple_message_types_independent() {
        let mut app = App::new();
        app.add_message::<CoinCollected>();
        app.add_message::<LevelChange>();
        app.world_mut()
            .resource_mut::<Messages<CoinCollected>>()
            .write(CoinCollected {
                amount: 5,
                position: (1.0, 2.0),
            });
        app.world_mut()
            .resource_mut::<Messages<LevelChange>>()
            .write(LevelChange { new_level: 3 });
        app.world_mut()
            .resource_mut::<Messages<CoinCollected>>()
            .update();
        app.world_mut()
            .resource_mut::<Messages<LevelChange>>()
            .update();
        let coins = app.world_mut().resource_mut::<Messages<CoinCollected>>();
        let mut c1 = coins.get_cursor();
        assert_eq!(c1.read(&coins).collect::<Vec<_>>().len(), 1);
        let levels = app.world_mut().resource_mut::<Messages<LevelChange>>();
        let mut c2 = levels.get_cursor();
        assert_eq!(c2.read(&levels).collect::<Vec<_>>().len(), 1);
    }
}
