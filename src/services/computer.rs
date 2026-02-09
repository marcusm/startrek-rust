use std::io::{self, Write};

use crate::models::constants::{Device, GALAXY_SIZE};
use crate::models::galaxy::Galaxy;
use crate::models::position::SectorPosition;

/// Library Computer — Command 7 (spec section 6.7).
pub fn library_computer(galaxy: &mut Galaxy) {
    if galaxy.enterprise.is_damaged(Device::Computer) {
        println!("COMPUTER DISABLED");
        return;
    }

    println!("COMPUTER ACTIVE AND AWAITING COMMAND");
    let input = read_line("");
    let input = input.trim();

    match input {
        "0" => cumulative_galactic_record(galaxy),
        "1" => status_report(galaxy),
        "2" => photon_torpedo_data(galaxy),
        _ => print_computer_menu(),
    }
}

/// Option 0 — Cumulative Galactic Record (spec section 6.7).
fn cumulative_galactic_record(galaxy: &Galaxy) {
    let qx = galaxy.enterprise.quadrant.x;
    let qy = galaxy.enterprise.quadrant.y;
    println!("COMPUTER RECORD OF GALAXY FOR QUADRANT {},{}", qx, qy);

    let border = "-------------------------------------------------";
    for y in 0..GALAXY_SIZE {
        println!("{}", border);
        let mut cells: Vec<String> = Vec::new();
        for x in 0..GALAXY_SIZE {
            let val = galaxy.computer_memory[y][x];
            if val < 0 {
                cells.push("???".to_string());
            } else {
                cells.push(format!("{:03}", val));
            }
        }
        println!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            cells[0], cells[1], cells[2], cells[3], cells[4], cells[5], cells[6], cells[7]
        );
    }
    println!("{}", border);
}

/// Option 1 — Status Report (spec section 6.7).
/// Prints status info then falls through to the damage control report.
fn status_report(galaxy: &Galaxy) {
    println!("   STATUS REPORT");
    println!();
    println!("NUMBER OF KLINGONS LEFT  = {}", galaxy.total_klingons);
    let stardates_left =
        (galaxy.starting_stardate + galaxy.mission_duration) - galaxy.stardate;
    println!("NUMBER OF STARDATES LEFT = {}", stardates_left as i32);
    println!("NUMBER OF STARBASES LEFT = {}", galaxy.total_starbases);

    // Falls through to damage control report (spec section 6.7)
    galaxy.enterprise.damage_report();
}

/// Option 2 — Photon Torpedo Data (spec section 6.7).
/// Displays direction and distance to each Klingon, then offers calculator.
fn photon_torpedo_data(galaxy: &Galaxy) {
    // Display data for each living Klingon
    for klingon in &galaxy.sector_map.klingons {
        if !klingon.is_alive() {
            continue; // Skip dead Klingons
        }

        let (direction, distance) = calculate_direction_and_distance(
            galaxy.enterprise.sector,
            klingon.sector,
        );

        println!("DIRECTION = {:.2}", direction);
        println!("DISTANCE  = {:.2}", distance);
    }

    // Calculator option
    println!("ENTER 1 TO USE THE CALCULATOR");
    let input = read_line("");
    if input.trim() == "1" {
        use_calculator(galaxy);
    }
}

/// Calculator sub-feature of photon torpedo data (spec section 6.7).
/// Allows player to calculate direction/distance between any two coordinates.
fn use_calculator(galaxy: &Galaxy) {
    println!(
        "YOU ARE AT QUADRANT {},{} SECTOR {},{}",
        galaxy.enterprise.quadrant.x,
        galaxy.enterprise.quadrant.y,
        galaxy.enterprise.sector.x,
        galaxy.enterprise.sector.y
    );
    println!("SHIP'S & TARGET'S COORDINATES ARE");

    let input = read_line("");
    let coords: Vec<&str> = input.trim().split(',').collect();
    if coords.len() != 4 {
        return;
    }

    let source_x: i32 = coords[0].trim().parse().unwrap_or(0);
    let source_y: i32 = coords[1].trim().parse().unwrap_or(0);
    let target_x: i32 = coords[2].trim().parse().unwrap_or(0);
    let target_y: i32 = coords[3].trim().parse().unwrap_or(0);

    let source = SectorPosition {
        x: source_x,
        y: source_y,
    };
    let target = SectorPosition {
        x: target_x,
        y: target_y,
    };

    let (direction, distance) = calculate_direction_and_distance(source, target);

    println!("DIRECTION = {:.2}", direction);
    println!("DISTANCE  = {:.2}", distance);

    // Warp units calculation (max of absolute deltas)
    let warp_units = ((target_x - source_x).abs()).max((target_y - source_y).abs());
    let plural = if warp_units != 1 { "S" } else { "" };
    println!("   ({} WARP UNIT{})", warp_units, plural);
}

/// Direction and distance calculation (spec section 7.4).
/// Uses the original ratio-based algorithm from the spec.
fn calculate_direction_and_distance(
    source: SectorPosition,
    target: SectorPosition,
) -> (f64, f64) {
    let delta_x = (target.x - source.x) as f64;
    let delta_y = (source.y - target.y) as f64; // Inverted per spec

    let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

    // Direction calculation (spec section 7.4)
    let direction = if delta_x >= 0.0 && delta_y >= 0.0 {
        // Case 1: right and/or up
        let base = if delta_x > 0.0 || delta_y > 0.0 {
            1.0
        } else {
            5.0
        };
        if delta_y.abs() <= delta_x.abs() {
            base + delta_y.abs() / delta_x.abs()
        } else {
            base + (delta_y.abs() - delta_x.abs() + delta_y.abs()) / delta_y.abs()
        }
    } else if delta_x < 0.0 && delta_y > 0.0 {
        // Case 2: left and up
        let base = 3.0;
        if delta_y.abs() >= delta_x.abs() {
            base + delta_x.abs() / delta_y.abs()
        } else {
            base + (delta_x.abs() - delta_y.abs() + delta_x.abs()) / delta_x.abs()
        }
    } else if delta_x >= 0.0 && delta_y < 0.0 {
        // Case 3: right and down
        let base = 7.0;
        if delta_y.abs() >= delta_x.abs() {
            base + delta_x.abs() / delta_y.abs()
        } else {
            base + (delta_x.abs() - delta_y.abs() + delta_x.abs()) / delta_x.abs()
        }
    } else {
        // Case 4: left and down
        let base = 5.0;
        if delta_y.abs() <= delta_x.abs() {
            base + delta_y.abs() / delta_x.abs()
        } else {
            base + (delta_y.abs() - delta_x.abs() + delta_y.abs()) / delta_y.abs()
        }
    };

    (direction, distance)
}

fn print_computer_menu() {
    println!("FUNCTIONS AVAILABLE FROM COMPUTER");
    println!("   0 = CUMULATIVE GALACTIC RECORD");
    println!("   1 = STATUS REPORT");
    println!("   2 = PHOTON TORPEDO DATA");
}

fn read_line(prompt: &str) -> String {
    if !prompt.is_empty() {
        print!("{} ", prompt);
    }
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::Device;
    use crate::models::galaxy::Galaxy;

    #[test]
    fn galactic_record_shows_unscanned_as_negative() {
        let galaxy = Galaxy::new(42);
        // Most quadrants should still be -1 (unscanned) except the starting one
        let mut unscanned_count = 0;
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                if galaxy.computer_memory[y][x] < 0 {
                    unscanned_count += 1;
                }
            }
        }
        // 64 total minus the starting quadrant = 63 unscanned
        assert_eq!(unscanned_count, 63);
    }

    #[test]
    fn starting_quadrant_is_recorded() {
        let galaxy = Galaxy::new(42);
        let qx = galaxy.enterprise.quadrant.x;
        let qy = galaxy.enterprise.quadrant.y;
        let mem = galaxy.computer_memory[(qy - 1) as usize][(qx - 1) as usize];
        let actual = galaxy.quadrants[(qy - 1) as usize][(qx - 1) as usize].encoded();
        assert_eq!(mem, actual);
    }

    #[test]
    fn record_blocked_when_computer_damaged() {
        let mut galaxy = Galaxy::new(42);
        galaxy.enterprise.devices[Device::Computer as usize] = -1.0;

        // Pick a quadrant we know is unscanned
        let qx = galaxy.enterprise.quadrant.x;
        let qy = galaxy.enterprise.quadrant.y;
        let target_x = if qx < 8 { qx + 1 } else { qx - 1 };

        // Should still be -1 (unscanned)
        assert_eq!(
            galaxy.computer_memory[(qy - 1) as usize][(target_x - 1) as usize],
            -1
        );

        // Try to record — should be blocked
        galaxy.record_quadrant_to_memory(target_x, qy);
        assert_eq!(
            galaxy.computer_memory[(qy - 1) as usize][(target_x - 1) as usize],
            -1
        );
    }

    #[test]
    fn status_report_stardates_remaining() {
        let galaxy = Galaxy::new(42);
        let expected = (galaxy.starting_stardate + galaxy.mission_duration) - galaxy.stardate;
        // At game start, stardate == starting_stardate, so remaining == mission_duration
        assert_eq!(expected as i32, galaxy.mission_duration as i32);
    }

    #[test]
    fn status_report_stardates_decrease_over_time() {
        let mut galaxy = Galaxy::new(42);
        let initial_remaining =
            (galaxy.starting_stardate + galaxy.mission_duration) - galaxy.stardate;
        galaxy.stardate += 5.0;
        let after_remaining =
            (galaxy.starting_stardate + galaxy.mission_duration) - galaxy.stardate;
        assert_eq!((initial_remaining - after_remaining) as i32, 5);
    }

    #[test]
    fn status_report_displays_without_panic() {
        let galaxy = Galaxy::new(99);
        // Verify the function runs without panicking (output goes to stdout)
        status_report(&galaxy);
    }

    #[test]
    fn status_report_falls_through_to_damage_report() {
        let mut galaxy = Galaxy::new(99);
        // Damage a device so we can verify the damage report portion runs
        galaxy.enterprise.devices[Device::WarpEngines as usize] = -2.0;
        // Should not panic — status report prints then falls through to damage_report
        status_report(&galaxy);
    }

    #[test]
    fn status_report_with_damage_control_damaged() {
        let mut galaxy = Galaxy::new(99);
        galaxy.enterprise.devices[Device::DamageControl as usize] = -1.0;
        // The fall-through damage report should print "not available" but not panic
        status_report(&galaxy);
    }

    // --- Photon Torpedo Data Tests (Option 2) ---

    #[test]
    fn direction_calculation_east() {
        // Course 1 (east): from (4,4) to (7,4) → direction should be ~1.0
        let source = SectorPosition { x: 4, y: 4 };
        let target = SectorPosition { x: 7, y: 4 };
        let (direction, _distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            (direction - 1.0).abs() < 0.01,
            "east should be ~1.0, got {}",
            direction
        );
    }

    #[test]
    fn direction_calculation_north() {
        // Course 3 (north): from (4,4) to (4,1) → direction should be ~3.0
        let source = SectorPosition { x: 4, y: 4 };
        let target = SectorPosition { x: 4, y: 1 };
        let (direction, _distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            (direction - 3.0).abs() < 0.01,
            "north should be ~3.0, got {}",
            direction
        );
    }

    #[test]
    fn direction_calculation_west() {
        // Course 5 (west): from (4,4) to (1,4) → direction should be ~5.0
        let source = SectorPosition { x: 4, y: 4 };
        let target = SectorPosition { x: 1, y: 4 };
        let (direction, _distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            (direction - 5.0).abs() < 0.01,
            "west should be ~5.0, got {}",
            direction
        );
    }

    #[test]
    fn direction_calculation_south() {
        // Course 7 (south): from (4,4) to (4,7) → direction should be ~7.0
        let source = SectorPosition { x: 4, y: 4 };
        let target = SectorPosition { x: 4, y: 7 };
        let (direction, _distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            (direction - 7.0).abs() < 0.01,
            "south should be ~7.0, got {}",
            direction
        );
    }

    #[test]
    fn distance_calculation_horizontal() {
        // 3 units east
        let source = SectorPosition { x: 2, y: 5 };
        let target = SectorPosition { x: 5, y: 5 };
        let (_direction, distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            (distance - 3.0).abs() < 0.01,
            "distance should be 3.0, got {}",
            distance
        );
    }

    #[test]
    fn distance_calculation_diagonal() {
        // 3 units east, 4 units south → distance = 5 (3-4-5 triangle)
        let source = SectorPosition { x: 2, y: 2 };
        let target = SectorPosition { x: 5, y: 6 };
        let (_direction, distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            (distance - 5.0).abs() < 0.01,
            "distance should be 5.0, got {}",
            distance
        );
    }

    #[test]
    fn distance_calculation_same_position() {
        let source = SectorPosition { x: 4, y: 4 };
        let target = SectorPosition { x: 4, y: 4 };
        let (_direction, distance) = super::calculate_direction_and_distance(source, target);
        assert!(
            distance.abs() < 0.01,
            "distance should be 0.0, got {}",
            distance
        );
    }
}
