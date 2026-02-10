use startrek::{GameEngine, GameState, DefeatReason};

#[test]
fn game_engine_initialization() {
    let engine = GameEngine::new(42);

    // Verify initial state
    assert!(matches!(engine.state(), GameState::Playing));

    let galaxy = engine.galaxy();
    assert!(galaxy.total_klingons() > 0);
    assert!(galaxy.total_starbases() > 0);
    assert!(galaxy.enterprise().energy() > 0.0);
    assert!(galaxy.enterprise().torpedoes() > 0);
}

#[test]
fn deterministic_gameplay_same_seed() {
    // Same seed should produce identical galaxy
    let engine1 = GameEngine::new(100);
    let engine2 = GameEngine::new(100);

    // Verify galaxies are identical
    assert_eq!(
        engine1.galaxy().total_klingons(),
        engine2.galaxy().total_klingons()
    );
    assert_eq!(
        engine1.galaxy().total_starbases(),
        engine2.galaxy().total_starbases()
    );
    assert_eq!(
        engine1.galaxy().enterprise().quadrant(),
        engine2.galaxy().enterprise().quadrant()
    );
    assert_eq!(
        engine1.galaxy().enterprise().sector(),
        engine2.galaxy().enterprise().sector()
    );
}

#[test]
fn different_seeds_produce_different_galaxies() {
    let engine1 = GameEngine::new(1);
    let engine2 = GameEngine::new(2);

    // At least one thing should be different
    let different =
        engine1.galaxy().total_klingons() != engine2.galaxy().total_klingons() ||
        engine1.galaxy().total_starbases() != engine2.galaxy().total_starbases() ||
        engine1.galaxy().enterprise().quadrant() != engine2.galaxy().enterprise().quadrant();

    assert!(different, "Different seeds should produce different galaxies");
}

#[test]
fn victory_condition_detected() {
    let mut engine = GameEngine::new(42);

    // Manually set all Klingons to 0 to simulate victory
    engine.galaxy_mut().set_total_klingons(0);

    // Check game over
    let state = engine.check_game_over();

    assert!(matches!(
        state,
        Some(GameState::Victory { .. })
    ), "Should detect victory when no Klingons remain");
}

#[test]
fn ship_destroyed_defeat_detected() {
    let mut engine = GameEngine::new(42);

    // Manually set shields below 0 to simulate destruction
    engine.galaxy_mut().enterprise_mut().set_shields(-1.0);

    // Check game over
    let state = engine.check_game_over();

    assert!(matches!(
        state,
        Some(GameState::Defeat {
            reason: DefeatReason::ShipDestroyed
        })
    ), "Should detect defeat when shields < 0");
}

#[test]
fn time_expired_defeat_detected() {
    let mut engine = GameEngine::new(42);

    // Manually advance time beyond mission duration
    let mission_duration = engine.galaxy().mission_duration();
    let _starting_stardate = engine.galaxy().stardate();

    for _ in 0..(mission_duration as i32 + 2) {
        engine.galaxy_mut().advance_time(1.0);
    }

    // Verify time has expired
    assert!(engine.galaxy().is_time_expired());

    // Check game over
    let state = engine.check_game_over();

    assert!(matches!(
        state,
        Some(GameState::Defeat {
            reason: DefeatReason::TimeExpired
        })
    ), "Should detect defeat when time expires");
}

#[test]
fn game_state_persists_after_check() {
    let mut engine = GameEngine::new(42);

    // Set victory condition
    engine.galaxy_mut().set_total_klingons(0);

    // Check multiple times
    let state1 = engine.check_game_over();
    let state2 = engine.check_game_over();

    // Both should return the same victory state
    assert!(matches!(state1, Some(GameState::Victory { .. })));
    assert!(matches!(state2, Some(GameState::Victory { .. })));

    // States should be identical
    if let (Some(GameState::Victory { rating: r1 }),
             Some(GameState::Victory { rating: r2 })) = (state1, state2) {
        assert_eq!(r1, r2);
    }
}

#[test]
fn efficiency_rating_calculated() {
    let mut engine = GameEngine::new(42);

    // Advance time slightly to avoid division by zero in efficiency rating
    engine.galaxy_mut().advance_time(1.0);

    // Set victory condition
    engine.galaxy_mut().set_total_klingons(0);

    // Check game over
    if let Some(GameState::Victory { rating }) = engine.check_game_over() {
        // Rating should be reasonable (based on initial Klingons / time used)
        // The formula is: (initial_klingons / elapsed_time) * 1000
        // With seed 42 and 1.0 time unit elapsed, rating should be reasonable
        assert!(rating > 0, "Efficiency rating should be positive");
        assert!(rating < 100000, "Efficiency rating should be reasonable");
    } else {
        panic!("Expected victory state");
    }
}

#[test]
fn galaxy_accessors_work() {
    let engine = GameEngine::new(42);
    let galaxy = engine.galaxy();

    // Test all major accessors
    let _klingons = galaxy.total_klingons();
    let _starbases = galaxy.total_starbases();
    let _enterprise = galaxy.enterprise();
    let _sector_map = galaxy.sector_map();
    let _quadrants = galaxy.quadrants();
    let stardate = galaxy.stardate();
    let _mission_duration = galaxy.mission_duration();

    // If we get here, all accessors work
    assert!(stardate > 0.0);
}

#[test]
fn mutable_galaxy_access() {
    let mut engine = GameEngine::new(42);

    // Get mutable access
    let galaxy = engine.galaxy_mut();

    // Perform mutations
    galaxy.advance_time(1.0);
    let _enterprise_mut = galaxy.enterprise_mut();

    // Verify time advanced
    let new_stardate = engine.galaxy().stardate();
    assert!(new_stardate > 0.0);
}

#[test]
fn shield_energy_manipulation() {
    let mut engine = GameEngine::new(42);
    let initial_shields = engine.galaxy().enterprise().shields();
    let initial_energy = engine.galaxy().enterprise().energy();

    // Verify initial values (shields start at 0.0, energy is positive)
    assert_eq!(initial_shields, 0.0);
    assert!(initial_energy > 0.0);

    // Modify shields
    engine.galaxy_mut().enterprise_mut().set_shields(100.0);
    assert_eq!(engine.galaxy().enterprise().shields(), 100.0);

    // Modify energy
    engine.galaxy_mut().enterprise_mut().set_energy(1500.0);
    assert_eq!(engine.galaxy().enterprise().energy(), 1500.0);
}

#[test]
fn docking_check_integration() {
    let mut engine = GameEngine::new(42);

    // Try to dock (may or may not succeed depending on starbase location)
    let _docked = engine.galaxy_mut().check_docking();

    // Result should be a boolean (true if docked, false otherwise)
    // We can't guarantee the result, but we can verify it doesn't crash
    // (test passes if we get here without panicking)
}

#[test]
fn condition_evaluation_integration() {
    let engine = GameEngine::new(42);

    // Evaluate condition (depends on current state)
    let condition = engine.galaxy().evaluate_condition();

    // Should return one of the valid conditions
    // We just verify this doesn't crash and returns a valid enum value
    println!("Current condition: {:?}", condition);
}

#[test]
fn torpedo_consumption() {
    let mut engine = GameEngine::new(42);
    let initial_torpedoes = engine.galaxy().enterprise().torpedoes();

    // Verify we have torpedoes
    assert!(initial_torpedoes > 0);

    // Consume a torpedo
    let result = engine.galaxy_mut().enterprise_mut().consume_torpedo();
    assert!(result.is_ok());

    // Verify torpedo count decreased
    assert_eq!(
        engine.galaxy().enterprise().torpedoes(),
        initial_torpedoes - 1
    );
}

#[test]
fn energy_consumption() {
    let mut engine = GameEngine::new(42);
    let initial_energy = engine.galaxy().enterprise().energy();

    // Consume energy
    let amount = 100.0;
    let result = engine
        .galaxy_mut()
        .enterprise_mut()
        .consume_energy(amount);

    assert!(result.is_ok());
    assert_eq!(
        engine.galaxy().enterprise().energy(),
        initial_energy - amount
    );
}

#[test]
fn insufficient_energy_handling() {
    let mut engine = GameEngine::new(42);

    // Set energy to a low value
    engine.galaxy_mut().enterprise_mut().set_energy(50.0);

    // Try to consume more energy than available
    let result = engine
        .galaxy_mut()
        .enterprise_mut()
        .consume_energy(100.0);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Insufficient energy");

    // Energy should remain unchanged
    assert_eq!(engine.galaxy().enterprise().energy(), 50.0);
}

#[test]
fn device_damage_and_repair() {
    use startrek::models::constants::Device;

    let mut engine = GameEngine::new(42);

    // Damage the shield control device
    engine
        .galaxy_mut()
        .enterprise_mut()
        .damage_device(Device::ShieldControl, 2.5);

    // Verify damage was applied
    let damage_state = engine.galaxy().enterprise().devices()[Device::ShieldControl as usize];
    assert_eq!(damage_state, -2.5);

    // Repair the device
    engine
        .galaxy_mut()
        .enterprise_mut()
        .repair_device(Device::ShieldControl, 1.5);

    // Verify repair was applied
    let new_damage_state = engine.galaxy().enterprise().devices()[Device::ShieldControl as usize];
    assert_eq!(new_damage_state, -1.0);
}

#[test]
fn quadrant_klingon_tracking() {
    let engine = GameEngine::new(42);
    let total_klingons = engine.galaxy().total_klingons();

    // Count Klingons in all quadrants
    let mut quadrant_sum = 0;
    for quadrant_row in engine.galaxy().quadrants() {
        for quadrant in quadrant_row {
            quadrant_sum += quadrant.klingons;
        }
    }

    // Total should match sum of all quadrants
    assert_eq!(total_klingons, quadrant_sum);
}

#[test]
fn starbase_tracking() {
    let engine = GameEngine::new(42);
    let total_starbases = engine.galaxy().total_starbases();

    // Count starbases in all quadrants
    let mut quadrant_sum = 0;
    for quadrant_row in engine.galaxy().quadrants() {
        for quadrant in quadrant_row {
            quadrant_sum += quadrant.starbases;
        }
    }

    // Total should match sum of all quadrants
    assert_eq!(total_starbases, quadrant_sum);
}
