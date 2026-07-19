use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();
    let e1 = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)))
        .id();
    let _e2 = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)))
        .id();
    let _e3 = app
        .world_mut()
        .spawn(Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)))
        .id();

    app.add_systems(Update, debug_transforms);
    app.update();

    {
        app.world_mut()
            .get_mut::<Transform>(e1)
            .unwrap()
            .translation
            .x = 42.0;
    }
    app.update();
}

fn debug_transforms(query: Query<Ref<Transform>>) {
    let (mut changed, mut total) = (0, 0);
    for tf in &query {
        total += 1;
        if tf.is_changed() {
            changed += 1;
            println!("Entity moved to {:?}", tf.translation);
        }
    }
    println!("Total: {}, Changed: {}", total, changed);
}
