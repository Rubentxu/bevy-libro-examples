// Capítulo 6. Eventos (Messages en Bevy 0.19)
//
// NOTE CRÍTICA: Bevy 0.19 renombró toda la API de eventos:
//   Event         → Message
//   EventWriter   → MessageWriter
//   EventReader   → MessageReader
//   Events<T>     → Messages<T>
//   add_event     → add_message
//
// El libro original usa la API vieja (Event/EventWriter/EventReader).
// Este código usa la API correcta de Bevy 0.19.

use bevy::prelude::*;

// ============================================================================
// MESSAGE TYPES (snippets §6.2)
// ============================================================================

/// Daño recibido por una entity (snippet §6.2)
#[derive(Message, Debug)]
pub struct DanoRecibido {
    pub entidad: Entity,
    pub cantidad: f32,
}

/// Moneda recogida por el player (snippet §6.2)
#[derive(Message, Debug)]
pub struct MonedaRecogida {
    pub cantidad: u32,
    pub posicion: (f32, f32),
}

/// Cambio de nivel (snippet §6.2)
#[derive(Message, Debug)]
pub struct CambioDeNivel {
    pub nuevo_nivel: usize,
}

// ============================================================================
// COMBAT EXAMPLE TYPES (snippet §6.7)
// ============================================================================

/// Damage message — target recibe cantidad de daño
#[derive(Message, Debug)]
pub struct DamageMessage {
    pub target: Entity,
    pub cantidad: f32,
}

/// Death message — victim ha muerto
#[derive(Message, Debug)]
pub struct DeathMessage {
    pub victim: Entity,
    pub killer: Option<Entity>,
}

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component, Debug)]
pub struct Vida {
    pub actual: f32,
    pub max: f32,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Hitbox {
    pub debe_recibir_dano: bool,
}

// ============================================================================
// RESOURCES
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct Puntuacion(pub u32);

// ============================================================================
// APP BUILDER
// ============================================================================

/// App de combate headless con messages registrados
pub fn build_combat_app() -> App {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Puntuacion>();

    // Registrar messages (Bevy 0.19 API)
    app.add_message::<DamageMessage>();
    app.add_message::<DeathMessage>();
    app.add_message::<DanoRecibido>();
    app.add_message::<MonedaRecogida>();
    app.add_message::<CambioDeNivel>();

    // Spawn player + enemies
    app.world_mut().spawn((
        Vida {
            actual: 100.0,
            max: 100.0,
        },
        Player,
    ));
    app.world_mut().spawn((
        Vida {
            actual: 50.0,
            max: 50.0,
        },
        Enemy,
    ));
    app.world_mut().spawn((
        Vida {
            actual: 30.0,
            max: 30.0,
        },
        Enemy,
    ));

    app
}

// ============================================================================
// COMBAT SYSTEMS (snippet §6.7)
// ============================================================================

/// System que envía DamageMessage al primer enemy
pub fn atacar(enemies: Query<Entity, With<Enemy>>, mut writer: MessageWriter<DamageMessage>) {
    if let Some(target) = enemies.iter().next() {
        writer.write(DamageMessage {
            target,
            cantidad: 25.0,
        });
    }
}

/// System que procesa daño y emite muerte si vida <= 0
pub fn procesar_dano(
    mut reader: MessageReader<DamageMessage>,
    mut query: Query<&mut Vida>,
    mut death_writer: MessageWriter<DeathMessage>,
) {
    for evento in reader.read() {
        if let Ok(mut vida) = query.get_mut(evento.target) {
            vida.actual -= evento.cantidad;
            if vida.actual <= 0.0 {
                death_writer.write(DeathMessage {
                    victim: evento.target,
                    killer: None,
                });
            }
        }
    }
}

/// System que reacciona a la muerte: suma puntos, despawnea
pub fn reaccionar_a_muerte(
    mut reader: MessageReader<DeathMessage>,
    mut puntos: ResMut<Puntuacion>,
    mut commands: Commands,
) {
    for evento in reader.read() {
        println!("¡Entity {:?} ha muerto!", evento.victim);
        puntos.0 += 100;
        commands.entity(evento.victim).despawn();
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
        // §6.3-6.4 — MessageWriter sends, MessageReader receives
        let mut app = App::new();
        app.add_message::<DanoRecibido>();

        // Write a message directly via Messages resource
        app.world_mut()
            .resource_mut::<Messages<DanoRecibido>>()
            .write(DanoRecibido {
                entidad: Entity::PLACEHOLDER,
                cantidad: 25.0,
            });

        // Swap buffer so cursor can read it
        app.world_mut()
            .resource_mut::<Messages<DanoRecibido>>()
            .update();

        // Read via cursor
        let messages = app.world_mut().resource_mut::<Messages<DanoRecibido>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].cantidad, 25.0);
    }

    #[test]
    fn multiple_messages_same_frame() {
        // §6.3 — Multiple writes in one frame
        let mut app = App::new();
        app.add_message::<MonedaRecogida>();

        // Write 3 messages
        {
            let mut messages = app.world_mut().resource_mut::<Messages<MonedaRecogida>>();
            messages.write(MonedaRecogida {
                cantidad: 5,
                posicion: (10.0, 20.0),
            });
            messages.write(MonedaRecogida {
                cantidad: 10,
                posicion: (30.0, 40.0),
            });
            messages.write(MonedaRecogida {
                cantidad: 15,
                posicion: (50.0, 60.0),
            });
        }

        app.world_mut()
            .resource_mut::<Messages<MonedaRecogida>>()
            .update();

        let messages = app.world_mut().resource_mut::<Messages<MonedaRecogida>>();
        let mut cursor = messages.get_cursor();
        let events: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].cantidad, 5);
        assert_eq!(events[1].cantidad, 10);
        assert_eq!(events[2].cantidad, 15);
    }

    #[test]
    fn combat_damage_reduces_health() {
        // §6.7 — DamageMessage reduces Vida
        // Messages have 1-frame delay due to double-buffering
        let mut app = build_combat_app();
        app.add_systems(Update, (atacar, procesar_dano, reaccionar_a_muerte).chain());

        // Run multiple frames to allow message propagation
        for _ in 0..5 {
            app.update();
        }

        // After 5 frames, the 50 HP enemy should have taken significant damage
        // (25 per hit, delayed by 1 frame each)
        let enemy_health = app
            .world_mut()
            .query_filtered::<&Vida, With<Enemy>>()
            .iter(app.world())
            .next();

        if let Some(h) = enemy_health {
            assert!(
                h.actual < 50.0,
                "enemy should have taken damage, got {}",
                h.actual
            );
        }
        // If no enemy found, they all died — also valid
    }

    #[test]
    fn combat_death_triggers_on_zero_health() {
        // §6.7 — Eventually all enemies die from repeated attacks
        let mut app = build_combat_app();
        app.add_systems(Update, (atacar, procesar_dano, reaccionar_a_muerte).chain());

        // Run enough frames for all enemies to die
        for _ in 0..10 {
            app.update();
        }

        // Score should have increased from kills
        let score = app.world().resource::<Puntuacion>();
        assert!(score.0 > 0, "should have kills, got {} points", score.0);
    }

    #[test]
    fn double_buffer_persists_messages_across_frames() {
        // §6.5 — Double-buffering: message written in frame N is readable in frame N+1
        let mut app = App::new();
        app.add_message::<CambioDeNivel>();

        // Write message
        app.world_mut()
            .resource_mut::<Messages<CambioDeNivel>>()
            .write(CambioDeNivel { nuevo_nivel: 2 });

        // Create cursor BEFORE update, then swap buffer
        let mut cursor = {
            let mut messages = app.world_mut().resource_mut::<Messages<CambioDeNivel>>();
            let c = messages.get_cursor();
            messages.update(); // rotate buffer
            c
        };

        // Cursor should be able to read the message after rotation
        let messages = app.world().resource::<Messages<CambioDeNivel>>();
        let events: Vec<_> = cursor.read(messages).collect();
        assert!(
            !events.is_empty(),
            "message should persist via double-buffering"
        );
    }

    #[test]
    fn reader_read_consumes_events() {
        // §6.8 Error común #2: reader.read() dos veces → segunda vez vacío
        let mut app = App::new();
        app.add_message::<DanoRecibido>();

        // Write a message
        app.world_mut()
            .resource_mut::<Messages<DanoRecibido>>()
            .write(DanoRecibido {
                entidad: Entity::PLACEHOLDER,
                cantidad: 10.0,
            });
        app.world_mut()
            .resource_mut::<Messages<DanoRecibido>>()
            .update();

        // First read — should get 1 event
        let messages = app.world_mut().resource_mut::<Messages<DanoRecibido>>();
        // Note: cursor tracks position, so second read with SAME cursor gives 0
        let mut cursor = messages.get_cursor();
        let first_read: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(first_read.len(), 1);

        // Second read with same cursor — should be 0 (already consumed)
        let second_read: Vec<_> = cursor.read(&messages).collect();
        assert_eq!(second_read.len(), 0, "second read should be empty");
    }

    #[test]
    fn multiple_message_types_independent() {
        // §6.3 — Multiple message types don't interfere
        let mut app = App::new();
        app.add_message::<MonedaRecogida>();
        app.add_message::<CambioDeNivel>();

        // Write different message types
        app.world_mut()
            .resource_mut::<Messages<MonedaRecogida>>()
            .write(MonedaRecogida {
                cantidad: 5,
                posicion: (1.0, 2.0),
            });
        app.world_mut()
            .resource_mut::<Messages<CambioDeNivel>>()
            .write(CambioDeNivel { nuevo_nivel: 3 });

        app.world_mut()
            .resource_mut::<Messages<MonedaRecogida>>()
            .update();
        app.world_mut()
            .resource_mut::<Messages<CambioDeNivel>>()
            .update();

        // Read MonedaRecogida
        let coins = app.world_mut().resource_mut::<Messages<MonedaRecogida>>();
        let mut coin_cursor = coins.get_cursor();
        let coin_events: Vec<_> = coin_cursor.read(&coins).collect();
        assert_eq!(coin_events.len(), 1);

        // Read CambioDeNivel
        let levels = app.world_mut().resource_mut::<Messages<CambioDeNivel>>();
        let mut level_cursor = levels.get_cursor();
        let level_events: Vec<_> = level_cursor.read(&levels).collect();
        assert_eq!(level_events.len(), 1);
    }
}
