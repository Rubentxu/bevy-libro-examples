// snippet 2, 3, 4 from cap-04.html — Component variants
use bevy::prelude::*;
use bevy_book_chapter_04::{Enemy, Player, Position, PositionDefault, State};

fn main() {
    let mut app = App::new();

    app.world_mut().spawn(PositionDefault::default());
    app.world_mut().spawn((Player, Position { x: 0.0, y: 0.0 }));
    app.world_mut().spawn((Enemy, Position { x: 10.0, y: 5.0 }));
    app.world_mut()
        .spawn((Position { x: 1.0, y: 2.0 }, State::Idle));
    app.world_mut()
        .spawn((Position { x: 3.0, y: 4.0 }, State::Walking));

    app.update();

    let mut players_q = app.world_mut().query::<(&Player, &Position)>();
    let mut enemies_q = app.world_mut().query::<(&Enemy, &Position)>();
    let mut states_q = app.world_mut().query::<(&State, &Position)>();
    println!("Players: {}", players_q.iter(app.world()).count());
    println!("Enemies: {}", enemies_q.iter(app.world()).count());
    println!(
        "Entities with State: {}",
        states_q.iter(app.world()).count()
    );
}
