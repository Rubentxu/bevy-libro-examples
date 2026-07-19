// snippet 12 from cap-04.html — Transform with Velocity2D
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_04::Velocity2D;
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, move_transform);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let (vel, tf) = app
        .world_mut()
        .query::<(&Velocity2D, &Transform)>()
        .iter(app.world())
        .next()
        .expect("one entity");

    println!("Velocity2D: ({}, {})", vel.x, vel.y);
    println!(
        "Transform: ({}, {}, {})",
        tf.translation.x, tf.translation.y, tf.translation.z
    );
    assert!(tf.translation.x > 0.0, "Transform should have moved in X");
}

fn setup(mut commands: Commands) {
    commands.spawn((Velocity2D { x: 30.0, y: 20.0 }, Transform::default()));
}

fn move_transform(mut query: Query<(&Velocity2D, &mut Transform)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (vel, mut tf) in &mut query {
        tf.translation.x += vel.x * dt;
        tf.translation.y += vel.y * dt;
    }
}
