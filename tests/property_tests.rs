use proptest::prelude::*;
use startrek::{GameEngine, GameState};
use startrek::models::galaxy::Galaxy;
use startrek::models::quadrant::QuadrantData;
use startrek::models::position::SectorPosition;
use startrek::services::combat::calculate_distance;

proptest! {
    /// Property: Total Klingons always equals sum of quadrant Klingons
    #[test]
    fn klingon_count_invariant(seed in any::<u64>()) {
        let galaxy = Galaxy::new(seed);

        let quadrant_sum: i32 = galaxy.quadrants()
            .iter()
            .flatten()
            .map(|q| q.klingons)
            .sum();

        prop_assert_eq!(
            galaxy.total_klingons(),
            quadrant_sum,
            "Total Klingon count {} doesn't match quadrant sum {}",
            galaxy.total_klingons(),
            quadrant_sum
        );
    }

    /// Property: Total starbases always equals sum of quadrant starbases
    #[test]
    fn starbase_count_invariant(seed in any::<u64>()) {
        let galaxy = Galaxy::new(seed);

        let quadrant_sum: i32 = galaxy.quadrants()
            .iter()
            .flatten()
            .map(|q| q.starbases)
            .sum();

        prop_assert_eq!(
            galaxy.total_starbases(),
            quadrant_sum,
            "Total starbase count {} doesn't match quadrant sum {}",
            galaxy.total_starbases(),
            quadrant_sum
        );
    }

    /// Property: Galaxy generation always succeeds and has valid state
    #[test]
    fn galaxy_generation_succeeds(seed in any::<u64>()) {
        let galaxy = Galaxy::new(seed);

        // Property: Galaxy always has at least one Klingon
        prop_assert!(
            galaxy.total_klingons() > 0,
            "Galaxy must have at least one Klingon"
        );

        // Property: Galaxy always has at least one starbase
        prop_assert!(
            galaxy.total_starbases() > 0,
            "Galaxy must have at least one starbase"
        );

        // Property: Enterprise position is valid
        let e = galaxy.enterprise();
        prop_assert!(e.quadrant().x >= 1 && e.quadrant().x <= 8);
        prop_assert!(e.quadrant().y >= 1 && e.quadrant().y <= 8);
        prop_assert!(e.sector().x >= 1 && e.sector().x <= 8);
        prop_assert!(e.sector().y >= 1 && e.sector().y <= 8);
    }

    /// Property: Shield control preserves total energy
    #[test]
    fn shield_control_energy_conservation(
        seed in any::<u64>(),
        transfer in 0.1f64..3000.0f64
    ) {
        let mut galaxy = Galaxy::new(seed);
        let enterprise = galaxy.enterprise_mut();

        let initial_total = enterprise.energy() + enterprise.shields();

        // Attempt shield control
        if enterprise.shield_control(transfer).is_ok() {
            let final_total = enterprise.energy() + enterprise.shields();
            prop_assert!(
                (final_total - initial_total).abs() < 0.01,
                "Energy conservation violated: {} != {}",
                initial_total,
                final_total
            );
        }
    }

    /// Property: Quadrant encoding round-trips correctly
    #[test]
    fn quadrant_encoding_roundtrip(
        klingons in 0i32..10,
        starbases in 0i32..2,
        stars in 0i32..10
    ) {
        let data = QuadrantData { klingons, starbases, stars };
        let encoded = data.encoded();

        // Decode by extracting digits
        let decoded_klingons = encoded / 100;
        let decoded_starbases = (encoded / 10) % 10;
        let decoded_stars = encoded % 10;

        prop_assert_eq!(decoded_klingons, klingons);
        prop_assert_eq!(decoded_starbases, starbases);
        prop_assert_eq!(decoded_stars, stars);
    }

    /// Property: Distance calculation is symmetric
    #[test]
    fn distance_is_symmetric(
        x1 in 1i32..=8, y1 in 1i32..=8,
        x2 in 1i32..=8, y2 in 1i32..=8
    ) {
        let pos1 = SectorPosition { x: x1, y: y1 };
        let pos2 = SectorPosition { x: x2, y: y2 };

        let d1 = calculate_distance(pos1, pos2);
        let d2 = calculate_distance(pos2, pos1);

        prop_assert!(
            (d1 - d2).abs() < 0.001,
            "Distance should be symmetric"
        );
    }

    /// Property: GameEngine state transitions are valid
    #[test]
    fn game_state_transitions_valid(seed in any::<u64>()) {
        let mut engine = GameEngine::new(seed);

        // Check game over returns consistent state
        if let Some(state) = engine.check_game_over() {
            // If game is over, calling again should return same state
            let state2 = engine.check_game_over();
            prop_assert!(state2.is_some());

            // State should not be Playing if game is over
            prop_assert!(!matches!(state, GameState::Playing));
        }
    }

    /// Property: Distance calculation is always non-negative
    #[test]
    fn distance_is_non_negative(
        x1 in 1i32..=8, y1 in 1i32..=8,
        x2 in 1i32..=8, y2 in 1i32..=8
    ) {
        let pos1 = SectorPosition { x: x1, y: y1 };
        let pos2 = SectorPosition { x: x2, y: y2 };

        let distance = calculate_distance(pos1, pos2);

        prop_assert!(
            distance >= 0.0,
            "Distance must be non-negative, got {}",
            distance
        );
    }

    /// Property: Distance satisfies triangle inequality
    #[test]
    fn distance_triangle_inequality(
        x1 in 1i32..=8, y1 in 1i32..=8,
        x2 in 1i32..=8, y2 in 1i32..=8,
        x3 in 1i32..=8, y3 in 1i32..=8
    ) {
        let pos1 = SectorPosition { x: x1, y: y1 };
        let pos2 = SectorPosition { x: x2, y: y2 };
        let pos3 = SectorPosition { x: x3, y: y3 };

        let d12 = calculate_distance(pos1, pos2);
        let d23 = calculate_distance(pos2, pos3);
        let d13 = calculate_distance(pos1, pos3);

        // Triangle inequality: d(a,c) <= d(a,b) + d(b,c)
        prop_assert!(
            d13 <= d12 + d23 + 0.001, // Small epsilon for floating point
            "Triangle inequality violated: {} > {} + {}",
            d13, d12, d23
        );
    }

    /// Property: Initial and total Klingons start equal
    #[test]
    fn initial_klingons_equals_total(seed in any::<u64>()) {
        let galaxy = Galaxy::new(seed);

        prop_assert_eq!(
            galaxy.initial_klingons(),
            galaxy.total_klingons(),
            "Initial Klingon count should equal total at start"
        );
    }
}
