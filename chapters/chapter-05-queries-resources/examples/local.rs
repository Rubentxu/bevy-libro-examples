use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_systems(Update, frame_counter);
    for _ in 0..5 {
        app.update();
    }
}

fn frame_counter(mut counter: Local<u32>) {
    *counter += 1;
    println!("Frames elapsed (in this system): {}", *counter);
}
