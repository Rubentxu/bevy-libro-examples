// snippet 1 from cap-05.html §5.1 — Basic Query<&Transform>
// Run: cargo run --example query_basic
use bevy::prelude::*;
use bevy_book_chapter_05::build_combat_app;

fn main() {
    let mut app = build_combat_app();
    app.add_systems(Update, listar_todo);

    app.update();
}

/// Lists every entity's transform position
fn listar_todo(query: Query<&Transform>) {
    for transform in &query {
        println!("Posición: {:?}", transform.translation);
    }
}
