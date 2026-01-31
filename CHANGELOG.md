# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-31

### Changed

- Migrated to Bevy 0.18
- Updated README with Bevy version compatibility table

### Fixed

- Fixed clippy warnings

### Added

- Added standard Rust `.gitignore`
- Removed build artifacts from git tracking (`target/`, `Cargo.lock`)

## [0.2.0] - 2026-01-20

### Changed

- Migrated to Bevy 0.17
- Updated `msg_rng` dependency to v0.2.0
- Updated README with Bevy 0.17 compatibility note

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
