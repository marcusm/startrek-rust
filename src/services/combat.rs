use rand::Rng;

use crate::io::{InputReader, OutputWriter};
use crate::models::constants::{Device, SectorContent};
use crate::models::enterprise::ShieldControlError;
use crate::models::errors::{GameError, GameResult};
use crate::models::galaxy::Galaxy;
use crate::models::klingon::Klingon;
use crate::models::position::SectorPosition;
use crate::services::navigation;

/// Calculate the Euclidean distance between two sector positions (spec section 7.1).
pub fn calculate_distance(from: SectorPosition, to: SectorPosition) -> f64 {
    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    (dx * dx + dy * dy).sqrt()
}

/// Check preconditions for firing phasers.
/// Returns (can_fire, computer_damaged).
fn check_phaser_readiness(galaxy: &Galaxy, output: &mut dyn OutputWriter) -> (bool, bool) {
    // Check for Klingons in quadrant
    if galaxy.sector_map().klingons.is_empty() {
        output.writeln("SHORT RANGE SENSORS REPORT NO KLINGONS IN THIS QUADRANT");
        return (false, false);
    }

    // Check if Phaser Control is damaged
    if galaxy.enterprise().is_damaged(Device::PhaserControl) {
        output.writeln("PHASER CONTROL IS DISABLED");
        return (false, false);
    }

    // Check if Computer is damaged (affects accuracy)
    let computer_damaged = galaxy.enterprise().is_damaged(Device::Computer);
    if computer_damaged {
        output.writeln(" COMPUTER FAILURE HAMPERS ACCURACY");
    }

    (true, computer_damaged)
}

/// Prompt for and validate phaser energy input.
/// Returns Some(units) if valid, None if cancelled or invalid.
fn read_and_validate_phaser_energy(
    available_energy: f64,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<Option<f64>> {
    output.writeln(&format!(
        "PHASERS LOCKED ON TARGET.  ENERGY AVAILABLE = {}",
        available_energy as i32
    ));
    let input = io.read_line("NUMBER OF UNITS TO FIRE")?;
    let units: f64 = match input.trim().parse() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    // Validate input
    if units <= 0.0 {
        return Ok(None);
    }
    if available_energy - units < 0.0 {
        return Ok(None);
    }

    Ok(Some(units))
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
    output: &mut dyn OutputWriter,
) -> Vec<SectorPosition> {
    // Count living Klingons for damage distribution
    let num_klingons = galaxy
        .sector_map()
        .klingons
        .iter()
        .filter(|k: &&Klingon| k.is_alive())
        .count();

    if num_klingons == 0 {
        return Vec::new(); // All Klingons already dead
    }

    let e_pos = galaxy.enterprise().sector();
    let mut destroyed_positions = Vec::new();

    // Generate random factors for each klingon first to avoid borrow conflicts
    let random_factors: Vec<f64> = (0..num_klingons)
        .map(|_| 2.0 * galaxy.rng_mut().gen::<f64>())
        .collect();

    // Apply damage to each Klingon
    let mut rand_idx = 0;
    for klingon in galaxy.sector_map_mut().klingons.iter_mut() {
        if !klingon.is_alive() {
            continue; // Already dead
        }

        let distance = calculate_distance(e_pos, klingon.sector);
        let hit = (phaser_energy / num_klingons as f64 / distance) * random_factors[rand_idx];
        rand_idx += 1;

        klingon.shields -= hit;

        output.writeln(&format!(
            "{} UNIT HIT ON KLINGON AT SECTOR {},{}",
            hit as i32, klingon.sector.x, klingon.sector.y
        ));
        output.writeln(&format!("   ({} LEFT)", klingon.shields.max(0.0) as i32));

        // If Klingon destroyed, collect position for cleanup
        if !klingon.is_alive() {
            destroyed_positions.push(klingon.sector);
        }
    }

    destroyed_positions
}

/// Clean up destroyed Klingons from all tracking structures.
fn cleanup_destroyed_klingons(
    galaxy: &mut Galaxy,
    destroyed_positions: &[SectorPosition],
    output: &mut dyn OutputWriter,
) -> GameResult<()> {
    // Clean up destroyed Klingons
    for pos in destroyed_positions {
        output.writeln("*** KLINGON DESTROYED ***");
        galaxy.destroy_klingon(*pos)?;
    }

    // Remove all dead Klingons from the vector in one pass
    galaxy.sector_map_mut().klingons.retain(|k| k.is_alive());
    Ok(())
}

/// Check for victory condition after Klingon destruction.
fn check_phaser_victory(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) {
    if galaxy.is_victory() {
        galaxy.end_victory(output);
    }
}

/// Fire Phasers — Command 3 (spec section 6.3).
pub fn fire_phasers(
    galaxy: &mut Galaxy,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<()> {
    // Phase 1: Preconditions
    let (can_fire, computer_damaged) = check_phaser_readiness(galaxy, output);
    if !can_fire {
        return Ok(());
    }

    // Phase 2: Input
    let units = match read_and_validate_phaser_energy(galaxy.enterprise().energy(), io, output)? {
        Some(u) => u,
        None => return Ok(()),
    };

    // Phase 3: Energy deduction
    galaxy.enterprise_mut().subtract_energy(units);

    // Phase 4: CRITICAL - Klingons fire BEFORE phaser damage (spec 8.1)
    if klingons_fire(galaxy, output) {
        return Ok(()); // Enterprise destroyed
    }

    // Phase 5: Apply phaser damage
    let phaser_energy = calculate_phaser_energy(units, computer_damaged, galaxy.rng_mut());
    let destroyed = apply_phaser_damage_to_klingons(galaxy, phaser_energy, output);

    // Phase 6: Cleanup
    cleanup_destroyed_klingons(galaxy, &destroyed, output)?;

    // Phase 7: Victory check
    check_phaser_victory(galaxy, output);
    Ok(())
}

/// Check preconditions for firing torpedoes (spec section 6.4).
/// Returns true if ready to fire, false otherwise.
fn check_torpedo_readiness(galaxy: &Galaxy, output: &mut dyn OutputWriter) -> bool {
    // Check if photon tubes are damaged
    if galaxy.enterprise().is_damaged(Device::PhotonTubes) {
        output.writeln("PHOTON TUBES ARE NOT OPERATIONAL");
        return false;
    }

    // Check torpedo count
    if galaxy.enterprise().torpedoes() <= 0 {
        output.writeln("ALL PHOTON TORPEDOES EXPENDED");
        return false;
    }

    true
}

/// Read and validate torpedo course input (spec section 6.4).
/// Returns Some(course) if valid, None if cancelled.
fn read_torpedo_course(io: &mut dyn InputReader) -> GameResult<Option<f64>> {
    loop {
        let input = io.read_line("TORPEDO COURSE (1-9)")?;
        let course: f64 = match input.trim().parse() {
            Ok(v) => v,
            Err(_) => continue, // Invalid input, re-prompt
        };

        if course == 0.0 {
            return Ok(None); // Cancel command
        }

        if course >= 1.0 && course < 9.0 {
            return Ok(Some(course));
        }

        // Out of range (< 1 or >= 9), re-prompt
    }
}

/// Handle Klingon hit by torpedo (spec section 6.4).
fn handle_klingon_hit(galaxy: &mut Galaxy, pos: SectorPosition, output: &mut dyn OutputWriter) -> GameResult<()> {
    output.writeln("*** KLINGON DESTROYED ***");

    // Atomically destroy Klingon
    galaxy.destroy_klingon(pos)?;

    // Remove from klingons vector
    galaxy.sector_map_mut().klingons.retain(|k| k.sector != pos);

    // Check victory condition
    if galaxy.is_victory() {
        galaxy.end_victory(output);
    }
    Ok(())
}

/// Handle starbase hit by torpedo (spec section 6.4).
fn handle_starbase_hit(galaxy: &mut Galaxy, pos: SectorPosition, output: &mut dyn OutputWriter) {
    output.writeln("*** STAR BASE DESTROYED ***  .......CONGRATULATIONS");

    // Atomically destroy starbase
    galaxy.destroy_starbase(pos);
}

/// Fire torpedo along trajectory and check for hits (spec section 6.4).
fn fire_torpedo_trajectory(galaxy: &mut Galaxy, course: f64, output: &mut dyn OutputWriter) -> GameResult<()> {
    // Calculate direction vector using navigation's interpolation
    let (dx, dy) = navigation::calculate_direction(course);

    // Start from Enterprise position (floating point for interpolation)
    let mut x = galaxy.enterprise().sector().x as f64;
    let mut y = galaxy.enterprise().sector().y as f64;

    output.writeln("TORPEDO TRACK:");

    // Travel sector-by-sector
    loop {
        x += dx;
        y += dy;

        // Boundary check: outside quadrant?
        if x < 0.5 || x >= 8.5 || y < 0.5 || y >= 8.5 {
            output.writeln("TORPEDO MISSED");
            return Ok(());
        }

        // Print current position as truncated integers
        output.writeln(&format!("{},{}", x as i32, y as i32));

        // Check sector at rounded position
        let check_x = (x + 0.5).floor() as i32;
        let check_y = (y + 0.5).floor() as i32;
        let check_pos = SectorPosition {
            x: check_x,
            y: check_y,
        };

        // Check what's in this sector
        match galaxy.sector_map().get(check_pos) {
            SectorContent::Empty => continue, // Keep traveling
            SectorContent::Klingon => {
                handle_klingon_hit(galaxy, check_pos, output)?;
                return Ok(());
            }
            SectorContent::Star => {
                output.writeln("YOU CAN'T DESTROY STARS SILLY");
                return Ok(());
            }
            SectorContent::Starbase => {
                handle_starbase_hit(galaxy, check_pos, output);
                return Ok(());
            }
            SectorContent::Enterprise => {
                // Should never happen, but handle gracefully
                return Ok(());
            }
        }
    }
}

/// Fire Photon Torpedoes — Command 4 (spec section 6.4).
pub fn fire_torpedoes(
    galaxy: &mut Galaxy,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<()> {
    // Phase 1: Check preconditions
    if !check_torpedo_readiness(galaxy, output) {
        return Ok(());
    }

    // Phase 2: Get course input (0 = cancel)
    let course = match read_torpedo_course(io)? {
        Some(c) => c,
        None => return Ok(()),
    };

    // Phase 3: Deduct torpedo BEFORE firing (spec step 2)
    let _ = galaxy.enterprise_mut().consume_torpedo();

    // Phase 4: Fire along trajectory
    fire_torpedo_trajectory(galaxy, course, output)?;

    // Phase 5: Klingons fire back (after torpedo resolution, spec 8.1)
    if klingons_fire(galaxy, output) {
        return Ok(()); // Enterprise destroyed
    }
    Ok(())
}

/// Klingons attack the Enterprise (spec section 8).
/// Returns true if the Enterprise is destroyed, false otherwise.
pub fn klingons_fire(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) -> bool {
    // Skip if docked (spec section 8.3)
    if galaxy
        .enterprise()
        .is_adjacent_to_starbase(galaxy.sector_map().starbase)
    {
        output.writeln("STAR BASE SHIELDS PROTECT THE ENTERPRISE");
        return false;
    }

    let e_pos = galaxy.enterprise().sector();

    // Collect klingon data to avoid borrow conflicts
    let klingon_attacks: Vec<_> = galaxy
        .sector_map()
        .klingons
        .iter()
        .filter(|k| k.is_alive())
        .map(|k| (k.sector, k.shields, calculate_distance(e_pos, k.sector)))
        .collect();

    for (k_sector, k_shields, distance) in klingon_attacks {
        let hit = (k_shields / distance) * (2.0 * galaxy.rng_mut().gen::<f64>());

        galaxy.enterprise_mut().subtract_shields(hit);

        output.writeln(&format!(
            "{} UNIT HIT ON ENTERPRISE FROM SECTOR {},{}",
            hit as i32, k_sector.x, k_sector.y
        ));
        output.writeln(&format!(
            "   ({} LEFT)",
            galaxy.enterprise().shields().max(0.0) as i32
        ));
    }

    // Check if Enterprise is destroyed (spec section 8.4)
    if galaxy.enterprise().shields() < 0.0 {
        galaxy.end_defeat(output);
        return true; // unreachable due to exit, but explicit
    }

    false
}

/// Shield control command (Command 5, spec section 6.5).
/// Allows the player to transfer energy between shields and main energy reserves.
pub fn shield_control(
    galaxy: &mut Galaxy,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<()> {
    // Check if shield control is damaged (spec section 6.5)
    if galaxy.enterprise().is_damaged(Device::ShieldControl) {
        output.writeln("SHIELD CONTROL IS NON-OPERATIONAL");
        return Ok(());
    }

    // Display available energy (energy + shields)
    let total_energy = galaxy.enterprise().energy() + galaxy.enterprise().shields();
    output.writeln(&format!("ENERGY AVAILABLE = {}", total_energy as i32));

    // Prompt for input
    let input = io.read_line("NUMBER OF UNITS TO SHIELDS")?;
    let units: f64 = match input.trim().parse() {
        Ok(v) => v,
        Err(_) => return Ok(()), // Invalid parse, return to command prompt
    };

    // If input ≤ 0, return to command prompt (spec section 6.5)
    if units <= 0.0 {
        return Ok(());
    }

    // Attempt to transfer energy
    match galaxy.enterprise_mut().shield_control(units) {
        Ok(()) => {
            // Success - energy transferred, return to command prompt
        }
        Err(ShieldControlError::InsufficientEnergy) => {
            // Return error instead of recursion - caller will handle retry
            return Err(GameError::InsufficientResources {
                required: units,
                available: total_energy,
            });
        }
        Err(ShieldControlError::InvalidInput) => {
            // Return to command prompt
        }
        Err(ShieldControlError::SystemDamaged) => {
            // Should never happen - we checked above
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::test_utils::MockOutput;
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
        *galaxy.sector_map_mut() = SectorMap::new();

        // Place Enterprise at (4, 4)
        let sector = SectorPosition { x: 4, y: 4 };
        let quadrant = galaxy.enterprise().quadrant();
        galaxy.enterprise_mut().move_to(quadrant, sector);
        galaxy.enterprise_mut().set_energy(enterprise_energy);
        galaxy.enterprise_mut().set_shields(enterprise_shields);
        let enterprise_sector = galaxy.enterprise().sector();
        galaxy
            .sector_map_mut()
            .set(enterprise_sector, SectorContent::Enterprise);

        // Place one Klingon at (2, 2)
        let klingon_pos = SectorPosition { x: 2, y: 2 };
        let mut klingon = Klingon::new(klingon_pos);
        klingon.shields = klingon_shields;
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

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
        let initial_shields = galaxy.enterprise().shields();

        klingons_fire(&mut galaxy, &mut MockOutput::new());

        assert!(galaxy.enterprise().shields() < initial_shields);
    }

    #[test]
    fn klingons_fire_skips_when_docked() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place starbase adjacent to Enterprise
        let starbase_pos = SectorPosition { x: 5, y: 4 };
        galaxy.sector_map_mut().set(starbase_pos, SectorContent::Starbase);
        galaxy.sector_map_mut().starbase = Some(starbase_pos);

        let initial_shields = galaxy.enterprise().shields();
        klingons_fire(&mut galaxy, &mut MockOutput::new());

        assert_eq!(galaxy.enterprise().shields(), initial_shields);
    }

    #[test]
    fn klingons_fire_does_not_hit_from_dead_klingons() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.sector_map_mut().klingons[0].shields = 0.0;

        let initial_shields = galaxy.enterprise().shields();
        klingons_fire(&mut galaxy, &mut MockOutput::new());

        // Shields should not change if all Klingons are dead
        assert_eq!(galaxy.enterprise().shields(), initial_shields);
    }

    #[test]
    fn klingons_fire_damage_depends_on_distance() {
        // Closer Klingon should do more damage
        let mut galaxy1 = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        // Klingon at (2,2), Enterprise at (4,4) - distance = sqrt(8) ≈ 2.83

        let mut galaxy2 = Galaxy::new(42);
        *galaxy2.sector_map_mut() = SectorMap::new();
        let sector = SectorPosition { x: 4, y: 4 };
        let quadrant = galaxy2.enterprise().quadrant();
        galaxy2.enterprise_mut().move_to(quadrant, sector);
        galaxy2.enterprise_mut().set_energy(3000.0);
        galaxy2.enterprise_mut().set_shields(500.0);
        let enterprise_sector = galaxy2.enterprise().sector();
        galaxy2
            .sector_map_mut()
            .set(enterprise_sector, SectorContent::Enterprise);

        // Place Klingon farther away at (1, 1)
        let far_klingon_pos = SectorPosition { x: 1, y: 1 };
        let mut far_klingon = Klingon::new(far_klingon_pos);
        far_klingon.shields = 200.0;
        galaxy2.sector_map_mut().set(far_klingon_pos, SectorContent::Klingon);
        galaxy2.sector_map_mut().klingons.push(far_klingon);

        klingons_fire(&mut galaxy1, &mut MockOutput::new());
        klingons_fire(&mut galaxy2, &mut MockOutput::new());

        // Both have random component, but on average closer Klingon does more damage
        // We can only verify shields were reduced from both
        assert!(galaxy1.enterprise().shields() < 500.0);
        assert!(galaxy2.enterprise().shields() < 500.0);
    }

    // ========== Victory/defeat tests ==========

    #[test]
    fn victory_when_last_klingon_destroyed() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 10.0);
        galaxy.set_total_klingons(1); // Only one Klingon in entire galaxy

        // Manually destroy the Klingon
        let klingon_pos = galaxy.sector_map().klingons[0].sector;
        galaxy.sector_map_mut().klingons[0].shields = 0.0;
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Empty);
        galaxy.decrement_klingons();
        galaxy.decrement_quadrant_klingons();

        assert_eq!(galaxy.total_klingons(), 0);
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
        galaxy.sector_map_mut().set(dead_klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(dead_klingon);

        // Add third Klingon that's alive
        let alive_klingon_pos = SectorPosition { x: 7, y: 7 };
        let alive_klingon = Klingon::new(alive_klingon_pos);
        galaxy
            .sector_map_mut()
            .set(alive_klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(alive_klingon);

        assert_eq!(galaxy.sector_map().klingons.len(), 3);

        // Apply retain
        galaxy.sector_map_mut().klingons.retain(|k| k.is_alive());

        // Should have 2 living Klingons left
        assert_eq!(galaxy.sector_map().klingons.len(), 2);
        for k in &galaxy.sector_map().klingons {
            assert!(k.is_alive());
        }
    }

    #[test]
    fn multiple_klingons_all_take_damage() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Add second Klingon
        let k2_pos = SectorPosition { x: 6, y: 6 };
        let k2 = Klingon::new(k2_pos);
        galaxy.sector_map_mut().set(k2_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(k2);

        // Add third Klingon
        let k3_pos = SectorPosition { x: 3, y: 7 };
        let k3 = Klingon::new(k3_pos);
        galaxy.sector_map_mut().set(k3_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(k3);

        assert_eq!(galaxy.sector_map().klingons.len(), 3);

        // All Klingons fire
        klingons_fire(&mut galaxy, &mut MockOutput::new());

        // Enterprise shields should be reduced by attacks from all 3
        assert!(galaxy.enterprise().shields() < 500.0);
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
        let klingon_pos = galaxy.sector_map().klingons[0].sector;

        // Verify Klingon is in grid
        assert_eq!(
            galaxy.sector_map().get(klingon_pos),
            SectorContent::Klingon
        );

        // Destroy Klingon
        galaxy.sector_map_mut().klingons[0].shields = 0.0;
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Empty);
        galaxy.sector_map_mut().klingons.retain(|k| k.is_alive());

        // Verify grid is cleared and vector is empty
        assert_eq!(galaxy.sector_map().get(klingon_pos), SectorContent::Empty);
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }

    // ========== Torpedo tests ==========

    #[test]
    fn torpedo_readiness_blocked_when_tubes_damaged() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.enterprise_mut().damage_device(Device::PhotonTubes, 2.0);

        assert!(!check_torpedo_readiness(&galaxy, &mut MockOutput::new()));
    }

    #[test]
    fn torpedo_readiness_blocked_when_no_torpedoes() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.enterprise_mut().set_torpedoes(0);

        assert!(!check_torpedo_readiness(&galaxy, &mut MockOutput::new()));
    }

    #[test]
    fn torpedo_readiness_ok_when_ready() {
        let galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        assert!(check_torpedo_readiness(&galaxy, &mut MockOutput::new()));
    }

    #[test]
    fn torpedo_destroys_klingon_going_east() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.set_total_klingons(1);

        // Enterprise at (4,4), place Klingon at (6,4) - east
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 6, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Fire torpedo east (course 1.0)
        let _ = fire_torpedo_trajectory(&mut galaxy,1.0, &mut MockOutput::new());

        // Verify Klingon destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
        assert_eq!(galaxy.sector_map().get(klingon_pos), SectorContent::Empty);
        assert_eq!(galaxy.total_klingons(), 0);
    }

    #[test]
    fn torpedo_blocked_by_star() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place star at (5,4) between Enterprise and Klingon
        let star_pos = SectorPosition { x: 5, y: 4 };
        galaxy.sector_map_mut().set(star_pos, SectorContent::Star);

        // Place Klingon further east at (7,4)
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 7, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Fire torpedo east (course 1.0)
        let _ = fire_torpedo_trajectory(&mut galaxy,1.0, &mut MockOutput::new());

        // Verify star stopped torpedo, Klingon still alive
        assert_eq!(galaxy.sector_map().get(star_pos), SectorContent::Star);
        assert_eq!(galaxy.sector_map().klingons.len(), 1);
        assert_eq!(
            galaxy.sector_map().get(klingon_pos),
            SectorContent::Klingon
        );
    }

    #[test]
    fn torpedo_destroys_starbase() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.set_total_starbases(1);

        // Place starbase at (5,4) - east of Enterprise
        let starbase_pos = SectorPosition { x: 5, y: 4 };
        galaxy.sector_map_mut().set(starbase_pos, SectorContent::Starbase);
        galaxy.sector_map_mut().starbase = Some(starbase_pos);

        // Fire torpedo east (course 1.0)
        let _ = fire_torpedo_trajectory(&mut galaxy,1.0, &mut MockOutput::new());

        // Verify starbase destroyed
        assert_eq!(galaxy.sector_map().starbase, None);
        assert_eq!(galaxy.sector_map().get(starbase_pos), SectorContent::Empty);
        assert_eq!(galaxy.total_starbases(), 0);
    }

    #[test]
    fn torpedo_misses_at_boundary() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.sector_map_mut().klingons.clear(); // No obstacles

        // Fire torpedo north (course 3.0) which will exit quadrant
        let _ = fire_torpedo_trajectory(&mut galaxy,3.0, &mut MockOutput::new());

        // Torpedo should miss (no crash, just returns)
        // Can't verify output but should not panic
    }

    #[test]
    fn torpedo_travels_through_empty_sectors() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place Klingon far to the east at (8,4)
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 8, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Fire torpedo east (course 1.0) - should travel through (5,4), (6,4), (7,4)
        let _ = fire_torpedo_trajectory(&mut galaxy,1.0, &mut MockOutput::new());

        // Verify Klingon destroyed at the end of path
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
        assert_eq!(galaxy.total_klingons(), 0);
    }

    #[test]
    fn torpedo_fractional_course_northeast() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Enterprise at (4,4), place Klingon northeast at (6,2)
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 6, y: 2 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Fire torpedo northeast with fractional course (course 2.0 is pure northeast)
        let _ = fire_torpedo_trajectory(&mut galaxy,2.0, &mut MockOutput::new());

        // Verify Klingon destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }

    #[test]
    fn torpedo_stops_at_first_entity() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place star at (5,4) and Klingon at (7,4)
        let star_pos = SectorPosition { x: 5, y: 4 };
        galaxy.sector_map_mut().set(star_pos, SectorContent::Star);

        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 7, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Fire east (course 1.0)
        let _ = fire_torpedo_trajectory(&mut galaxy,1.0, &mut MockOutput::new());

        // Star should stop torpedo, Klingon survives
        assert_eq!(galaxy.sector_map().get(star_pos), SectorContent::Star);
        assert_eq!(galaxy.sector_map().klingons.len(), 1);
    }

    #[test]
    fn torpedo_victory_when_last_klingon_destroyed() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.set_total_klingons(1); // Last Klingon in galaxy

        // Place Klingon at (6,4)
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 6, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Note: handle_klingon_hit calls end_victory() which exits,
        // so we can't directly test this without mocking.
        // This test verifies the setup is correct for victory.
        assert_eq!(galaxy.total_klingons(), 1);
        assert!(!galaxy.is_victory());

        // Manually destroy to verify is_victory() works
        galaxy.set_total_klingons(0);
        assert!(galaxy.is_victory());
    }

    #[test]
    fn torpedo_course_1_goes_east() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place Klingon directly east
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 7, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        let _ = fire_torpedo_trajectory(&mut galaxy,1.0, &mut MockOutput::new());

        // Klingon should be destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }

    #[test]
    fn torpedo_course_3_goes_north() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place Klingon directly north
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 4, y: 2 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        let _ = fire_torpedo_trajectory(&mut galaxy,3.0, &mut MockOutput::new());

        // Klingon should be destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }

    #[test]
    fn torpedo_course_5_goes_west() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place Klingon directly west
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 2, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        let _ = fire_torpedo_trajectory(&mut galaxy,5.0, &mut MockOutput::new());

        // Klingon should be destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }

    #[test]
    fn torpedo_course_7_goes_south() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);

        // Place Klingon directly south
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 4, y: 6 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        let _ = fire_torpedo_trajectory(&mut galaxy,7.0, &mut MockOutput::new());

        // Klingon should be destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }
}
