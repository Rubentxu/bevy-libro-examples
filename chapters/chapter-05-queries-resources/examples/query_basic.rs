use bevy::prelude::*;
use bevy_book_chapter_05::build_combat_app;

fn main() {
    let mut app = build_combat_app();
    app.add_systems(Update, list_all);
    app.update();
}

fn list_all(query: Query<&Transform>) {
    for transform in &query {
        println!("Position: {:?}", transform.translation);
    }
}
