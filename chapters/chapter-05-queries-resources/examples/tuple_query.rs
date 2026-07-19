use bevy::prelude::*;
use bevy_book_chapter_05::{AngularVelocity, Player, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut().entity_mut(player).insert(AngularVelocity {
        x: 10.0,
        y: 0.0,
        rotation: 0.5,
    });

    app.add_systems(Update, (move_and_rotate, optional_velocity));
    app.update();
}

fn move_and_rotate(mut query: Query<(&mut Transform, &AngularVelocity)>, time: Res<Time>) {
    let dt = time.delta_secs();
    let mut count = 0;
    for (mut tf, vel) in &mut query {
        tf.translation.x += vel.x * dt;
        tf.translation.y += vel.y * dt;
        tf.rotate_z(vel.rotation * dt);
        count += 1;
    }
    println!("move_and_rotate: {} entities", count);
}

fn optional_velocity(query: Query<(&Transform, Option<&AngularVelocity>)>) {
    let (mut with, mut without) = (0, 0);
    for (tf, vel) in &query {
        match vel {
            Some(v) => {
                with += 1;
                println!("  With vel {} at {:?}", v.x, tf.translation);
            }
            None => {
                without += 1;
            }
        }
    }
    println!("optional_velocity: {} with, {} without", with, without);
}
