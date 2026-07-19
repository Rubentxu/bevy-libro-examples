// snippet 18 from cap-05.html §5.6 — FromWorld initialization
// Run: cargo run --example from_world
use bevy::prelude::*;
use bevy_book_chapter_05::MejorPuntuacion;

fn main() {
    let mut app = App::new();
    // init_resource calls FromWorld::from_world() automatically
    app.init_resource::<MejorPuntuacion>();
    app.add_systems(Update, mostrar_mejor_puntuacion);

    app.update();
}

fn mostrar_mejor_puntuacion(mejor: Res<MejorPuntuacion>) {
    println!("Mejor puntuación cargada: {}", mejor.valor);
}
