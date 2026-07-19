// snippets 6, 7 from cap-05.html §5.2bis — Or filter combinations
// Run: cargo run --example or_filter
use bevy::prelude::*;
use bevy_book_chapter_05::{Enemy, Muerto, Player, Vida, build_combat_app};

fn main() {
    let mut app = build_combat_app();
    // Add Vida to some entities for the example
    let entities: Vec<Entity> = app
        .world_mut()
        .query::<Entity>()
        .iter(app.world())
        .collect();
    for e in entities {
        app.world_mut().entity_mut(e).insert(Vida {
            actual: 50,
            max: 100,
        });
    }
    // Mark one enemy as dead
    let enemy = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .next()
        .unwrap();
    app.world_mut().entity_mut(enemy).insert(Muerto);

    app.add_systems(Update, (refrescar_barra_de_vida, listar_combatientes));

    println!("--- Frame 1 ---");
    app.update();

    // Spawn a new enemy
    app.world_mut().spawn((
        Enemy,
        Vida {
            actual: 20,
            max: 20,
        },
    ));
    println!("\n--- After spawning new enemy ---");
    app.update();
}

/// Entities whose Enemy was Added OR Vida changed
fn refrescar_barra_de_vida(query: Query<&Vida, Or<(Added<Enemy>, Changed<Vida>)>>) {
    let mut count = 0;
    for vida in &query {
        count += 1;
        println!("Refresco el HUD con vida {}/{}", vida.actual, vida.max);
    }
    println!("refrescar_barra_de_vida: {} entities", count);
}

/// Players OR enemies, but NOT dead (Muerto)
fn listar_combatientes(query: Query<Entity, (Or<(With<Player>, With<Enemy>)>, Without<Muerto>)>) {
    for entity in &query {
        println!("Combate: {:?}", entity);
    }
}
