use bevy::prelude::*;
use bevy_book_chapter_05::{GameConfig, GlobalScore};

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (increase_score, show_volume));
    app.update();
    app.update();
    app.update();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(GlobalScore::default());
    commands.insert_resource(GameConfig {
        volume: 0.8,
        music: true,
        effects: true,
    });
}

fn increase_score(mut score: ResMut<GlobalScore>) {
    score.points += 10;
    println!("Score increased to {}", score.points);
}

fn show_volume(config: Res<GameConfig>) {
    println!("Volume: {}", config.volume);
}
