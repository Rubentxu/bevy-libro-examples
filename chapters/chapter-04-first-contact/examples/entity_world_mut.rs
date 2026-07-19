// snippet 9 from cap-04.html — EntityWorldMut
use bevy::prelude::*;
use bevy_book_chapter_04::{Player, Position, Velocity};

fn main() {
    let mut app = App::new();
    app.add_systems(Startup, spawn_with_position);
    app.update();

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

fn spawn_with_position(mut commands: Commands) {
    let entity = commands.spawn((Position { x: 0.0, y: 0.0 }, Player)).id();
    commands.entity(entity).insert(Velocity { x: 50.0, y: 0.0 });
}
