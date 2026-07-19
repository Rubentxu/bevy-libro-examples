// snippet 12 from cap-04.html — Transform with Velocidad2D
// Run: cargo run --example transform_velocity
use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_04::Velocidad2D;

fn main() {
    let mut app = App::new();

    // Headless runner
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>(); // Time needed for delta_secs()

    app.add_systems(Startup, setup);
    app.add_systems(Update, mover);

    // Advance time and run
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let (vel, tf) = app
        .world_mut()
        .query::<(&Velocidad2D, &Transform)>()
        .iter(app.world())
        .next()
        .expect("one entity");

    println!("Velocidad2D: ({}, {})", vel.x, vel.y);
    println!(
        "Transform translation: ({}, {}, {})",
        tf.translation.x, tf.translation.y, tf.translation.z
    );
    // After 1/60s at 30px/s → 0.5px movement
    assert!(tf.translation.x > 0.0, "Transform should have moved in X");
}

fn setup(mut commands: Commands) {
    commands.spawn((Velocidad2D { x: 30.0, y: 20.0 }, Transform::default()));
}

fn mover(mut query: Query<(&Velocidad2D, &mut Transform)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (vel, mut tf) in &mut query {
        tf.translation.x += vel.x * dt;
        tf.translation.y += vel.y * dt;
    }
}
