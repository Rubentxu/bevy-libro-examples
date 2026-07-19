// snippet 2, 3, 4 from cap-04.html — Component variants
// Run: cargo run --example components_variants
use bevy::prelude::*;
use bevy_book_chapter_04::{Enemy, Estado, Player, Position, PositionDefault};

fn main() {
    let mut app = App::new();

    // snippet 2: Component with Default derive
    app.world_mut().spawn(PositionDefault::default());

    // snippet 3: Tag components (no fields)
    app.world_mut().spawn((Player, Position { x: 0.0, y: 0.0 }));
    app.world_mut().spawn((Enemy, Position { x: 10.0, y: 5.0 }));

    // snippet 4: State enum component
    app.world_mut()
        .spawn((Position { x: 1.0, y: 2.0 }, Estado::Idle));
    app.world_mut()
        .spawn((Position { x: 3.0, y: 4.0 }, Estado::Caminando));

    app.update();

    // Verify entities (query requires &mut World)
    let mut players_q = app.world_mut().query::<(&Player, &Position)>();
    let mut enemies_q = app.world_mut().query::<(&Enemy, &Position)>();
    let mut estados_q = app.world_mut().query::<(&Estado, &Position)>();

    println!("Players: {}", players_q.iter(app.world()).count());
    println!("Enemies: {}", enemies_q.iter(app.world()).count());
    println!(
        "Entities with Estado: {}",
        estados_q.iter(app.world()).count()
    );
}
