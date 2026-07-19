// snippet 20 from cap-05.html §5.9 — Complete example: marcador + player
// Run: cargo run --example complete
//
// NOTE: Original book example uses DefaultPlugins (window). This headless variant
// uses ScheduleRunnerPlugin + simulates Space/A/D key presses via ButtonInput.
use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_book_chapter_05::{Player, PuntuacionGlobal};

fn main() {
    let mut app = App::new();
    app.add_plugins(ScheduleRunnerPlugin::run_once());
    app.init_resource::<Time>();
    app.init_resource::<ButtonInput<KeyCode>>();

    app.init_resource::<PuntuacionGlobal>();
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Transform::default(), Player));
    });
    app.add_systems(Update, (input_y_subir, mover_player, mostrar_puntos));

    // Simulate Space + A pressed
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

    // Release Space
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::Space);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();
}

fn input_y_subir(input: Res<ButtonInput<KeyCode>>, mut puntos: ResMut<PuntuacionGlobal>) {
    if input.just_pressed(KeyCode::Space) {
        puntos.puntos += 10;
        println!("+10 puntos! Total: {}", puntos.puntos);
    }
}

fn mover_player(
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
    for mut t in &mut query {
        t.translation.x += dir * 100.0 * time.delta_secs();
    }
}

fn mostrar_puntos(puntos: Res<PuntuacionGlobal>) {
    if puntos.puntos > 0 && puntos.is_changed() {
        println!("Puntos: {}", puntos.puntos);
    }
}
