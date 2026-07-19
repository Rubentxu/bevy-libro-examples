// snippet 19 from cap-05.html §5.8 — Local<T> system-private state
// Run: cargo run --example local
use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_systems(Update, contador_de_frames);

    // Run 5 frames to demonstrate persistence
    for _ in 0..5 {
        app.update();
    }
}

/// Counts frames using Local<u32>. Persists between invocations of THIS system only.
fn contador_de_frames(mut contador: Local<u32>) {
    *contador += 1;
    println!("Han pasado {} frames (en este system)", *contador);
}
