// snippet 14 from cap-05.html §5.4 — Entity id in queries + despawn
// Run: cargo run --example entity_query
use bevy::prelude::*;
use bevy_book_chapter_05::Health;

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();

    // Spawn entities and capture IDs
    let e1 = app
        .world_mut()
        .spawn((Health {
            actual: 100,
            max: 100,
        },))
        .id();
    let e2 = app
        .world_mut()
        .spawn((Health {
            actual: 50,
            max: 50,
        },))
        .id();
    let e3 = app
        .world_mut()
        .spawn((Health {
            actual: 30,
            max: 30,
        },))
        .id();

    // Kill one entity before running the system
    app.world_mut().get_mut::<Health>(e1).unwrap().actual = 0;
    println!("Entity {:?} has been killed (health = 0)", e1);

    app.add_systems(Update, matar_a_todo_lo_que_tenga_bandage);
    app.update();

    println!("\nFrame done. Remaining entities:");
    println!(
        "  e1 {:?} — alive? {}",
        e1,
        app.world().get::<Health>(e1).is_some()
    );
    println!(
        "  e2 {:?} — alive? {}",
        e2,
        app.world().get::<Health>(e2).is_some()
    );
    println!(
        "  e3 {:?} — alive? {}",
        e3,
        app.world().get::<Health>(e3).is_some()
    );
}

/// Despawns entities whose Health is <= 0
fn matar_a_todo_lo_que_tenga_bandage(query: Query<(Entity, &Health)>, mut commands: Commands) {
    for (entity, health) in &query {
        if health.actual <= 0 {
            println!("Despawning {:?} (health {})", entity, health.actual);
            commands.entity(entity).despawn();
        }
    }
}
