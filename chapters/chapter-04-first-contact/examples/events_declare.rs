// snippet 7 from cap-04.html — Message declaration (Bevy 0.19: Event→Message)
use bevy::prelude::*;
use bevy_book_chapter_04::CoinCollected;

fn main() {
    let mut app = App::new();
    app.add_message::<CoinCollected>();

    app.world_mut()
        .resource_mut::<Messages<CoinCollected>>()
        .write(CoinCollected { amount: 5 });

    let mut messages = app.world_mut().resource_mut::<Messages<CoinCollected>>();
    let mut cursor = messages.get_cursor();
    messages.update();

    let events: Vec<_> = cursor.read(&messages).collect();
    println!("Messages received: {}", events.len());
    for e in &events {
        println!("  CoinCollected {{ amount: {} }}", e.amount);
    }
}
