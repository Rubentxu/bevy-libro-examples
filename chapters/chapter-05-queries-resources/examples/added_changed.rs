use bevy::prelude::*;
use bevy_book_chapter_05::{Alert, Enemy, Health, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    app.add_systems(Update, (react_to_new_enemies, log_health_changes));
    app.update();
    app.world_mut().spawn((
        Enemy,
        Health {
            current: 25,
            max: 25,
        },
    ));
    app.update();
    let enemy = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut().get_mut::<Health>(enemy).unwrap().current -= 10;
    app.update();
}

fn react_to_new_enemies(query: Query<Entity, Added<Enemy>>, mut commands: Commands) {
    for entity in &query {
        println!("New Enemy spawned: {:?}!", entity);
        commands.entity(entity).insert(Alert(true));
    }
}

fn log_health_changes(query: Query<&Health, Changed<Health>>) {
    for health in &query {
        println!("Health changed to {}", health.current);
    }
}
