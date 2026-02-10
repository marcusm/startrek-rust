use rand::Rng;

use crate::io::{InputReader, OutputWriter};
use crate::models::constants::Device;
use crate::models::errors::GameResult;
use crate::models::galaxy::Galaxy;
use crate::models::klingon::Klingon;
use crate::models::position::SectorPosition;
use crate::ui::presenters::CombatPresenter;

use super::klingon_attack::klingons_fire;

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

        CombatPresenter::show_klingon_hit(hit, klingon.sector, klingon.shields, output);

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
        CombatPresenter::show_klingon_destroyed(output);
        galaxy.destroy_klingon(*pos)?;
    }

    // Remove all dead Klingons from the vector in one pass
    galaxy.sector_map_mut().klingons.retain(|k| k.is_alive());
    Ok(())
}

/// Check for victory condition after Klingon destruction.
/// This is a no-op now - game loop will check victory condition.
fn check_phaser_victory(_galaxy: &mut Galaxy, _output: &mut dyn OutputWriter) {
    // Victory check moved to game loop / GameEngine
}

/// Fires phasers at all Klingons in the current sector (Command 3)
///
/// Prompts the player for phaser energy units to fire. Energy is distributed
/// among all Klingons based on distance, with closer targets receiving more damage.
/// Computer damage reduces phaser accuracy. Klingons fire back before phaser damage
/// is applied per spec 8.1.
///
/// # Arguments
///
/// * `galaxy` - The game galaxy state
/// * `io` - Input reader for getting player choices
/// * `output` - Output writer for displaying results
///
/// # Returns
///
/// * `Ok(())` on successful execution
/// * `Err` if I/O operations fail
///
/// # Specification
///
/// See spec section 6.3 for full details on phaser combat mechanics.
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

    #[test]
    fn distance_symmetry() {
        let p1 = SectorPosition { x: 2, y: 3 };
        let p2 = SectorPosition { x: 6, y: 8 };
        assert_eq!(calculate_distance(p1, p2), calculate_distance(p2, p1));
    }
}
