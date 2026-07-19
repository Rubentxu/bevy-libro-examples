// §6.7 — Complete combat example with DamageMessage/DeathMessage
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_06::{
    DamageMessage, DeathMessage, Enemy, Health, Player, Score, attack, process_damage,
    react_to_death,
};
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>();
    app.add_message::<DamageMessage>();
    app.add_message::<DeathMessage>();
    app.init_resource::<Score>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (attack, process_damage, react_to_death).chain());

    for i in 0..8 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
        println!("--- Frame {} done ---", i + 1);
    }
    println!(
        "\nFinal score: {} points",
        app.world().resource::<Score>().0
    );
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Health {
            current: 100.0,
            max: 100.0,
        },
        Player,
    ));
    commands.spawn((
        Health {
            current: 50.0,
            max: 50.0,
        },
        Enemy,
    ));
    commands.spawn((
        Health {
            current: 30.0,
            max: 30.0,
        },
        Enemy,
    ));
}
