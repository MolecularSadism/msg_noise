# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-15

### Added

- `NoisePlugin` for easy Bevy integration with seeded or GlobalRng-derived initialization
- `NoiseSource` resource for creating derived noise generators with unique keys
- `Noise` struct with configurable Perlin noise generation
- Builder pattern: `with_scale()`, `with_range()`, `with_offset()`
- 2D noise: `get_raw()`, `get_normalized()`, `get_absolute()`
- 3D noise: `get_raw_3d()`, `get_normalized_3d()`
- Fractal noise: `get_fractal()`, `get_fractal_scaled()`
- Full documentation with examples
