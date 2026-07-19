// snippets 5, 6 from cap-04.html — Resources
// Run: cargo run --example resources
use bevy::prelude::*;
use bevy_book_chapter_04::{GameConfig, PuntuacionGlobal};

fn main() {
    let mut app = App::new();

    // snippet 5: Insert resources
    app.insert_resource(GameConfig {
        volumen: 0.8,
        dificultad: 1.5,
        fullscreen: false,
    });
    app.insert_resource(PuntuacionGlobal(0));

    // snippet 6: Read resources from a system
    app.add_systems(Update, mostrar_puntuacion);
    app.add_systems(Update, incrementar_puntuacion);

    app.update();
    app.update();
    app.update();

    let final_score = app.world().resource::<PuntuacionGlobal>();
    println!("Final score: {}", final_score.0);
}

fn mostrar_puntuacion(puntuacion: Res<PuntuacionGlobal>) {
    println!("Llevas {} punticos.", puntuacion.0);
}

fn incrementar_puntuacion(mut score: ResMut<PuntuacionGlobal>) {
    score.0 += 10;
}
