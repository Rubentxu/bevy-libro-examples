use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy_book_chapter_05::{Enemy, Player, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    app.add_systems(Update, compare_positions);
    app.update();
}

fn compare_positions(
    mut q: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
) {
    let player_pos = q.p0().single().expect("one player").translation;
    for mut enemy_tf in q.p1().iter_mut() {
        let distance = (enemy_tf.translation - player_pos).length();
        println!("Distance to enemy: {:.2}", distance);
        enemy_tf.translation.x += 0.1;
    }
}
