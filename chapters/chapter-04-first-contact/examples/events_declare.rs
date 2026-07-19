// snippet 7 from cap-04.html — Events declaration
// Run: cargo run --example events_declare
//
// NOTE: Bevy 0.19 changed from Event/EventWriter/EventReader to
// Message/MessageWriter/MessageReader. Round-trip test done in lib tests.
use bevy::prelude::*;
use bevy_book_chapter_04::MonedaRecogida;

fn main() {
    let mut app = App::new();

    // Register the message type (Bevy 0.19 API)
    app.add_message::<MonedaRecogida>();

    // Write a message via the Messages resource
    app.world_mut()
        .resource_mut::<Messages<MonedaRecogida>>()
        .write(MonedaRecogida { cantidad: 5 });

    // Create cursor BEFORE swapping buffers
    let mut messages = app.world_mut().resource_mut::<Messages<MonedaRecogida>>();
    let mut cursor = messages.get_cursor();
    messages.update(); // swap buffer — now the written message is readable

    let events: Vec<_> = cursor.read(&messages).collect();

    println!("Messages received: {}", events.len());
    for e in &events {
        println!("  MonedaRecogida {{ cantidad: {} }}", e.cantidad);
    }
}
