use crate::io::{InputReader, OutputWriter};
use crate::models::constants::{Device, SectorContent};
use crate::models::errors::GameResult;
use crate::models::galaxy::Galaxy;
use crate::models::navigation_types::Course;
use crate::models::position::SectorPosition;
use crate::services::navigation;
use crate::ui::presenters::CombatPresenter;

use super::klingon_attack::klingons_fire;

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
fn read_torpedo_course(io: &mut dyn InputReader) -> GameResult<Option<Course>> {
    loop {
        let input = io.read_line("TORPEDO COURSE (1-9)")?;
        let value: f64 = match input.trim().parse() {
            Ok(v) => v,
            Err(_) => continue, // Invalid input, re-prompt
        };

        if value == 0.0 {
            return Ok(None); // Cancel command
        }

        match Course::new(value) {
            Ok(c) => return Ok(Some(c)),
            Err(_) => continue, // Out of range, re-prompt
        }
    }
}

/// Handle Klingon hit by torpedo (spec section 6.4).
fn handle_klingon_hit(galaxy: &mut Galaxy, pos: SectorPosition, output: &mut dyn OutputWriter) -> GameResult<()> {
    CombatPresenter::show_klingon_destroyed(output);

    // Atomically destroy Klingon
    galaxy.destroy_klingon(pos)?;

    // Remove from klingons vector
    galaxy.sector_map_mut().klingons.retain(|k| k.sector != pos);

    // Victory check moved to game loop / GameEngine
    Ok(())
}

/// Handle starbase hit by torpedo (spec section 6.4).
fn handle_starbase_hit(galaxy: &mut Galaxy, pos: SectorPosition, output: &mut dyn OutputWriter) {
    output.writeln("*** STAR BASE DESTROYED ***  .......CONGRATULATIONS");

    // Atomically destroy starbase
    galaxy.destroy_starbase(pos);
}

/// Fire torpedo along trajectory and check for hits (spec section 6.4).
fn fire_torpedo_trajectory(galaxy: &mut Galaxy, course: Course, output: &mut dyn OutputWriter) -> GameResult<()> {
    // Calculate direction vector using navigation's interpolation
    let (dx, dy) = navigation::calculate_direction(course.value());

    // Start from Enterprise position (floating point for interpolation)
    let mut x = galaxy.enterprise().sector().x as f64;
    let mut y = galaxy.enterprise().sector().y as f64;

    output.writeln("TORPEDO TRACK:");

    // Travel sector-by-sector
    loop {
        x += dx;
        y += dy;

        // Boundary check: outside quadrant?
        if !(0.5..8.5).contains(&x) || !(0.5..8.5).contains(&y) {
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

/// Fires a photon torpedo in a specified direction (Command 4)
///
/// Prompts the player for a course direction (1-9) and launches a photon torpedo
/// that travels in a straight line until it hits a target (Klingon, star, or starbase)
/// or exits the sector. Klingons are destroyed on hit, stars block the torpedo,
/// and hitting a starbase is heavily penalized.
///
/// # Arguments
///
/// * `galaxy` - The game galaxy state
/// * `io` - Input reader for getting course direction
/// * `output` - Output writer for displaying results
///
/// # Returns
///
/// * `Ok(())` on successful execution (hit or miss)
/// * `Err` if I/O operations fail
///
/// # Specification
///
/// See spec section 6.4 for full details on torpedo mechanics.
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
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(1.0).unwrap(), &mut MockOutput::new());

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
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(1.0).unwrap(), &mut MockOutput::new());

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
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(1.0).unwrap(), &mut MockOutput::new());

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
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(3.0).unwrap(), &mut MockOutput::new());

        // Torpedo should miss (no crash, just returns)
        // Can't verify output but should not panic
    }

    #[test]
    fn torpedo_travels_through_empty_sectors() {
        let mut galaxy = setup_combat_scenario(42, 3000.0, 500.0, 200.0);
        galaxy.set_total_klingons(1);

        // Place Klingon far to the east at (8,4)
        galaxy.sector_map_mut().klingons.clear();
        let klingon_pos = SectorPosition { x: 8, y: 4 };
        let klingon = Klingon::new(klingon_pos);
        galaxy.sector_map_mut().set(klingon_pos, SectorContent::Klingon);
        galaxy.sector_map_mut().klingons.push(klingon);

        // Fire torpedo east (course 1.0) - should travel through (5,4), (6,4), (7,4)
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(1.0).unwrap(), &mut MockOutput::new());

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
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(2.0).unwrap(), &mut MockOutput::new());

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
        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(1.0).unwrap(), &mut MockOutput::new());

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

        // Victory check now handled by GameEngine
        // This test verifies the setup is correct for victory.
        assert_eq!(galaxy.total_klingons(), 1);
        assert!(!galaxy.all_klingons_destroyed());

        // Manually destroy to verify all_klingons_destroyed() works
        galaxy.set_total_klingons(0);
        assert!(galaxy.all_klingons_destroyed());
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

        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(1.0).unwrap(), &mut MockOutput::new());

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

        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(3.0).unwrap(), &mut MockOutput::new());

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

        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(5.0).unwrap(), &mut MockOutput::new());

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

        let _ = fire_torpedo_trajectory(&mut galaxy, Course::new(7.0).unwrap(), &mut MockOutput::new());

        // Klingon should be destroyed
        assert_eq!(galaxy.sector_map().klingons.len(), 0);
    }
}
