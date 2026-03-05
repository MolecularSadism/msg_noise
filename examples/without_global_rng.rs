//! Example: Using NoisePlugin with an explicit seed (no GlobalRng).
//!
//! This uses `NoisePlugin::seeded()` directly, so `msg_rng::RngPlugin`
//! is not required.
//!
//! Run with: `cargo run --example without_global_rng`

use bevy::prelude::*;
use msg_noise::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(NoisePlugin::seeded(42))
        .add_systems(Startup, generate_terrain)
        .run();
}

fn generate_terrain(noise_source: Res<NoiseSource>) {
    println!("Noise seed (explicit): {}", noise_source.seed());

    let terrain = noise_source.create(0x5445_5252);

    println!("\nTerrain heights (x=0..10):");
    for x in 0..10 {
        let height = terrain.get_normalized(f64::from(x), 0.0);
        let bar_len = (height * 40.0) as usize;
        println!("  x={x}: {height:.4} |{}", "#".repeat(bar_len));
    }

    // Fractal noise for more natural terrain
    let fractal_terrain = noise_source.create(0x4652_4143).with_range(0.0, 100.0);
    println!("\nFractal terrain (4 octaves, x=0..10):");
    for x in 0..10 {
        let height = fractal_terrain.get_fractal_scaled(f64::from(x), 0.0, 4, 0.5, 2.0);
        println!("  x={x}: {height:.2}");
    }
}
