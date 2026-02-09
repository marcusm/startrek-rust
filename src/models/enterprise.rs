use super::constants::{Device, INITIAL_ENERGY, INITIAL_SHIELDS, INITIAL_TORPEDOES, NUM_DEVICES};
use super::position::{QuadrantPosition, SectorPosition};

/// The player's starship.
pub struct Enterprise {
    pub quadrant: QuadrantPosition,
    pub sector: SectorPosition,
    pub energy: f64,
    pub torpedoes: i32,
    pub shields: f64,
    /// Damage state for each of the 8 devices.
    /// 0 = operational, negative = damaged, positive = improved.
    pub devices: [f64; NUM_DEVICES],
}

impl Enterprise {
    pub fn new(quadrant: QuadrantPosition, sector: SectorPosition) -> Self {
        Enterprise {
            quadrant,
            sector,
            energy: INITIAL_ENERGY,
            torpedoes: INITIAL_TORPEDOES,
            shields: INITIAL_SHIELDS,
            devices: [0.0; NUM_DEVICES],
        }
    }

    pub fn is_damaged(&self, device: Device) -> bool {
        self.devices[device as usize] < 0.0
    }

    /// Reset ship resources when docking at a starbase (spec section 9.2).
    pub fn dock(&mut self) {
        self.energy = INITIAL_ENERGY;
        self.torpedoes = INITIAL_TORPEDOES;
        self.shields = INITIAL_SHIELDS;
    }

    /// Check if the Enterprise is adjacent to (or at) a starbase (spec section 9.1).
    pub fn is_adjacent_to_starbase(&self, starbase: Option<SectorPosition>) -> bool {
        if let Some(base) = starbase {
            (self.sector.x - base.x).abs() <= 1 && (self.sector.y - base.y).abs() <= 1
        } else {
            false
        }
    }

    /// Print the damage control report (spec section 6.6).
    /// If the Damage Control device is damaged, prints a failure message.
    /// Otherwise, lists all 8 devices and their repair states (truncated to integer).
    pub fn damage_report(&self) {
        if self.is_damaged(Device::DamageControl) {
            println!("DAMAGE CONTROL REPORT IS NOT AVAILABLE");
            return;
        }

        println!("{:<14}{}", "DEVICE", "STATE OF REPAIR");
        for device in Device::ALL.iter() {
            let state = self.devices[*device as usize] as i32;
            println!("{:<14}{}", device.name(), state);
        }
    }

    /// Check if the Enterprise is adjacent to a starbase and dock if so.
    /// Returns true if docked (spec section 9.1-9.2).
    pub fn check_docking(&mut self, starbase: Option<SectorPosition>) -> bool {
        if self.is_adjacent_to_starbase(starbase) {
            self.dock();
            println!("SHIELDS DROPPED FOR DOCKING PURPOSES");
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::constants::{INITIAL_ENERGY, INITIAL_SHIELDS, INITIAL_TORPEDOES};
    use crate::models::position::SectorPosition;

    /// Helper: create an Enterprise with reduced resources at a given sector.
    fn enterprise_at(sector: SectorPosition) -> Enterprise {
        let mut e = Enterprise::new(
            QuadrantPosition { x: 1, y: 1 },
            sector,
        );
        e.energy = 1000.0;
        e.shields = 500.0;
        e.torpedoes = 3;
        e
    }

    #[test]
    fn docking_when_adjacent_horizontally() {
        let mut e = enterprise_at(SectorPosition { x: 4, y: 4 });
        let starbase = Some(SectorPosition { x: 5, y: 4 });

        assert!(e.check_docking(starbase));
        assert_eq!(e.energy, INITIAL_ENERGY);
        assert_eq!(e.torpedoes, INITIAL_TORPEDOES);
        assert_eq!(e.shields, INITIAL_SHIELDS);
    }

    #[test]
    fn docking_when_adjacent_diagonally() {
        let mut e = enterprise_at(SectorPosition { x: 3, y: 3 });
        let starbase = Some(SectorPosition { x: 4, y: 4 });

        assert!(e.check_docking(starbase));
    }

    #[test]
    fn no_docking_when_too_far() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        let starbase = Some(SectorPosition { x: 4, y: 4 });

        assert!(!e.check_docking(starbase));
        assert_eq!(e.energy, 1000.0);
        assert_eq!(e.torpedoes, 3);
    }

    #[test]
    fn no_docking_when_no_starbase() {
        let mut e = enterprise_at(SectorPosition { x: 4, y: 4 });

        assert!(!e.check_docking(None));
    }

    #[test]
    fn docking_when_distance_exactly_one() {
        let base = SectorPosition { x: 4, y: 4 };
        let adjacent_positions = [
            SectorPosition { x: 3, y: 3 },
            SectorPosition { x: 4, y: 3 },
            SectorPosition { x: 5, y: 3 },
            SectorPosition { x: 3, y: 4 },
            SectorPosition { x: 5, y: 4 },
            SectorPosition { x: 3, y: 5 },
            SectorPosition { x: 4, y: 5 },
            SectorPosition { x: 5, y: 5 },
        ];
        for pos in &adjacent_positions {
            let mut e = enterprise_at(*pos);
            assert!(
                e.check_docking(Some(base)),
                "should dock at ({}, {}) next to base at (4, 4)",
                pos.x,
                pos.y
            );
        }
    }

    #[test]
    fn damage_report_shows_all_devices_when_undamaged() {
        let e = enterprise_at(SectorPosition { x: 1, y: 1 });
        // All devices at 0.0, DamageControl is not damaged, so report should work.
        // We just verify it doesn't panic. (Output goes to stdout.)
        e.damage_report();
    }

    #[test]
    fn damage_report_blocked_when_damage_control_damaged() {
        use crate::models::constants::Device;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        e.devices[Device::DamageControl as usize] = -1.0;
        assert!(e.is_damaged(Device::DamageControl));
        // Should print the unavailable message and return early.
        e.damage_report();
    }

    #[test]
    fn damage_report_available_when_other_devices_damaged() {
        use crate::models::constants::Device;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        // Damage everything except DamageControl
        e.devices[Device::WarpEngines as usize] = -3.0;
        e.devices[Device::ShortRangeSensors as usize] = -1.0;
        e.devices[Device::Computer as usize] = -5.0;
        assert!(!e.is_damaged(Device::DamageControl));
        // Should still produce the report.
        e.damage_report();
    }

    #[test]
    fn damage_report_truncates_values() {
        use crate::models::constants::Device;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        e.devices[Device::WarpEngines as usize] = -3.7;
        e.devices[Device::PhaserControl as usize] = 2.9;
        // Truncation: -3.7 as i32 = -3, 2.9 as i32 = 2
        assert_eq!(e.devices[Device::WarpEngines as usize] as i32, -3);
        assert_eq!(e.devices[Device::PhaserControl as usize] as i32, 2);
        e.damage_report();
    }
}
