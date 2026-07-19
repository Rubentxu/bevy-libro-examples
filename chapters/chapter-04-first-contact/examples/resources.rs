// snippets 5, 6 from cap-04.html — Resources
use bevy::prelude::*;
use bevy_book_chapter_04::{GameConfig, GlobalScore};

fn main() {
    let mut app = App::new();
    app.insert_resource(GameConfig {
        volume: 0.8,
        difficulty: 1.5,
        fullscreen: false,
    });
    app.insert_resource(GlobalScore(0));
    app.add_systems(Update, (show_score, increment_score));

    app.update();
    app.update();
    app.update();

    let final_score = app.world().resource::<GlobalScore>();
    println!("Final score: {}", final_score.0);
}

fn show_score(score: Res<GlobalScore>) {
    println!("Score: {}", score.0);
}

fn increment_score(mut score: ResMut<GlobalScore>) {
    score.0 += 10;
}
