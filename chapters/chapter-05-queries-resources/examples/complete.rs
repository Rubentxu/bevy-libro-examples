// snippet §5.9 — Complete: score + player with keyboard input (headless)
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_05::{GlobalScore, Player};
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<GlobalScore>();
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Transform::default(), Player));
    });
    app.add_systems(Update, (handle_input, move_player, show_points));

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyA);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::Space);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();
}

fn handle_input(input: Res<ButtonInput<KeyCode>>, mut score: ResMut<GlobalScore>) {
    if input.just_pressed(KeyCode::Space) {
        score.points += 10;
        println!("+10 points! Total: {}", score.points);
    }
}

fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut dir = 0.0;
    if input.pressed(KeyCode::KeyA) {
        dir -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        dir += 1.0;
    }
    for mut tf in &mut query {
        tf.translation.x += dir * 100.0 * time.delta_secs();
    }
}

fn show_points(score: Res<GlobalScore>) {
    if score.points > 0 && score.is_changed() {
        println!("Points: {}", score.points);
    }
}
