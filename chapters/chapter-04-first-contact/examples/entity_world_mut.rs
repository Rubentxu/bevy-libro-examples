// snippet 9 from cap-04.html — EntityWorldMut
// Run: cargo run --example entity_world_mut
use bevy::prelude::*;
use bevy_book_chapter_04::{Player, Position, Velocity};

fn main() {
    let mut app = App::new();

    app.add_systems(Startup, spawnear_con_posicion);

    app.update();

    // Verify the entity has both Position and Velocity
    let (entity, pos) = app
        .world_mut()
        .query::<(Entity, &Position)>()
        .iter(app.world())
        .next()
        .expect("entity with Position + Velocity");

    let vel = app.world().get::<Velocity>(entity).unwrap();
    println!(
        "Entity {} has Position({}, {}) and Velocity({}, {})",
        entity, pos.x, pos.y, vel.x, vel.y
    );
}

fn spawnear_con_posicion(mut commands: Commands) {
    // EntityWorldMut returned by spawn allows immediate .insert()
    let entity = commands.spawn((Position { x: 0.0, y: 0.0 }, Player)).id();
    // Insert Velocity without waiting for the system to end
    commands.entity(entity).insert(Velocity { x: 50.0, y: 0.0 });
}
