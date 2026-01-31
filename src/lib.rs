//! Seeded noise generation for Bevy games.
//!
//! This crate provides deterministic, reproducible noise generation
//! through a centralized [`NoiseSource`] resource that derives its seed
//! from [`msg_rng::GlobalRng`].
//!
//! # Features
//!
//! - **Seeded noise**: Reproducible Perlin noise generation
//! - **Global source**: Single seed for all noise generators
//! - **Factory pattern**: Create derived noise generators with unique keys
//! - **Configurable**: Scale, range, offset, and fractal parameters
//!
//! # Quick Start
//!
//! ```rust
//! use bevy::prelude::*;
//! use msg_rng::prelude::*;
//! use msg_noise::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(RngPlugin::seeded(12345))
//!         .add_plugins(NoisePlugin::from_global_rng())
//!         .add_systems(Update, generate_terrain);
//!     // .run() would start the app loop
//! }
//!
//! fn generate_terrain(noise_source: Res<NoiseSource>) {
//!     let terrain = noise_source.create(0x5445_5252); // "TERR"
//!     let value = terrain.get_normalized(10.0, 20.0);
//! }
//! ```

use bevy::prelude::*;
use msg_rng::GlobalRng;
use noise::{NoiseFn, Perlin, ScalePoint};
use std::fmt;

const DEFAULT_NOISE_SCALE: f64 = 0.008;

/// Plugin for adding noise generation to a Bevy app.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use msg_rng::RngPlugin;
/// use msg_noise::NoisePlugin;
///
/// // Derive from GlobalRng (recommended)
/// App::new()
///     .add_plugins(RngPlugin::seeded(42))
///     .add_plugins(NoisePlugin::from_global_rng());
///
/// // Or use explicit seed
/// App::new()
///     .add_plugins(NoisePlugin::seeded(12345));
/// ```
pub struct NoisePlugin {
    seed: Option<u32>,
}

impl NoisePlugin {
    /// Create a noise plugin with an explicit seed.
    #[must_use]
    pub fn seeded(seed: u32) -> Self {
        Self { seed: Some(seed) }
    }

    /// Create a noise plugin that derives its seed from [`GlobalRng`] at startup.
    ///
    /// Requires [`msg_rng::RngPlugin`] to be added before this plugin.
    #[must_use]
    pub fn from_global_rng() -> Self {
        Self { seed: None }
    }
}

impl Plugin for NoisePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<NoiseSource>();

        match self.seed {
            Some(seed) => {
                app.insert_resource(NoiseSource::new(seed));
            }
            None => {
                app.add_systems(PreStartup, init_from_global_rng);
            }
        }
    }
}

fn init_from_global_rng(mut commands: Commands, rng: Res<GlobalRng>) {
    // Use lower 32 bits of u64 seed for u32-based Perlin noise
    let seed = (rng.seed() & u64::from(u32::MAX))
        .try_into()
        .expect("Bitmasked value should always fit in u32");
    commands.insert_resource(NoiseSource::new(seed));
}

/// Global noise source resource.
///
/// This is the primary source for creating noise generators.
/// All noise generators are derived from this resource's seed.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use msg_noise::NoiseSource;
///
/// fn my_system(noise: Res<NoiseSource>) {
///     // Create noise with a unique key
///     let terrain = noise.create(0x5445_5252);
///
///     // Get noise value at coordinates
///     let height = terrain.get_normalized(10.0, 20.0);
/// }
/// ```
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct NoiseSource {
    seed: u32,
}

impl NoiseSource {
    /// Create a new noise source with the given seed.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    /// Get the current seed.
    #[must_use]
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Reseed the noise source.
    ///
    /// Use this when transitioning to a new level.
    pub fn reseed(&mut self, seed: u32) {
        self.seed = seed;
    }

    /// Create a noise generator with a derived seed.
    ///
    /// The key is combined with the base seed to create a unique
    /// but deterministic noise generator.
    #[must_use]
    pub fn create(&self, key: u32) -> Noise {
        let derived = hash_combine(self.seed, key);
        Noise::new(derived)
    }

    /// Create a noise generator using an additional salt value.
    ///
    /// Useful when you need multiple noise layers with the same key.
    #[must_use]
    pub fn create_salted(&self, key: u32, salt: u32) -> Noise {
        let combined = hash_combine(key, salt);
        let derived = hash_combine(self.seed, combined);
        Noise::new(derived)
    }
}

/// Combine two u32 values into a deterministic hash.
#[inline]
fn hash_combine(a: u32, b: u32) -> u32 {
    let mut h = a;
    h ^= b;
    h = h.wrapping_mul(0x517c_c1b7);
    h ^= h >> 16;
    h
}

/// A configurable Perlin noise generator.
///
/// Create instances via [`NoiseSource::create`] rather than directly.
///
/// # Examples
///
/// ```rust
/// use msg_noise::Noise;
///
/// let noise = Noise::new(12345)
///     .with_scale(0.01)
///     .with_range(0.0, 255.0);
///
/// let value = noise.get_normalized(10.0, 20.0);
/// ```
#[derive(Clone)]
pub struct Noise {
    generator: ScalePoint<Perlin>,
    scale: f64,
    offset: f64,
    range_min: f64,
    range_max: f64,
}

impl Default for Noise {
    fn default() -> Self {
        Self::new(0)
    }
}

impl fmt::Debug for Noise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Noise")
            .field("scale", &self.scale)
            .field("offset", &self.offset)
            .field("range_min", &self.range_min)
            .field("range_max", &self.range_max)
            .finish_non_exhaustive()
    }
}

impl Noise {
    /// Create a new noise generator with the specified seed.
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self {
            generator: ScalePoint::new(Perlin::new(seed)),
            scale: DEFAULT_NOISE_SCALE,
            offset: 0.0,
            range_min: 0.0,
            range_max: 1.0,
        }
    }

    /// Create a noise generator from a base seed and a key.
    ///
    /// Combines the seeds using wrapping addition for a derived seed.
    #[must_use]
    pub fn from_base(base_seed: u32, key: u32) -> Self {
        let combined = base_seed.wrapping_add(key);
        Self::new(combined)
    }

    /// Set the noise scale (frequency).
    ///
    /// Lower values create smoother, larger features.
    /// Higher values create noisier, smaller features.
    #[must_use]
    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    /// Set the output range for scaled values.
    #[must_use]
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.range_min = min;
        self.range_max = max;
        self
    }

    /// Set an offset to add to the noise coordinates.
    #[must_use]
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    /// Get raw Perlin noise value (typically between -1.0 and 1.0).
    #[must_use]
    pub fn get_raw(&self, x: f64, y: f64) -> f64 {
        self.generator.get([
            (x + self.offset) * self.scale,
            (y + self.offset) * self.scale,
        ])
    }

    /// Get raw 3D Perlin noise value.
    #[must_use]
    pub fn get_raw_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        self.generator.get([
            (x + self.offset) * self.scale,
            (y + self.offset) * self.scale,
            (z + self.offset) * self.scale,
        ])
    }

    /// Get absolute noise value (0.0 to 1.0).
    #[must_use]
    pub fn get_absolute(&self, x: f64, y: f64) -> f64 {
        self.get_raw(x, y).abs()
    }

    /// Get normalized noise value (0.0 to 1.0).
    #[must_use]
    pub fn get_normalized(&self, x: f64, y: f64) -> f64 {
        (self.get_raw(x, y) + 1.0) * 0.5
    }

    /// Get normalized 3D noise value (0.0 to 1.0).
    #[must_use]
    pub fn get_normalized_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        (self.get_raw_3d(x, y, z) + 1.0) * 0.5
    }

    /// Generate fractal noise by combining multiple octaves.
    #[must_use]
    pub fn get_fractal(
        &self,
        x: f64,
        y: f64,
        octaves: u32,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += self.get_raw(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        value / max_value
    }

    /// Get fractal noise scaled to the configured range.
    #[must_use]
    pub fn get_fractal_scaled(
        &self,
        x: f64,
        y: f64,
        octaves: u32,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        let fractal = self.get_fractal(x, y, octaves, persistence, lacunarity);
        let normalized = (fractal + 1.0) * 0.5;
        self.range_min + normalized * (self.range_max - self.range_min)
    }

    /// Update the internal scale.
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// Update the output range.
    pub fn set_range(&mut self, min: f64, max: f64) {
        self.range_min = min;
        self.range_max = max;
    }

    /// Update the coordinate offset.
    pub fn set_offset(&mut self, offset: f64) {
        self.offset = offset;
    }
}

/// Prelude module for convenient imports.
pub mod prelude {
    pub use super::{Noise, NoisePlugin, NoiseSource};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_noise_is_deterministic() {
        let source1 = NoiseSource::new(12345);
        let source2 = NoiseSource::new(12345);

        let noise1 = source1.create(0xABCD);
        let noise2 = source2.create(0xABCD);

        let values1: Vec<f64> = (0..10)
            .map(|i| noise1.get_normalized(i as f64, 0.0))
            .collect();
        let values2: Vec<f64> = (0..10)
            .map(|i| noise2.get_normalized(i as f64, 0.0))
            .collect();

        assert_eq!(values1, values2);
    }

    #[test]
    fn different_seeds_produce_different_noise() {
        let source1 = NoiseSource::new(12345);
        let source2 = NoiseSource::new(54321);

        let noise1 = source1.create(0xABCD);
        let noise2 = source2.create(0xABCD);

        let values1: Vec<f64> = (0..10)
            .map(|i| noise1.get_normalized(i as f64, 0.0))
            .collect();
        let values2: Vec<f64> = (0..10)
            .map(|i| noise2.get_normalized(i as f64, 0.0))
            .collect();

        assert_ne!(values1, values2);
    }

    #[test]
    fn different_keys_produce_different_noise() {
        let source = NoiseSource::new(12345);

        let noise1 = source.create(0x0001);
        let noise2 = source.create(0x0002);

        let values1: Vec<f64> = (0..10)
            .map(|i| noise1.get_normalized(i as f64, 0.0))
            .collect();
        let values2: Vec<f64> = (0..10)
            .map(|i| noise2.get_normalized(i as f64, 0.0))
            .collect();

        assert_ne!(values1, values2);
    }

    #[test]
    fn reseed_changes_noise() {
        let mut source = NoiseSource::new(12345);
        let noise1 = source.create(0xABCD);
        let values1: Vec<f64> = (0..5)
            .map(|i| noise1.get_normalized(i as f64, 0.0))
            .collect();

        source.reseed(54321);
        let noise2 = source.create(0xABCD);
        let values2: Vec<f64> = (0..5)
            .map(|i| noise2.get_normalized(i as f64, 0.0))
            .collect();

        assert_ne!(values1, values2);
    }

    #[test]
    fn normalized_values_in_range() {
        let source = NoiseSource::new(42);
        let noise = source.create(0x1234);

        for x in -100..100 {
            for y in -100..100 {
                let value = noise.get_normalized(x as f64, y as f64);
                assert!(
                    (0.0..=1.0).contains(&value),
                    "Value {} out of range at ({}, {})",
                    value,
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn scale_affects_frequency() {
        let source = NoiseSource::new(42);

        let smooth = source.create(0x1234).with_scale(0.001);
        let rough = source.create(0x1234).with_scale(0.1);

        // Sample at two nearby points
        let smooth_diff = (smooth.get_normalized(0.0, 0.0) - smooth.get_normalized(1.0, 0.0)).abs();
        let rough_diff = (rough.get_normalized(0.0, 0.0) - rough.get_normalized(1.0, 0.0)).abs();

        // Rougher noise should have larger differences between nearby points
        assert!(rough_diff > smooth_diff);
    }
}
