// snippet 2 from cap-05.html §5.1 — With / Without filters
// Run: cargo run --example with_without
use bevy::prelude::*;
use bevy_book_chapter_05::{Enemy, Player, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    app.add_systems(
        Update,
        (mover_jugador, mover_no_enemigos, mover_players_puros),
    );

    app.update();
    println!("Frame completed.");
}

/// Only entities that have Player component
fn mover_jugador(mut query: Query<&mut Transform, With<Player>>) {
    let mut count = 0;
    for mut transform in &mut query {
        transform.translation.x += 1.0;
        count += 1;
    }
    println!("mover_jugador: {} entities", count);
}

/// Only entities that DO NOT have Enemy component
fn mover_no_enemigos(mut query: Query<&mut Transform, Without<Enemy>>) {
    let mut count = 0;
    for mut transform in &mut query {
        transform.translation.x += 1.0;
        count += 1;
    }
    println!("mover_no_enemigos: {} entities", count);
}

/// Entities that have Player AND do NOT have Enemy
fn mover_players_puros(mut query: Query<&mut Transform, (With<Player>, Without<Enemy>)>) {
    let mut count = 0;
    for mut transform in &mut query {
        transform.translation.x += 1.0;
        count += 1;
    }
    println!("mover_players_puros: {} entities", count);
}
