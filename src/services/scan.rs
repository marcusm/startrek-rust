use crate::models::constants::{Device, SECTOR_SIZE};
use crate::models::galaxy::Galaxy;

/// Short Range Sensor Scan — Command 1 (spec section 6.1).
pub fn short_range_scan(galaxy: &mut Galaxy) {
    galaxy.check_docking();
    let condition = galaxy.evaluate_condition();

    if galaxy.enterprise.is_damaged(Device::ShortRangeSensors) {
        println!("*** SHORT RANGE SENSORS ARE OUT ***");
        return;
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

    println!("{}", border);
    for y in 1..=SECTOR_SIZE as i32 {
        let row = galaxy.sector_map.render_row(y);
        let idx = (y - 1) as usize;
        if !status[idx].is_empty() {
            println!("{}        {}", row, status[idx]);
        } else {
            println!("{}", row);
        }
    }
    println!("{}", border);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::Device;

    #[test]
    fn short_range_scan_does_not_panic() {
        let mut galaxy = Galaxy::new(42);
        // Just verify it runs without panicking
        short_range_scan(&mut galaxy);
    }

    #[test]
    fn short_range_scan_blocked_when_sensors_damaged() {
        let mut galaxy = Galaxy::new(42);
        galaxy.enterprise.devices[Device::ShortRangeSensors as usize] = -1.0;
        // Should print damage message and return without panicking
        short_range_scan(&mut galaxy);
    }
}
