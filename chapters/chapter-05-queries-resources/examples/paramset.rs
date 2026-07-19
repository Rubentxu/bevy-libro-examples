// snippet 13 from cap-05.html §5.3 — ParamSet for conflicting queries
// Run: cargo run --example paramset
//
// NOTE: Bevy 0.18+ deprecated QuerySet in favor of ParamSet.
// ParamSet gives you exclusive access to one slot at a time via p0(), p1().
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy_book_chapter_05::{Enemy, Player, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    app.add_systems(Update, comparar_posiciones);
    app.update();
}

/// Uses ParamSet to access two conflicting queries (same component, different filters).
/// Each slot is accessed via p0(), p1(), etc. — only one at a time.
fn comparar_posiciones(
    mut q: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
) {
    // Read player position from slot 0
    // NOTE: single() returns Result in Bevy 0.19
    let player_pos = q.p0().single().expect("one player").translation;

    // Move each enemy away from player in slot 1
    for mut enemy_tf in q.p1().iter_mut() {
        let diff = enemy_tf.translation - player_pos;
        let distance = diff.length();
        println!("Distancia al enemy: {:.2}", distance);
        enemy_tf.translation.x += 0.1;
    }
}
