// snippets §6.2-6.4 — Basic message send/receive
// Run: cargo run --example message_basics
use bevy::prelude::*;
use bevy_book_chapter_06::{CambioDeNivel, DanoRecibido, MonedaRecogida};

fn main() {
    let mut app = App::new();
    app.add_message::<DanoRecibido>();
    app.add_message::<MonedaRecogida>();
    app.add_message::<CambioDeNivel>();

    // §6.3 — Write messages via MessageWriter
    app.add_systems(Update, enviar_mensajes);
    // §6.4 — Read messages via MessageReader
    app.add_systems(Update, leer_mensajes.after(enviar_mensajes));

    app.update();
    app.update();
}

fn enviar_mensajes(mut w1: MessageWriter<MonedaRecogida>, mut w2: MessageWriter<CambioDeNivel>) {
    w1.write(MonedaRecogida {
        cantidad: 5,
        posicion: (10.0, 20.0),
    });
    w2.write(CambioDeNivel { nuevo_nivel: 2 });
    println!("Mensajes enviados");
}

fn leer_mensajes(mut r1: MessageReader<MonedaRecogida>, mut r2: MessageReader<CambioDeNivel>) {
    for msg in r1.read() {
        println!("Moneda: {} en {:?}", msg.cantidad, msg.posicion);
    }
    for msg in r2.read() {
        println!("Nivel cambiado a: {}", msg.nuevo_nivel);
    }
}
