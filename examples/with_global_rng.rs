//! Example: Using NoisePlugin seeded from the global entropy source.
//!
//! This derives the noise seed from `bevy_rand`'s global entropy source,
//! ensuring all randomness in the app shares a single reproducible seed.
//!
//! Run with: `cargo run --example with_global_rng`

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use msg_noise::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(EntropyPlugin::<WyRand>::with_seed(12345u64.to_le_bytes()))
        .add_plugins(NoisePlugin::from_global_rng())
        .add_systems(Update, generate_terrain.run_if(run_once))
        .run();
}

fn generate_terrain(noise_source: Res<NoiseSource>) {
    println!(
        "Noise seed (derived from the global entropy source): {}",
        noise_source.seed()
    );

    let terrain = noise_source.create(0x5445_5252); // "TERR"
    let caves = noise_source.create(0x4341_5645); // "CAVE"

    println!("\nTerrain heights (x=0..10):");
    for x in 0..10 {
        let height = terrain.get_normalized(f64::from(x), 0.0);
        let bar_len = (height * 40.0) as usize;
        println!("  x={x}: {height:.4} |{}", "#".repeat(bar_len));
    }

    println!("\nCave density (x=0..10):");
    for x in 0..10 {
        let density = caves.get_normalized(f64::from(x), 0.0);
        println!("  x={x}: {density:.4}");
    }
}
