// snippets 10, 11 from cap-04.html — Time-based movement (headless)
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_04::{Position, Velocity};
use std::time::Duration;

fn main() {
    let mut app = App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, move_cube);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();
    println!("After frame 1: {:?}", get_positions(&mut app));

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();
    println!("After frame 2: {:?}", get_positions(&mut app));
}

fn setup(mut commands: Commands) {
    commands.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 50.0, y: 0.0 }));
    commands.spawn((Position { x: 100.0, y: 0.0 }, Velocity { x: 80.0, y: 0.0 }));
}

fn move_cube(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in &mut query {
        pos.x += vel.x * time.delta_secs();
    }
}

fn get_positions(app: &mut App) -> Vec<(f32, f32)> {
    let mut positions = app.world_mut().query::<&Position>();
    positions.iter(app.world()).map(|p| (p.x, p.y)).collect()
}
