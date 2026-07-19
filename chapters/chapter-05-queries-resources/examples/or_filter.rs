use bevy::prelude::*;
use bevy_book_chapter_05::{Dead, Enemy, Health, Player, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    let entities: Vec<Entity> = app
        .world_mut()
        .query::<Entity>()
        .iter(app.world())
        .collect();
    for e in entities {
        app.world_mut().entity_mut(e).insert(Health {
            current: 50,
            max: 100,
        });
    }
    let enemy = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut().entity_mut(enemy).insert(Dead);

    app.add_systems(Update, (refresh_health_bar, list_combatants));
    app.update();
    app.world_mut().spawn((
        Enemy,
        Health {
            current: 20,
            max: 20,
        },
    ));
    app.update();
}

fn refresh_health_bar(query: Query<&Health, Or<(Added<Enemy>, Changed<Health>)>>) {
    let mut count = 0;
    for health in &query {
        count += 1;
        println!("Refresh HUD: health {}/{}", health.current, health.max);
    }
    println!("refresh_health_bar: {} entities", count);
}

fn list_combatants(query: Query<Entity, (Or<(With<Player>, With<Enemy>)>, Without<Dead>)>) {
    for entity in &query {
        println!("Combatant: {:?}", entity);
    }
}
