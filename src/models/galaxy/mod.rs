//! Galaxy model
//!
//! Represents the game universe with 8x8 quadrants, each containing
//! Klingons, starbases, stars, and the Enterprise.

mod generation;
mod quadrant_ops;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fmt;

use super::constants::{
    Condition, GALAXY_SIZE, INITIAL_ENERGY, MISSION_DURATION, SectorContent,
};
use super::enterprise::Enterprise;
use super::errors::GameResult;
use super::position::{QuadrantPosition, SectorPosition};
use super::quadrant::QuadrantData;
use super::sector_map::SectorMap;

use generation::generate_galaxy;
use quadrant_ops::{
    decrement_quadrant_klingons, decrement_quadrant_starbases, enter_quadrant,
    record_quadrant_to_memory,
};

/// Consolidated Klingon count tracking
struct KlingonCount {
    total: i32,
    initial: i32,
}

/// Top-level game state container.
pub struct Galaxy {
    stardate: f64,
    starting_stardate: f64,
    mission_duration: f64,
    /// 8x8 grid of quadrant data. Internal 0-based: quadrants[y-1][x-1].
    quadrants: [[QuadrantData; GALAXY_SIZE]; GALAXY_SIZE],
    /// Computer's knowledge of the galaxy. None = unscanned, Some = scanned quadrant data.
    computer_memory: [[Option<QuadrantData>; GALAXY_SIZE]; GALAXY_SIZE],
    klingon_count: KlingonCount,
    total_starbases: i32,
    enterprise: Enterprise,
    sector_map: SectorMap,
    rng: StdRng,
}

impl Galaxy {
    /// Create and initialize a new game from the player's seed number.
    pub fn new(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Starting stardate (spec 3.2): floor(random * 20 + 20) * 100
        let starting_stardate = (rng.gen::<f64>() * 20.0 + 20.0).floor() * 100.0;

        // Generate galaxy with regeneration guard (spec 3.4, 3.5)
        let (quadrants, total_klingons, total_starbases) = generate_galaxy(&mut rng);

        // Random starting position (spec 3.3)
        let quadrant = QuadrantPosition {
            x: rng.gen_range(1..=8),
            y: rng.gen_range(1..=8),
        };
        let sector = SectorPosition {
            x: rng.gen_range(1..=8),
            y: rng.gen_range(1..=8),
        };

        let mut galaxy = Galaxy {
            stardate: starting_stardate,
            starting_stardate,
            mission_duration: MISSION_DURATION,
            quadrants,
            computer_memory: [[None; GALAXY_SIZE]; GALAXY_SIZE],
            klingon_count: KlingonCount {
                total: total_klingons,
                initial: total_klingons,
            },
            total_starbases,
            enterprise: Enterprise::new(quadrant, sector),
            sector_map: SectorMap::new(),
            rng,
        };

        // Enter the starting quadrant (populates sector map)
        galaxy.enter_quadrant();

        // Record starting quadrant to computer memory
        galaxy.record_quadrant_to_memory(
            galaxy.enterprise.quadrant().x,
            galaxy.enterprise.quadrant().y,
        );

        galaxy
    }

    // ========== Accessor Methods ==========

    /// Get current stardate
    pub fn stardate(&self) -> f64 {
        self.stardate
    }

    /// Get starting stardate
    pub fn starting_stardate(&self) -> f64 {
        self.starting_stardate
    }

    /// Get mission duration
    pub fn mission_duration(&self) -> f64 {
        self.mission_duration
    }

    /// Get total Klingons remaining
    pub fn total_klingons(&self) -> i32 {
        self.klingon_count.total
    }

    /// Get initial Klingon count
    pub fn initial_klingons(&self) -> i32 {
        self.klingon_count.initial
    }

    /// Get total starbases
    pub fn total_starbases(&self) -> i32 {
        self.total_starbases
    }

    /// Get reference to Enterprise
    pub fn enterprise(&self) -> &Enterprise {
        &self.enterprise
    }

    /// Get mutable reference to Enterprise
    pub fn enterprise_mut(&mut self) -> &mut Enterprise {
        &mut self.enterprise
    }

    /// Get reference to sector map
    pub fn sector_map(&self) -> &SectorMap {
        &self.sector_map
    }

    /// Get mutable reference to sector map
    pub fn sector_map_mut(&mut self) -> &mut SectorMap {
        &mut self.sector_map
    }

    /// Get reference to quadrants array
    pub fn quadrants(&self) -> &[[QuadrantData; GALAXY_SIZE]; GALAXY_SIZE] {
        &self.quadrants
    }

    /// Get mutable reference to RNG
    pub fn rng_mut(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    /// Advance stardate by delta
    pub fn advance_time(&mut self, delta: f64) {
        self.stardate += delta;
    }

    /// Decrement total Klingon count
    pub fn decrement_klingons(&mut self) {
        self.klingon_count.total -= 1;
    }

    /// Decrement total starbase count
    pub fn decrement_starbases(&mut self) {
        self.total_starbases -= 1;
    }

    /// Get reference to computer memory
    pub fn computer_memory(&self) -> &[[Option<QuadrantData>; GALAXY_SIZE]; GALAXY_SIZE] {
        &self.computer_memory
    }

    /// Get mutable reference to computer memory
    pub fn computer_memory_mut(&mut self) -> &mut [[Option<QuadrantData>; GALAXY_SIZE]; GALAXY_SIZE] {
        &mut self.computer_memory
    }

    // Test-only setters
    #[cfg(test)]
    pub fn set_total_klingons(&mut self, count: i32) {
        self.klingon_count.total = count;
    }

    #[cfg(test)]
    pub fn set_initial_klingons(&mut self, count: i32) {
        self.klingon_count.initial = count;
    }

    #[cfg(test)]
    pub fn set_total_starbases(&mut self, count: i32) {
        self.total_starbases = count;
    }

    #[cfg(test)]
    pub fn set_stardate(&mut self, stardate: f64) {
        self.stardate = stardate;
    }

    #[cfg(test)]
    pub fn set_starting_stardate(&mut self, stardate: f64) {
        self.starting_stardate = stardate;
    }

    // ========== End Accessor Methods ==========

    // ========== Atomic Update Methods ==========

    /// Atomically destroy a Klingon, updating all tracking locations
    pub fn destroy_klingon(&mut self, pos: SectorPosition) -> GameResult<()> {
        // Remove from sector map
        self.sector_map.set(pos, SectorContent::Empty);

        // Decrement global count
        self.klingon_count.total -= 1;

        // Decrement quadrant count
        let q = self.enterprise.quadrant();
        let qy = (q.y - 1) as usize;
        let qx = (q.x - 1) as usize;
        self.quadrants[qy][qx].klingons -= 1;

        Ok(())
    }

    /// Atomically destroy a starbase, updating all tracking locations
    pub fn destroy_starbase(&mut self, pos: SectorPosition) {
        // Remove from sector map
        self.sector_map.set(pos, SectorContent::Empty);
        self.sector_map.starbase = None;

        // Decrement global count
        self.total_starbases -= 1;

        // Decrement quadrant count
        let q = self.enterprise.quadrant();
        let qy = (q.y - 1) as usize;
        let qx = (q.x - 1) as usize;
        self.quadrants[qy][qx].starbases = 0;
    }

    // ========== End Atomic Update Methods ==========

    /// Enter the current quadrant: clear sector map and place all entities.
    /// Called on game start and every quadrant transition (spec section 4).
    pub fn enter_quadrant(&mut self) {
        enter_quadrant(
            &mut self.sector_map,
            &self.enterprise,
            &self.quadrants,
            &mut self.rng,
        );
    }

    /// Check if the Enterprise is adjacent to a starbase and dock if so.
    /// Returns true if docked (spec section 9.1-9.2).
    pub fn check_docking(&mut self) -> bool {
        self.enterprise.check_docking(self.sector_map.starbase)
    }

    /// Record a quadrant's data into computer memory.
    /// Does nothing if the Computer device is damaged or coordinates are out of range.
    pub fn record_quadrant_to_memory(&mut self, x: i32, y: i32) {
        record_quadrant_to_memory(
            &mut self.computer_memory,
            &self.quadrants,
            &self.enterprise,
            x,
            y,
        );
    }

    /// Evaluate the ship's condition code (spec section 9.4).
    pub fn evaluate_condition(&self) -> Condition {
        if self.enterprise.is_adjacent_to_starbase(self.sector_map.starbase) {
            return Condition::Docked;
        }

        if !self.sector_map.klingons.is_empty() {
            Condition::Red
        } else if self.enterprise.energy() < INITIAL_ENERGY * 0.1 {
            Condition::Yellow
        } else {
            Condition::Green
        }
    }

    /// Check if all Klingons have been destroyed (spec section 10.1).
    pub fn all_klingons_destroyed(&self) -> bool {
        self.klingon_count.total == 0
    }

    /// Check if time has expired (spec section 10.3).
    pub fn is_time_expired(&self) -> bool {
        self.stardate > self.starting_stardate + self.mission_duration
    }

    /// Calculate the efficiency rating (spec section 7.7).
    pub fn efficiency_rating(&self) -> i32 {
        let elapsed = self.stardate - self.starting_stardate;
        ((self.klingon_count.initial as f64 / elapsed) * 1000.0) as i32
    }

    /// Update the quadrant's klingon count after removing one.
    pub fn decrement_quadrant_klingons(&mut self) {
        decrement_quadrant_klingons(&mut self.quadrants, &self.enterprise);
    }

    /// Update the quadrant's starbase count after removing one.
    pub fn decrement_quadrant_starbases(&mut self) {
        decrement_quadrant_starbases(&mut self.quadrants, &self.enterprise);
    }
}

// Custom Debug that doesn't expose RNG internals
impl fmt::Debug for Galaxy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Galaxy")
            .field("stardate", &self.stardate)
            .field("total_klingons", &self.total_klingons())
            .field("starbases", &self.total_starbases())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::{
        Condition, GALAXY_SIZE, INITIAL_ENERGY, INITIAL_SHIELDS, INITIAL_TORPEDOES,
        MISSION_DURATION, SECTOR_SIZE, SectorContent,
    };

    // ========== Galaxy initialization tests ==========

    #[test]
    fn new_galaxy_has_positive_klingons_and_starbases() {
        let galaxy = Galaxy::new(0);
        assert!(galaxy.total_klingons() > 0, "must have at least one Klingon");
        assert!(
            galaxy.total_starbases() > 0,
            "must have at least one starbase"
        );
    }

    #[test]
    fn initial_klingons_equals_total_klingons() {
        let galaxy = Galaxy::new(42);
        assert_eq!(galaxy.initial_klingons(), galaxy.total_klingons());
    }

    #[test]
    fn stardate_is_multiple_of_100() {
        for seed in 0..20 {
            let galaxy = Galaxy::new(seed);
            assert_eq!(
                galaxy.stardate() % 100.0,
                0.0,
                "seed {}: stardate {} should be a multiple of 100",
                seed,
                galaxy.stardate()
            );
        }
    }

    #[test]
    fn stardate_in_valid_range() {
        for seed in 0..20 {
            let galaxy = Galaxy::new(seed);
            assert!(
                galaxy.stardate() >= 2000.0 && galaxy.stardate() <= 3900.0,
                "seed {}: stardate {} out of range [2000, 3900]",
                seed,
                galaxy.stardate()
            );
        }
    }

    #[test]
    fn mission_duration_is_30() {
        let galaxy = Galaxy::new(0);
        assert_eq!(galaxy.mission_duration(), MISSION_DURATION);
    }

    #[test]
    fn enterprise_position_in_valid_range() {
        for seed in 0..20 {
            let galaxy = Galaxy::new(seed);
            let q = galaxy.enterprise.quadrant();
            let s = galaxy.enterprise.sector();
            assert!(q.x >= 1 && q.x <= 8, "quadrant x out of range");
            assert!(q.y >= 1 && q.y <= 8, "quadrant y out of range");
            assert!(s.x >= 1 && s.x <= 8, "sector x out of range");
            assert!(s.y >= 1 && s.y <= 8, "sector y out of range");
        }
    }

    #[test]
    fn enterprise_starts_with_full_resources() {
        let galaxy = Galaxy::new(0);
        assert_eq!(galaxy.enterprise.energy(), INITIAL_ENERGY);
        assert_eq!(galaxy.enterprise.torpedoes(), INITIAL_TORPEDOES);
        assert_eq!(galaxy.enterprise.shields(), INITIAL_SHIELDS);
    }

    #[test]
    fn quadrant_klingon_counts_sum_to_total() {
        let galaxy = Galaxy::new(42);
        let mut sum = 0;
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                let k = galaxy.quadrants[y][x].klingons;
                assert!(k >= 0 && k <= 3, "klingon count out of [0,3]");
                sum += k;
            }
        }
        assert_eq!(sum, galaxy.total_klingons());
    }

    #[test]
    fn quadrant_starbase_counts_sum_to_total() {
        let galaxy = Galaxy::new(42);
        let mut sum = 0;
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                let b = galaxy.quadrants[y][x].starbases;
                assert!(b == 0 || b == 1, "starbase count not 0 or 1");
                sum += b;
            }
        }
        assert_eq!(sum, galaxy.total_starbases());
    }

    #[test]
    fn stars_in_valid_range() {
        let galaxy = Galaxy::new(42);
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                let s = galaxy.quadrants[y][x].stars;
                assert!(s >= 1 && s <= 8, "stars {} out of range [1,8]", s);
            }
        }
    }

    #[test]
    fn computer_memory_starts_unscanned_except_starting_quadrant() {
        let galaxy = Galaxy::new(0);
        let qx = galaxy.enterprise.quadrant().x;
        let qy = galaxy.enterprise.quadrant().y;
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                if x == (qx - 1) as usize && y == (qy - 1) as usize {
                    // Starting quadrant should be recorded
                    let expected = galaxy.quadrants[y][x];
                    assert_eq!(galaxy.computer_memory[y][x], Some(expected));
                } else {
                    // All other quadrants should be unscanned
                    assert_eq!(galaxy.computer_memory[y][x], None);
                }
            }
        }
    }

    #[test]
    fn sector_map_has_enterprise_after_init() {
        let galaxy = Galaxy::new(42);
        let content = galaxy.sector_map.get(galaxy.enterprise.sector());
        assert_eq!(content, SectorContent::Enterprise);
    }

    #[test]
    fn sector_map_entity_counts_match_quadrant_data() {
        let galaxy = Galaxy::new(42);
        let q = galaxy.enterprise.quadrant();
        let qdata = galaxy.quadrants[(q.y - 1) as usize][(q.x - 1) as usize];

        assert_eq!(
            galaxy.sector_map.klingons.len() as i32,
            qdata.klingons,
            "klingon count mismatch"
        );

        if qdata.starbases > 0 {
            assert!(galaxy.sector_map.starbase.is_some());
        } else {
            assert!(galaxy.sector_map.starbase.is_none());
        }

        // Count stars in the sector map
        let mut star_count = 0;
        for y in 1..=8 {
            for x in 1..=8 {
                if galaxy.sector_map.get(SectorPosition { x, y }) == SectorContent::Star {
                    star_count += 1;
                }
            }
        }
        assert_eq!(star_count, qdata.stars, "star count mismatch");
    }

    #[test]
    fn deterministic_with_same_seed() {
        let g1 = Galaxy::new(123);
        let g2 = Galaxy::new(123);
        assert_eq!(g1.stardate, g2.stardate);
        assert_eq!(g1.total_klingons(), g2.total_klingons());
        assert_eq!(g1.total_starbases, g2.total_starbases);
        assert_eq!(g1.enterprise.quadrant(), g2.enterprise.quadrant());
        assert_eq!(g1.enterprise.sector(), g2.enterprise.sector());
    }

    #[test]
    fn different_seeds_produce_different_galaxies() {
        let g1 = Galaxy::new(1);
        let g2 = Galaxy::new(2);
        // At least one of these should differ
        let same = g1.stardate == g2.stardate
            && g1.total_klingons() == g2.total_klingons()
            && g1.enterprise.quadrant() == g2.enterprise.quadrant();
        assert!(!same, "different seeds should produce different state");
    }



    // ========== Condition evaluation tests ==========

    #[test]
    fn condition_green_no_klingons_full_energy() {
        let mut galaxy = Galaxy::new(42);
        galaxy.sector_map = SectorMap::new();
        let sector = SectorPosition { x: 4, y: 4 };
        galaxy.enterprise.move_to(galaxy.enterprise.quadrant(), sector);
        galaxy
            .sector_map
            .set(galaxy.enterprise.sector(), SectorContent::Enterprise);
        galaxy.enterprise.set_energy(INITIAL_ENERGY);

        assert_eq!(galaxy.evaluate_condition(), Condition::Green);
    }

    #[test]
    fn condition_yellow_low_energy() {
        let mut galaxy = Galaxy::new(42);
        galaxy.sector_map = SectorMap::new();
        let sector = SectorPosition { x: 4, y: 4 };
        galaxy.enterprise.move_to(galaxy.enterprise.quadrant(), sector);
        galaxy
            .sector_map
            .set(galaxy.enterprise.sector(), SectorContent::Enterprise);
        galaxy.enterprise.set_energy(INITIAL_ENERGY * 0.05); // below 10%

        assert_eq!(galaxy.evaluate_condition(), Condition::Yellow);
    }

    #[test]
    fn condition_red_klingons_present() {
        let mut galaxy = Galaxy::new(42);
        galaxy.sector_map = SectorMap::new();
        let sector = SectorPosition { x: 4, y: 4 };
        galaxy.enterprise.move_to(galaxy.enterprise.quadrant(), sector);
        galaxy
            .sector_map
            .set(galaxy.enterprise.sector(), SectorContent::Enterprise);
        // Add a Klingon
        let kpos = SectorPosition { x: 1, y: 1 };
        galaxy.sector_map.set(kpos, SectorContent::Klingon);
        galaxy
            .sector_map
            .klingons
            .push(crate::models::klingon::Klingon::new(kpos));

        assert_eq!(galaxy.evaluate_condition(), Condition::Red);
    }

    /// Helper: set up a galaxy with a starbase at a known position.
    fn setup_galaxy_with_starbase(
        enterprise_sector: SectorPosition,
        starbase_sector: SectorPosition,
    ) -> Galaxy {
        let mut galaxy = Galaxy::new(42);
        galaxy.sector_map = SectorMap::new();
        galaxy.enterprise.move_to(galaxy.enterprise.quadrant(), enterprise_sector);
        galaxy
            .sector_map
            .set(enterprise_sector, SectorContent::Enterprise);
        galaxy
            .sector_map
            .set(starbase_sector, SectorContent::Starbase);
        galaxy.sector_map.starbase = Some(starbase_sector);
        galaxy
    }

    #[test]
    fn condition_docked_adjacent_to_starbase() {
        let enterprise = SectorPosition { x: 4, y: 4 };
        let starbase = SectorPosition { x: 5, y: 4 };
        let galaxy = setup_galaxy_with_starbase(enterprise, starbase);

        assert_eq!(galaxy.evaluate_condition(), Condition::Docked);
    }

    #[test]
    fn render_row_shows_enterprise_symbol() {
        let galaxy = Galaxy::new(42);
        let ey = galaxy.enterprise.sector().y;
        let row = galaxy.sector_map.render_row(ey);
        assert!(
            row.contains("<*>"),
            "row {} should contain Enterprise symbol <*>, got: {}",
            ey,
            row
        );
    }

    #[test]
    fn render_row_length_is_24_chars() {
        let galaxy = Galaxy::new(42);
        for y in 1..=SECTOR_SIZE as i32 {
            let row = galaxy.sector_map.render_row(y);
            assert_eq!(
                row.len(),
                SECTOR_SIZE * 3,
                "row {} should be {} chars, got {}",
                y,
                SECTOR_SIZE * 3,
                row.len()
            );
        }
    }

    // ========== Game over condition tests ==========

    #[test]
    fn all_klingons_destroyed_when_no_klingons() {
        let mut galaxy = Galaxy::new(42);
        galaxy.set_total_klingons(0);
        assert!(galaxy.all_klingons_destroyed());
    }

    #[test]
    fn not_all_klingons_destroyed_when_klingons_remain() {
        let galaxy = Galaxy::new(42);
        assert!(!galaxy.all_klingons_destroyed());
        assert!(galaxy.total_klingons() > 0);
    }

    #[test]
    fn efficiency_rating_calculation() {
        let mut galaxy = Galaxy::new(42);
        galaxy.set_initial_klingons(15);
        galaxy.set_stardate(2010.0);
        galaxy.set_starting_stardate(2000.0);
        // (15 / 10) * 1000 = 1500
        assert_eq!(galaxy.efficiency_rating(), 1500);
    }

    #[test]
    fn efficiency_rating_truncates_to_integer() {
        let mut galaxy = Galaxy::new(42);
        galaxy.set_initial_klingons(17);
        galaxy.set_stardate(2007.0);
        galaxy.set_starting_stardate(2000.0);
        // (17 / 7) * 1000 = 2428.571... truncated to 2428
        assert_eq!(galaxy.efficiency_rating(), 2428);
    }

    #[test]
    fn decrement_quadrant_klingons_updates_count() {
        let mut galaxy = Galaxy::new(42);
        let q = galaxy.enterprise.quadrant();
        let initial_count = galaxy.quadrants[(q.y - 1) as usize][(q.x - 1) as usize].klingons;

        galaxy.decrement_quadrant_klingons();

        let new_count = galaxy.quadrants[(q.y - 1) as usize][(q.x - 1) as usize].klingons;
        assert_eq!(new_count, initial_count - 1);
    }
}
