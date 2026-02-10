use crate::io::{InputReader, OutputWriter};
use crate::models::constants::{Device, SectorContent};
use crate::models::errors::GameResult;
use crate::models::galaxy::Galaxy;
use crate::models::navigation_types::{Course, WarpFactor};
use crate::models::position::SectorPosition;
use crate::services::combat;

use super::course::{calculate_direction, calculate_quadrant_crossing};
use super::damage::{auto_repair_devices, random_damage_event};

/// Engages warp engines to move the Enterprise (Command 0)
///
/// Prompts the player for a course direction (1-9) and warp factor (0-8).
/// The Enterprise travels at the specified warp speed in the given direction,
/// consuming energy and advancing stardate. Random device damage may occur
/// during warp travel. Blocked movement (hitting objects) consumes partial energy.
///
/// # Arguments
///
/// * `galaxy` - The game galaxy state
/// * `io` - Input reader for getting course and warp factor
/// * `output` - Output writer for displaying navigation results
///
/// # Returns
///
/// * `Ok(())` on successful navigation (complete or blocked)
/// * `Err` if I/O operations fail
///
/// # Specification
///
/// See spec section 5.1 for full details on navigation mechanics.
pub fn navigate(
    galaxy: &mut Galaxy,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<()> {
    let (course, warp_factor) = match read_course_and_warp(galaxy, io, output)? {
        Some(values) => values,
        None => return Ok(()),
    };

    // If Klingons present, they fire before warp move (spec section 8.1)
    if !galaxy.sector_map().klingons.is_empty()
        && combat::klingons_fire(galaxy, output)
    {
        return Ok(()); // Enterprise destroyed, game ended
    }

    // Energy/shields check (no-Klingons path, spec section 10.4)
    if galaxy.enterprise().energy() <= 0.0 {
        if galaxy.enterprise().shields() < 1.0 {
            output.writeln("THE ENTERPRISE IS DEAD IN SPACE. IF YOU SURVIVE ALL IMPENDING");
            output.writeln("ATTACK YOU WILL BE DEMOTED TO THE RANK OF PRIVATE");

            // Klingons fire repeatedly until Enterprise destroyed or survives (spec 10.4)
            combat::dead_in_space_loop(galaxy, output);
            return Ok(()); // Game ended (either destroyed or demoted)
        } else {
            output.writeln(&format!(
                "YOU HAVE {} UNITS OF ENERGY",
                galaxy.enterprise().energy() as i32
            ));
            output.writeln(&format!(
                "SUGGEST YOU GET SOME FROM YOUR SHIELDS WHICH HAVE {} UNITS LEFT",
                galaxy.enterprise().shields() as i32
            ));
            return Ok(()); // Prevent movement
        }
    }

    execute_move(galaxy, course, warp_factor, output);
    Ok(())
}

/// Prompt the player for course and warp factor. Returns None if the player
/// cancels (course 0) or input is invalid in a way that aborts navigation.
fn read_course_and_warp(
    galaxy: &Galaxy,
    io: &mut dyn InputReader,
    output: &mut dyn OutputWriter,
) -> GameResult<Option<(Course, WarpFactor)>> {
    // Course input loop
    let course: Course = loop {
        let input = io.read_line("COURSE (1-9)")?;
        let value: f64 = match input.trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if value == 0.0 {
            return Ok(None);
        }
        match Course::new(value) {
            Ok(c) => break c,
            Err(_) => continue, // Invalid range — re-prompt
        }
    };

    // Warp factor input
    let input = io.read_line("WARP FACTOR (0-8)")?;
    let warp_value: f64 = match input.trim().parse() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let warp_factor = match WarpFactor::new(warp_value) {
        Ok(w) => w,
        Err(_) => return Ok(None),
    };

    // Check for damaged warp engines
    if galaxy.enterprise().is_damaged(Device::WarpEngines) && warp_factor.value() > 0.2 {
        output.writeln("WARP ENGINES ARE DAMAGED, MAXIMUM SPEED = WARP .2");
        return Ok(None);
    }

    Ok(Some((course, warp_factor)))
}

/// Execute the warp move: step through sectors, handle collisions and
/// quadrant boundary crossings, update energy and stardate.
fn execute_move(galaxy: &mut Galaxy, course: Course, warp_factor: WarpFactor, output: &mut dyn OutputWriter) {
    let (dx, dy) = calculate_direction(course.value());
    let n = (warp_factor.value() * 8.0).floor() as i32;

    if n == 0 {
        return;
    }

    let old_sector = galaxy.enterprise().sector();
    let old_quadrant = galaxy.enterprise().quadrant();

    let mut sx = galaxy.enterprise().sector().x as f64;
    let mut sy = galaxy.enterprise().sector().y as f64;
    let mut crossed_boundary = false;

    // Remove Enterprise from current position before moving
    galaxy
        .sector_map_mut()
        .set(old_sector, SectorContent::Empty);

    for _ in 0..n {
        sx += dx;
        sy += dy;

        // Boundary check: leaving the quadrant?
        if !(0.5..8.5).contains(&sx) || !(0.5..8.5).contains(&sy) {
            crossed_boundary = true;
            break;
        }

        // Collision check: is the next sector occupied?
        let check_x = (sx + 0.5).floor() as i32;
        let check_y = (sy + 0.5).floor() as i32;
        let check_pos = SectorPosition {
            x: check_x,
            y: check_y,
        };
        if galaxy.sector_map().get(check_pos) != SectorContent::Empty {
            // Back up one step
            sx -= dx;
            sy -= dy;
            let stop_x = (sx + 0.5).floor() as i32;
            let stop_y = (sy + 0.5).floor() as i32;
            output.writeln(&format!(
                "WARP ENGINES SHUTDOWN AT SECTOR {},{} DUE TO BAD NAVIGATION",
                stop_x, stop_y
            ));
            break;
        }
    }

    if crossed_boundary {
        // Quadrant boundary crossing
        let (new_quadrant, new_sector) = calculate_quadrant_crossing(
            old_quadrant.x,
            old_quadrant.y,
            old_sector.x,
            old_sector.y,
            dx,
            dy,
            n,
        );

        galaxy.enterprise_mut().move_to(new_quadrant, new_sector);
        galaxy.enter_quadrant();

        // Record the new quadrant to computer memory
        galaxy.record_quadrant_to_memory(
            galaxy.enterprise().quadrant().x,
            galaxy.enterprise().quadrant().y,
        );

        // Boundary crossing always advances stardate by 1
        galaxy.advance_time(1.0);
        check_time_limit(galaxy, output);
    } else {
        // Intra-quadrant move: update sector map
        let final_x = (sx + 0.5).floor() as i32;
        let final_y = (sy + 0.5).floor() as i32;
        let new_sector = SectorPosition {
            x: final_x,
            y: final_y,
        };

        let quadrant = galaxy.enterprise().quadrant();
        galaxy
            .sector_map_mut()
            .set(new_sector, SectorContent::Enterprise);
        galaxy.enterprise_mut().move_to(quadrant, new_sector);

        // Advance stardate only for warp >= 1
        if warp_factor.is_warp() {
            galaxy.advance_time(1.0);
            check_time_limit(galaxy, output);
        }
    }

    // Energy cost: N - 5 (short moves can gain energy)
    let cost = (n - 5) as f64;
    if cost > 0.0 {
        galaxy.enterprise_mut().subtract_energy(cost);
    } else {
        galaxy.enterprise_mut().add_energy(-cost);
    }

    // Automatic repair (spec section 5.2)
    auto_repair_devices(galaxy);

    // Random damage/repair events - 20% chance (spec section 5.3)
    random_damage_event(galaxy, output);
}

/// Check if the time limit has been exceeded (spec section 10.3).
/// Time expiration is now checked by GameEngine.
fn check_time_limit(_galaxy: &Galaxy, _output: &mut dyn OutputWriter) {
    // Time limit check moved to GameEngine
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::test_utils::MockOutput;
    use crate::models::galaxy::Galaxy;
    use crate::models::position::QuadrantPosition;

    // --- Energy cost tests ---

    #[test]
    fn energy_cost_warp_1() {
        let mut galaxy = Galaxy::new(42);
        let initial_energy = galaxy.enterprise().energy();
        // Place Enterprise somewhere safe with clear path
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 4);

        // Warp 1.0 → n=8, cost = 8-5 = 3
        execute_move(&mut galaxy, Course::new(3.0).unwrap(), WarpFactor::new(1.0).unwrap(), &mut MockOutput::new());
        let expected = initial_energy - 3.0;
        assert!(
            (galaxy.enterprise().energy() - expected).abs() < 1e-10,
            "warp 1.0: expected energy {}, got {}",
            expected,
            galaxy.enterprise().energy(),
        );
    }

    #[test]
    fn energy_cost_warp_half_gains_energy() {
        let mut galaxy = Galaxy::new(42);
        let initial_energy = galaxy.enterprise().energy();
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 4);

        // Warp 0.5 → n=4, cost = 4-5 = -1 → gains 1 energy
        execute_move(&mut galaxy, Course::new(3.0).unwrap(), WarpFactor::new(0.5).unwrap(), &mut MockOutput::new());
        let expected = initial_energy + 1.0;
        assert!(
            (galaxy.enterprise().energy() - expected).abs() < 1e-10,
            "warp 0.5: expected energy {}, got {}",
            expected,
            galaxy.enterprise().energy(),
        );
    }

    #[test]
    fn energy_cost_warp_8() {
        let mut galaxy = Galaxy::new(42);
        let initial_energy = galaxy.enterprise().energy();
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 4);

        // Warp 8.0 → n=64, cost = 64-5 = 59
        // Will cross boundary, but energy cost still applies
        execute_move(&mut galaxy, Course::new(3.0).unwrap(), WarpFactor::new(8.0).unwrap(), &mut MockOutput::new());
        let expected = initial_energy - 59.0;
        assert!(
            (galaxy.enterprise().energy() - expected).abs() < 1e-10,
            "warp 8.0: expected energy {}, got {}",
            expected,
            galaxy.enterprise().energy(),
        );
    }

    // --- Time advancement tests ---

    #[test]
    fn time_advances_at_warp_1() {
        let mut galaxy = Galaxy::new(42);
        let initial_stardate = galaxy.stardate();
        place_enterprise_for_test(&mut galaxy, 4, 4, 1, 4);

        // Course 1 (east), warp 1.0 — will cross quadrant boundary (8 steps from sector 1)
        // Boundary crossing always advances stardate
        execute_move(&mut galaxy, Course::new(1.0).unwrap(), WarpFactor::new(1.0).unwrap(), &mut MockOutput::new());
        assert!(
            galaxy.stardate() > initial_stardate,
            "stardate should advance at warp >= 1.0",
        );
    }

    #[test]
    fn time_unchanged_sub_warp_no_crossing() {
        let mut galaxy = Galaxy::new(42);
        let initial_stardate = galaxy.stardate();
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 4);

        // Course 3 (north), warp 0.25 → n=2 steps, stays in quadrant
        execute_move(&mut galaxy, Course::new(3.0).unwrap(), WarpFactor::new(0.25).unwrap(), &mut MockOutput::new());
        assert!(
            (galaxy.stardate() - initial_stardate).abs() < 1e-10,
            "stardate should not advance for sub-warp without crossing",
        );
    }

    // --- Intra-quadrant movement tests ---

    #[test]
    fn move_east_within_quadrant() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 2, 4);

        // Course 1 (east), warp 0.25 → n=2 steps
        execute_move(&mut galaxy, Course::new(1.0).unwrap(), WarpFactor::new(0.25).unwrap(), &mut MockOutput::new());
        assert_eq!(galaxy.enterprise().sector().x, 4);
        assert_eq!(galaxy.enterprise().sector().y, 4);
    }

    #[test]
    fn move_north_within_quadrant() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 6);

        // Course 3 (north, dy=-1), warp 0.375 → n=3 steps
        execute_move(&mut galaxy, Course::new(3.0).unwrap(), WarpFactor::new(0.375).unwrap(), &mut MockOutput::new());
        assert_eq!(galaxy.enterprise().sector().x, 4);
        assert_eq!(galaxy.enterprise().sector().y, 3);
    }

    #[test]
    fn move_south_within_quadrant() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 2);

        // Course 7 (south, dy=+1), warp 0.25 → n=2 steps
        execute_move(&mut galaxy, Course::new(7.0).unwrap(), WarpFactor::new(0.25).unwrap(), &mut MockOutput::new());
        assert_eq!(galaxy.enterprise().sector().x, 4);
        assert_eq!(galaxy.enterprise().sector().y, 4);
    }

    // --- Collision detection test ---

    #[test]
    fn collision_stops_before_occupied_sector() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 1, 4);

        // Place a star at sector (4, 4)
        galaxy
            .sector_map_mut()
            .set(SectorPosition { x: 4, y: 4 }, SectorContent::Star);

        // Course 1 (east), warp 0.5 → n=4 steps from sector (1,4)
        // Should stop at (3,4) — one before the star
        execute_move(&mut galaxy, Course::new(1.0).unwrap(), WarpFactor::new(0.5).unwrap(), &mut MockOutput::new());
        assert_eq!(galaxy.enterprise().sector().x, 3);
        assert_eq!(galaxy.enterprise().sector().y, 4);
    }

    // --- Quadrant boundary crossing integration test ---

    #[test]
    fn crosses_quadrant_boundary_east() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 7, 4);

        let initial_quad_x = galaxy.enterprise().quadrant().x;

        // Course 1 (east), warp 0.5 → n=4 steps from sector 7
        // Steps: 8 (boundary check: 8 < 8.5 is false at >= 8.5), so step 2 → sx=9 → crosses
        execute_move(&mut galaxy, Course::new(1.0).unwrap(), WarpFactor::new(0.5).unwrap(), &mut MockOutput::new());

        // Should have crossed into a new quadrant
        assert_ne!(
            galaxy.enterprise().quadrant().x, initial_quad_x,
            "should have crossed to a new quadrant"
        );
    }

    // --- Helper ---

    /// Place the Enterprise at a specific position, clearing the sector map
    /// around it for clean test setup.
    fn place_enterprise_for_test(
        galaxy: &mut Galaxy,
        quad_x: i32,
        quad_y: i32,
        sect_x: i32,
        sect_y: i32,
    ) {
        // Clear old Enterprise position
        let old_sector = galaxy.enterprise().sector();
        galaxy
            .sector_map_mut()
            .set(old_sector, SectorContent::Empty);

        let quadrant = QuadrantPosition {
            x: quad_x,
            y: quad_y,
        };
        let sector = SectorPosition {
            x: sect_x,
            y: sect_y,
        };
        galaxy.enterprise_mut().move_to(quadrant, sector);

        // Clear the sector map and place Enterprise
        *galaxy.sector_map_mut() = crate::models::sector_map::SectorMap::new();
        let new_sector = galaxy.enterprise().sector();
        galaxy
            .sector_map_mut()
            .set(new_sector, SectorContent::Enterprise);
    }
}
