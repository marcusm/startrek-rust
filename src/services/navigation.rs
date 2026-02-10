use crate::io::{InputReader, OutputWriter};
use crate::models::constants::{COURSE_VECTORS, Device, SectorContent};
use crate::models::errors::GameResult;
use crate::models::galaxy::Galaxy;
use crate::models::position::{QuadrantPosition, SectorPosition};
use crate::services::combat;

/// Warp Engine Control — Command 0 (spec section 5.1).
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
    if !galaxy.sector_map().klingons.is_empty() {
        if combat::klingons_fire(galaxy, output) {
            return Ok(()); // Enterprise destroyed, game ended
        }
    }

    // Energy/shields check (no-Klingons path, spec section 10.4)
    if galaxy.enterprise().energy() <= 0.0 {
        if galaxy.enterprise().shields() < 1.0 {
            output.writeln("THE ENTERPRISE IS DEAD IN SPACE. IF YOU SURVIVE ALL IMPENDING");
            output.writeln("ATTACK YOU WILL BE DEMOTED TO THE RANK OF PRIVATE");

            // Klingons fire repeatedly until Enterprise destroyed or survives (spec 10.4)
            dead_in_space_loop(galaxy, output);
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
) -> GameResult<Option<(f64, f64)>> {
    // Course input loop
    let course: f64 = loop {
        let input = io.read_line("COURSE (1-9)")?;
        let value: f64 = match input.trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        if value == 0.0 {
            return Ok(None);
        }
        if value >= 1.0 && value < 9.0 {
            break value;
        }
        // Invalid range — re-prompt
    };

    // Warp factor input
    let input = io.read_line("WARP FACTOR (0-8)")?;
    let warp_factor: f64 = match input.trim().parse() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if warp_factor < 0.0 || warp_factor > 8.0 {
        return Ok(None);
    }

    // Check for damaged warp engines
    if galaxy.enterprise().is_damaged(Device::WarpEngines) && warp_factor > 0.2 {
        output.writeln("WARP ENGINES ARE DAMAGED, MAXIMUM SPEED = WARP .2");
        return Ok(None);
    }

    Ok(Some((course, warp_factor)))
}

/// Calculate the direction vector for a given course value (1.0 ..< 9.0).
/// Uses linear interpolation between adjacent integer course vectors.
pub fn calculate_direction(course: f64) -> (f64, f64) {
    let r = course.floor() as usize;
    let frac = course - course.floor();
    let dx = COURSE_VECTORS[r].0 + (COURSE_VECTORS[r + 1].0 - COURSE_VECTORS[r].0) * frac;
    let dy = COURSE_VECTORS[r].1 + (COURSE_VECTORS[r + 1].1 - COURSE_VECTORS[r].1) * frac;
    (dx, dy)
}

/// Calculate the new quadrant and sector position after a quadrant boundary
/// crossing. Uses absolute galactic coordinates with sector-zero correction
/// and galaxy-edge clamping.
pub fn calculate_quadrant_crossing(
    quad_x: i32,
    quad_y: i32,
    sect_x: i32,
    sect_y: i32,
    dx: f64,
    dy: f64,
    n: i32,
) -> (QuadrantPosition, SectorPosition) {
    let abs_x = quad_x as f64 * 8.0 + sect_x as f64 + dx * n as f64;
    let abs_y = quad_y as f64 * 8.0 + sect_y as f64 + dy * n as f64;

    let mut new_quad_x = (abs_x / 8.0).floor() as i32;
    let mut new_quad_y = (abs_y / 8.0).floor() as i32;
    let mut new_sect_x = (abs_x - new_quad_x as f64 * 8.0 + 0.5).floor() as i32;
    let mut new_sect_y = (abs_y - new_quad_y as f64 * 8.0 + 0.5).floor() as i32;

    // Sector-zero correction
    if new_sect_x == 0 {
        new_quad_x -= 1;
        new_sect_x = 8;
    }
    if new_sect_y == 0 {
        new_quad_y -= 1;
        new_sect_y = 8;
    }

    // Clamp quadrant to galaxy boundaries (1-8)
    new_quad_x = new_quad_x.clamp(1, 8);
    new_quad_y = new_quad_y.clamp(1, 8);

    // Clamp sector to valid range (1-8) in case of edge effects
    new_sect_x = new_sect_x.clamp(1, 8);
    new_sect_y = new_sect_y.clamp(1, 8);

    (
        QuadrantPosition {
            x: new_quad_x,
            y: new_quad_y,
        },
        SectorPosition {
            x: new_sect_x,
            y: new_sect_y,
        },
    )
}

/// Execute the warp move: step through sectors, handle collisions and
/// quadrant boundary crossings, update energy and stardate.
fn execute_move(galaxy: &mut Galaxy, course: f64, warp_factor: f64, output: &mut dyn OutputWriter) {
    let (dx, dy) = calculate_direction(course);
    let n = (warp_factor * 8.0).floor() as i32;

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
        if sx < 0.5 || sx >= 8.5 || sy < 0.5 || sy >= 8.5 {
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
        if warp_factor >= 1.0 {
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

/// Check if the time limit has been exceeded and end the game if so (spec section 10.3).
fn check_time_limit(galaxy: &Galaxy, output: &mut dyn OutputWriter) {
    let time_limit = galaxy.starting_stardate() + galaxy.mission_duration();
    if galaxy.stardate() > time_limit {
        output.writeln("");
        output.writeln(&format!("IT IS STARDATE {}", galaxy.stardate() as i32));
        output.writeln(&format!(
            "THERE ARE STILL {} KLINGON BATTLE CRUISERS",
            galaxy.total_klingons()
        ));
        std::process::exit(0);
    }
}

/// Automatic device repair on navigation moves (spec section 5.2).
/// Each damaged device (value < 0) is incremented by 1.
fn auto_repair_devices(galaxy: &mut Galaxy) {
    use crate::models::constants::Device;
    for device in Device::ALL.iter() {
        if galaxy.enterprise().is_damaged(*device) {
            galaxy.enterprise_mut().repair_device(*device, 1.0);
        }
    }
}

/// Random damage/repair events on navigation moves (spec section 5.3).
/// 20% chance of event affecting a random device.
/// FIXED: Now uses galaxy.rng instead of thread_rng() for determinism
fn random_damage_event(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) {
    use rand::Rng;

    // 20% chance of event - FIXED: using galaxy.rng for determinism!
    if galaxy.rng_mut().gen::<f64>() > 0.2 {
        return;
    }

    // Select random device (0-7 index)
    let device_index = (galaxy.rng_mut().gen::<f64>() * 8.0).floor() as usize;

    // Determine severity (1-5)
    let severity = (galaxy.rng_mut().gen::<f64>() * 5.0).floor() + 1.0;

    // 50% chance of damage vs repair
    let is_repair = galaxy.rng_mut().gen::<f64>() >= 0.5;

    let device = Device::ALL[device_index];

    output.writeln("");
    if is_repair {
        galaxy.enterprise_mut().repair_device(device, severity);
        output.writeln(&format!(
            "DAMAGE CONTROL REPORT: {} STATE OF REPAIR IMPROVED",
            device.name()
        ));
    } else {
        galaxy.enterprise_mut().damage_device(device, severity);
        output.writeln(&format!("DAMAGE CONTROL REPORT: {} DAMAGED", device.name()));
    }
    output.writeln("");
}


/// Handle the dead-in-space scenario where Klingons fire repeatedly (spec 10.4).
/// The Enterprise is stuck with no energy and minimal shields. All Klingons in the
/// quadrant fire until either the Enterprise is destroyed or miraculously survives.
fn dead_in_space_loop(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) {
    loop {
        // Check if there are any Klingons left to fire
        if galaxy.sector_map().klingons.is_empty() {
            // No Klingons to fire - Enterprise survives, demoted to private
            output.writeln("");
            output.writeln(&format!(
                "THERE ARE STILL {} KLINGON BATTLE CRUISERS",
                galaxy.total_klingons()
            ));
            std::process::exit(0);
        }

        // Klingons fire (uses existing combat::klingons_fire function)
        // This function returns true if Enterprise is destroyed (shields < 0)
        if combat::klingons_fire(galaxy, output) {
            return; // Enterprise destroyed, end_defeat() already called
        }

        // If we reach here, shields are still >= 0 despite the attack
        // In practice, this is extremely unlikely with shields < 1
        // But the spec says "fire repeatedly until" so we continue the loop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::test_utils::MockOutput;
    use crate::models::galaxy::Galaxy;

    // --- Direction vector tests ---

    #[test]
    fn direction_integer_courses() {
        let cases = [
            (1.0, (1.0, 0.0)),   // east
            (2.0, (1.0, -1.0)),  // northeast
            (3.0, (0.0, -1.0)),  // north
            (4.0, (-1.0, -1.0)), // northwest
            (5.0, (-1.0, 0.0)),  // west
            (6.0, (-1.0, 1.0)),  // southwest
            (7.0, (0.0, 1.0)),   // south
            (8.0, (1.0, 1.0)),   // southeast
        ];
        for (course, (expected_dx, expected_dy)) in &cases {
            let (dx, dy) = calculate_direction(*course);
            assert!(
                (dx - expected_dx).abs() < 1e-10 && (dy - expected_dy).abs() < 1e-10,
                "course {} expected ({}, {}), got ({}, {})",
                course,
                expected_dx,
                expected_dy,
                dx,
                dy,
            );
        }
    }

    #[test]
    fn direction_fractional_interpolation() {
        // Course 1.5: midpoint between course 1 (1,0) and course 2 (1,-1) → (1.0, -0.5)
        let (dx, dy) = calculate_direction(1.5);
        assert!((dx - 1.0).abs() < 1e-10);
        assert!((dy - (-0.5)).abs() < 1e-10);

        // Course 4.5: midpoint between course 4 (-1,-1) and course 5 (-1,0) → (-1.0, -0.5)
        let (dx, dy) = calculate_direction(4.5);
        assert!((dx - (-1.0)).abs() < 1e-10);
        assert!((dy - (-0.5)).abs() < 1e-10);
    }

    // --- Quadrant crossing tests ---

    #[test]
    fn quadrant_crossing_basic_east() {
        // Quadrant (1,1), sector (8,4), moving east (dx=1, dy=0), 8 steps
        let (quad, sect) = calculate_quadrant_crossing(1, 1, 8, 4, 1.0, 0.0, 8);
        assert_eq!(quad.x, 2, "should move to quadrant 2");
        assert_eq!(quad.y, 1, "y quadrant unchanged");
        // abs_x = 1*8 + 8 + 1*8 = 24, new_quad_x = floor(24/8) = 3,
        // new_sect_x = floor(24 - 3*8 + 0.5) = floor(0.5) = 0 → sector-zero correction
        // → quad_x = 2, sect_x = 8
        assert_eq!(quad.x, 2);
        assert_eq!(sect.x, 8);
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_west() {
        // Quadrant (1,4), sector (1,4), moving west (dx=-1, dy=0), 8 steps
        // abs_x = 1*8 + 1 + (-1)*8 = 1, new_quad_x = floor(1/8) = 0 → clamp to 1
        let (quad, _sect) = calculate_quadrant_crossing(1, 4, 1, 4, -1.0, 0.0, 8);
        assert_eq!(quad.x, 1, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_north() {
        // Quadrant (4,1), sector (4,1), moving north (dx=0, dy=-1), 8 steps
        let (quad, _sect) = calculate_quadrant_crossing(4, 1, 4, 1, 0.0, -1.0, 8);
        assert_eq!(quad.y, 1, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_east() {
        // Quadrant (8,4), sector (8,4), moving east (dx=1, dy=0), 8 steps
        let (quad, _sect) = calculate_quadrant_crossing(8, 4, 8, 4, 1.0, 0.0, 8);
        assert_eq!(quad.x, 8, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_galaxy_edge_south() {
        // Quadrant (4,8), sector (4,8), moving south (dx=0, dy=1), 8 steps
        let (quad, _sect) = calculate_quadrant_crossing(4, 8, 4, 8, 0.0, 1.0, 8);
        assert_eq!(quad.y, 8, "should clamp to galaxy edge");
    }

    #[test]
    fn quadrant_crossing_sector_zero_correction() {
        // Set up a scenario where abs coordinate gives sector = 0.
        // Quadrant (2,2), sector (8,8), moving east 8 steps:
        // abs_x = 2*8 + 8 + 1*8 = 32, new_quad_x = floor(32/8) = 4
        // new_sect_x = floor(32 - 4*8 + 0.5) = floor(0.5) = 0
        // Correction: quad_x = 3, sect_x = 8
        let (quad, sect) = calculate_quadrant_crossing(2, 2, 8, 8, 1.0, 0.0, 8);
        assert_eq!(quad.x, 3);
        assert_eq!(sect.x, 8);
    }

    // --- Energy cost tests ---

    #[test]
    fn energy_cost_warp_1() {
        let mut galaxy = Galaxy::new(42);
        let initial_energy = galaxy.enterprise().energy();
        // Place Enterprise somewhere safe with clear path
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 4);

        // Warp 1.0 → n=8, cost = 8-5 = 3
        execute_move(&mut galaxy,3.0, 1.0, &mut MockOutput::new());
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
        execute_move(&mut galaxy,3.0, 0.5, &mut MockOutput::new());
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
        execute_move(&mut galaxy,3.0, 8.0, &mut MockOutput::new());
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
        execute_move(&mut galaxy,1.0, 1.0, &mut MockOutput::new());
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
        execute_move(&mut galaxy,3.0, 0.25, &mut MockOutput::new());
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
        execute_move(&mut galaxy,1.0, 0.25, &mut MockOutput::new());
        assert_eq!(galaxy.enterprise().sector().x, 4);
        assert_eq!(galaxy.enterprise().sector().y, 4);
    }

    #[test]
    fn move_north_within_quadrant() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 6);

        // Course 3 (north, dy=-1), warp 0.375 → n=3 steps
        execute_move(&mut galaxy,3.0, 0.375, &mut MockOutput::new());
        assert_eq!(galaxy.enterprise().sector().x, 4);
        assert_eq!(galaxy.enterprise().sector().y, 3);
    }

    #[test]
    fn move_south_within_quadrant() {
        let mut galaxy = Galaxy::new(42);
        place_enterprise_for_test(&mut galaxy, 4, 4, 4, 2);

        // Course 7 (south, dy=+1), warp 0.25 → n=2 steps
        execute_move(&mut galaxy,7.0, 0.25, &mut MockOutput::new());
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
        execute_move(&mut galaxy,1.0, 0.5, &mut MockOutput::new());
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
        execute_move(&mut galaxy,1.0, 0.5, &mut MockOutput::new());

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
