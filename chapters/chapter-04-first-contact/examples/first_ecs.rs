use bevy_book_chapter_04::{Position, build_app};

fn main() {
    let mut app = build_app();
    app.update();

    let mut positions = app.world_mut().query::<&Position>();
    for position in positions.iter(app.world()) {
        println!("Player position: ({}, {})", position.x, position.y);
    }
}
