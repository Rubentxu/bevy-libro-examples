// snippets 8, 9, 10, 11 from cap-05.html §5.2ter — Tuple queries, Option<T>
// Run: cargo run --example tuple_query
use bevy::prelude::*;
use bevy_book_chapter_05::{Player, Velocidad, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    // Give player a Velocidad
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut().entity_mut(player).insert(Velocidad {
        x: 10.0,
        y: 0.0,
        giro: 0.5,
    });

    app.add_systems(Update, (mover_y_girar, opcional_velocidad));
    app.update();
}

/// Tuple query: (&mut Transform, &Velocidad) — snippet 8
fn mover_y_girar(mut query: Query<(&mut Transform, &Velocidad)>, time: Res<Time>) {
    let dt = time.delta_secs();
    let mut count = 0;
    for (mut transform, vel) in &mut query {
        transform.translation.x += vel.x * dt;
        transform.translation.y += vel.y * dt;
        transform.rotate_z(vel.giro * dt);
        count += 1;
    }
    println!("mover_y_girar: {} entities with Velocidad", count);
}

/// Option<&Velocidad> — snippet 11. Returns Some where present, None otherwise.
fn opcional_velocidad(query: Query<(&Transform, Option<&Velocidad>)>) {
    let mut with_vel = 0;
    let mut without_vel = 0;
    for (transform, maybe_vel) in &query {
        match maybe_vel {
            Some(vel) => {
                with_vel += 1;
                println!("  With vel {} at {:?}", vel.x, transform.translation);
            }
            None => {
                without_vel += 1;
            }
        }
    }
    println!(
        "opcional_velocidad: {} with, {} without",
        with_vel, without_vel
    );
}
