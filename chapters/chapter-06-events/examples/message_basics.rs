// §6.2-6.4 — Basic message send/receive
use bevy::prelude::*;
use bevy_book_chapter_06::{CoinCollected, DamageReceived, LevelChange};

fn main() {
    let mut app = App::new();
    app.add_message::<DamageReceived>();
    app.add_message::<CoinCollected>();
    app.add_message::<LevelChange>();
    app.add_systems(Update, (send_messages, read_messages.after(send_messages)));
    app.update();
    app.update();
}

fn send_messages(mut w1: MessageWriter<CoinCollected>, mut w2: MessageWriter<LevelChange>) {
    w1.write(CoinCollected {
        amount: 5,
        position: (10.0, 20.0),
    });
    w2.write(LevelChange { new_level: 2 });
    println!("Messages sent");
}

fn read_messages(mut r1: MessageReader<CoinCollected>, mut r2: MessageReader<LevelChange>) {
    for msg in r1.read() {
        println!("Coin: {} at {:?}", msg.amount, msg.position);
    }
    for msg in r2.read() {
        println!("Level changed to: {}", msg.new_level);
    }
}
