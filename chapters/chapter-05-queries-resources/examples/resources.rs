// snippets 15, 16, 17 from cap-05.html §5.5 — Resources basics
// Run: cargo run --example resources
use bevy::prelude::*;
use bevy_book_chapter_05::{ConfiguracionJuego, PuntuacionGlobal};

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();

    app.add_systems(Startup, setup);
    app.add_systems(Update, (subir_puntuacion, mostrar_volumen));

    app.update();
    app.update();
    app.update();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(PuntuacionGlobal::default());
    commands.insert_resource(ConfiguracionJuego {
        volumen: 0.8,
        musica: true,
        efectos: true,
    });
}

fn subir_puntuacion(mut puntos: ResMut<PuntuacionGlobal>) {
    puntos.puntos += 10;
    println!("Puntuación subió a {}", puntos.puntos);
}

fn mostrar_volumen(config: Res<ConfiguracionJuego>) {
    println!("Volumen: {}", config.volumen);
}
