use bevy::prelude::*;
use bevy_book_chapter_05::{Enemy, Player, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    app.add_systems(Update, (move_player, move_non_enemies, move_pure_players));
    app.update();
}

fn move_player(mut query: Query<&mut Transform, With<Player>>) {
    for mut tf in &mut query {
        tf.translation.x += 1.0;
    }
    println!("move_player: {} entities", query.iter().count());
}

fn move_non_enemies(mut query: Query<&mut Transform, Without<Enemy>>) {
    for mut tf in &mut query {
        tf.translation.x += 1.0;
    }
    println!("move_non_enemies: {} entities", query.iter().count());
}

fn move_pure_players(mut query: Query<&mut Transform, (With<Player>, Without<Enemy>)>) {
    for mut tf in &mut query {
        tf.translation.x += 1.0;
    }
    println!("move_pure_players: {} entities", query.iter().count());
}
