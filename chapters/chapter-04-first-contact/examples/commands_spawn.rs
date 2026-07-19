// snippet 8 from cap-04.html — Commands spawn with Name
// Run: cargo run --example commands_spawn
use bevy::prelude::*;
use bevy_book_chapter_04::{Enemy, Player, Position};

fn main() {
    let mut app = App::new();

    app.add_systems(Startup, spawnear_enemigo);
    app.add_systems(Startup, spawnear_jugador);

    app.update();

    // Verify — query requires &mut World
    let mut all = app.world_mut().query::<(&Name, &Position)>();
    for (name, pos) in all.iter(app.world()) {
        println!("{} at ({}, {})", name, pos.x, pos.y);
    }
}

fn spawnear_enemigo(mut commands: Commands) {
    commands.spawn((Name::new("Enemigo"), Position { x: 100.0, y: 50.0 }, Enemy));
}

fn spawnear_jugador(mut commands: Commands) {
    commands.spawn((Name::new("Player"), Position { x: 0.0, y: 0.0 }, Player));
}
