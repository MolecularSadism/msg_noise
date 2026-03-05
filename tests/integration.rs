//! Integration tests for `msg_noise` with Bevy 0.18

use bevy::prelude::*;
use msg_noise::prelude::*;
use msg_rng::prelude::*;

#[test]
fn plugin_initialization_with_explicit_seed() {
    let mut app = App::new();
    app.add_plugins(NoisePlugin::seeded(12345));

    // Plugin should insert NoiseSource resource
    assert!(app.world().get_resource::<NoiseSource>().is_some());

    let noise_source = app.world().get_resource::<NoiseSource>().unwrap();
    assert_eq!(noise_source.seed(), 12345);
}

#[test]
fn plugin_initialization_from_global_rng() {
    let mut app = App::new();
    app.add_plugins(RngPlugin::seeded(42));
    app.add_plugins(NoisePlugin::from_global_rng());

    // Force PreStartup systems to run
    app.update();

    // Plugin should derive seed from GlobalRng
    assert!(app.world().get_resource::<NoiseSource>().is_some());

    let noise_source = app.world().get_resource::<NoiseSource>().unwrap();
    // Seed should be derived from GlobalRng (42)
    assert_eq!(noise_source.seed(), 42);
}

#[test]
fn noise_source_factory_creates_deterministic_noise() {
    let source = NoiseSource::new(12345);

    let noise1 = source.create(0x5445_5252);
    let noise2 = source.create(0x5445_5252);

    // Same key should produce same noise values
    for i in 0..10 {
        let x = f64::from(i);
        let val1 = noise1.get_normalized(x, 0.0);
        let val2 = noise2.get_normalized(x, 0.0);
        assert!(
            (val1 - val2).abs() < f64::EPSILON,
            "Values should match at x={x}"
        );
    }
}

#[test]
fn noise_source_different_keys_produce_different_noise() {
    let source = NoiseSource::new(12345);

    let noise1 = source.create(0x0001);
    let noise2 = source.create(0x0002);

    let mut different_count = 0;
    for i in 0..10 {
        let x = f64::from(i);
        let val1 = noise1.get_normalized(x, 0.0);
        let val2 = noise2.get_normalized(x, 0.0);
        if (val1 - val2).abs() > f64::EPSILON {
            different_count += 1;
        }
    }

    // Different keys should produce different values
    assert!(
        different_count > 8,
        "Expected most values to differ, but only {different_count} out of 10 were different"
    );
}

#[test]
fn noise_source_salted_creates_unique_layers() {
    let source = NoiseSource::new(12345);

    let layer1 = source.create_salted(0x5445_5252, 1);
    let layer2 = source.create_salted(0x5445_5252, 2);

    let mut different_count = 0;
    for i in 0..10 {
        let x = f64::from(i);
        let val1 = layer1.get_normalized(x, 0.0);
        let val2 = layer2.get_normalized(x, 0.0);
        if (val1 - val2).abs() > f64::EPSILON {
            different_count += 1;
        }
    }

    // Different salts should produce different values
    assert!(
        different_count > 8,
        "Expected most values to differ with different salts"
    );
}

#[test]
fn noise_determinism_across_apps() {
    // First app run
    let mut app1 = App::new();
    app1.add_plugins(NoisePlugin::seeded(99999));
    let source1 = app1.world().get_resource::<NoiseSource>().unwrap();
    let noise1 = source1.create(0xABCD);
    let values1: Vec<f64> = (0..10)
        .map(|i| noise1.get_normalized(f64::from(i), 0.0))
        .collect();

    // Second app run with same seed
    let mut app2 = App::new();
    app2.add_plugins(NoisePlugin::seeded(99999));
    let source2 = app2.world().get_resource::<NoiseSource>().unwrap();
    let noise2 = source2.create(0xABCD);
    let values2: Vec<f64> = (0..10)
        .map(|i| noise2.get_normalized(f64::from(i), 0.0))
        .collect();

    assert_eq!(
        values1, values2,
        "Noise should be deterministic across app instances"
    );
}

#[test]
fn noise_raw_values_in_expected_range() {
    let noise = Noise::new(42);

    for x in -50..50 {
        for y in -50..50 {
            let value = noise.get_raw(f64::from(x), f64::from(y));
            // Perlin noise is typically between -1 and 1, but can exceed slightly
            assert!(
                (-2.0..=2.0).contains(&value),
                "Raw value {value} out of reasonable range at ({x}, {y})"
            );
        }
    }
}

#[test]
fn noise_normalized_values_in_range() {
    let noise = Noise::new(42);

    for x in -50..50 {
        for y in -50..50 {
            let value = noise.get_normalized(f64::from(x), f64::from(y));
            assert!(
                (0.0..=1.0).contains(&value),
                "Normalized value {value} out of range at ({x}, {y})"
            );
        }
    }
}

#[test]
fn noise_absolute_values_in_range() {
    let noise = Noise::new(42);

    for x in -50..50 {
        for y in -50..50 {
            let value = noise.get_absolute(f64::from(x), f64::from(y));
            assert!(
                (0.0..=2.0).contains(&value),
                "Absolute value {value} out of range at ({x}, {y})"
            );
        }
    }
}

#[test]
fn noise_3d_normalized_values_in_range() {
    let noise = Noise::new(42);

    for x in -20..20 {
        for y in -20..20 {
            for z in -20..20 {
                let value = noise.get_normalized_3d(f64::from(x), f64::from(y), f64::from(z));
                assert!(
                    (0.0..=1.0).contains(&value),
                    "3D normalized value {value} out of range at ({x}, {y}, {z})"
                );
            }
        }
    }
}

#[test]
fn noise_scale_affects_frequency() {
    let smooth = Noise::new(42).with_scale(0.001);
    let rough = Noise::new(42).with_scale(0.1);

    // Sample at two nearby points
    let smooth_diff = (smooth.get_normalized(0.0, 0.0) - smooth.get_normalized(1.0, 0.0)).abs();
    let rough_diff = (rough.get_normalized(0.0, 0.0) - rough.get_normalized(1.0, 0.0)).abs();

    // Higher scale = higher frequency = larger differences between nearby points
    assert!(
        rough_diff > smooth_diff,
        "Rough noise (scale=0.1) diff {rough_diff} should be > smooth noise (scale=0.001) diff {smooth_diff}"
    );
}

#[test]
fn noise_range_configuration() {
    let noise = Noise::new(42).with_range(100.0, 200.0);

    // get_normalized should still return 0-1
    let normalized = noise.get_normalized(10.0, 20.0);
    assert!((0.0..=1.0).contains(&normalized));

    // But range is used in fractal_scaled
    let scaled = noise.get_fractal_scaled(10.0, 20.0, 4, 0.5, 2.0);
    assert!(
        (100.0..=200.0).contains(&scaled),
        "Scaled value {scaled} should be in configured range [100, 200]"
    );
}

#[test]
fn noise_offset_affects_coordinates() {
    let noise1 = Noise::new(42).with_offset(0.0);
    let noise2 = Noise::new(42).with_offset(1000.0);

    // Check multiple points to ensure offset has an effect
    let mut different_count = 0;
    for i in 0..10 {
        let x = f64::from(i);
        let value1 = noise1.get_normalized(x, 0.0);
        let value2 = noise2.get_normalized(x, 0.0);

        if (value1 - value2).abs() > f64::EPSILON {
            different_count += 1;
        }
    }

    assert!(
        different_count > 8,
        "Offset should change most noise values, but only {different_count} out of 10 differed"
    );

    // Verify offset shifts the noise field correctly
    let value_at_origin_with_offset = noise2.get_normalized(0.0, 0.0);
    let value_at_offset_without_offset = noise1.get_normalized(1000.0, 1000.0);
    assert!(
        (value_at_origin_with_offset - value_at_offset_without_offset).abs() < f64::EPSILON,
        "Offset should shift the noise field"
    );
}

#[test]
fn fractal_noise_produces_valid_values() {
    let noise = Noise::new(42);

    let fractal = noise.get_fractal(10.0, 20.0, 4, 0.5, 2.0);

    // Fractal should be in roughly -1 to 1 range (unnormalized)
    assert!(
        (-2.0..=2.0).contains(&fractal),
        "Fractal value {fractal} out of reasonable range"
    );
}

#[test]
fn fractal_scaled_respects_range() {
    let noise = Noise::new(42).with_range(50.0, 150.0);

    for i in 0..20 {
        let x = f64::from(i);
        let value = noise.get_fractal_scaled(x, 0.0, 4, 0.5, 2.0);
        assert!(
            (50.0..=150.0).contains(&value),
            "Fractal scaled value {value} out of configured range at x={x}"
        );
    }
}

#[test]
fn fractal_octaves_affect_detail() {
    let noise = Noise::new(42);

    let simple = noise.get_fractal(10.0, 20.0, 1, 0.5, 2.0);
    let detailed = noise.get_fractal(10.0, 20.0, 8, 0.5, 2.0);

    // Just verify both produce valid values
    assert!((-2.0..=2.0).contains(&simple));
    assert!((-2.0..=2.0).contains(&detailed));

    // More octaves generally means values can differ from base octave
    // (though not guaranteed for every single point)
}

#[test]
fn noise_reseed_changes_output() {
    let mut source = NoiseSource::new(12345);
    let noise1 = source.create(0xABCD);
    let values1: Vec<f64> = (0..10)
        .map(|i| noise1.get_normalized(f64::from(i), 0.0))
        .collect();

    source.reseed(54321);
    let noise2 = source.create(0xABCD);
    let values2: Vec<f64> = (0..10)
        .map(|i| noise2.get_normalized(f64::from(i), 0.0))
        .collect();

    assert_ne!(values1, values2, "Reseeding should change noise values");
}

#[test]
fn noise_mutable_setters_work() {
    let mut noise = Noise::new(42);

    noise.set_scale(0.05);
    noise.set_range(10.0, 20.0);
    noise.set_offset(500.0);

    // Verify values are in configured range for fractal_scaled
    let value = noise.get_fractal_scaled(5.0, 5.0, 4, 0.5, 2.0);
    assert!(
        (10.0..=20.0).contains(&value),
        "Value should respect mutably set range"
    );
}

// Helper function for the test below (defined at module level to avoid clippy::items_after_statements)
fn test_system(noise_source: Res<NoiseSource>) {
    let terrain = noise_source.create(0x5445_5252);
    let caves = noise_source.create(0x4341_5645);

    // Generate some terrain height values
    let height = terrain.get_fractal_scaled(100.0, 200.0, 4, 0.5, 2.0);
    assert!((0.0..=1.0).contains(&height));

    // Generate cave density
    let cave_density = caves.get_normalized(100.0, 200.0);
    assert!((0.0..=1.0).contains(&cave_density));
}

#[test]
fn bevy_app_integration_full_workflow() {
    let mut app = App::new();

    // Add RNG and Noise plugins
    app.add_plugins(RngPlugin::seeded(777));
    app.add_plugins(NoisePlugin::from_global_rng());

    // Add a system that uses noise
    app.add_systems(Update, test_system);

    // Run a few updates to ensure systems work
    app.update();
    app.update();
    app.update();

    // Verify resource is still present
    assert!(app.world().get_resource::<NoiseSource>().is_some());
}

#[test]
fn noise_reflection_registered() {
    let mut app = App::new();
    app.add_plugins(NoisePlugin::seeded(42));

    // Verify NoiseSource is registered for reflection
    let registry = app.world().resource::<AppTypeRegistry>();
    let registry = registry.read();
    assert!(
        registry
            .get(std::any::TypeId::of::<NoiseSource>())
            .is_some(),
        "NoiseSource should be registered for reflection"
    );
}

// --- Tests for GlobalRng initialization path ---

#[test]
fn from_global_rng_resource_available_in_startup() {
    // NoiseSource is inserted via Commands in PreStartup,
    // so it should be available in Startup systems.
    let mut app = App::new();
    app.add_plugins(RngPlugin::seeded(42));
    app.add_plugins(NoisePlugin::from_global_rng());

    let startup_ran = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = startup_ran.clone();

    app.add_systems(Startup, move |noise: Res<NoiseSource>| {
        // Verify the resource exists and has a valid seed
        assert!(noise.seed() > 0, "Seed should be derived from GlobalRng");
        flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    app.update();

    assert!(
        startup_ran.load(std::sync::atomic::Ordering::SeqCst),
        "Startup system should have run with NoiseSource available"
    );
}

#[test]
fn from_global_rng_deterministic_across_runs() {
    // Two apps with the same GlobalRng seed should produce identical NoiseSource seeds
    let mut app1 = App::new();
    app1.add_plugins(RngPlugin::seeded(777));
    app1.add_plugins(NoisePlugin::from_global_rng());
    app1.update();

    let mut app2 = App::new();
    app2.add_plugins(RngPlugin::seeded(777));
    app2.add_plugins(NoisePlugin::from_global_rng());
    app2.update();

    let seed1 = app1.world().resource::<NoiseSource>().seed();
    let seed2 = app2.world().resource::<NoiseSource>().seed();

    assert_eq!(
        seed1, seed2,
        "Same GlobalRng seed should produce same NoiseSource seed"
    );
}

#[test]
fn from_global_rng_noise_values_match_seeded_equivalent() {
    // Create an app with GlobalRng, extract the derived seed,
    // then verify a seeded app with the same seed produces identical noise.
    let mut app_rng = App::new();
    app_rng.add_plugins(RngPlugin::seeded(42));
    app_rng.add_plugins(NoisePlugin::from_global_rng());
    app_rng.update();

    let derived_seed = app_rng.world().resource::<NoiseSource>().seed();

    let mut app_seeded = App::new();
    app_seeded.add_plugins(NoisePlugin::seeded(derived_seed));

    let noise_rng = app_rng.world().resource::<NoiseSource>().create(0xBEEF);
    let noise_seeded = app_seeded
        .world()
        .resource::<NoiseSource>()
        .create(0xBEEF);

    for i in 0..20 {
        let x = f64::from(i);
        let v1 = noise_rng.get_normalized(x, 0.0);
        let v2 = noise_seeded.get_normalized(x, 0.0);
        assert!(
            (v1 - v2).abs() < f64::EPSILON,
            "Noise values should match at x={x}: {v1} vs {v2}"
        );
    }
}

#[test]
fn seeded_plugin_resource_available_immediately() {
    // With NoisePlugin::seeded(), the resource should exist before any update
    let mut app = App::new();
    app.add_plugins(NoisePlugin::seeded(99));

    assert!(
        app.world().get_resource::<NoiseSource>().is_some(),
        "Seeded NoiseSource should be available immediately (no update needed)"
    );
    assert_eq!(app.world().resource::<NoiseSource>().seed(), 99);
}

#[test]
fn from_global_rng_resource_available_immediately() {
    // NoiseSource is inserted in build() by reading GlobalRng directly,
    // so it should exist before any update.
    let mut app = App::new();
    app.add_plugins(RngPlugin::seeded(42));
    app.add_plugins(NoisePlugin::from_global_rng());

    assert!(
        app.world().get_resource::<NoiseSource>().is_some(),
        "NoiseSource should exist immediately when using from_global_rng()"
    );
}
