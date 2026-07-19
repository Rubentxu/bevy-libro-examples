use bevy::prelude::*;
use bevy_book_chapter_05::BestScore;

fn main() {
    let mut app = App::new();
    app.init_resource::<BestScore>();
    app.add_systems(Update, show_best_score);
    app.update();
}

fn show_best_score(best: Res<BestScore>) {
    println!("Best score loaded: {}", best.value);
}
