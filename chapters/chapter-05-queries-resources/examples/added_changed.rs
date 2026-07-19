// snippets 3, 4 from cap-05.html §5.1 — Added and Changed filters
// Run: cargo run --example added_changed
use bevy::prelude::*;
use bevy_book_chapter_05::{Alerta, Enemy, Health, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    app.add_systems(
        Update,
        (reaccionar_a_nuevos_enemigos, loggear_cambios_de_vida),
    );

    // Frame 1: existing enemies, nothing new yet
    println!("--- Frame 1 ---");
    app.update();

    // Spawn a new enemy → Added<Enemy> should fire
    println!("\n--- Spawn new enemy ---");
    app.world_mut().spawn((
        Enemy,
        Health {
            actual: 25,
            max: 25,
        },
    ));

    println!("\n--- Frame 2 ---");
    app.update();

    // Mutate an existing enemy's health → Changed<Health> should fire
    println!("\n--- Damage an enemy ---");
    let enemy = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .next()
        .unwrap();
    {
        let mut health = app.world_mut().get_mut::<Health>(enemy).unwrap();
        health.actual -= 10;
    }

    println!("\n--- Frame 3 ---");
    app.update();
}

/// Fires only for entities where Enemy was Added this frame
fn reaccionar_a_nuevos_enemigos(query: Query<Entity, Added<Enemy>>, mut commands: Commands) {
    for entity in &query {
        println!("¡Enemy nuevo aparecido: {:?}!", entity);
        commands.entity(entity).insert(Alerta(true));
    }
}

/// Fires only for entities whose Health changed this frame
fn loggear_cambios_de_vida(query: Query<&Health, Changed<Health>>) {
    for health in &query {
        println!("Vida cambió a {}", health.actual);
    }
}
