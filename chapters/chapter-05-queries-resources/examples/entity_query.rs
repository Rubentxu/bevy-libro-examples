use bevy::prelude::*;
use bevy_book_chapter_05::Health;

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();
    let e1 = app
        .world_mut()
        .spawn(Health {
            current: 100,
            max: 100,
        })
        .id();
    let e2 = app
        .world_mut()
        .spawn(Health {
            current: 50,
            max: 50,
        })
        .id();
    let e3 = app
        .world_mut()
        .spawn(Health {
            current: 30,
            max: 30,
        })
        .id();

    app.world_mut().get_mut::<Health>(e1).unwrap().current = 0;
    println!("Entity {:?} killed (health = 0)", e1);

    app.add_systems(Update, despawn_dead_entities);
    app.update();

    println!("\nRemaining:");
    println!(
        "  e1 {:?} alive? {}",
        e1,
        app.world().get::<Health>(e1).is_some()
    );
    println!(
        "  e2 {:?} alive? {}",
        e2,
        app.world().get::<Health>(e2).is_some()
    );
    println!(
        "  e3 {:?} alive? {}",
        e3,
        app.world().get::<Health>(e3).is_some()
    );
}

fn despawn_dead_entities(query: Query<(Entity, &Health)>, mut commands: Commands) {
    for (entity, health) in &query {
        if health.current <= 0 {
            println!("Despawning {:?} (health {})", entity, health.current);
            commands.entity(entity).despawn();
        }
    }
}
