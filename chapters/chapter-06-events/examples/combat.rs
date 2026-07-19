// snippet §6.7 — Complete combat example with DamageMessage/DeathMessage
// Run: cargo run --example combat
//
// NOTE: Bevy 0.19 renombró Event→Message. Este código usa la API correcta.
use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_06::{
    DamageMessage, DeathMessage, Enemy, Player, Puntuacion, Vida, atacar, procesar_dano,
    reaccionar_a_muerte,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>();

    // Register message types (Bevy 0.19 API)
    app.add_message::<DamageMessage>();
    app.add_message::<DeathMessage>();
    app.init_resource::<Puntuacion>();

    // Spawn entities
    app.add_systems(Startup, setup);

    // Combat systems chained: atacar → procesar_dano → reaccionar_a_muerte
    app.add_systems(Update, (atacar, procesar_dano, reaccionar_a_muerte).chain());

    // Run several frames to simulate combat
    for i in 0..8 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
        println!("--- Frame {} done ---", i + 1);
    }

    let score = app.world().resource::<Puntuacion>();
    println!("\nFinal score: {} points", score.0);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Vida {
            actual: 100.0,
            max: 100.0,
        },
        Player,
    ));
    commands.spawn((
        Vida {
            actual: 50.0,
            max: 50.0,
        },
        Enemy,
    ));
    commands.spawn((
        Vida {
            actual: 30.0,
            max: 30.0,
        },
        Enemy,
    ));
}
