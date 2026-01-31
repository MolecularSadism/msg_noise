# msg_noise

[![CI](https://github.com/MolecularSadism/msg_noise/workflows/CI/badge.svg)](https://github.com/MolecularSadism/msg_noise/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/MolecularSadism/msg_noise#license)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-blue.svg)](https://bevyengine.org/)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)

Seeded noise generation for Bevy games.

## Features

- **Seeded noise**: Reproducible Perlin noise generation
- **Global source**: Single seed for all noise generators
- **Factory pattern**: Create derived noise generators with unique keys
- **Configurable**: Scale, range, offset, and fractal parameters

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
msg_noise = { git = "https://github.com/MolecularSadism/msg_noise", tag = "v0.3.0" }
msg_rng = { git = "https://github.com/MolecularSadism/msg_rng", tag = "v0.3.0" }
```

## Quick Start

```rust
use bevy::prelude::*;
use msg_rng::prelude::*;
use msg_noise::prelude::*;

fn main() {
    App::new()
        .add_plugins(RngPlugin::seeded(12345))
        .add_plugins(NoisePlugin::from_global_rng())
        .add_systems(Update, generate_terrain)
        .run();
}

fn generate_terrain(noise_source: Res<NoiseSource>) {
    // Create noise with a unique key
    let terrain = noise_source.create(0x5445_5252); // "TERR"

    // Get normalized value (0.0 to 1.0)
    let height = terrain.get_normalized(10.0, 20.0);
}
```

## API Overview

### NoisePlugin

```rust
// Derive seed from GlobalRng (recommended)
NoisePlugin::from_global_rng()

// Use explicit seed
NoisePlugin::seeded(12345)
```

### NoiseSource

```rust
fn my_system(noise: Res<NoiseSource>) {
    // Create noise generators with unique keys
    let terrain = noise.create(0x5445_5252);
    let caves = noise.create(0x4341_5645);

    // Or use salt for multiple layers with same key
    let layer1 = noise.create_salted(0x5445_5252, 1);
    let layer2 = noise.create_salted(0x5445_5252, 2);
}
```

### Noise

```rust
let noise = Noise::new(12345)
    .with_scale(0.01)      // Frequency (lower = smoother)
    .with_range(0.0, 255.0) // Output range
    .with_offset(1000.0);   // Coordinate offset

// 2D noise
let raw = noise.get_raw(x, y);           // -1.0 to 1.0
let normalized = noise.get_normalized(x, y); // 0.0 to 1.0
let absolute = noise.get_absolute(x, y);     // 0.0 to 1.0

// 3D noise
let value_3d = noise.get_normalized_3d(x, y, z);

// Fractal noise (multiple octaves)
let fractal = noise.get_fractal(x, y, octaves, persistence, lacunarity);
let fractal_scaled = noise.get_fractal_scaled(x, y, 4, 0.5, 2.0);
```

## Bevy Version Compatibility

| `msg_noise` | Bevy |
|-------------|------|
| 0.3         | 0.18 |
| 0.2         | 0.17 |
| 0.1         | 0.16 |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
