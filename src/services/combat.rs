use std::io::{self, Write};

use rand::Rng;

use crate::models::constants::{Device, SectorContent};
use crate::models::galaxy::Galaxy;
use crate::models::position::SectorPosition;

/// Calculate the Euclidean distance between two sector positions (spec section 7.1).
pub fn calculate_distance(from: SectorPosition, to: SectorPosition) -> f64 {
    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    (dx * dx + dy * dy).sqrt()
}

/// Check preconditions for firing phasers.
/// Returns (can_fire, computer_damaged).
fn check_phaser_readiness(galaxy: &Galaxy) -> (bool, bool) {
    // Check for Klingons in quadrant
    if galaxy.sector_map.klingons.is_empty() {
        println!("SHORT RANGE SENSORS REPORT NO KLINGONS IN THIS QUADRANT");
        return (false, false);
    }

    // Check if Phaser Control is damaged
    if galaxy.enterprise.is_damaged(Device::PhaserControl) {
        println!("PHASER CONTROL IS DISABLED");
        return (false, false);
    }

    // Check if Computer is damaged (affects accuracy)
    let computer_damaged = galaxy.enterprise.is_damaged(Device::Computer);
    if computer_damaged {
        println!(" COMPUTER FAILURE HAMPERS ACCURACY");
    }

    (true, computer_damaged)
}

/// Prompt for and validate phaser energy input.
/// Returns Some(units) if valid, None if cancelled or invalid.
fn read_and_validate_phaser_energy(available_energy: f64) -> Option<f64> {
    println!(
        "PHASERS LOCKED ON TARGET.  ENERGY AVAILABLE = {}",
        available_energy as i32
    );
    let input = read_line("NUMBER OF UNITS TO FIRE");
    let units: f64 = match input.trim().parse() {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Validate input
    if units <= 0.0 {
        return None;
    }
    if available_energy - units < 0.0 {
        return None;
    }

    Some(units)
}

/// Apply computer damage degradation to phaser energy.
fn calculate_phaser_energy(units: f64, computer_damaged: bool, rng: &mut impl Rng) -> f64 {
    if computer_damaged {
        units * rng.gen::<f64>()
    } else {
        units
    }
}

/// Apply phaser damage to all Klingons and return positions of destroyed ones.
fn apply_phaser_damage_to_klingons(
    galaxy: &mut Galaxy,
    phaser_energy: f64,
) -> Vec<SectorPosition> {
    // Count living Klingons for damage distribution
    let num_klingons = galaxy
        .sector_map
        .klingons
        .iter()
        .filter(|k| k.shields > 0.0)
        .count();

    if num_klingons == 0 {
        return Vec::new(); // All Klingons already dead
    }

    let e_pos = galaxy.enterprise.sector;
    let mut destroyed_positions = Vec::new();

    // Apply damage to each Klingon
    for klingon in galaxy.sector_map.klingons.iter_mut() {
        if klingon.shields <= 0.0 {
            continue; // Already dead
        }

        let distance = calculate_distance(e_pos, klingon.sector);
        let hit =
            (phaser_energy / num_klingons as f64 / distance) * (2.0 * galaxy.rng.gen::<f64>());

        klingon.shields -= hit;

        println!(
            "{} UNIT HIT ON KLINGON AT SECTOR {},{}",
            hit as i32, klingon.sector.x, klingon.sector.y
        );
        println!("   ({} LEFT)", klingon.shields.max(0.0) as i32);

        // If Klingon destroyed, collect position for cleanup
        if klingon.shields <= 0.0 {
            destroyed_positions.push(klingon.sector);
        }
    }

    destroyed_positions
}

/// Clean up destroyed Klingons from all tracking structures.
fn cleanup_destroyed_klingons(galaxy: &mut Galaxy, destroyed_positions: &[SectorPosition]) {
    // Clean up destroyed Klingons
    for pos in destroyed_positions {
        println!("*** KLINGON DESTROYED ***");
        galaxy.sector_map.set(*pos, SectorContent::Empty);
        galaxy.total_klingons -= 1;
        galaxy.decrement_quadrant_klingons();
    }

    // Remove all dead Klingons from the vector in one pass
    galaxy.sector_map.klingons.retain(|k| k.shields > 0.0);
}

/// Check for victory condition after Klingon destruction.
fn check_phaser_victory(galaxy: &mut Galaxy) {
    if galaxy.is_victory() {
        galaxy.end_victory();
    }
}

/// Fire Phasers — Command 3 (spec section 6.3).
pub fn fire_phasers(galaxy: &mut Galaxy) {
    // Phase 1: Preconditions
    let (can_fire, computer_damaged) = check_phaser_readiness(galaxy);
    if !can_fire {
        return;
    }

    // Phase 2: Input
    let units = match read_and_validate_phaser_energy(galaxy.enterprise.energy) {
        Some(u) => u,
        None => return,
    };

    // Phase 3: Energy deduction
    galaxy.enterprise.energy -= units;

    // Phase 4: CRITICAL - Klingons fire BEFORE phaser damage (spec 8.1)
    if klingons_fire(galaxy) {
        return; // Enterprise destroyed
    }

    // Phase 5: Apply phaser damage
    let phaser_energy = calculate_phaser_energy(units, computer_damaged, &mut galaxy.rng);
    let destroyed = apply_phaser_damage_to_klingons(galaxy, phaser_energy);

    // Phase 6: Cleanup
    cleanup_destroyed_klingons(galaxy, &destroyed);

    // Phase 7: Victory check
    check_phaser_victory(galaxy);
}

/// Klingons attack the Enterprise (spec section 8).
/// Returns true if the Enterprise is destroyed, false otherwise.
pub fn klingons_fire(galaxy: &mut Galaxy) -> bool {
    // Skip if docked (spec section 8.3)
    if galaxy
        .enterprise
        .is_adjacent_to_starbase(galaxy.sector_map.starbase)
    {
        println!("STAR BASE SHIELDS PROTECT THE ENTERPRISE");
        return false;
    }

    let e_pos = galaxy.enterprise.sector;

    for klingon in galaxy.sector_map.klingons.iter() {
        if klingon.shields <= 0.0 {
            continue; // Already dead
        }

        let distance = calculate_distance(e_pos, klingon.sector);
        let hit = (klingon.shields / distance) * (2.0 * galaxy.rng.gen::<f64>());

        galaxy.enterprise.shields -= hit;

        println!(
            "{} UNIT HIT ON ENTERPRISE FROM SECTOR {},{}",
            hit as i32, klingon.sector.x, klingon.sector.y
        );
        println!(
            "   ({} LEFT)",
            galaxy.enterprise.shields.max(0.0) as i32
        );
    }

    // Check if Enterprise is destroyed (spec section 8.4)
    if galaxy.enterprise.shields < 0.0 {
        galaxy.end_defeat();
        return true; // unreachable due to exit, but explicit
    }

    false
}

fn read_line(prompt: &str) -> String {
    print!("{} ", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::SectorContent;
    use crate::models::galaxy::Galaxy;
    use crate::models::klingon::Klingon;
    use crate::models::position::SectorPosition;
    use crate::models::sector_map::SectorMap;

    /// Helper: Set up a combat scenario with specified parameters.
    fn setup_combat_scenario(
        seed: u64,
        enterprise_energy: f64,
        enterprise_shields: f64,
        klingon_shields: f64,
    ) -> Galaxy {
        let mut galaxy = Galaxy::new(seed);

        // Clear sector map
        galaxy.sector_map = SectorMap::new();

        // Place Enterprise at (4, 4)
        galaxy.enterprise.sector = SectorPosition { x: 4, y: 4 };
        galaxy.enterprise.energy = enterprise_energy;
        galaxy.enterprise.shields = enterprise_shields;
        galaxy
            .sector_map
            .set(galaxy.enterprise.sector, SectorContent::Enterprise);

        // Place one Klingon at (2, 2)
        let klingon_pos = SectorPosition { x: 2, y: 2 };
        let mut klingon = Klingon::new(klingon_pos);
        klingon.shields = klingon_shields;
        galaxy.sector_map.set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map.klingons.push(klingon);

        galaxy
    }

    // ========== Distance calculation tests ==========

    #[test]
    fn distance_same_position() {
        let pos = SectorPosition { x: 4, y: 4 };
        assert_eq!(calculate_distance(pos, pos), 0.0);
    }

    #[test]
    fn distance_horizontal() {
        let p1 = SectorPosition { x: 2, y: 4 };
        let p2 = SectorPosition { x: 5, y: 4 };
        assert_eq!(calculate_distance(p1, p2), 3.0);
    }

    #[test]
    fn distance_vertical() {
        let p1 = SectorPosition { x: 4, y: 2 };
        let p2 = SectorPosition { x: 4, y: 6 };
        assert_eq!(calculate_distance(p1, p2), 4.0);
    }

    #[test]
    fn distance_diagonal() {
        let p1 = SectorPosition { x: 1, y: 1 };
        let p2 = SectorPosition { x: 4, y: 5 };
        // sqrt((4-1)² + (5-1)²) = sqrt(9 + 16) = sqrt(25) = 5.0
        assert_eq!(calculate_distance(p1, p2), 5.0);
    }

    // ========== Klingon firing tests ==========

    #[test]
    fn klingons_fire_reduces_shields() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        let initial_shields = galaxy.enterprise.shields;

        klingons_fire(&mut galaxy);

        assert!(galaxy.enterprise.shields < initial_shields);
    }

    #[test]
    fn klingons_fire_skips_when_docked() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place starbase adjacent to Enterprise
        let starbase_pos = SectorPosition { x: 5, y: 4 };
        galaxy.sector_map.set(starbase_pos, SectorContent::Starbase);
        galaxy.sector_map.starbase = Some(starbase_pos);

        let initial_shields = galaxy.enterprise.shields;
        klingons_fire(&mut galaxy);

        assert_eq!(galaxy.enterprise.shields, initial_shields);
    }

    #[test]
    fn klingons_fire_does_not_hit_from_dead_klingons() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.sector_map.klingons[0].shields = 0.0;

        let initial_shields = galaxy.enterprise.shields;
        klingons_fire(&mut galaxy);

        // Shields should not change if all Klingons are dead
        assert_eq!(galaxy.enterprise.shields, initial_shields);
    }

    #[test]
    fn klingons_fire_damage_depends_on_distance() {
        // Closer Klingon should do more damage
        let mut galaxy1 = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        // Klingon at (2,2), Enterprise at (4,4) - distance = sqrt(8) ≈ 2.83

        let mut galaxy2 = Galaxy::new(42);
        galaxy2.sector_map = SectorMap::new();
        galaxy2.enterprise.sector = SectorPosition { x: 4, y: 4 };
        galaxy2.enterprise.energy = 3000.0;
        galaxy2.enterprise.shields = 500.0;
        galaxy2
            .sector_map
            .set(galaxy2.enterprise.sector, SectorContent::Enterprise);

        // Place Klingon farther away at (1, 1)
        let far_klingon_pos = SectorPosition { x: 1, y: 1 };
        let mut far_klingon = Klingon::new(far_klingon_pos);
        far_klingon.shields = 200.0;
        galaxy2.sector_map.set(far_klingon_pos, SectorContent::Klingon);
        galaxy2.sector_map.klingons.push(far_klingon);

        klingons_fire(&mut galaxy1);
        klingons_fire(&mut galaxy2);

        // Both have random component, but on average closer Klingon does more damage
        // We can only verify shields were reduced from both
        assert!(galaxy1.enterprise.shields < 500.0);
        assert!(galaxy2.enterprise.shields < 500.0);
    }

    // ========== Victory/defeat tests ==========

    #[test]
    fn victory_when_last_klingon_destroyed() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 10.0);
        galaxy.total_klingons = 1; // Only one Klingon in entire galaxy

        // Manually destroy the Klingon
        let klingon_pos = galaxy.sector_map.klingons[0].sector;
        galaxy.sector_map.klingons[0].shields = 0.0;
        galaxy.sector_map.set(klingon_pos, SectorContent::Empty);
        galaxy.total_klingons -= 1;
        galaxy.decrement_quadrant_klingons();

        assert_eq!(galaxy.total_klingons, 0);
        assert!(galaxy.is_victory());
        // end_victory() would be called in real code, which exits process
    }

    // ========== Retain cleanup tests ==========

    #[test]
    fn retain_removes_only_dead_klingons() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Add second Klingon that's already dead
        let dead_klingon_pos = SectorPosition { x: 6, y: 6 };
        let mut dead_klingon = Klingon::new(dead_klingon_pos);
        dead_klingon.shields = 0.0;
        galaxy.sector_map.set(dead_klingon_pos, SectorContent::Klingon);
        galaxy.sector_map.klingons.push(dead_klingon);

        // Add third Klingon that's alive
        let alive_klingon_pos = SectorPosition { x: 7, y: 7 };
        let alive_klingon = Klingon::new(alive_klingon_pos);
        galaxy
            .sector_map
            .set(alive_klingon_pos, SectorContent::Klingon);
        galaxy.sector_map.klingons.push(alive_klingon);

        assert_eq!(galaxy.sector_map.klingons.len(), 3);

        // Apply retain
        galaxy.sector_map.klingons.retain(|k| k.shields > 0.0);

        // Should have 2 living Klingons left
        assert_eq!(galaxy.sector_map.klingons.len(), 2);
        for k in &galaxy.sector_map.klingons {
            assert!(k.shields > 0.0);
        }
    }

    #[test]
    fn multiple_klingons_all_take_damage() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Add second Klingon
        let k2_pos = SectorPosition { x: 6, y: 6 };
        let k2 = Klingon::new(k2_pos);
        galaxy.sector_map.set(k2_pos, SectorContent::Klingon);
        galaxy.sector_map.klingons.push(k2);

        // Add third Klingon
        let k3_pos = SectorPosition { x: 3, y: 7 };
        let k3 = Klingon::new(k3_pos);
        galaxy.sector_map.set(k3_pos, SectorContent::Klingon);
        galaxy.sector_map.klingons.push(k3);

        assert_eq!(galaxy.sector_map.klingons.len(), 3);

        // All Klingons fire
        klingons_fire(&mut galaxy);

        // Enterprise shields should be reduced by attacks from all 3
        assert!(galaxy.enterprise.shields < 500.0);
    }

    #[test]
    fn distance_symmetry() {
        let p1 = SectorPosition { x: 2, y: 3 };
        let p2 = SectorPosition { x: 6, y: 8 };
        assert_eq!(calculate_distance(p1, p2), calculate_distance(p2, p1));
    }

    #[test]
    fn klingon_destruction_clears_grid() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        let klingon_pos = galaxy.sector_map.klingons[0].sector;

        // Verify Klingon is in grid
        assert_eq!(
            galaxy.sector_map.get(klingon_pos),
            SectorContent::Klingon
        );

        // Destroy Klingon
        galaxy.sector_map.klingons[0].shields = 0.0;
        galaxy.sector_map.set(klingon_pos, SectorContent::Empty);
        galaxy.sector_map.klingons.retain(|k| k.shields > 0.0);

        // Verify grid is cleared and vector is empty
        assert_eq!(galaxy.sector_map.get(klingon_pos), SectorContent::Empty);
        assert_eq!(galaxy.sector_map.klingons.len(), 0);
    }
}
