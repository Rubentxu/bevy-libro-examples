// snippet 8 from cap-04.html — Commands spawn with Name
use bevy::prelude::*;
use bevy_book_chapter_04::{Enemy, Player, Position};

fn main() {
    let mut app = App::new();
    app.add_systems(Startup, (spawn_enemy, spawn_player_entity));

    app.update();

    let mut all = app.world_mut().query::<(&Name, &Position)>();
    for (name, pos) in all.iter(app.world()) {
        println!("{} at ({}, {})", name, pos.x, pos.y);
    }
}

fn spawn_enemy(mut commands: Commands) {
    commands.spawn((Name::new("Enemy"), Position { x: 100.0, y: 50.0 }, Enemy));
}

fn spawn_player_entity(mut commands: Commands) {
    commands.spawn((Name::new("Player"), Position { x: 0.0, y: 0.0 }, Player));
}
