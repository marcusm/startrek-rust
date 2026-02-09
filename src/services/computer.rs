use std::io::{self, Write};

use crate::models::constants::{Device, GALAXY_SIZE};
use crate::models::galaxy::Galaxy;

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
        "2" => println!("NOT YET IMPLEMENTED"),
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
}
