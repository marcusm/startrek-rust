use super::constants::{Device, INITIAL_ENERGY, INITIAL_SHIELDS, INITIAL_TORPEDOES, NUM_DEVICES};
use super::position::{QuadrantPosition, SectorPosition};

/// The player's starship.
#[derive(Debug)]
pub struct Enterprise {
    quadrant: QuadrantPosition,
    sector: SectorPosition,
    energy: f64,
    torpedoes: i32,
    shields: f64,
    /// Damage state for each of the 8 devices.
    /// 0 = operational, negative = damaged, positive = improved.
    devices: [f64; NUM_DEVICES],
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

    // Getters
    pub fn quadrant(&self) -> QuadrantPosition {
        self.quadrant
    }

    pub fn sector(&self) -> SectorPosition {
        self.sector
    }

    pub fn energy(&self) -> f64 {
        self.energy
    }

    pub fn shields(&self) -> f64 {
        self.shields
    }

    pub fn torpedoes(&self) -> i32 {
        self.torpedoes
    }

    pub fn devices(&self) -> &[f64; NUM_DEVICES] {
        &self.devices
    }

    // Controlled mutations
    pub fn consume_energy(&mut self, amount: f64) -> Result<(), &'static str> {
        if self.energy >= amount {
            self.energy -= amount;
            Ok(())
        } else {
            Err("Insufficient energy")
        }
    }

    pub fn move_to(&mut self, quadrant: QuadrantPosition, sector: SectorPosition) {
        self.quadrant = quadrant;
        self.sector = sector;
    }

    pub fn set_shields(&mut self, value: f64) {
        self.shields = value;
    }

    pub fn consume_torpedo(&mut self) -> Result<(), &'static str> {
        if self.torpedoes > 0 {
            self.torpedoes -= 1;
            Ok(())
        } else {
            Err("No torpedoes remaining")
        }
    }

    pub fn damage_device(&mut self, device: Device, amount: f64) {
        self.devices[device as usize] -= amount;
    }

    pub fn repair_device(&mut self, device: Device, amount: f64) {
        self.devices[device as usize] += amount;
    }

    pub fn set_energy(&mut self, value: f64) {
        self.energy = value;
    }

    pub fn set_torpedoes(&mut self, value: i32) {
        self.torpedoes = value;
    }

    pub fn add_energy(&mut self, amount: f64) {
        self.energy += amount;
    }

    pub fn subtract_energy(&mut self, amount: f64) {
        self.energy -= amount;
    }

    pub fn subtract_shields(&mut self, amount: f64) {
        self.shields -= amount;
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

    /// Shield control (spec section 6.5).
    /// Transfers energy between shields and main energy reserves.
    /// Returns Ok(()) on success, or Err with an error message.
    pub fn shield_control(&mut self, new_shield_value: f64) -> Result<(), ShieldControlError> {
        // Check if shield control is damaged (D[7] < 0)
        if self.is_damaged(Device::ShieldControl) {
            return Err(ShieldControlError::SystemDamaged);
        }

        // Input validation: reject non-positive values
        if new_shield_value <= 0.0 {
            return Err(ShieldControlError::InvalidInput);
        }

        // Check if we have enough total energy (energy + shields)
        let total_available = self.energy + self.shields;
        if new_shield_value > total_available {
            return Err(ShieldControlError::InsufficientEnergy);
        }

        // Perform the energy transfer (conserving total energy)
        self.energy = total_available - new_shield_value;
        self.shields = new_shield_value;

        Ok(())
    }
}

/// Errors that can occur during shield control operations.
#[derive(Debug, PartialEq)]
pub enum ShieldControlError {
    /// Shield control system is damaged
    SystemDamaged,
    /// Requested shield value is invalid (≤ 0)
    InvalidInput,
    /// Not enough total energy available
    InsufficientEnergy,
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
        e.set_energy(1000.0);
        e.set_shields(500.0);
        e.set_torpedoes(3);
        e
    }

    #[test]
    fn docking_when_adjacent_horizontally() {
        let mut e = enterprise_at(SectorPosition { x: 4, y: 4 });
        let starbase = Some(SectorPosition { x: 5, y: 4 });

        assert!(e.check_docking(starbase));
        assert_eq!(e.energy(), INITIAL_ENERGY);
        assert_eq!(e.torpedoes(), INITIAL_TORPEDOES);
        assert_eq!(e.shields(), INITIAL_SHIELDS);
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
        assert_eq!(e.energy(), 1000.0);
        assert_eq!(e.torpedoes(), 3);
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


    // Shield Control Tests (spec section 6.5)

    #[test]
    fn shield_control_transfers_energy_to_shields() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        // Initial: energy = 1000, shields = 500
        let initial_total = e.energy() + e.shields(); // 1500

        // Transfer 300 more to shields (total shields = 800)
        let result = e.shield_control(800.0);

        assert!(result.is_ok());
        assert_eq!(e.shields(), 800.0);
        assert_eq!(e.energy(), 700.0);
        assert_eq!(e.energy() + e.shields(), initial_total); // Total conserved
    }

    #[test]
    fn shield_control_transfers_shields_to_energy() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        // Initial: energy = 1000, shields = 500

        // Transfer shields back to energy (reduce shields to 100)
        let result = e.shield_control(100.0);

        assert!(result.is_ok());
        assert_eq!(e.shields(), 100.0);
        assert_eq!(e.energy(), 1400.0);
    }

    #[test]
    fn shield_control_blocked_when_system_damaged() {
        use super::ShieldControlError;
        use crate::models::constants::Device;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        e.damage_device(Device::ShieldControl, 2.0);

        let result = e.shield_control(600.0);

        assert_eq!(result, Err(ShieldControlError::SystemDamaged));
        // Energy and shields unchanged
        assert_eq!(e.energy(), 1000.0);
        assert_eq!(e.shields(), 500.0);
    }

    #[test]
    fn shield_control_rejects_zero_input() {
        use super::ShieldControlError;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });

        let result = e.shield_control(0.0);

        assert_eq!(result, Err(ShieldControlError::InvalidInput));
        // Energy and shields unchanged
        assert_eq!(e.energy(), 1000.0);
        assert_eq!(e.shields(), 500.0);
    }

    #[test]
    fn shield_control_rejects_negative_input() {
        use super::ShieldControlError;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });

        let result = e.shield_control(-100.0);

        assert_eq!(result, Err(ShieldControlError::InvalidInput));
        // Energy and shields unchanged
        assert_eq!(e.energy(), 1000.0);
        assert_eq!(e.shields(), 500.0);
    }

    #[test]
    fn shield_control_rejects_insufficient_energy() {
        use super::ShieldControlError;
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        // Total available: 1000 + 500 = 1500

        let result = e.shield_control(2000.0);

        assert_eq!(result, Err(ShieldControlError::InsufficientEnergy));
        // Energy and shields unchanged
        assert_eq!(e.energy(), 1000.0);
        assert_eq!(e.shields(), 500.0);
    }

    #[test]
    fn shield_control_can_use_all_energy_for_shields() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        let total = e.energy() + e.shields(); // 1500

        // Put all energy into shields
        let result = e.shield_control(total);

        assert!(result.is_ok());
        assert_eq!(e.shields(), total);
        assert_eq!(e.energy(), 0.0);
    }

    #[test]
    fn shield_control_can_remove_all_shields() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        e.set_shields(1000.0);
        e.set_energy(500.0);

        // Minimum valid input is slightly above 0
        let result = e.shield_control(0.1);

        assert!(result.is_ok());
        assert_eq!(e.shields(), 0.1);
        assert_eq!(e.energy(), 1499.9);
    }

    #[test]
    fn shield_control_exact_boundary_at_total_energy() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        e.set_shields(800.0);
        e.set_energy(200.0);

        // Exactly at the boundary (should succeed)
        let result = e.shield_control(1000.0);

        assert!(result.is_ok());
        assert_eq!(e.shields(), 1000.0);
        assert_eq!(e.energy(), 0.0);

        // Just above the boundary (should fail)
        let result = e.shield_control(1000.1);
        assert_eq!(result, Err(ShieldControlError::InsufficientEnergy));
    }

    #[test]
    fn shield_control_preserves_total_energy() {
        let mut e = enterprise_at(SectorPosition { x: 1, y: 1 });
        e.set_energy(2000.0);
        e.set_shields(300.0);
        let initial_total = 2300.0;

        // Multiple transfers
        let _ = e.shield_control(1000.0);
        assert_eq!(e.energy() + e.shields(), initial_total);

        let _ = e.shield_control(500.0);
        assert_eq!(e.energy() + e.shields(), initial_total);

        let _ = e.shield_control(2000.0);
        assert_eq!(e.energy() + e.shields(), initial_total);
    }
}
