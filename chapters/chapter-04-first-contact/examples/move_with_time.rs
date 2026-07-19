// snippets 10, 11 from cap-04.html — Time-based movement (headless)
// snippet 11 uses ScheduleRunnerPlugin instead of DefaultPlugins + Window
// Run: cargo run --example move_with_time
use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_04::{Posicion, Velocidad};

fn main() {
    let mut app = App::new();

    // Headless: run a fixed number of frames instead of opening a window
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>(); // Time needed for delta_secs()

    app.add_systems(Startup, setup);
    app.add_systems(Update, mover_cubo);

    // Advance time by 1/60s before each update for deterministic tests
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
    // Two entities, two velocities (matching book snippet 11)
    commands.spawn((Posicion(0.0, 0.0), Velocidad(50.0)));
    commands.spawn((Posicion(100.0, 0.0), Velocidad(80.0)));
}

fn mover_cubo(time: Res<Time>, mut query: Query<(&mut Posicion, &Velocidad)>) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_secs();
    }
}

fn get_positions(app: &mut App) -> Vec<(f32, f32)> {
    let mut positions = app.world_mut().query::<&Posicion>();
    positions.iter(app.world()).map(|p| (p.0, p.1)).collect()
}
