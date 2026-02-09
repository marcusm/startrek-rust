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
}
