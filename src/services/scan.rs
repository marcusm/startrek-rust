use crate::io::OutputWriter;
use crate::models::constants::{Device, GALAXY_SIZE, SECTOR_SIZE};
use crate::models::errors::GameResult;
use crate::models::galaxy::Galaxy;

/// Long Range Sensor Scan — Command 2 (spec section 6.2).
pub fn long_range_scan(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) -> GameResult<()> {
    if galaxy.enterprise.is_damaged(Device::LongRangeSensors) {
        output.writeln("LONG RANGE SENSORS ARE INOPERABLE");
        return Ok(());
    }

    let qx = galaxy.enterprise.quadrant.x;
    let qy = galaxy.enterprise.quadrant.y;
    output.writeln(&format!("LONG RANGE SENSOR SCAN FOR QUADRANT {},{}", qx, qy));

    let border = "-------------------";
    for dy in -1..=1_i32 {
        output.writeln(&border);
        let mut cells: Vec<String> = Vec::new();
        for dx in -1..=1_i32 {
            let scan_x = qx + dx;
            let scan_y = qy + dy;
            if scan_x < 1 || scan_x > GALAXY_SIZE as i32 || scan_y < 1 || scan_y > GALAXY_SIZE as i32
            {
                cells.push("xxx".to_string());
            } else {
                let encoded = galaxy.quadrants[(scan_y - 1) as usize][(scan_x - 1) as usize]
                    .encoded();
                cells.push(format!("{:03}", encoded));
                galaxy.record_quadrant_to_memory(scan_x, scan_y);
            }
        }
        output.writeln(&format!("| {} | {} | {} |", cells[0], cells[1], cells[2]));
    }
    output.writeln(&border);
    Ok(())
}

/// Short Range Sensor Scan — Command 1 (spec section 6.1).
pub fn short_range_scan(galaxy: &mut Galaxy, output: &mut dyn OutputWriter) -> GameResult<()> {
    galaxy.check_docking();
    let condition = galaxy.evaluate_condition();

    if galaxy.enterprise.is_damaged(Device::ShortRangeSensors) {
        output.writeln("*** SHORT RANGE SENSORS ARE OUT ***");
        return Ok(());
    }

    let border = "-=--=--=--=--=--=--=--=-";
    let e = &galaxy.enterprise;
    let status: [String; SECTOR_SIZE] = [
        format!("STARDATE  {}", galaxy.stardate as i32),
        format!("CONDITION {}", condition.label()),
        format!("QUADRANT  {},{}", e.quadrant.x, e.quadrant.y),
        format!("SECTOR    {},{}", e.sector.x, e.sector.y),
        format!("ENERGY    {}", e.energy as i32),
        format!("SHIELDS   {}", e.shields as i32),
        format!("PHOTON TORPEDOES {}", e.torpedoes),
        String::new(),
    ];

    output.writeln(&border);
    for y in 1..=SECTOR_SIZE as i32 {
        let row = galaxy.sector_map.render_row(y);
        let idx = (y - 1) as usize;
        if !status[idx].is_empty() {
            output.writeln(&format!("{}        {}", row, status[idx]));
        } else {
            output.writeln(&row);
        }
    }
    output.writeln(&border);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::{Device, GALAXY_SIZE};

    #[test]
    fn short_range_scan_does_not_panic() {
        use crate::io::test_utils::MockOutput;
        let mut galaxy = Galaxy::new(42);
        let mut output = MockOutput::new();
        // Just verify it runs without panicking
        short_range_scan(&mut galaxy, &mut output).unwrap();
    }

    #[test]
    fn short_range_scan_blocked_when_sensors_damaged() {
        use crate::io::test_utils::MockOutput;
        let mut galaxy = Galaxy::new(42);
        let mut output = MockOutput::new();
        galaxy.enterprise.devices[Device::ShortRangeSensors as usize] = -1.0;
        // Should print damage message and return without panicking
        short_range_scan(&mut galaxy, &mut output).unwrap();
    }

    #[test]
    fn long_range_scan_does_not_panic() {
        use crate::io::test_utils::MockOutput;
        let mut galaxy = Galaxy::new(42);
        let mut output = MockOutput::new();
        long_range_scan(&mut galaxy, &mut output).unwrap();
    }

    #[test]
    fn long_range_scan_blocked_when_sensors_damaged() {
        use crate::io::test_utils::MockOutput;
        let mut galaxy = Galaxy::new(42);
        let mut output = MockOutput::new();
        galaxy.enterprise.devices[Device::LongRangeSensors as usize] = -1.0;
        // Should print damage message and return without panicking
        long_range_scan(&mut galaxy, &mut output).unwrap();
    }

    #[test]
    fn long_range_scan_updates_computer_memory() {
        use crate::io::test_utils::MockOutput;
        let mut galaxy = Galaxy::new(42);
        let mut output = MockOutput::new();
        // Reset computer memory to verify LRS populates it
        galaxy.computer_memory = [[-1; GALAXY_SIZE]; GALAXY_SIZE];

        long_range_scan(&mut galaxy, &mut output).unwrap();

        let qx = galaxy.enterprise.quadrant.x;
        let qy = galaxy.enterprise.quadrant.y;

        // The current quadrant and its in-bounds neighbors should now be recorded
        for dy in -1..=1_i32 {
            for dx in -1..=1_i32 {
                let sx = qx + dx;
                let sy = qy + dy;
                if sx >= 1 && sx <= 8 && sy >= 1 && sy <= 8 {
                    let mem = galaxy.computer_memory[(sy - 1) as usize][(sx - 1) as usize];
                    let actual =
                        galaxy.quadrants[(sy - 1) as usize][(sx - 1) as usize].encoded();
                    assert_eq!(mem, actual, "memory at ({},{}) should match quadrant data", sx, sy);
                }
            }
        }
    }

    #[test]
    fn long_range_scan_does_not_record_when_computer_damaged() {
        use crate::io::test_utils::MockOutput;
        let mut galaxy = Galaxy::new(42);
        let mut output = MockOutput::new();
        galaxy.computer_memory = [[-1; GALAXY_SIZE]; GALAXY_SIZE];
        galaxy.enterprise.devices[Device::Computer as usize] = -1.0;

        long_range_scan(&mut galaxy, &mut output).unwrap();

        // All memory should remain unscanned (-1)
        for y in 0..GALAXY_SIZE {
            for x in 0..GALAXY_SIZE {
                assert_eq!(galaxy.computer_memory[y][x], -1);
            }
        }
    }
}
