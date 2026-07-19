// snippet 5 from cap-05.html §5.2 — Ref<T> change detection
// Run: cargo run --example ref_change_detection
//
// Demonstrates that Ref<T> detects mutations, and warns against
// the anti-pattern of requesting &mut "just in case" (which marks
// everything as changed).
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();

    // Spawn entities and capture IDs
    let e1 = app
        .world_mut()
        .spawn((Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),))
        .id();
    let _e2 = app
        .world_mut()
        .spawn((Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),))
        .id();
    let _e3 = app
        .world_mut()
        .spawn((Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)),))
        .id();

    app.add_systems(Update, depurar_transforms);

    println!("--- Frame 1 (initial spawn, Added fires) ---");
    app.update();

    // Mutate one transform
    {
        let mut tf = app.world_mut().get_mut::<Transform>(e1).unwrap();
        tf.translation.x = 42.0;
    }
    println!("\n--- Mutated entity {:?} ---", e1);

    println!("\n--- Frame 2 (only one should be changed) ---");
    app.update();
}

/// Uses Ref<T> to detect changes without mutating.
fn depurar_transforms(query: Query<Ref<Transform>>) {
    let mut changed = 0;
    let mut total = 0;
    for transform in &query {
        total += 1;
        if transform.is_changed() {
            changed += 1;
            println!("La entity se movió a {:?}", transform.translation);
        }
    }
    println!("Total: {}, Changed: {}", total, changed);
}
